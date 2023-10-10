// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::path::PathBuf;

use ancvm_program::thread_context::ThreadContext;
use ancvm_types::{ExternalLibraryType, OPERAND_SIZE_IN_BYTES};

pub fn extcall(thread_context: &mut ThreadContext) {
    // `fn (external_func_index:i32)`
    //
    // the 'external_func_index' is the index within a specific module, it is not
    // the 'unified_external_func_index'.

    let external_function_index = thread_context.stack.pop_i64_u() as usize;
    let module_index = thread_context.pc.module_index;

    // get the unified external function index
    let (unified_external_function_index, type_index) = thread_context
        .program_context
        .external_func_index_section
        .get_item_unified_external_func_index_and_type_index(module_index, external_function_index);

    // get the data types of params and results of the external function
    let (param_datatypes, result_datatypes) = thread_context.program_context.program_modules
        [module_index]
        .type_section
        .get_item_params_and_results(type_index);

    let opt_func_pointer_and_wrapper = {
        let table = thread_context.external_function_table.as_ref().borrow();
        table.get_external_function_pointer_and_wrapper_function(unified_external_function_index)
    };

    let func_pointer_and_wrapper = opt_func_pointer_and_wrapper.unwrap_or_else(|| {
        // get the name of the external function and
        // the index of the unified external library
        let (external_function_name, unified_external_library_index) = thread_context
            .program_context
            .unified_external_func_section
            .get_item_name_and_unified_external_library_index(unified_external_function_index);

        // get the file path or name of the external library
        let (external_library_name, external_library_type) = thread_context
            .program_context
            .unified_external_library_section
            .get_item_name_and_external_library_type(unified_external_library_index);

        let external_library_file_path_or_name = match external_library_type {
            ExternalLibraryType::User => {
                let mut path_buf = PathBuf::from(&thread_context.program_settings.source_path);
                if !thread_context.program_settings.is_multiple_scripts {
                    path_buf.pop();
                }
                path_buf.push("lib");
                path_buf.push(external_library_name);
                path_buf.as_os_str().to_string_lossy().to_string()
            }
            ExternalLibraryType::Shared => {
                let mut path_buf = PathBuf::from(&thread_context.program_settings.runtime_path);
                path_buf.push("lib");
                path_buf.push(external_library_name);
                path_buf.as_os_str().to_string_lossy().to_string()
            }
            ExternalLibraryType::System => external_library_name.to_owned(),
        };

        let mut table = thread_context.external_function_table.as_ref().borrow_mut();
        table
            .add_external_function(
                unified_external_function_index,
                unified_external_library_index,
                &external_library_file_path_or_name,
                external_function_name,
                param_datatypes,
                result_datatypes,
            )
            .unwrap()
    });

    // call the wrapper function:
    //
    // ```rust
    // type WrapperFunction = extern "C" fn(
    //     external_function_pointer: *const c_void,
    //     params_ptr: *const u8,
    //     results_ptr: *mut u8,
    // );
    // ```

    let params = thread_context.stack.pop_operands(param_datatypes.len());
    let mut results = [0u8; OPERAND_SIZE_IN_BYTES];

    (func_pointer_and_wrapper.1)(
        func_pointer_and_wrapper.0,
        params.as_ptr(),
        results.as_mut_ptr(),
    );

    // push the result on the stack

    if !result_datatypes.is_empty() {
        let dst = thread_context.stack.push_operand_from_memory();
        unsafe { std::ptr::copy(results.as_ptr(), dst, OPERAND_SIZE_IN_BYTES) };
    }
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        module_image::type_section::TypeEntry,
        utils::{
            build_module_binary_with_single_function_and_external_functions, BytecodeWriter,
            HelperExternalFunctionEntry,
        },
    };
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{
        ecallcode::ECallCode, opcode::Opcode, DataType, ExternalLibraryType, ForeignValue,
    };

    use crate::{in_memory_program_source::InMemoryProgramSource, interpreter::process_function};

    #[test]
    fn test_ecall_heap_capacity() {
        // init_runtime();

        // bytecodes
        //
        // 0x0000 ecall                261
        // 0x0008 i32_imm              0x2
        // 0x0010 ecall                262
        // 0x0018 i32_imm              0x4
        // 0x0020 ecall                262
        // 0x0028 i32_imm              0x1
        // 0x0030 ecall                262
        // 0x0038 ecall                261
        // 0x0040 end
        //
        // () -> (i64, i64, i64, i64, i64)

        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::ecall, ECallCode::extcall as u32)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        // `man 3 getuid`
        // 'uid_t getuid(void);'

        let binary0 = build_module_binary_with_single_function_and_external_functions(
            vec![
                TypeEntry {
                    params: vec![],
                    results: vec![DataType::I32],
                }, // getuid
                TypeEntry {
                    params: vec![],
                    results: vec![DataType::I32],
                }, // main
            ], // params
            1,
            vec![], // local varslist which
            code0,
            vec![HelperExternalFunctionEntry {
                external_library_type: ExternalLibraryType::System,
                library_name: "libc.so".to_string(),
                function_name: "getuid".to_string(),
                type_index: 0,
            }],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.new_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &vec![]);
        let results0 = result0.unwrap();
        assert!(matches!(results0[0], ForeignValue::UInt64(uid) if uid > 0 ));
    }
}
