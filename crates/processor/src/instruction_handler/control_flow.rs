// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;
use anc_stack::{FrameType, ProgramCounter};

use crate::{process::EXIT_CURRENT_HANDLER_LOOP_BIT, TERMINATE_CODE_STACK_OVERFLOW};

use super::HandleResult;

/// end a function or block.
///
/// both instruction `end` and `break` can end
/// a function or a block, they are the same actually except that
/// the `break` instruction can specify `layers`
/// and `next_inst_offset` parameters.
pub fn end(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // () -> NO_RETURN
    const INSTRUCTION_END_LENGTH: u32 = 2;
    do_break(thread_context, 0, INSTRUCTION_END_LENGTH)
}

pub fn block(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param type_index:i32 local_variable_list_index:i32)
    let (type_index, local_variable_list_index) = thread_context.get_param_i32_i32();

    let ProgramCounter {
        instruction_address: _,
        function_internal_index: _,
        module_index,
    } = thread_context.pc;
    let module = &thread_context.module_common_instances[module_index];
    let type_item = &module.type_section.items[type_index as usize];
    let local_variables_with_arguments_allocated_bytes =
        module.local_variable_section.lists[local_variable_list_index as usize].allocated_bytes;

    match thread_context.stack.create_frame(
        type_item.params_count,
        type_item.results_count,
        local_variable_list_index,
        local_variables_with_arguments_allocated_bytes,
        None,
    ) {
        Ok(_) => HandleResult::Move(12),
        Err(_) => {
            // stack overflow
            HandleResult::Terminate(TERMINATE_CODE_STACK_OVERFLOW)
        }
    }
}

pub fn block_alt(/* _handler: &Handler, */ thread_context: &mut ThreadContext,) -> HandleResult {
    // (param type_index:i32 local_variable_list_index:i32 next_inst_offset:i32)
    let condition = thread_context.stack.pop_i32_u();
    let (type_index, local_variable_list_index, next_inst_offset) =
        thread_context.get_param_i32_i32_i32();

    let ProgramCounter {
        instruction_address: _,
        function_internal_index: _,
        module_index,
    } = thread_context.pc;
    let module = &thread_context.module_common_instances[module_index];
    let type_item = &module.type_section.items[type_index as usize];

    let local_variables_with_arguments_allocated_bytes =
        module.local_variable_section.lists[local_variable_list_index as usize].allocated_bytes;

    match thread_context.stack.create_frame(
        type_item.params_count,
        type_item.results_count,
        local_variable_list_index,
        local_variables_with_arguments_allocated_bytes,
        None,
    ) {
        Ok(_) => {
            if condition == 0 {
                HandleResult::Move(next_inst_offset as isize)
            } else {
                HandleResult::Move(16) // inst length = 16 bytes
            }
        }
        Err(_) => {
            // stack overflow
            HandleResult::Terminate(TERMINATE_CODE_STACK_OVERFLOW)
        }
    }
}

pub fn block_nez(/* _handler: &Handler, */ thread_context: &mut ThreadContext,) -> HandleResult {
    // (param local_variable_list_index:i32 next_inst_offset:i32) NO_RETURN

    let condition = thread_context.stack.pop_i32_u();
    let (local_variable_list_index, next_inst_offset) = thread_context.get_param_i32_i32();

    if condition == 0 {
        HandleResult::Move(next_inst_offset as isize)
    } else {
        let ProgramCounter {
            instruction_address: _,
            function_internal_index: _,
            module_index,
        } = thread_context.pc;
        let module = &thread_context.module_common_instances[module_index];
        let local_variables_with_arguments_allocated_bytes =
            module.local_variable_section.lists[local_variable_list_index as usize].allocated_bytes;

        // 'block_nez' has no type (i.e. has no params and returns)
        match thread_context.stack.create_frame(
            0,
            0,
            local_variable_list_index,
            local_variables_with_arguments_allocated_bytes,
            None,
        ) {
            Ok(_) => {
                HandleResult::Move(12) // 96 bits instruction
            }
            Err(_) => {
                // stack overflow
                HandleResult::Terminate(TERMINATE_CODE_STACK_OVERFLOW)
            }
        }
    }
}

/// note that both instruction 'end' and 'break' can end
/// a function or a block, they are the same actually except
/// the 'break' instruction can specify the 'layers'
/// and 'next_inst_offset'.
// thus `end` == `break layers=0 next_inst_offset=2`
pub fn break_(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 next_inst_offset:i32) NO_RETURN
    let (layers, next_inst_offset) = thread_context.get_param_i16_i32();
    do_break(thread_context, layers, next_inst_offset)
}

// `break_alt next` == `break 0 next`
pub fn break_alt(/* _handler: &Handler, */ thread_context: &mut ThreadContext,) -> HandleResult {
    // (param next_inst_offset:i32) -> NO_RETURN
    let next_inst_offset = thread_context.get_param_i32();
    do_break(thread_context, 0, next_inst_offset)
}

fn do_break(
    thread_context: &mut ThreadContext,
    layers: u16,
    next_inst_offset: u32,
) -> HandleResult {
    let opt_return_pc = thread_context.stack.remove_frames(layers);

    if let Some(return_pc) = opt_return_pc {
        // current function end
        //
        // the `EXIT_CURRENT_HANDLER_LOOP_BIT` flag is used to indicated
        // the current function is the last function of "calling path" (each
        // callback function will generate a new calling path).
        //
        // if the current function is the last function of "calling path",
        // the `process_continuous_instructions()` should be terminated.
        if return_pc.module_index & EXIT_CURRENT_HANDLER_LOOP_BIT == EXIT_CURRENT_HANDLER_LOOP_BIT {
            const EXIT_CURRENT_HANDLER_LOOP_BIT_INVERT: usize = !EXIT_CURRENT_HANDLER_LOOP_BIT;

            // remove the EXIT_CURRENT_HANDLER_LOOP_BIT flag
            let original_module_index =
                return_pc.module_index & EXIT_CURRENT_HANDLER_LOOP_BIT_INVERT;

            let original_pc = ProgramCounter {
                instruction_address: return_pc.instruction_address,
                function_internal_index: return_pc.function_internal_index,
                module_index: original_module_index,
            };

            HandleResult::End(original_pc)
        } else {
            HandleResult::Jump(return_pc)
        }
    } else {
        // current block end
        //
        // just move on
        HandleResult::Move(next_inst_offset as isize)
    }
}

pub fn recur(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 start_inst_offset:i32) -> NO_RETURN
    let (layers, start_inst_offset) = thread_context.get_param_i16_i32();
    do_recur(thread_context, layers, start_inst_offset)
}

fn do_recur(
    thread_context: &mut ThreadContext,
    layers: u16,
    start_inst_offset: u32,
) -> HandleResult {
    let frame_type = thread_context.stack.reset_frames(layers);
    if frame_type == FrameType::Function {
        // the target frame is a function frame
        // the value of 'start_inst_offset' is ignored.
        let ProgramCounter {
            instruction_address,
            function_internal_index,
            module_index,
        } = thread_context.pc;
        let function_item = &thread_context.module_common_instances[module_index]
            .function_section
            .items[function_internal_index];
        let relate_offset = function_item.code_offset as isize - instruction_address as isize;
        HandleResult::Move(relate_offset)
    } else {
        // the target frame is a block frame
        HandleResult::Move(-(start_inst_offset as isize))
    }
}

#[cfg(test)]
mod tests {
    use anc_context::program_source::ProgramSource;
    use anc_image::{
        bytecode_writer::BytecodeWriterHelper,
        utils::{helper_build_module_binary_with_single_function_and_blocks, HelperBlockEntry},
    };
    use anc_isa::{opcode::Opcode, ForeignValue, OperandDataType};

    use crate::{
        in_memory_program_source::InMemoryProgramSource, process::process_function, ProcessorError,
        ProcessorErrorType, TERMINATE_CODE_UNREACHABLE,
    };

    #[test]
    fn test_handler_control_flow_block() {
        // fn () -> (i32, i32, i32, i32)    ;; type idx 0
        //     imm_i32(11)
        //     imm_i32(13)
        //     block () -> ()               ;; type idx 1
        //         imm_i32(17)
        //         imm_i32(19)
        //     end
        //     imm_i32(23)
        //     imm_i32(29)
        // end
        //
        // expect (11, 13, 23, 29)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 11)
            .append_opcode_i32(Opcode::imm_i32, 13)
            .append_opcode_i32_i32(Opcode::block, 1, 1) // block type = 1, local variable index = 1
            .append_opcode_i32(Opcode::imm_i32, 17)
            .append_opcode_i32(Opcode::imm_i32, 19)
            .append_opcode(Opcode::end)
            .append_opcode_i32(Opcode::imm_i32, 23)
            .append_opcode_i32(Opcode::imm_i32, 29)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![], // params
            vec![
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
            ], // results
            vec![], // local variables
            code0,
            vec![HelperBlockEntry {
                params: vec![],
                results: vec![],
                local_variable_item_entries_without_args: vec![],
            }],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(/* &handler, */ &mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U32(11),
                ForeignValue::U32(13),
                ForeignValue::U32(23),
                ForeignValue::U32(29),
            ]
        );
    }

    #[test]
    fn test_handler_control_flow_block_with_args_and_results() {
        // fn () -> (i32, i32, i32)
        //     imm_i32(11)
        //     imm_i32(13)
        //     block (i32) -> (i32)
        //         local_load(0)
        //         imm_i32(17)
        //         add_i32()
        //     end
        //     imm_i32(19)
        // end
        //
        // expect (11, 30, 19)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 11)
            .append_opcode_i32(Opcode::imm_i32, 13)
            .append_opcode_i32_i32(Opcode::block, 1, 1) // block type = 1, local list index = 1
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i32(Opcode::imm_i32, 17)
            .append_opcode(Opcode::add_i32)
            .append_opcode(Opcode::end)
            .append_opcode_i32(Opcode::imm_i32, 19)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![], // params
            vec![
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
            ], // results
            vec![], // local variables
            code0,
            vec![HelperBlockEntry {
                params: vec![OperandDataType::I32],
                results: vec![OperandDataType::I32],
                local_variable_item_entries_without_args: vec![],
            }],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(/* &handler, */ &mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U32(11),
                ForeignValue::U32(30),
                ForeignValue::U32(19),
            ]
        );
    }

    #[test]
    fn test_handler_control_flow_block_with_local_variables() {
        // fn (a/0:i32, b/1:i32) -> (i32,i32,i32,i32,i32,i32,i32,i32)
        //     [local c/2:i32, d/3:i32]
        //     c=a+1                            ;; 20
        //     d=b+1                            ;; 12
        //     block () -> (i32, i32, i32,i32)  ;; type idx 1
        //         [local p/0:i32, q/1:i32]
        //         a=a-1                        ;; 18
        //         b=b-1                        ;; 10
        //         p=c+d                        ;; 32
        //         q=c-d                        ;; 8
        //         load c
        //         load d
        //         block (x/0:i32, y/1:i32) -> (i32,i32)    ;; type idx 2
        //             d=d+1                    ;; 13
        //             q=q-1                    ;; 7
        //             x+q                      ;; 27 (ret #0)
        //             y+p                      ;; 44 (ret #1)
        //         end
        //         load p (ret #2)
        //         load q (ret #3)
        //     end
        //     load a (ret #4)
        //     load b (ret #5)
        //     load c (ret #6)
        //     load d (ret #7)
        // end
        //
        // expect (19, 11) -> (27, 44, 32, 7, 18, 10, 20, 13)

        let code0 = BytecodeWriterHelper::new()
            // c=a+1
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i16(Opcode::add_imm_i32, 1)
            .append_opcode_i16_i32(Opcode::local_store_i32, 0, 2)
            // d=b+1
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode_i16(Opcode::add_imm_i32, 1)
            .append_opcode_i16_i32(Opcode::local_store_i32, 0, 3)
            // block 1
            .append_opcode_i32_i32(Opcode::block, 1, 1)
            // a=a-1
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode_i16(Opcode::sub_imm_i32, 1)
            .append_opcode_i16_i32(Opcode::local_store_i32, 1, 0)
            // b=b-1
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 1)
            .append_opcode_i16(Opcode::sub_imm_i32, 1)
            .append_opcode_i16_i32(Opcode::local_store_i32, 1, 1)
            // p=c+d
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 2)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 3)
            .append_opcode(Opcode::add_i32)
            .append_opcode_i16_i32(Opcode::local_store_i32, 0, 0)
            // q=c-d
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 2)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 3)
            .append_opcode(Opcode::sub_i32)
            .append_opcode_i16_i32(Opcode::local_store_i32, 0, 1)
            // load c, d
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 2)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 3)
            // block 2
            .append_opcode_i32_i32(Opcode::block, 2, 2)
            // d=d+1
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 2, 3)
            .append_opcode_i16(Opcode::add_imm_i32, 1)
            .append_opcode_i16_i32(Opcode::local_store_i32, 2, 3)
            // q=q-1
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 1)
            .append_opcode_i16(Opcode::sub_imm_i32, 1)
            .append_opcode_i16_i32(Opcode::local_store_i32, 1, 1)
            // x+q
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 1)
            .append_opcode(Opcode::add_i32)
            // y+p
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode(Opcode::add_i32)
            //
            .append_opcode(Opcode::end)
            // load p, q
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            //
            .append_opcode(Opcode::end)
            // load a, b, c, d
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 2)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 3)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![OperandDataType::I32, OperandDataType::I32], // params
            vec![
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
            ], // results
            vec![OperandDataType::I32, OperandDataType::I32], // local variables
            code0,
            vec![
                HelperBlockEntry {
                    params: vec![],
                    results: vec![
                        OperandDataType::I32,
                        OperandDataType::I32,
                        OperandDataType::I32,
                        OperandDataType::I32,
                    ],
                    local_variable_item_entries_without_args: vec![
                        OperandDataType::I32,
                        OperandDataType::I32,
                    ],
                },
                HelperBlockEntry {
                    params: vec![OperandDataType::I32, OperandDataType::I32],
                    results: vec![OperandDataType::I32, OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
            ],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(19), ForeignValue::U32(11)],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U32(27),
                ForeignValue::U32(44),
                ForeignValue::U32(32),
                ForeignValue::U32(7),
                ForeignValue::U32(18),
                ForeignValue::U32(10),
                ForeignValue::U32(20),
                ForeignValue::U32(13),
            ]
        );
    }

    #[test]
    fn test_handler_control_flow_break_function() {
        // fn () -> (i32, i32)
        //     imm_i32(11)
        //     imm_i32(13)
        //     break(0)
        //     imm_i32(17)
        //     imm_i32(19)
        // end
        //
        // expect (11, 13)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 11)
            .append_opcode_i32(Opcode::imm_i32, 13)
            .append_opcode_i16_i32(Opcode::break_, 0, 0)
            .append_opcode_i32(Opcode::imm_i32, 17)
            .append_opcode_i32(Opcode::imm_i32, 19)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![],                                           // params
            vec![OperandDataType::I32, OperandDataType::I32], // results
            vec![],                                           // local variables
            code0,
            vec![],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(/* &handler, */ &mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::U32(11), ForeignValue::U32(13),]
        );
    }

    #[test]
    fn test_handler_control_flow_break_block() {
        // fn () -> (i32, i32, i32, i32)
        //     imm_i32(11)
        //     imm_i32(13)
        //     block () -> (i32, i32)
        //         imm_i32(17)
        //         imm_i32(19)
        //         break(0)
        //         imm_i32(23)
        //         imm_i32(29)
        //     end
        //     imm_i32(31)
        //     imm_i32(37)
        // end
        //
        // expect (17, 19, 31, 37)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 11)
            .append_opcode_i32(Opcode::imm_i32, 13)
            .append_opcode_i32_i32(Opcode::block, 1, 1) // block type = 1
            .append_opcode_i32(Opcode::imm_i32, 17)
            .append_opcode_i32(Opcode::imm_i32, 19)
            .append_opcode_i16_i32(Opcode::break_, 0, 0x1a)
            .append_opcode_i32(Opcode::imm_i32, 23)
            .append_opcode_i32(Opcode::imm_i32, 29)
            .append_opcode(Opcode::end)
            .append_opcode_i32(Opcode::imm_i32, 31)
            .append_opcode_i32(Opcode::imm_i32, 37)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![], // params
            vec![
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
            ], // results
            vec![], // local variables
            code0,
            vec![HelperBlockEntry {
                params: vec![],
                results: vec![OperandDataType::I32, OperandDataType::I32],
                local_variable_item_entries_without_args: vec![],
            }],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(/* &handler, */ &mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U32(17),
                ForeignValue::U32(19),
                ForeignValue::U32(31),
                ForeignValue::U32(37),
            ]
        );
    }

    #[test]
    fn test_handler_control_flow_break_block_to_function() {
        // fn () -> (i32, i32)
        //     imm_i32 11()
        //     imm_i32 13()
        //     block () -> (i32 i32)
        //         imm_i32(17)
        //         imm_i32(19)
        //         break(1)
        //         imm_i32(23)
        //         imm_i32(29)
        //     end
        //     imm_i32(31)
        //     imm_i32(37)
        // end
        //
        // expect (17, 19)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 11)
            .append_opcode_i32(Opcode::imm_i32, 13)
            .append_opcode_i32_i32(Opcode::block, 1, 1) // block type = 1, local variable index = 1
            .append_opcode_i32(Opcode::imm_i32, 17)
            .append_opcode_i32(Opcode::imm_i32, 19)
            .append_opcode_i16_i32(Opcode::break_, 1, 0)
            .append_opcode_i32(Opcode::imm_i32, 23)
            .append_opcode_i32(Opcode::imm_i32, 29)
            .append_opcode(Opcode::end)
            .append_opcode_i32(Opcode::imm_i32, 31)
            .append_opcode_i32(Opcode::imm_i32, 37)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![],                                           // params
            vec![OperandDataType::I32, OperandDataType::I32], // results
            vec![],                                           // local variables
            code0,
            vec![HelperBlockEntry {
                params: vec![],
                results: vec![OperandDataType::I32, OperandDataType::I32],
                local_variable_item_entries_without_args: vec![],
            }],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(/* &handler, */ &mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::U32(17), ForeignValue::U32(19),]
        );
    }

    #[test]
    fn test_handler_control_flow_structure_when() {
        // fn max (left/0:i32, right/1:i32) -> (i32)    ;; type idx 0
        //     [local ret/2 i32]
        //
        //     local_load32(0, 0)
        //     local_store_i32(0, 2)
        //
        //     local_load32(0, 0)
        //     local_load32(0, 1)
        //     lt_i32_u
        //     block_nez ()->()                         ;; type idx 1
        //          local_load32(1, 1)
        //          local_store_i32(1, 2)
        //     end
        //     local_load32(0, 2)
        // end
        //
        // assert (11, 13) -> (13)
        // assert (19, 17) -> (19)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i16_i32(Opcode::local_store_i32, 0, 2)
            //
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode(Opcode::lt_i32_u)
            .append_opcode_i32_i32(Opcode::block_nez, 1, 0x1e)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 1)
            .append_opcode_i16_i32(Opcode::local_store_i32, 1, 2)
            .append_opcode(Opcode::end)
            //
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 2)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![OperandDataType::I32, OperandDataType::I32], // params
            vec![OperandDataType::I32],                       // results
            vec![OperandDataType::I32],                       // local variables
            code0,
            vec![HelperBlockEntry {
                params: vec![],
                results: vec![],
                local_variable_item_entries_without_args: vec![],
            }],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(11), ForeignValue::U32(13)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(13)]);

        let result1 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(19), ForeignValue::U32(17)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::U32(19)]);
    }

    #[test]
    fn test_handler_control_flow_when_with_break() {
        // break crossing block
        //
        // fn (/0:i32) -> (i32 i32 i32 i32)     ;; type idx 0
        //     imm_i32(11)
        //     imm_i32(13)
        //     block () -> (i32 i32)            ;; type idx 1
        //         imm_i32(17)
        //         imm_i32(19)
        //         local_load_i32_u(1, 0)       ;; == true
        //         block_nez
        //             imm_i32(23)
        //             imm_i32(29)
        //             break(1)
        //             imm_i32(31)
        //             imm_i32(37)
        //         end
        //         imm_i32(41)
        //         imm_i32(43)
        //     end
        //     imm_i32(51)
        //     imm_i32(53)
        // end
        //
        // expect (1) -> (23, 29, 51, 53)
        // expect (0) -> (41, 43, 51, 53)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 11)
            .append_opcode_i32(Opcode::imm_i32, 13)
            .append_opcode_i32_i32(Opcode::block, 1, 1) // block type = 1
            .append_opcode_i32(Opcode::imm_i32, 17)
            .append_opcode_i32(Opcode::imm_i32, 19)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode_i32_i32(Opcode::block_nez, 2, 0x36) // block type = 2
            .append_opcode_i32(Opcode::imm_i32, 23)
            .append_opcode_i32(Opcode::imm_i32, 29)
            .append_opcode_i16_i32(Opcode::break_, 1, 0x2e)
            .append_opcode_i32(Opcode::imm_i32, 31)
            .append_opcode_i32(Opcode::imm_i32, 37)
            .append_opcode(Opcode::end)
            .append_opcode_i32(Opcode::imm_i32, 41)
            .append_opcode_i32(Opcode::imm_i32, 43)
            .append_opcode(Opcode::end)
            .append_opcode_i32(Opcode::imm_i32, 51)
            .append_opcode_i32(Opcode::imm_i32, 53)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![OperandDataType::I32], // params
            vec![
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
            ], // results
            vec![],                     // local variables
            code0,
            vec![
                HelperBlockEntry {
                    params: vec![],
                    results: vec![OperandDataType::I32, OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![],
                    results: vec![],
                    local_variable_item_entries_without_args: vec![],
                },
            ],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(1)]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U32(23),
                ForeignValue::U32(29),
                ForeignValue::U32(51),
                ForeignValue::U32(53),
            ]
        );

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(0)]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U32(41),
                ForeignValue::U32(43),
                ForeignValue::U32(51),
                ForeignValue::U32(53),
            ]
        );
    }

    #[test]
    fn test_handler_control_flow_structure_if() {
        // fn max (i32, i32) -> (i32)
        //     local_load32(0, 0)
        //     local_load32(0, 1)
        //     gt_i32_u
        //     block_alt ()->(i32)
        //         local_load32(1, 0)
        //     break_alt
        //         local_load32(1, 1)
        //     end
        // end
        //
        // assert (11, 13) -> (13)
        // assert (19, 17) -> (19)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode(Opcode::gt_i32_u)
            .append_opcode_i32_i32_i32(Opcode::block_alt, 1, 1, 0x20)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode_i32(Opcode::break_alt, 0x12)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 1)
            .append_opcode(Opcode::end)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![OperandDataType::I32, OperandDataType::I32], // params
            vec![OperandDataType::I32],                       // results
            vec![],                                           // local variables
            code0,
            vec![HelperBlockEntry {
                params: vec![],
                results: vec![OperandDataType::I32],
                local_variable_item_entries_without_args: vec![],
            }],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(11), ForeignValue::U32(13)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(13)]);

        let result1 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(19), ForeignValue::U32(17)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::U32(19)]);
    }

    #[test]
    fn test_handler_control_flow_structure_if_nested() {
        // fn level (0/:i32) -> (i32)
        //     local_load32(0, 0)
        //     imm_i32(85)
        //     gt_i32_u
        //     block_alt ()->(i32)              ;; type idx 1
        //         imm_i32(65)                  ;; 'A' (85, 100]
        //     break_alt
        //         local_load32(1, 0)
        //         imm_i32(70)
        //         gt_i32_u
        //         block_alt ()->(i32)          ;; block 2 2
        //             imm_i32(66)              ;; 'B' (70,85]
        //         break_alt
        //             local_load32(2, 0)
        //             imm_i32(55)
        //             gt_i32_u
        //             block_alt ()->(i32)      ;; block 3 3
        //                 imm_i32(67)          ;; 'C' (55, 70]
        //             break_alt
        //                 imm_i32(68)          ;; 'D' [0, 55]
        //             end
        //         end
        //     end
        // end
        //
        // assert (90) -> (65) 'A'
        // assert (80) -> (66) 'B'
        // assert (70) -> (67) 'C'
        // assert (60) -> (67) 'C'
        // assert (50) -> (68) 'D'
        // assert (40) -> (68) 'D'

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i32(Opcode::imm_i32, 85)
            .append_opcode(Opcode::gt_i32_u)
            .append_opcode_i32_i32_i32(Opcode::block_alt, 1, 1, 0x20)
            .append_opcode_i32(Opcode::imm_i32, 65)
            .append_opcode_i32(Opcode::break_alt, 0x7e)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode_i32(Opcode::imm_i32, 70)
            .append_opcode(Opcode::gt_i32_u)
            .append_opcode_i32_i32_i32(Opcode::block_alt, 2, 2, 0x20)
            .append_opcode_i32(Opcode::imm_i32, 66)
            .append_opcode_i32(Opcode::break_alt, 0x48)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 2, 0)
            .append_opcode_i32(Opcode::imm_i32, 55)
            .append_opcode(Opcode::gt_i32_u)
            .append_opcode_i32_i32_i32(Opcode::block_alt, 3, 3, 0x20)
            .append_opcode_i32(Opcode::imm_i32, 67)
            .append_opcode_i32(Opcode::break_alt, 0x12)
            .append_opcode_i32(Opcode::imm_i32, 68)
            .append_opcode(Opcode::end)
            .append_opcode(Opcode::end)
            .append_opcode(Opcode::end)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![OperandDataType::I32], // params
            vec![OperandDataType::I32], // results
            vec![],                     // local variables
            code0,
            vec![
                HelperBlockEntry {
                    params: vec![], // 'block_alt' has no PARAMS but RESULTS
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![], // 'block_alt' has no PARAMS but RESULTS
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![], // 'block_alt' has no PARAMS but RESULTS
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
            ],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(90)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(65)]);

        let result1 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(80)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::U32(66)]);

        let result2 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(70)]);
        assert_eq!(result2.unwrap(), vec![ForeignValue::U32(67)]);

        let result3 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(60)]);
        assert_eq!(result3.unwrap(), vec![ForeignValue::U32(67)]);

        let result4 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(50)]);
        assert_eq!(result4.unwrap(), vec![ForeignValue::U32(68)]);

        let result5 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(40)]);
        assert_eq!(result5.unwrap(), vec![ForeignValue::U32(68)]);
    }

    #[test]
    fn test_handler_control_flow_structure_branch() {
        // fn level (i32) -> (i32)
        //     block ()->(i32)              ;; block 1 1
        //                                  ;; case 1
        //         local_load32(0, 0)
        //         imm_i32(85)
        //         gt_i32_u
        //         block_nez ()->()         ;; block 2 2
        //             imm_i32(65)          ;; 'A' (85, 100]
        //             break(1)
        //         end
        //                                  ;; case 2
        //         local_load32(0, 0)
        //         imm_i32(70)
        //         gt_i32_u
        //         block_nez ()->()         ;; block 3 3
        //             imm_i32(66)          ;; 'B' (70,85]
        //             break(1)
        //         end
        //                                  ;; case 3
        //         local_load32(0, 0)
        //         imm_i32(55)
        //         gt_i32_u
        //         block_nez ()->()         ;; block 4 4
        //             imm_i32(67)          ;; 'C' (55, 70]
        //             break(1)
        //         end
        //                                  ;; default
        //         imm_i32(68)              ;; 'D' [0, 55]
        //     end
        // end
        //
        // assert (90) -> (65) 'A'
        // assert (80) -> (66) 'B'
        // assert (70) -> (67) 'C'
        // assert (60) -> (67) 'C'
        // assert (50) -> (68) 'D'
        // assert (40) -> (68) 'D'

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32_i32(Opcode::block, 1, 1)
            // case 1
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode_i32(Opcode::imm_i32, 85)
            .append_opcode(Opcode::gt_i32_u)
            .append_opcode_i32_i32(Opcode::block_nez, 2, 0x1e)
            .append_opcode_i32(Opcode::imm_i32, 65)
            .append_opcode_i16_i32(Opcode::break_, 1, 0x7e)
            .append_opcode(Opcode::end)
            // case 2
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode_i32(Opcode::imm_i32, 70)
            .append_opcode(Opcode::gt_i32_u)
            .append_opcode_i32_i32(Opcode::block_nez, 3, 0x1e)
            .append_opcode_i32(Opcode::imm_i32, 66)
            .append_opcode_i16_i32(Opcode::break_, 1, 0x4a)
            .append_opcode(Opcode::end)
            // case 3
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode_i32(Opcode::imm_i32, 55)
            .append_opcode(Opcode::gt_i32_u)
            .append_opcode_i32_i32(Opcode::block_nez, 4, 0x1e)
            .append_opcode_i32(Opcode::imm_i32, 67)
            .append_opcode_i16_i32(Opcode::break_, 1, 0x16)
            .append_opcode(Opcode::end)
            // default
            .append_opcode_i32(Opcode::imm_i32, 68)
            // block end
            .append_opcode(Opcode::end)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![OperandDataType::I32], // params
            vec![OperandDataType::I32], // results
            vec![],                     // local variables
            code0,
            vec![
                HelperBlockEntry {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![],
                    results: vec![],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![],
                    results: vec![],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![],
                    results: vec![],
                    local_variable_item_entries_without_args: vec![],
                },
            ],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(90)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(65)]);

        let result1 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(80)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::U32(66)]);

        let result2 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(70)]);
        assert_eq!(result2.unwrap(), vec![ForeignValue::U32(67)]);

        let result3 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(60)]);
        assert_eq!(result3.unwrap(), vec![ForeignValue::U32(67)]);

        let result4 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(50)]);
        assert_eq!(result4.unwrap(), vec![ForeignValue::U32(68)]);

        let result5 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(40)]);
        assert_eq!(result5.unwrap(), vec![ForeignValue::U32(68)]);
    }

    #[test]
    fn test_handler_control_flow_structure_branch_without_default_arm() {
        // note
        // this test requires the instruction 'panic'

        // fn level (i32) -> (i32)
        //     block ()->(i32)              ;; type idx 1
        //                                  ;; case 1
        //         local_load32(0, 0)
        //         imm_i32(85)
        //         gt_i32_u
        //         block_nez ()->()         ;; type idx 2
        //             imm_i32(65)          ;; 'A' (85, 100]
        //             break(1)
        //         end
        //                                  ;; case 2
        //         local_load32(0, 0)
        //         imm_i32(70)
        //         gt_i32_u
        //         block_nez ()->()         ;; type idx 3
        //             imm_i32(66)          ;; 'B' (70,85]
        //             break(1)
        //         end
        //         panic
        //     end
        // end
        //
        // assert (90) -> (65) 'A'
        // assert (80) -> (66) 'B'
        // assert (70) -> panic
        // assert (60) -> panic

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32_i32(Opcode::block, 1, 1)
            // case 1
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode_i32(Opcode::imm_i32, 85)
            .append_opcode(Opcode::gt_i32_u)
            .append_opcode_i32_i32(Opcode::block_nez, 2, 0x1e)
            .append_opcode_i32(Opcode::imm_i32, 65)
            .append_opcode_i16_i32(Opcode::break_, 1, 0x4a)
            .append_opcode(Opcode::end)
            // case 2
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode_i32(Opcode::imm_i32, 70)
            .append_opcode(Opcode::gt_i32_u)
            .append_opcode_i32_i32(Opcode::block_nez, 3, 0x1e)
            .append_opcode_i32(Opcode::imm_i32, 66)
            .append_opcode_i16_i32(Opcode::break_, 1, 0x16)
            .append_opcode(Opcode::end)
            // unreachable
            .append_opcode_i32(Opcode::terminate, TERMINATE_CODE_UNREACHABLE as u32)
            // block end
            .append_opcode(Opcode::end)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![OperandDataType::I32], // params
            vec![OperandDataType::I32], // results
            vec![],                     // local variables
            code0,
            vec![
                HelperBlockEntry {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![],
                    results: vec![],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![],
                    results: vec![],
                    local_variable_item_entries_without_args: vec![],
                },
            ],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(90)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(65)]);

        let result1 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(80)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::U32(66)]);

        let result2 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(70)]);
        assert!(matches!(
            result2,
            Err(ProcessorError {
                error_type: ProcessorErrorType::Terminate(TERMINATE_CODE_UNREACHABLE)
            })
        ));

        let result3 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(60)]);
        assert!(matches!(
            result3,
            Err(ProcessorError {
                error_type: ProcessorErrorType::Terminate(TERMINATE_CODE_UNREACHABLE)
            })
        ));
    }

    #[test]
    fn test_handler_control_flow_structure_loop() {
        // fn accu (n/0:i32) -> (i32)
        //     [local sum/1:i32]
        //     block ()->()
        //                                  ;; break if n==0
        //         local_load32(1, 0)
        //         eqz_i32
        //         block_nez
        //             break(1)
        //         end
        //                                  ;; sum = sum + n
        //         local_load32(1, 0)
        //         local_load32(1, 1)
        //         add_i32
        //         local_store_i32(1, 1)
        //                                  ;; n = n - 1
        //         local_load32(1, 0)
        //         sub_imm_i32(1)
        //         local_store_i32(1, 0)
        //                                  ;; recur
        //         (recur 0)
        //     end
        //     (local_load32 0 1)
        // end
        //
        // assert (10) -> (55)
        // assert (100) -> (5050)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32_i32(Opcode::block, 1, 1)
            //
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode(Opcode::eqz_i32)
            .append_opcode_i32_i32(Opcode::block_nez, 2, 0x16)
            .append_opcode_i16_i32(Opcode::break_, 1, 0x46)
            .append_opcode(Opcode::end)
            // sum = sum + n
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 1)
            .append_opcode(Opcode::add_i32)
            .append_opcode_i16_i32(Opcode::local_store_i32, 1, 1)
            // n = n - 1
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode_i16(Opcode::sub_imm_i32, 1)
            .append_opcode_i16_i32(Opcode::local_store_i32, 1, 0)
            //
            .append_opcode_i16_i32(Opcode::recur, 0, 0x54)
            // block end
            .append_opcode(Opcode::end)
            //
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![OperandDataType::I32], // params
            vec![OperandDataType::I32], // results
            vec![OperandDataType::I32], // local variables
            code0,
            vec![
                HelperBlockEntry {
                    params: vec![],
                    results: vec![],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![],
                    results: vec![],
                    local_variable_item_entries_without_args: vec![],
                },
            ],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(10)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(55)]);

        let result1 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(100)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::U32(5050)]);
    }

    #[test]
    fn test_handler_control_flow_structure_loop_with_block_parameters() {
        // fn accu (count/0:i32) -> (i32)
        //     imm_i32(0)                   ;; sum
        //     local_load32(0, 0)           ;; count
        //     block                        ;; (sum/0:i32, n/1:i32)->(i32)
        //                                  ;; break if n==0
        //         local_load32(0, 1)
        //         eqz_i32
        //         block_nez
        //             local_load32(0, 1)
        //             break(1)
        //         end
        //                                  ;; sum + n
        //         local_load32(0, 0)
        //         local_load32(0, 1)
        //         add_i32
        //                                  ;; n - 1
        //         local_load32(0, 1)
        //         sub_imm_i32(1)
        //                                  ;; recur
        //         recur(0)
        //     end
        // end
        //
        // assert (10) -> (55)
        // assert (100) -> (5050)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            // block start
            .append_opcode_i32_i32(Opcode::block, 1, 1)
            // if n==0
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode(Opcode::eqz_i32)
            .append_opcode_i32_i32(Opcode::block_nez, 2, 0x1e)
            // load sum, break sum
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode_i16_i32(Opcode::break_, 1, 0x36)
            .append_opcode(Opcode::end)
            // end if
            // sum + n
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode(Opcode::add_i32)
            // n - 1
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode_i16(Opcode::sub_imm_i32, 1)
            // recur
            .append_opcode_i16_i32(Opcode::recur, 0, 0x4c)
            // block end
            .append_opcode(Opcode::end)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![OperandDataType::I32], // params
            vec![OperandDataType::I32], // results
            vec![],                     // local variables
            code0,
            vec![
                HelperBlockEntry {
                    params: vec![OperandDataType::I32, OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
                // since the image builder helper just creates types for each block,
                // thus there is also a "type" for block_nez.
                HelperBlockEntry {
                    params: vec![],
                    results: vec![],
                    local_variable_item_entries_without_args: vec![],
                },
            ],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(10)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(55)]);

        let result1 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(100)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::U32(5050)]);
    }

    #[test]
    fn test_handler_control_flow_structure_loop_with_block_parameters_and_if() {
        // fn accu (count/0:i32) -> (i32)
        //     imm_i32(0)                   ;; sum
        //     local_load32(0, 0)           ;; count
        //     block (sum/0:i32, n/1:i32)->(i32)
        //                                  ;; if n==0
        //         local_load32(0, 1)
        //         eqz_i32
        //         block_alt
        //             local_load32(0, 1)
        //             break(1)
        //         break_alt
        //                                  ;; sum + n
        //             local_load32(0, 0)
        //             local_load32(0, 1)
        //             add_i32
        //                                  ;; n - 1
        //             local_load32(0, 1)
        //             sub_imm_i32(1)
        //                                  ;; recur
        //             recur(1)
        //         end
        //     end
        // end
        //
        // assert (10) -> (55)
        // assert (100) -> (5050)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            //
            .append_opcode_i32_i32(Opcode::block, 1, 1)
            // if
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode(Opcode::eqz_i32)
            .append_opcode_i32_i32_i32(Opcode::block_alt, 2, 2, 0x28)
            // then
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode_i16_i32(Opcode::break_, 1, 0x3c)
            // else
            .append_opcode_i32(Opcode::break_alt, 0x32)
            // sum + n
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 1)
            .append_opcode(Opcode::add_i32)
            // n - 1
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 1)
            .append_opcode_i16(Opcode::sub_imm_i32, 1)
            .append_opcode_i16_i32(Opcode::recur, 1, 0x54)
            // end if
            .append_opcode(Opcode::end)
            // block end
            .append_opcode(Opcode::end)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![OperandDataType::I32], // params
            vec![OperandDataType::I32], // results
            vec![],                     // local variables
            code0,
            vec![
                HelperBlockEntry {
                    params: vec![OperandDataType::I32, OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![],
                    results: vec![],
                    local_variable_item_entries_without_args: vec![],
                },
            ],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(10)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(55)]);

        let result1 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(100)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::U32(5050)]);
    }

    #[test]
    fn test_handler_control_flow_function_tail_call() {
        // fn accu (sum/0:i32, n/1:i32) -> (i32)
        //                              ;; sum = sum + n
        //     local_load32(0, 0)
        //     local_load32(0, 1)
        //     add_i32
        //     local_store_i32(0, 0)
        //                              ;; n = n - 1
        //     local_load32(0, 1)
        //     sub_imm_i32(1)
        //     local_store_i32(0, 1)
        //                              ;; if n > 0 recur (sum,n)
        //     local_load32(0, 1)
        //     imm_i32(0)
        //     gt_i32_u
        //     block_nez () -> ()
        //         local_load32(0, 0)
        //         local_load32(0, 1)
        //         recur(1)
        //     end
        //     local_load32(0, 0)       ;; load sum
        // end
        //
        // assert (0, 10) -> (55)
        // assert (0, 100) -> (5050)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode(Opcode::add_i32)
            .append_opcode_i16_i32(Opcode::local_store_i32, 0, 0)
            //
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode_i16(Opcode::sub_imm_i32, 1)
            .append_opcode_i16_i32(Opcode::local_store_i32, 0, 1)
            //
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            // .append_opcode(Opcode::zero)
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode(Opcode::gt_i32_u)
            .append_opcode_i32_i32(Opcode::block_nez, 1, 0x26)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 1)
            .append_opcode_i16_i32(Opcode::recur, 1, 0)
            // block end
            .append_opcode(Opcode::end)
            //
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![OperandDataType::I32, OperandDataType::I32], // params
            vec![OperandDataType::I32],                       // results
            vec![],                                           // local variables
            code0,
            vec![HelperBlockEntry {
                params: vec![],
                results: vec![],
                local_variable_item_entries_without_args: vec![],
            }], // blocks
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(0), ForeignValue::U32(10)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(55)]);

        let result1 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(0), ForeignValue::U32(100)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::U32(5050)]);
    }

    #[test]
    fn test_handler_control_flow_function_tail_call_with_if() {
        // fn accu (sum:i32, n:i32) -> (i32)
        //     local_load32(0, 1)               ;; load n
        //     eqz_i32
        //     block_alt () -> (i32)            ;; if n == 0
        //         local_load32(1, 0)           ;; then sum
        //     break_alt                        ;; else
        //                                      ;; sum + n
        //         local_load32(1, 0)
        //         local_load32(1, 1)
        //         add_i32
        //                                      ;; n - 1
        //         local_load32(1, 1)
        //         sub_imm_i32(1)
        //         recur(1)                     ;; recur
        //     end
        // end
        //
        // assert (0, 10) -> (55)
        // assert (0, 100) -> (5050)

        let code0 = BytecodeWriterHelper::new()
            //
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode(Opcode::eqz_i32)
            .append_opcode_i32_i32_i32(Opcode::block_alt, 1, 1, 0x20)
            // then
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode_i32(Opcode::break_alt, 0x32)
            // else
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 1)
            .append_opcode(Opcode::add_i32)
            //
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 1, 1)
            .append_opcode_i16(Opcode::sub_imm_i32, 1)
            //
            .append_opcode_i16_i32(Opcode::recur, 1, 0)
            // block end
            .append_opcode(Opcode::end)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![OperandDataType::I32, OperandDataType::I32], // params
            vec![OperandDataType::I32],                       // results
            vec![],                                           // local variables
            code0,
            vec![HelperBlockEntry {
                params: vec![], // 'block_alt' has no PARAMS
                results: vec![OperandDataType::I32],
                local_variable_item_entries_without_args: vec![],
            }], // blocks
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(0), ForeignValue::U32(10)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(55)]);

        let result1 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(0), ForeignValue::U32(100)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::U32(5050)]);
    }
}
