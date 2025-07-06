// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;
use anc_isa::OPERAND_SIZE_IN_BYTES;
use anc_stack::ProgramCounter;

use crate::{
    envcall_handler::get_envcall_handlers,
    extcall_handler::get_or_create_external_function_wrapper_function,
    syscall_handler::get_syscall_handler, TERMINATE_CODE_FAILED_TO_LOAD_EXTERNAL_FUNCTION,
    TERMINATE_CODE_STACK_OVERFLOW,
};

use super::HandleResult;

pub fn call(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param function_public_index:i32) (operand args...) -> (values)
    let function_public_index = thread_context.get_param_i32();
    do_call(
        thread_context,
        thread_context.pc.module_index,
        function_public_index,
        8,
    )
}

pub fn call_dynamic(/* _handler: &Handler, */ thread_context: &mut ThreadContext,) -> HandleResult {
    // () (operand args... function_module_index:i32 function_public_index:i32) -> (values)
    let function_public_index = thread_context.stack.pop_i32_u();
    let module_index = thread_context.stack.pop_i32_u() as usize;
    do_call(thread_context, module_index, function_public_index, 2)
}

fn do_call(
    thread_context: &mut ThreadContext,
    module_index: usize,
    function_public_index: u32,
    instruction_length_in_bytes: usize,
) -> HandleResult {
    let ProgramCounter {
        instruction_address: return_instruction_address,
        function_internal_index: return_function_internal_index,
        module_index: return_module_index,
    } = thread_context.pc;

    let target_function_object =
        thread_context.get_target_function_object(module_index, function_public_index as usize);
    let function_info = thread_context.get_function_info(
        target_function_object.module_index,
        target_function_object.function_internal_index,
    );

    let type_item = &thread_context.module_common_instances[target_function_object.module_index]
        .type_section
        .items[function_info.type_index];

    let return_pc = ProgramCounter {
        // the length of instruction 'call' is 8 bytes (while 'dyncall' is 2 bytes).
        // so when the target function is finish, the next instruction should be the
        // instruction after the instruction 'call/dyncall'.
        instruction_address: return_instruction_address + instruction_length_in_bytes,
        function_internal_index: return_function_internal_index,
        module_index: return_module_index,
    };

    match thread_context.stack.create_frame(
        type_item.params_count,
        type_item.results_count,
        function_info.local_variable_list_index as u32,
        function_info.local_variables_with_arguments_allocated_bytes as u32,
        Some(return_pc),
    ) {
        Ok(_) => {
            let target_pc = ProgramCounter {
                instruction_address: function_info.code_offset,
                function_internal_index: target_function_object.function_internal_index,
                module_index: target_function_object.module_index,
            };

            HandleResult::Jump(target_pc)
        }
        Err(_) => HandleResult::Terminate(TERMINATE_CODE_STACK_OVERFLOW),
    }
}

pub fn syscall(/* handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand args... params_count:i32 syscall_num:i32) -> (return_value:i64 error_number:i32)
    //
    // The "syscall" instruction invokes a system call on Unix-like operating systems.
    //
    // The syscall arguments must be pushed onto the stack first, followed by the syscall number, and the number of parameters.
    //
    // For example:
    //
    // | params_count   | <-- stack end
    // | syscall_num    |
    // | arg6           |
    // | arg5           |
    // | arg4           |
    // | arg3           |
    // | arg2           |                     | error number   |
    // | arg1           |    -- returns -->   | return value   |
    // | ...            |                     | ...            |
    // \----------------/ <-- stack start --> \----------------/
    //
    // When the syscall completes, the return value is stored in the "rax" register. If the operation fails,
    // the value is negative (i.e., rax < 0).
    //
    // Note: Unlike the C standard library, there is no "errno" when calling syscalls directly from assembly.

    let syscall_num = thread_context.stack.pop_i32_u();
    let params_count = thread_context.stack.pop_i32_u();

    let function = get_syscall_handler(params_count as usize);
    let result = function(thread_context, syscall_num as usize);

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

pub fn envcall(/* handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param envcall_num:i32) (operand args...) -> (values)

    let envcall_num = thread_context.get_param_i32();
    let function = get_envcall_handlers(envcall_num);
    function(thread_context);
    HandleResult::Move(8)
}

pub fn extcall(/* handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param external_function_index:i32) (operand args...) -> return_value:void/i32/i64/f32/f64
    //
    // note that the `external_function_index` is the index within a specific module,
    // it is NOT the `unified_external_function_index`.

    let external_function_index = thread_context.get_param_i32() as usize;
    let module_index = thread_context.pc.module_index;

    let (external_function_pointer, wrapper_function, params_count, contains_return_value) =
        if let Ok(pwr) = get_or_create_external_function_wrapper_function(
            // handler,
            thread_context,
            module_index,
            external_function_index,
        ) {
            pwr
        } else {
            return HandleResult::Terminate(TERMINATE_CODE_FAILED_TO_LOAD_EXTERNAL_FUNCTION);
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

    let params_ptr = thread_context.stack.pop_operands_to_memory(params_count);
    let mut results = [0u8; OPERAND_SIZE_IN_BYTES];

    wrapper_function(external_function_pointer, params_ptr, results.as_mut_ptr());

    // push the result on the stack
    if contains_return_value {
        let dst = thread_context.stack.push_operand_from_memory();
        unsafe { std::ptr::copy(results.as_ptr(), dst, OPERAND_SIZE_IN_BYTES) };
    }

    HandleResult::Move(8)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anc_context::{
        process_property::{ProcessProperty, ProgramSourceType},
        program_source::ProgramSource,
    };
    use anc_image::{
        bytecode_writer::BytecodeWriterHelper,
        entry::{ExternalLibraryEntry, ReadOnlyDataEntry},
        utils::{
            helper_build_module_binary_with_functions_and_blocks,
            helper_build_module_binary_with_functions_and_data_and_external_functions,
            helper_build_module_binary_with_single_function, HelperBlockEntry,
            HelperExternalFunctionEntry, HelperFunctionEntry,
        },
    };
    use anc_isa::{opcode::Opcode, ExternalLibraryDependency, ForeignValue, OperandDataType};
    use dyncall_util::cstr_pointer_to_str;
    use syscall_util::{errno::Errno, number::SysCallNum};

    use crate::{in_memory_program_source::InMemoryProgramSource, process::process_function};

    #[test]
    fn test_handler_function_call() {
        // pesudo code:
        //
        // fn test (num/0:i32) -> (i32)             ;; type 0
        //     call(sum_square)                     ;; call (sum_square, n)
        // end
        //
        // fn sum_square (count/0:i32) -> (i32)     ;; type 1
        //     imm_i32(0)                           ;;
        //     local_load32(0, 0)                   ;;
        //     block (sum/0:i32, n/1:i32) -> (i32)  ;; type 3, let (sum,n) = (0,count)
        //         local_load32(0, 1)               ;;
        //         eqz_i32                          ;;
        //         block_alt () -> (i32)            ;; type 4, if n == 0 then
        //             local_load32(1, 0)           ;; sum
        //         break_alt()                      ;; else
        //             local_load32(1, 1)           ;;
        //             call(square)                 ;;
        //             local_load32(1, 0)           ;;
        //             add_i32                      ;; call(square, n) + sum
        //             local_load32(1, 1)           ;;
        //             sub_imm_i32(1)               ;; n - 1
        //             recur(1)                     ;; recur 1
        //         end                              ;; end if
        //     end
        // end
        //
        // fn square (num/0:i32) -> (i32)           ;; type 2
        //     local_load_i32s(0, 0)                ;;
        //     local_load_i32s(0, 0)                ;;
        //     mul_i32()                            ;; n * n
        // end
        //
        // expect:
        // arg: 5
        // returns: 55 (= 1 + 2^2 + 3^2 + 4^2 + 5^2)

        let code_main = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::call, 1)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code_sum_square = BytecodeWriterHelper::new()
            // let (sum,n) = (0,count)
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i32_i32(Opcode::block, 3, 3)
            // if n == 0
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode(Opcode::eqz_i32)
            .append_opcode_i32_i32_i32(Opcode::block_alt, 4, 4, 0x20)
            // then sum
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            // else
            .append_opcode_i32(Opcode::break_alt, 0x3a)
            // call(square, n)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 1)
            .append_opcode_i32(Opcode::call, 2)
            // + sum
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode(Opcode::add_i32)
            // n - 1
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 1)
            .append_opcode_i16(Opcode::sub_imm_i32, 1)
            // recur
            .append_opcode_i16_i32(Opcode::recur, 1, 0x54)
            // end if
            .append_opcode(Opcode::end)
            .append_opcode(Opcode::end)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code_square = BytecodeWriterHelper::new()
            // n * n
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode(Opcode::mul_i32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_blocks(
            // the binary building helper does not support merge types,
            // each function requires its own type item.
            &[
                HelperFunctionEntry {
                    params: vec![OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_main,
                },
                HelperFunctionEntry {
                    params: vec![OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_sum_square,
                },
                HelperFunctionEntry {
                    params: vec![OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_square,
                },
            ],
            // the binary building helper does not support merge types,
            // each block requires its own type item.
            &[
                HelperBlockEntry {
                    params: vec![OperandDataType::I32, OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
            ],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            /* &handler, */
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(5)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(55),]);
    }

    #[test]
    fn test_handler_function_call_dynamic() {
        // pesudo code:
        //
        // fn test () -> (i32, i32, i32, i32, i32)  ;; function public idx: 0
        //     get_function(thirteen)
        //     call_dynamic()
        //     get_function(nineteen)
        //     call_dynamic()
        //     get_function(seventeen)
        //     call_dynamic()
        //     get_function(eleven)
        //     call_dynamic()
        //     get_function(thirteen)
        //     call_dynamic()
        // end
        //
        // fn eleven () -> (i32)        ;; function public idx: 1
        //     imm_i32(11)
        // end
        //
        // fn thirteen () -> (i32)      ;; function public idx: 2
        //     imm_i32(13)
        // end
        //
        // fn seventeen () -> (i32)     ;; function public idx: 3
        //     imm_i32(17)
        // end
        //
        // fn nineteen () -> (i32)      ;; function public idx: 4
        //     imm_i32(19)
        // end
        //
        // expect:
        // args: ()
        // returns: (13, 19, 17, 11, 13)

        let code_main = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::get_function, 2)
            .append_opcode(Opcode::call_dynamic)
            .append_opcode_i32(Opcode::get_function, 4)
            .append_opcode(Opcode::call_dynamic)
            .append_opcode_i32(Opcode::get_function, 3)
            .append_opcode(Opcode::call_dynamic)
            .append_opcode_i32(Opcode::get_function, 1)
            .append_opcode(Opcode::call_dynamic)
            .append_opcode_i32(Opcode::get_function, 2)
            .append_opcode(Opcode::call_dynamic)
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
            &[
                HelperFunctionEntry {
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
                HelperFunctionEntry {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_eleven,
                },
                HelperFunctionEntry {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_thirteen,
                },
                HelperFunctionEntry {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_seventeen,
                },
                HelperFunctionEntry {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_nineteen,
                },
            ],
            &[],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(/* &handler, */ &mut thread_context0, 0, 0, &[]);
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
        // pesudo code:
        //
        // fn test () -> (result:i64 errno:i32)
        // syscall getpid()

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 0) // the amount of syscall args
            .append_opcode_i32(Opcode::imm_i32, SysCallNum::getpid as u32) // syscall num
            //
            .append_opcode(Opcode::syscall)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[],                                           // params
            &[OperandDataType::I64, OperandDataType::I32], // results
            &[],                                           // local variables
            code0,
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(/* &handler, */ &mut thread_context0, 0, 0, &[]);
        let result_values0 = result0.unwrap();

        let pid = std::process::id();

        assert!(matches!(result_values0[0], ForeignValue::U64(value) if value == pid as u64));
        assert_eq!(result_values0[1], ForeignValue::U32(0));
    }

    #[test]
    fn test_handler_syscall_with_2_args() {
        // pesudo code:
        //
        // fn test (buf_addr:i64, buf_len:i32) -> (result:i64 errno:i32)
        // syscall getcwd(buf_addr, buf_len)

        let code0 = BytecodeWriterHelper::new()
            // syscall args from 1 to 6
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            // "number of args" and "syscall number"
            .append_opcode_i32(Opcode::imm_i32, 2) // the amount of syscall args
            .append_opcode_i32(Opcode::imm_i32, SysCallNum::getcwd as u32) // syscall num
            //
            .append_opcode(Opcode::syscall)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[OperandDataType::I64, OperandDataType::I64], // params
            &[OperandDataType::I64, OperandDataType::I32], // results
            &[],                                           // local variables
            code0,
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        const BUF_LENGTH: u32 = 1024;
        let buf = [0u8; BUF_LENGTH as usize];
        let buf_addr = buf.as_ptr() as u64;

        let result0 = process_function(
            /* &handler, */
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U64(buf_addr), ForeignValue::U32(BUF_LENGTH)],
        );

        let results0 = result0.unwrap();

        // note
        //
        // the function 'getcwd' in the libc returns the pointer to the buf, but the
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
    fn test_handler_syscall_error_number() {
        // pesudo code:
        //
        // fn test (file_path_buf_addr:i64) -> (result:i64 errno:i32)
        // syscall open(file_path_buf_addr, flags)

        let code0 = BytecodeWriterHelper::new()
            // args from 1 to 6
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // file path addr
            .append_opcode_i32(Opcode::imm_i32, 0) // open flags
            // "number of args" and "syscall number"
            .append_opcode_i32(Opcode::imm_i32, 2) // the amount of syscall args
            .append_opcode_i32(Opcode::imm_i32, SysCallNum::open as u32) // syscall num
            //
            .append_opcode(Opcode::syscall)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[OperandDataType::I64],                       // params
            &[OperandDataType::I64, OperandDataType::I32], // results
            &[],                                           // local variables
            code0,
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let file_path0 = b"/this/file/should/not/exist\0";
        let file_path1 = b"/dev/zero\0";

        let file_path_addr0 = file_path0.as_ptr() as usize;
        let file_path_addr1 = file_path1.as_ptr() as usize;

        let result0 = process_function(
            /* &handler, */
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
            /* &handler, */
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

        // ref: `man 3 getuid`
        // signature: `uid_t getuid(void);`

        let binary0 = helper_build_module_binary_with_functions_and_data_and_external_functions(
            &[HelperFunctionEntry {
                params: vec![],
                results: vec![OperandDataType::I32],
                local_variable_item_entries_without_args: vec![],
                code: code0,
            }],
            &[],
            &[],
            &[],
            &[ExternalLibraryEntry::new(
                "libc".to_owned(),
                Box::new(ExternalLibraryDependency::File(
                    "system:libc.so.6".to_owned(),
                )),
            )],
            &[HelperExternalFunctionEntry {
                name: "getuid".to_string(),
                params: vec![],
                result: Some(OperandDataType::I32),
                external_library_index: 0,
            }],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(/* &handler, */ &mut thread_context0, 0, 0, &[]);

        assert!(result0.is_ok());
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

        // ref: `man 3 getenv`
        // signature: `char *getenv(const char *name);`

        let binary0 = helper_build_module_binary_with_functions_and_data_and_external_functions(
            &[HelperFunctionEntry {
                params: vec![],
                results: vec![OperandDataType::I64], // pointer
                local_variable_item_entries_without_args: vec![],
                code: code0,
            }],
            &[ReadOnlyDataEntry::from_bytes(b"PWD\0".to_vec(), 1)],
            &[],
            &[],
            &[ExternalLibraryEntry::new(
                "libc".to_owned(),
                Box::new(ExternalLibraryDependency::File(
                    "system:libc.so.6".to_owned(),
                )),
            )],
            &[HelperExternalFunctionEntry {
                external_library_index: 0,
                name: "getenv".to_string(),
                params: vec![OperandDataType::I64], // pointer
                result: Some(OperandDataType::I64), // pointer
            }],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(/* &handler, */ &mut thread_context0, 0, 0, &[]);
        let results0 = result0.unwrap();

        assert!(matches!(results0[0], ForeignValue::U64(addr) if {
            let pwd0 = cstr_pointer_to_str(addr as *const i8);
            !pwd0.to_string().is_empty()
        }));
    }

    #[test]
    fn test_handler_extcall_with_user_lib_libtest() {
        // pesudo code:
        //
        // import fn add (int,int) -> int from "libtest.so.1"
        // fn add (left:i32, right:i32) -> (i32)
        // extcall add(left, right)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0) // external function param 0
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1) // external function param 1
            //
            .append_opcode_i32(Opcode::extcall, 0) // 0 is the external function index
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_data_and_external_functions(
            &[HelperFunctionEntry {
                params: vec![OperandDataType::I32, OperandDataType::I32],
                results: vec![OperandDataType::I32],
                local_variable_item_entries_without_args: vec![],
                code: code0,
            }],
            &[],
            &[],
            &[],
            &[ExternalLibraryEntry::new(
                "libtest".to_owned(),
                Box::new(ExternalLibraryDependency::File(
                    "tests/resources/libtest/libtest.so.1".to_owned(),
                )),
            )],
            &[HelperExternalFunctionEntry {
                params: vec![OperandDataType::I32, OperandDataType::I32],
                result: Some(OperandDataType::I32),
                name: "add".to_string(),
                external_library_index: 0,
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

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::with_property(
            vec![binary0],
            ProcessProperty::new(
                pwd,
                ProgramSourceType::Module,
                vec![],
                HashMap::<String, String>::new(),
            ),
        );
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(11), ForeignValue::U32(13)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(24)]);

        let result1 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(211), ForeignValue::U32(223)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::U32(434)]);
    }
}
