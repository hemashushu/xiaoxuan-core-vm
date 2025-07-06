// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;
use anc_isa::{ForeignValue, OperandDataType, OPERAND_SIZE_IN_BYTES};
use anc_stack::ProgramCounter;

use crate::{
    instruction_handler::{get_instruction_handler, HandleResult},
    ProcessorError, ProcessorErrorType,
};

// The `EXIT_CURRENT_HANDLER_LOOP_BIT` flag is used to indicate
// that the current function is the last function in the current "calling path".
// Each callback function creates a new calling path.
//
// If the current function is the last in the calling path,
// `process_continuous_instructions()` should terminate.
pub const EXIT_CURRENT_HANDLER_LOOP_BIT: usize = 0x8000_0000;

pub fn process_function(
    thread_context: &mut ThreadContext,
    module_index: usize,
    function_public_index: usize,
    arguments: &[ForeignValue],
) -> Result<Vec<ForeignValue>, ProcessorError> {
    // Reset the stack before function execution.
    thread_context.stack.reset();

    let target_function_object =
        thread_context.get_target_function_object(module_index, function_public_index);
    let function_info = thread_context.get_function_info(
        target_function_object.module_index,
        target_function_object.function_internal_index,
    );

    let (params, results) = {
        let pars = thread_context.module_common_instances[target_function_object.module_index]
            .type_section
            .get_item_params_and_results(function_info.type_index);
        (pars.0.to_vec(), pars.1.to_vec())
    };

    // Check that the number of arguments matches the function signature.
    if arguments.len() != params.len() {
        return Err(ProcessorError::new(
            ProcessorErrorType::ParametersAmountMissmatch,
        ));
    }

    // Push arguments onto the stack.
    // ------------------------------
    // Arguments are pushed in order, so the first value is at the bottom of the stack:
    //
    // ```diagram
    // array [0, 1, 2] -> |  2  | <-- high addr
    //                    |  1  |
    //                    |  0  |
    //                    \-----/ <-- low addr
    // ```
    //
    // Arguments are pushed left-to-right so that the stack bottom (low address) contains arg0,
    // making it easy to copy consecutive arguments to an array.
    //
    // For simplicity, this procedure does not check the data type of arguments.
    for value in arguments {
        match value {
            ForeignValue::U32(value) => thread_context.stack.push_i32_u(*value),
            ForeignValue::U64(value) => thread_context.stack.push_i64_u(*value),
            ForeignValue::F32(value) => thread_context.stack.push_f32(*value),
            ForeignValue::F64(value) => thread_context.stack.push_f64(*value),
        }
    }

    // Create a function stack frame.
    thread_context
        .stack
        .create_frame(
            params.len() as u16,
            results.len() as u16,
            function_info.local_variable_list_index as u32,
            function_info.local_variables_with_arguments_allocated_bytes as u32,
            Some(ProgramCounter {
                instruction_address: 0,
                function_internal_index: 0,
                // Set MSB of 'return module index' to '1' to indicate END of the function call path.
                module_index: EXIT_CURRENT_HANDLER_LOOP_BIT,
            }),
        )
        .map_err(|_| ProcessorError::new(ProcessorErrorType::StackOverflow))?;

    // Set the new program counter (PC).
    thread_context.pc.module_index = target_function_object.module_index;
    thread_context.pc.function_internal_index = target_function_object.function_internal_index;
    thread_context.pc.instruction_address = function_info.code_offset;

    // Start processing instructions.
    if let Some(terminate_code) =
        process_continuous_instructions(/* handler, */ thread_context)
    {
        return Err(ProcessorError::new(ProcessorErrorType::Terminate(
            terminate_code,
        )));
    }

    // Pop results from the stack.
    // ---------------------------
    // Results are popped from the top of the stack and become the last elements of the result array.
    //
    // ```diagram
    // |  2  | -> array [0, 1, 2]
    // |  1  |
    // |  0  |
    // \-----/
    // ```
    //
    // Do not use the `pop_xxx` functions to pop results, as they require a stack frame.
    // After the entry function finishes, the stack has no frame.
    let result_operands = thread_context.stack.pop_last_operands(results.len());
    let result_values = results
        .iter()
        .enumerate()
        .map(|(idx, dt)| match dt {
            OperandDataType::I32 => ForeignValue::U32(u32::from_le_bytes(
                result_operands[(idx * OPERAND_SIZE_IN_BYTES)..(idx * OPERAND_SIZE_IN_BYTES + 4)]
                    .try_into()
                    .unwrap(),
            )),
            OperandDataType::I64 => ForeignValue::U64(u64::from_le_bytes(
                result_operands[(idx * OPERAND_SIZE_IN_BYTES)..((idx + 1) * OPERAND_SIZE_IN_BYTES)]
                    .try_into()
                    .unwrap(),
            )),
            OperandDataType::F32 => ForeignValue::F32(f32::from_le_bytes(
                result_operands[(idx * OPERAND_SIZE_IN_BYTES)..(idx * OPERAND_SIZE_IN_BYTES + 4)]
                    .try_into()
                    .unwrap(),
            )),
            OperandDataType::F64 => ForeignValue::F64(f64::from_le_bytes(
                result_operands[(idx * OPERAND_SIZE_IN_BYTES)..((idx + 1) * OPERAND_SIZE_IN_BYTES)]
                    .try_into()
                    .unwrap(),
            )),
        })
        .collect::<Vec<_>>();

    Ok(result_values)
}

pub fn process_continuous_instructions(
    thread_context: &mut ThreadContext,
) -> Option<i32> /* terminate code */ {
    loop {
        let result = process_instruction(/*handler, */ thread_context);
        match result {
            HandleResult::Move(relate_offset_in_bytes) => {
                let next_instruction_offset =
                    thread_context.pc.instruction_address as isize + relate_offset_in_bytes;
                thread_context.pc.instruction_address = next_instruction_offset as usize;
            }
            HandleResult::Jump(return_pc) => {
                thread_context.pc.module_index = return_pc.module_index;
                thread_context.pc.function_internal_index = return_pc.function_internal_index;
                thread_context.pc.instruction_address = return_pc.instruction_address;
            }
            HandleResult::End(original_pc) => {
                thread_context.pc.module_index = original_pc.module_index;
                thread_context.pc.function_internal_index = original_pc.function_internal_index;
                thread_context.pc.instruction_address = original_pc.instruction_address;

                // Break the instruction processing loop.
                break None;
            }
            HandleResult::Terminate(terminate_code) => {
                // Break the instruction processing loop with terminate code.
                break Some(terminate_code);
            }
        }
    }
}

#[inline]
fn process_instruction(thread_context: &mut ThreadContext) -> HandleResult {
    let opcode_num = thread_context.get_opcode_num();
    let function = get_instruction_handler(opcode_num);
    function(/* handler, */ thread_context)
}
