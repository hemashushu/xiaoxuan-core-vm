// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::thread_context::{ProgramCounter, ThreadContext};

use super::InterpretResult;

pub fn call(thread_context: &mut ThreadContext) -> InterpretResult {
    let function_public_index = thread_context.get_param_i32();
    do_call(thread_context, function_public_index, 8)
}

pub fn dyncall(thread_context: &mut ThreadContext) -> InterpretResult {
    let function_public_index = thread_context.stack.pop_i32_u();
    do_call(thread_context, function_public_index, 2)
}

fn do_call(
    thread_context: &mut ThreadContext,
    function_public_index: u32,
    instruction_length: usize,
) -> InterpretResult {
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

    let type_item = &thread_context.module_instances[target_module_index]
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

    InterpretResult::Jump(target_pc)
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        bytecode_writer::BytecodeWriter,
        utils::{
            helper_build_module_binary_with_functions_and_blocks, HelperBlockSignatureAndLocalVariablesEntry,
            HelperFunctionWithCodeAndSignatureAndLocalVariablesEntry,
        },
    };

    use crate::{in_memory_program_resource::InMemoryProgramResource, interpreter::process_function};
    use ancvm_context::program_resource::ProgramResource;
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    #[test]
    fn test_interpreter_function_call() {
        // fn $test (i32) -> (i32)
        //     (call $sum_square)
        // end
        //
        // fn $sum_square (count/1:i32) -> (i32)
        //     zero
        //     (local_load32 0 0)
        //     (block 3 3) (sum/0:i32, n/1:i32) -> (i32)
        //                                  ;; if n == 0
        //         (local_load32 0 1)
        //         i32_eqz
        //         (block_alt 4 4) () -> (i32)
        //             (local_load32 1 0)   ;; then sum
        //             (break 0)            ;; else
        //                                  ;; sum + n^2
        //             (local_load32 1 0)
        //             (local_load32 1 1)
        //             (call $square)
        //             i32_add
        //                                  ;; n - 1
        //             (local_load32 1 1)
        //             (i32_dec 1)
        //                                  ;; recur 1
        //             (recur 1)
        //         end
        //     end
        // end
        //
        // fn $square (i32) -> (i32)
        //     (local_load 32)
        //     (local_load 32)
        //     i32_mul
        // end

        // expect (5) -> 1 + 2^2 + 3^2 + 4^2 + 5^2 -> 1 + 4 + 9 + 16 + 25 -> 55

        let code_main = BytecodeWriter::new()
            .append_opcode_i32(Opcode::call, 1)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code_sum_square = BytecodeWriter::new()
            .append_opcode(Opcode::zero)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i32_i32(Opcode::block, 3, 3)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_eqz)
            .append_opcode_i32_i32_i32(Opcode::block_alt, 4, 4, 0x20)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 0)
            .append_opcode_i16_i32(Opcode::break_, 0, 0x3a)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 1)
            .append_opcode_i32(Opcode::call, 2)
            .append_opcode(Opcode::i32_add)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 1)
            .append_opcode_i16(Opcode::i32_dec, 1)
            //
            .append_opcode_i16_i32(Opcode::recur, 1, 0x54)
            //
            .append_opcode(Opcode::end)
            .append_opcode(Opcode::end)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code_square = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode(Opcode::i32_mul)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_blocks(
            vec![
                HelperFunctionWithCodeAndSignatureAndLocalVariablesEntry {
                    params: vec![DataType::I32],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_main,
                },
                HelperFunctionWithCodeAndSignatureAndLocalVariablesEntry {
                    params: vec![DataType::I32],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_sum_square,
                },
                HelperFunctionWithCodeAndSignatureAndLocalVariablesEntry {
                    params: vec![DataType::I32],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_square,
                },
            ],
            vec![
                HelperBlockSignatureAndLocalVariablesEntry {
                    params: vec![DataType::I32, DataType::I32],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockSignatureAndLocalVariablesEntry {
                    params: vec![],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
            ],
        );

        let program_resource0 = InMemoryProgramResource::new(vec![binary0]);
        let process_context0 = program_resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(5)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(55),]);
    }

    #[test]
    fn test_interpreter_function_call_dyncall() {
        // fn $test () -> (i32, i32, i32, i32, i32)
        //     (i32_imm 2)
        //     (dyncall)
        //     (i32_imm 4)
        //     (dyncall)
        //     (i32_imm 3)
        //     (dyncall)
        //     (i32_imm 1)
        //     (dyncall)
        //     (i32_imm 2)
        //     (dyncall)
        // end
        //
        // fn $eleven (;1;) () -> (i32)
        //     (i32_imm 11)
        // end
        //
        // fn $thirteen (;2;) () -> (i32)
        //     (i32_imm 13)
        // end
        //
        // fn $seventeen (;3;) () -> (i32)
        //     (i32_imm 17)
        // end
        //
        // fn $nineteen (;4;) () -> (i32)
        //     (i32_imm 19)
        // end

        // expect (13, 19, 17, 11, 13)

        let code_main = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 2)
            .append_opcode(Opcode::dyncall)
            .append_opcode_i32(Opcode::i32_imm, 4)
            .append_opcode(Opcode::dyncall)
            .append_opcode_i32(Opcode::i32_imm, 3)
            .append_opcode(Opcode::dyncall)
            .append_opcode_i32(Opcode::i32_imm, 1)
            .append_opcode(Opcode::dyncall)
            .append_opcode_i32(Opcode::i32_imm, 2)
            .append_opcode(Opcode::dyncall)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code_eleven = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 11)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code_thirteen = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 13)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code_seventeen = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 17)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code_nineteen = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 19)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_blocks(
            vec![
                HelperFunctionWithCodeAndSignatureAndLocalVariablesEntry {
                    params: vec![],
                    results: vec![
                        DataType::I32,
                        DataType::I32,
                        DataType::I32,
                        DataType::I32,
                        DataType::I32,
                    ],
                    local_variable_item_entries_without_args: vec![],
                    code: code_main,
                },
                HelperFunctionWithCodeAndSignatureAndLocalVariablesEntry {
                    params: vec![],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_eleven,
                },
                HelperFunctionWithCodeAndSignatureAndLocalVariablesEntry {
                    params: vec![],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_thirteen,
                },
                HelperFunctionWithCodeAndSignatureAndLocalVariablesEntry {
                    params: vec![],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_seventeen,
                },
                HelperFunctionWithCodeAndSignatureAndLocalVariablesEntry {
                    params: vec![],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_nineteen,
                },
            ],
            vec![],
        );

        let program_resource0 = InMemoryProgramResource::new(vec![binary0]);
        let process_context0 = program_resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
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
}
