// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_program::thread_context::{ProgramCounter, ThreadContext};

use super::InterpretResult;

pub fn call(thread_context: &mut ThreadContext) -> InterpretResult {
    let function_public_index = thread_context.get_param_i32();
    do_call(thread_context, function_public_index, 8)
}

pub fn dcall(thread_context: &mut ThreadContext) -> InterpretResult {
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

    let type_item = &thread_context.program_context.program_modules[target_module_index]
        .type_section
        .items[type_index];

    let return_pc = ProgramCounter {
        // the length of instruction 'call' is 8 bytes (while 'dcall' is 2 bytes).
        // so when the target function is finish, the next instruction should be the
        // instruction after the instruction 'call/dcall'.
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
    use ancvm_binary::utils::{
        build_module_binary_with_functions_and_blocks, BytecodeWriter, HelperBlockEntry,
        HelperFunctionEntry,
    };

    use crate::{in_memory_program_source::InMemoryProgramSource, interpreter::process_function};
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    #[test]
    fn test_process_function_call() {
        // function $main (i32) -> (i32)
        //     (call $sum_square)
        // end
        //
        // function $sum_square (n/1:i32) -> (i32)
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
        // function $square (i32) -> (i32)
        //     (local_load 32)
        //     (local_load 32)
        //     i32_mul
        // end

        // expect (5) -> 1 + 2^2 + 3^2 + 4^2 + 5^2 -> 1 + 4 + 9 + 16 + 25 -> 55

        let code_main = BytecodeWriter::new()
            .write_opcode_i32(Opcode::call, 1)
            .write_opcode(Opcode::end)
            .to_bytes();

        let code_sum_square = BytecodeWriter::new()
            .write_opcode(Opcode::zero)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode_i32_i32(Opcode::block, 3, 3)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_eqz)
            .write_opcode_i32_i32_i32(Opcode::block_alt, 4, 4, 0x20)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32, 1, 0, 0)
            .write_opcode_i16_i32(Opcode::break_, 0, 0x3a)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32, 1, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 1, 0, 1)
            .write_opcode_i32(Opcode::call, 2)
            .write_opcode(Opcode::i32_add)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32, 1, 0, 1)
            .write_opcode_i16(Opcode::i32_dec, 1)
            //
            .write_opcode_i16_i32(Opcode::recur, 1, 0x54)
            //
            .write_opcode(Opcode::end)
            .write_opcode(Opcode::end)
            .write_opcode(Opcode::end)
            .to_bytes();

        let code_square = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode(Opcode::i32_mul)
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}\n", BytecodeReader::new(&code_main).to_text());
        // println!("{}\n", BytecodeReader::new(&code_sum_square).to_text());
        // println!("{}\n", BytecodeReader::new(&code_square).to_text());

        let binary0 = build_module_binary_with_functions_and_blocks(
            vec![
                HelperFunctionEntry {
                    params: vec![DataType::I32],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_main,
                },
                HelperFunctionEntry {
                    params: vec![DataType::I32],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_sum_square,
                },
                HelperFunctionEntry {
                    params: vec![DataType::I32],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_square,
                },
            ],
            vec![
                HelperBlockEntry {
                    params: vec![DataType::I32, DataType::I32],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
            ],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(5)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(55),]);
    }

    #[test]
    fn test_process_function_call_dcall() {
        // function $main () -> (i32, i32, i32, i32, i32)
        //     (i32_imm 2)
        //     (dcall)
        //     (i32_imm 4)
        //     (dcall)
        //     (i32_imm 3)
        //     (dcall)
        //     (i32_imm 1)
        //     (dcall)
        //     (i32_imm 2)
        //     (dcall)
        // end
        //
        // function $eleven (;1;) () -> (i32)
        //     (i32_imm 11)
        // end
        //
        // function $thirteen (;2;) () -> (i32)
        //     (i32_imm 13)
        // end
        //
        // function $seventeen (;3;) () -> (i32)
        //     (i32_imm 17)
        // end
        //
        // function $nineteen (;4;) () -> (i32)
        //     (i32_imm 19)
        // end

        // expect (13, 19, 17, 11, 13)

        let code_main = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 2)
            .write_opcode(Opcode::dcall)
            .write_opcode_i32(Opcode::i32_imm, 4)
            .write_opcode(Opcode::dcall)
            .write_opcode_i32(Opcode::i32_imm, 3)
            .write_opcode(Opcode::dcall)
            .write_opcode_i32(Opcode::i32_imm, 1)
            .write_opcode(Opcode::dcall)
            .write_opcode_i32(Opcode::i32_imm, 2)
            .write_opcode(Opcode::dcall)
            .write_opcode(Opcode::end)
            .to_bytes();

        let code_eleven = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 11)
            .write_opcode(Opcode::end)
            .to_bytes();

        let code_thirteen = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 13)
            .write_opcode(Opcode::end)
            .to_bytes();

        let code_seventeen = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 17)
            .write_opcode(Opcode::end)
            .to_bytes();

        let code_nineteen = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 19)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_functions_and_blocks(
            vec![
                HelperFunctionEntry {
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
                HelperFunctionEntry {
                    params: vec![],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_eleven,
                },
                HelperFunctionEntry {
                    params: vec![],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_thirteen,
                },
                HelperFunctionEntry {
                    params: vec![],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_seventeen,
                },
                HelperFunctionEntry {
                    params: vec![],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code_nineteen,
                },
            ],
            vec![],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(13),
                ForeignValue::UInt32(19),
                ForeignValue::UInt32(17),
                ForeignValue::UInt32(11),
                ForeignValue::UInt32(13),
            ]
        );
    }
}
