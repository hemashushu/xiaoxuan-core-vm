// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use anc_context::thread_context::{ProgramCounter, ThreadContext};
use anc_isa::OPERAND_SIZE_IN_BYTES;

use crate::{
    extcall_handler::get_or_create_external_function, PANIC_CODE_EXTERNAL_FUNCTION_CREATE_FAILURE,
};

use super::{HandleResult, Handler};

pub fn call(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    let function_public_index = thread_context.get_param_i32();
    do_call(thread_context, function_public_index, 8)
}

pub fn dyncall(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    let function_public_index = thread_context.stack.pop_i32_u();
    do_call(thread_context, function_public_index, 2)
}

pub fn pub_index_function(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    let function_public_index = thread_context.get_param_i32();
    thread_context.stack.push_i32_u(function_public_index);
    HandleResult::Move(8)
}

fn do_call(
    thread_context: &mut ThreadContext,
    function_public_index: u32,
    instruction_length: usize,
) -> HandleResult {
    let ProgramCounter {
        instruction_address: return_instruction_address,
        function_internal_index: return_function_internal_index,
        module_index: return_module_index,
    } = thread_context.pc;

    let (target_module_index, target_function_internal_index) = thread_context
        .get_function_target_module_index_and_internal_index(
            return_module_index,
            function_public_index as usize,
        );
    let (type_index, local_list_index, code_offset, local_variables_allocate_bytes) =
        thread_context
            .get_function_type_and_local_list_index_and_code_offset_and_local_variables_allocate_bytes(
                target_module_index,
                target_function_internal_index,
            );

    let type_item = &thread_context.module_common_instances[target_module_index]
        .type_section
        .items[type_index];

    let return_pc = ProgramCounter {
        // the length of instruction 'call' is 8 bytes (while 'dyncall' is 2 bytes).
        // so when the target function is finish, the next instruction should be the
        // instruction after the instruction 'call/dyncall'.
        instruction_address: return_instruction_address + instruction_length,
        function_internal_index: return_function_internal_index,
        module_index: return_module_index,
    };

    thread_context.stack.create_frame(
        type_item.params_count,
        type_item.results_count,
        local_list_index as u32,
        local_variables_allocate_bytes,
        Some(return_pc),
    );

    let target_pc = ProgramCounter {
        instruction_address: code_offset,
        function_internal_index: target_function_internal_index,
        module_index: target_module_index,
    };

    HandleResult::Jump(target_pc)
}

pub fn syscall(handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (operand args..., syscall_num:i32 params_count: i32) -> (return_value:i64, error_no:i32)
    //
    // the syscall arguments should be pushed on the stack first, e.g.
    //
    // | params_count   |
    // | syscall_num    |
    // | arg6           |
    // | arg5           |
    // | arg4           |
    // | arg3           |
    // | arg2           |                  | error no       |
    // | arg1           |     return -->   | return value   |
    // | ...            |                  | ...            |
    // \----------------/ <-- stack start  \----------------/
    //
    // when a syscall complete, the return value is store into the 'rax' register,
    // if the operation fails, the value is a negative value (rax < 0).
    // there is no 'errno' if invoke syscall by assembly directly.

    let params_count = thread_context.stack.pop_i32_u();
    let syscall_num = thread_context.stack.pop_i32_u();

    let syscall_handler = handler.syscall_handlers[params_count as usize];
    let result = syscall_handler(handler, thread_context, syscall_num as usize);

    // push the result on the stack

    match result {
        Ok(ret_value) => {
            thread_context.stack.push_i64_u(ret_value as u64);
            thread_context.stack.push_i32_u(0);
        }
        Err(error_no) => {
            thread_context.stack.push_i64_u(0);
            thread_context.stack.push_i32_u(error_no as u32);
        }
    }

    HandleResult::Move(2)
}

pub fn envcall(handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param envcall_num:i32)
    let envcall_num = thread_context.get_param_i32();
    let envcall_handler = handler.envcall_handlers[envcall_num as usize];
    envcall_handler(handler, thread_context);
    HandleResult::Move(8)
}

pub fn extcall(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (operand external_function_index:i32) -> void/i32/i64/f32/f64
    //
    // the 'external_function_index' is the index within a specific module, it is not
    // the 'unified_external_function_index'.

    let external_function_index = thread_context.get_param_i32() as usize;
    let module_index = thread_context.pc.module_index;

    let (external_function_pointer, wrapper_function, params_count, has_return_value) =
        if let Ok(pwr) =
            get_or_create_external_function(thread_context, module_index, external_function_index)
        {
            pwr
        } else {
            return HandleResult::Panic(PANIC_CODE_EXTERNAL_FUNCTION_CREATE_FAILURE);
        };

    // call the wrapper function:
    //
    // ```rust
    // type WrapperFunction = extern "C" fn(
    //     external_function_pointer: *const c_void,
    //     params_ptr: *const u8,
    //     results_ptr: *mut u8,
    // );
    // ```

    let params = thread_context.stack.pop_operands(params_count);
    let mut results = [0u8; OPERAND_SIZE_IN_BYTES];

    wrapper_function(
        external_function_pointer,
        params.as_ptr(),
        results.as_mut_ptr(),
    );

    // push the result on the stack
    if has_return_value {
        let dst = thread_context.stack.push_operand_from_memory();
        unsafe { std::ptr::copy(results.as_ptr(), dst, OPERAND_SIZE_IN_BYTES) };
    }

    HandleResult::Move(8)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anc_context::{environment::Environment, resource::Resource};
    use anc_image::{
        bytecode_reader::format_bytecode_as_text,
        bytecode_writer::BytecodeWriterHelper,
        entry::{InitedDataEntry, TypeEntry},
        utils::{
            helper_build_module_binary_with_functions_and_blocks,
            helper_build_module_binary_with_functions_and_external_functions,
            helper_build_module_binary_with_single_function,
            HelperBlockEntryWithSignatureAndLocalVariables, HelperExternalFunctionEntry,
            HelperFunctionEntryWithCodeAndLocalVariables,
            HelperFunctionEntryWithCodeAndSignatureAndLocalVariables,
        },
    };
    use anc_isa::{opcode::Opcode, ExternalLibraryDependentValue, ForeignValue, OperandDataType};
    use dyncall_util::cstr_pointer_to_str;
    use syscall_util::{errno::Errno, number::SysCallNum};

    use crate::{
        handler::Handler, in_memory_resource::InMemoryResource, process::process_function,
    };

    #[test]
    fn test_handler_function_call() {
        // fn $test (i32) -> (i32)
        //     (call $sum_square)
        // end
        //
        // fn $sum_square (count/1:i32) -> (i32)
        //     zero
        //     (local_load32 0 0)
        //     (block #0) (sum/0:i32, n/1:i32) -> (i32)
        //                                  ;; if n == 0
        //         (local_load32 0 1)
        //         eqz_i32
        //         (block_alt #1) () -> (i32)
        //             (local_load32 1 0)   ;; then sum
        //         (break_alt)              ;; else
        //                                  ;; sum + n^2
        //             (local_load32 1 0)
        //             (local_load32 1 1)
        //             (call $square)
        //             add_i32
        //                                  ;; n - 1
        //             (local_load32 1 1)
        //             (sub_imm_i32 1)
        //                                  ;; recur 1
        //             (recur 1)
        //         end
        //     end
        // end
        //
        // fn $square (i32) -> (i32)
        //     (local_load 32)
        //     (local_load 32)
        //     mul_i32
        // end

        // expect (5) -> 1 + 2^2 + 3^2 + 4^2 + 5^2 -> 1 + 4 + 9 + 16 + 25 -> 55

        let code_main = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::call, 1)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code_sum_square = BytecodeWriterHelper::new()
            // .append_opcode(Opcode::zero)
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 0)
            .append_opcode_i32_i32(Opcode::block, 3, 3)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1)
            .append_opcode(Opcode::eqz_i32)
            .append_opcode_i32_i32(Opcode::block_alt, 4, 0x1c)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 1, 0, 0)
            .append_opcode_i32(Opcode::break_alt, 0x3a)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 1, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 1, 0, 1)
            .append_opcode_i32(Opcode::call, 2)
            .append_opcode(Opcode::add_i32)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 1, 0, 1)
            .append_opcode_i16(Opcode::sub_imm_i32, 1)
            //
            .append_opcode_i16_i32(Opcode::recur, 1, 0x50)
            //
            .append_opcode(Opcode::end)
            .append_opcode(Opcode::end)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code_square = BytecodeWriterHelper::new()
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 0)
            .append_opcode(Opcode::mul_i32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_blocks(
            vec![
                HelperFunctionEntryWithCodeAndSignatureAndLocalVariables {
                    params: vec![OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_main,
                },
                HelperFunctionEntryWithCodeAndSignatureAndLocalVariables {
                    params: vec![OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_sum_square,
                },
                HelperFunctionEntryWithCodeAndSignatureAndLocalVariables {
                    params: vec![OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_square,
                },
            ],
            vec![
                HelperBlockEntryWithSignatureAndLocalVariables {
                    params: vec![OperandDataType::I32, OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntryWithSignatureAndLocalVariables {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
            ],
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &handler,
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(5)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(55),]);
    }

    #[test]
    fn test_handler_pub_index_function() {
        // fn $test () -> (i32, i32)
        //     (pub_index_function 1)
        //     (pub_index_function 2)
        // end
        //
        // fn $one (;1;) () -> (i32)
        //     (imm_i32 11)
        // end
        //
        // fn $two (;2;) () -> (i32)
        //     (imm_i32 13)
        // end

        // expect (1, 2)
        let code_main = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::pub_index_function, 1)
            .append_opcode_i32(Opcode::pub_index_function, 2)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code_one = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 11)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code_two = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 13)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_blocks(
            vec![
                HelperFunctionEntryWithCodeAndSignatureAndLocalVariables {
                    params: vec![],
                    results: vec![OperandDataType::I32, OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_main,
                },
                HelperFunctionEntryWithCodeAndSignatureAndLocalVariables {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_one,
                },
                HelperFunctionEntryWithCodeAndSignatureAndLocalVariables {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_two,
                },
            ],
            vec![],
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::U32(1), ForeignValue::U32(2),]
        );
    }

    #[test]
    fn test_handler_function_call_dyncall() {
        // fn $test () -> (i32, i32, i32, i32, i32)
        //     (imm_i32 2)
        //     (dyncall)
        //     (imm_i32 4)
        //     (dyncall)
        //     (imm_i32 3)
        //     (dyncall)
        //     (imm_i32 1)
        //     (dyncall)
        //     (imm_i32 2)
        //     (dyncall)
        // end
        //
        // fn $eleven (;1;) () -> (i32)
        //     (imm_i32 11)
        // end
        //
        // fn $thirteen (;2;) () -> (i32)
        //     (imm_i32 13)
        // end
        //
        // fn $seventeen (;3;) () -> (i32)
        //     (imm_i32 17)
        // end
        //
        // fn $nineteen (;4;) () -> (i32)
        //     (imm_i32 19)
        // end

        // expect (13, 19, 17, 11, 13)

        let code_main = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 2)
            .append_opcode(Opcode::dyncall)
            .append_opcode_i32(Opcode::imm_i32, 4)
            .append_opcode(Opcode::dyncall)
            .append_opcode_i32(Opcode::imm_i32, 3)
            .append_opcode(Opcode::dyncall)
            .append_opcode_i32(Opcode::imm_i32, 1)
            .append_opcode(Opcode::dyncall)
            .append_opcode_i32(Opcode::imm_i32, 2)
            .append_opcode(Opcode::dyncall)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code_eleven = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 11)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code_thirteen = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 13)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code_seventeen = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 17)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code_nineteen = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 19)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_blocks(
            vec![
                HelperFunctionEntryWithCodeAndSignatureAndLocalVariables {
                    params: vec![],
                    results: vec![
                        OperandDataType::I32,
                        OperandDataType::I32,
                        OperandDataType::I32,
                        OperandDataType::I32,
                        OperandDataType::I32,
                    ],
                    local_variable_item_entries_without_args: vec![],
                    code: code_main,
                },
                HelperFunctionEntryWithCodeAndSignatureAndLocalVariables {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_eleven,
                },
                HelperFunctionEntryWithCodeAndSignatureAndLocalVariables {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_thirteen,
                },
                HelperFunctionEntryWithCodeAndSignatureAndLocalVariables {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_seventeen,
                },
                HelperFunctionEntryWithCodeAndSignatureAndLocalVariables {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_nineteen,
                },
            ],
            vec![],
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U32(13),
                ForeignValue::U32(19),
                ForeignValue::U32(17),
                ForeignValue::U32(11),
                ForeignValue::U32(13),
            ]
        );
    }

    #[test]
    fn test_handler_syscall_without_args() {
        // fn $test () -> (result:i64 errno:i32)

        // syscall:
        // `pid_t getpid(void);`

        let code0 = BytecodeWriterHelper::new()
            // push syscall args from 1 to 6
            // -none-
            // prepare syscall
            .append_opcode_i32(Opcode::imm_i32, SysCallNum::getpid as u32) // syscall num
            .append_opcode_i32(Opcode::imm_i32, 0) // the amount of syscall args
            //
            .append_opcode(Opcode::syscall)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        println!("{}", format_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                                           // params
            vec![OperandDataType::I64, OperandDataType::I32], // results
            vec![],                                           // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        let result_values0 = result0.unwrap();

        let pid = std::process::id();

        assert!(matches!(result_values0[0], ForeignValue::U64(value) if value == pid as u64));
        assert_eq!(result_values0[1], ForeignValue::U32(0));
    }

    #[test]
    fn test_handler_syscall_with_2_args() {
        // fn $test (buf_addr:i64, buf_len:i32) -> (result:i64 errno:i32)

        // syscall:
        // `char *getcwd(char buf[.size], size_t size);`

        let code0 = BytecodeWriterHelper::new()
            // push syscall args from 1 to 6
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1)
            // prepare syscall
            .append_opcode_i32(Opcode::imm_i32, SysCallNum::getcwd as u32) // syscall num
            .append_opcode_i32(Opcode::imm_i32, 2) // the amount of syscall args
            //
            .append_opcode(Opcode::syscall)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        println!("{}", format_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function(
            vec![OperandDataType::I64, OperandDataType::I64], // params
            vec![OperandDataType::I64, OperandDataType::I32], // results
            vec![],                                           // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        const BUF_LENGTH: u32 = 1024;
        let buf = [0u8; BUF_LENGTH as usize];
        let buf_addr = buf.as_ptr() as u64;

        let result0 = process_function(
            &handler,
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U64(buf_addr), ForeignValue::U32(BUF_LENGTH)],
        );

        let results0 = result0.unwrap();

        // note
        //
        // the syscall 'getcwd' in the libc returns the pointer to the buf, but the
        // raw syscall 'getcwd' returns the length of the path (includes the NULL terminated char)

        let null_pos = buf.iter().position(|u| *u == 0).unwrap();

        assert!(matches!(results0[0], ForeignValue::U64(value) if value == (null_pos + 1) as u64));
        assert_eq!(results0[1], ForeignValue::U32(0));

        let path0 = String::from_utf8_lossy(&buf[0..null_pos]);
        let cwd = std::env::current_dir().unwrap();
        let cwd0 = cwd.as_os_str().to_string_lossy();
        assert_eq!(path0, cwd0);
    }

    #[test]
    fn test_handler_syscall_error_no() {
        // fn $test (file_path_buf_addr:i64) -> (result:i64 errno:i32)

        // syscall:
        // `int open(const char *pathname, int flags)`

        let code0 = BytecodeWriterHelper::new()
            // push syscall args from 1 to 6
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0) // file path addr
            .append_opcode_i32(Opcode::imm_i32, 0) // open flags
            // prepare syscall
            .append_opcode_i32(Opcode::imm_i32, SysCallNum::open as u32) // syscall num
            .append_opcode_i32(Opcode::imm_i32, 2) // the amount of syscall args
            //
            .append_opcode(Opcode::syscall)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        println!("{}", format_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function(
            vec![OperandDataType::I64],                       // params
            vec![OperandDataType::I64, OperandDataType::I32], // results
            vec![],                                           // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let file_path0 = b"/this/file/should/not/exist\0";
        let file_path1 = b"/dev/zero\0";

        let file_path_addr0 = file_path0.as_ptr() as usize;
        let file_path_addr1 = file_path1.as_ptr() as usize;

        let result0 = process_function(
            &handler,
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U64(file_path_addr0 as u64)],
        );
        let results0 = result0.unwrap();

        assert_eq!(
            results0,
            vec![
                ForeignValue::U64(0),
                ForeignValue::U32(Errno::ENOENT as u32)
            ]
        );

        let result1 = process_function(
            &handler,
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U64(file_path_addr1 as u64)],
        );
        let results1 = result1.unwrap();

        assert!(matches!(results1[0], ForeignValue::U64(value) if value > 0));
        assert_eq!(results1[1], ForeignValue::U32(0));
    }

    #[test]
    fn test_handler_extcall_with_system_libc_getuid() {
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::extcall, 0) // 0 is the external function index
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        // `man 3 getuid`
        // 'uid_t getuid(void);'

        let binary0 = helper_build_module_binary_with_functions_and_external_functions(
            vec![
                TypeEntry {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                }, // getuid
                TypeEntry {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                }, // main
            ], // types
            vec![HelperFunctionEntryWithCodeAndLocalVariables {
                type_index: 1,
                local_variable_item_entries_without_args: vec![],
                code: code0,
            }],
            vec![],
            vec![],
            vec![],
            vec![HelperExternalFunctionEntry {
                // external_library_dependent_type: ExternalLibraryDependentType::System,
                library_name: "libc.so.6".to_string(),
                library_value: Box::new(ExternalLibraryDependentValue::System(
                    "libc.so.6".to_owned(),
                )),
                function_name: "getuid".to_string(),
                type_index: 0,
            }],
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        let results0 = result0.unwrap();

        assert!(matches!(results0[0], ForeignValue::U32(uid) if uid > 0 ));
    }

    #[test]
    fn test_handler_extcall_with_system_libc_getenv() {
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 0) // external function param 0
            //
            .append_opcode_i32(Opcode::extcall, 0) // 0 is the external function index
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        // `man 3 getenv`
        // 'char *getenv(const char *name);'

        let binary0 = helper_build_module_binary_with_functions_and_external_functions(
            vec![
                TypeEntry {
                    params: vec![OperandDataType::I64],  // pointer
                    results: vec![OperandDataType::I64], // pointer
                }, // getenv
                TypeEntry {
                    params: vec![],
                    results: vec![OperandDataType::I64], // pointer
                }, // main
            ], // types
            vec![HelperFunctionEntryWithCodeAndLocalVariables {
                type_index: 1,
                local_variable_item_entries_without_args: vec![],
                code: code0,
            }],
            vec![InitedDataEntry::from_raw(b"PWD\0".to_vec(), 1)],
            vec![],
            vec![],
            vec![HelperExternalFunctionEntry {
                // external_library_dependent_type: ExternalLibraryDependentType::System,
                library_name: "libc.so.6".to_string(),
                library_value: Box::new(ExternalLibraryDependentValue::System(
                    "libc.so.6".to_owned(),
                )),
                function_name: "getenv".to_string(),
                type_index: 0,
            }],
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        let results0 = result0.unwrap();

        assert!(matches!(results0[0], ForeignValue::U64(addr) if {
            let pwd0 = cstr_pointer_to_str(addr as *const i8);
            !pwd0.to_string().is_empty()
        }));
    }

    #[test]
    fn test_handler_extcall_with_user_lib() {
        // (i32,i32) -> (i32)

        // 'libtest0.so.1'
        // 'int add(int, int)'

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 0) // external function param 0
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1) // external function param 1
            //
            .append_opcode_i32(Opcode::extcall, 0) // 0 is the external function index
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_external_functions(
            vec![
                TypeEntry {
                    params: vec![OperandDataType::I32, OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                }, // getenv
                TypeEntry {
                    params: vec![OperandDataType::I32, OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                }, // main
            ], // types
            vec![HelperFunctionEntryWithCodeAndLocalVariables {
                type_index: 1,
                local_variable_item_entries_without_args: vec![],
                code: code0,
            }],
            vec![],
            vec![],
            vec![],
            vec![HelperExternalFunctionEntry {
                // external_library_dependent_type: ExternalLibraryDependentType::Local,
                library_name: "libtest0.so.1".to_string(),
                library_value: Box::new(ExternalLibraryDependentValue::Local(
                    "lib/libtest0.so.1".to_owned(),
                )),
                function_name: "add".to_string(),
                type_index: 0,
            }],
        );

        // it can not obtain the name of crate with the macro cfg, it
        // only supports several options:
        // https://doc.rust-lang.org/reference/conditional-compilation.html
        // https://doc.rust-lang.org/reference/attributes.html
        // https://doc.rust-lang.org/cargo/reference/environment-variables.html

        let mut pwd = std::env::current_dir().unwrap();
        // let pkg_name = env!("CARGO_PKG_NAME");
        let crate_folder_name = "processor";
        if !pwd.ends_with(crate_folder_name) {
            // in the VSCode editor `Debug` environment, the `current_dir()` returns
            // the project's root folder.
            // while in both `$ cargo test` and VSCode editor `Run Test` environment,
            // the `current_dir()` returns the current crate path.
            // here canonicalize the unit test resources path.
            pwd.push("crates");
            pwd.push(crate_folder_name);
        }
        pwd.push("tests");
        let application_path = pwd.to_str().unwrap();

        let handler = Handler::new();
        let resource0 = InMemoryResource::with_environment(
            vec![binary0],
            &Environment::new(
                application_path,
                true,
                "",
                &[""],
                "",
                HashMap::<String, String>::new(),
            ),
        );
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &handler,
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(11), ForeignValue::U32(13)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(24)]);

        let result1 = process_function(
            &handler,
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(211), ForeignValue::U32(223)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::U32(434)]);
    }
}
