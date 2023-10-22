// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// about the callback function
//
// on the XiaoXuan Core Script application, pass VM function as a callback function to the external C library.
//
//                                      runtime (native)
//                                   /------------------------\
//                                   |                        |
//                                   | external func list     |
//                                   | |--------------------| |
//                                   | | idx | lib  | name  | |
//                              /--> | | 0   | ".." | ".."  | |
//                              |    | |--------------------| |
//                              |    |                        |
//                              |    | wrapper func code 0    |
//  XiaoXuan core application   |    | 0x0000 0xb8, 0x34,     |
// /------------------------\   |    | 0x000a 0x12, 0x00...   | --\
// |                        |   |    |                        |   |
// | fn $demo () -> ()      |   |    |                        |   |
// |   extcall do_something | --/    | callback func table    |   |
// | end                    |        | |--------------------| |   |
// |                        |        | | mod idx | func idx | |   |      libxyz.so
// | fn $callback () -> ()  | <----- | | 0       | 0        | |   |    /----------------------\
// |   ...                  |        | | ...     | ...      | |   \--> | void do_something (  |
// | end                    |        | |--------------------| |        |     void* () cb) {   |
// |                        |        |                        |        |     ...              |
// \------------------------/        | bridge func code 0     | <----- |     (cb)(11, 13)     |
//                                   | 0x0000 0xb8, 0x34,     |        | }                    |
//                                   | 0x000a 0x12, 0x00...   |        |                      |
//                                   |                        |        \----------------------/
//                                   | bridge func code 1     |
//                                   | ...                    |
//                                   |                        |
//                                   \------------------------/
//

use ancvm_program::{jit_util::build_host_to_vm_function, thread_context::ThreadContext};

use crate::interpreter::process_callback_function_call;

pub fn host_addr_func(thread_context: &mut ThreadContext) {
    // `fn (func_pub_index:i32) -> i64/i32`

    let function_public_index = thread_context.stack.pop_i32_u() as usize;
    let module_index = thread_context.pc.module_index;

    // get the internal index of function
    let (target_module_index, function_internal_index) = thread_context
        .get_function_target_module_index_and_internal_index(module_index, function_public_index);

    let callback_function_ptr =
        get_callback_function_ptr(thread_context, target_module_index, function_internal_index)
            .unwrap();

    store_pointer_to_operand_stack(thread_context, callback_function_ptr);
}

fn store_pointer_to_operand_stack(thread_context: &mut ThreadContext, ptr: *const u8) {
    #[cfg(target_pointer_width = "64")]
    {
        let address = ptr as u64;
        thread_context.stack.push_i64_u(address);
    }

    #[cfg(target_pointer_width = "32")]
    {
        let address = ptr as u32;
        thread_context.stack.push_i32_u(address);
    }
}

fn get_callback_function_ptr(
    thread_context: &mut ThreadContext,
    target_module_index: usize,
    function_internal_index: usize,
) -> Result<*const u8, &'static str> {
    // check if the specified (target_module_index, function_internal_index) already
    // exists in the callback function table
    let opt_callback_function_ptr =
        thread_context.find_callback_function(target_module_index, function_internal_index);

    if let Some(callback_function_ptr) = opt_callback_function_ptr {
        return Ok(callback_function_ptr);
    }

    let type_index = thread_context.program_context.program_modules[target_module_index]
        .func_section
        .items[function_internal_index]
        .type_index;
    let (params, results) = thread_context.program_context.program_modules[target_module_index]
        .type_section
        .get_item_params_and_results(type_index as usize);

    if results.len() > 1 {
        return Err("The specified function has more than 1 return value.");
    }

    let delegate_function_addr = process_callback_function_call as *const u8 as usize;
    let thread_context_addr = thread_context as *const ThreadContext as *const u8 as usize;
    let callback_function_ptr = build_host_to_vm_function(
        delegate_function_addr,
        thread_context_addr,
        target_module_index,
        function_internal_index,
        params,
        results,
    );

    // store the function pointer into table
    thread_context.insert_callback_function(
        target_module_index,
        function_internal_index,
        callback_function_ptr,
    );

    Ok(callback_function_ptr)
}

#[cfg(test)]
mod tests {
    use std::env;

    use ancvm_binary::{
        module_image::type_section::TypeEntry,
        utils::{
            build_module_binary_with_functions_and_external_functions, BytecodeWriter,
            HelperExternalFunctionEntry, HelperSlimFunctionEntry,
        },
    };
    use ancvm_program::{program_settings::ProgramSettings, program_source::ProgramSource};
    use ancvm_types::{
        ecallcode::ECallCode, opcode::Opcode, DataType, ExternalLibraryType, ForeignValue,
    };

    use crate::{in_memory_program_source::InMemoryProgramSource, interpreter::process_function};

    #[test]
    fn test_ecall_host_addr_func_ie_callback_function() {
        // extern "C" do_something(callback_func, a:i32, b:i32) -> i32 {
        //     callback_func(a) + b
        // }
        //
        // func0 (a:i32, b:i32)->i32 {
        //     do_something(func1, a, b)
        // }
        //
        // func1 (a:i32) -> i32 {   ;; this is the callback function for external function 'do_something'
        //     a*2
        // }

        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 1) // func1 index
            .write_opcode_i32(Opcode::ecall, ECallCode::host_addr_func as u32) // get host address of the func1
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0) // external func param 1
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1) // external func param 2
            //
            .write_opcode_i32(Opcode::i32_imm, 0) // external func index
            .write_opcode_i32(Opcode::ecall, ECallCode::extcall as u32) // call external function
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let code1 = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode_i32(Opcode::i32_imm, 2)
            .write_opcode(Opcode::i32_mul)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_functions_and_external_functions(
            vec![
                TypeEntry {
                    params: vec![DataType::I64, DataType::I32, DataType::I32],
                    results: vec![DataType::I32],
                }, // do_something
                TypeEntry {
                    params: vec![DataType::I32, DataType::I32],
                    results: vec![DataType::I32],
                }, // func0
                TypeEntry {
                    params: vec![DataType::I32],
                    results: vec![DataType::I32],
                }, // func1
            ], // types
            vec![
                HelperSlimFunctionEntry {
                    type_index: 1,
                    local_variable_item_entries_without_args: vec![],
                    code: code0,
                },
                HelperSlimFunctionEntry {
                    type_index: 2,
                    local_variable_item_entries_without_args: vec![],
                    code: code1,
                },
            ],
            vec![],
            vec![],
            vec![],
            vec![HelperExternalFunctionEntry {
                external_library_type: ExternalLibraryType::User,
                library_name: "lib-test-0.so.1.0.0".to_string(),
                function_name: "do_something".to_string(),
                type_index: 0,
            }],
        );

        let mut pwd = env::current_dir().unwrap();
        if !pwd.ends_with("runtime") {
            // in the VSCode debug mode
            pwd.push("crates");
            pwd.push("runtime");
        }
        pwd.push("tests");
        let program_source_path = pwd.to_str().unwrap();

        let program_source0 = InMemoryProgramSource::with_settings(
            vec![binary0],
            ProgramSettings::new(program_source_path, true, "", ""),
        );

        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &vec![ForeignValue::UInt32(11), ForeignValue::UInt32(13)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(11 * 2 + 13)]);

        let result1 = process_function(
            &mut thread_context0,
            0,
            0,
            &vec![ForeignValue::UInt32(211), ForeignValue::UInt32(223)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(211 * 2 + 223)]);
    }
}
