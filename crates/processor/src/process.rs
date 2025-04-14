// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;
use anc_isa::{ForeignValue, OperandDataType, OPERAND_SIZE_IN_BYTES};
use anc_stack::ProgramCounter;

use crate::{
    handler::{HandleResult, Handler},
    ProcessorError, ProcessorErrorType,
};

// the `EXIT_CURRENT_HANDLER_LOOP_BIT` flag is used to indicated
// the current function is the last function of "calling path" (each
// callback function will generate a new calling path).
//
// if the current function is the last function of "calling path",
// the `process_continuous_instructions()` should be terminated.
pub const EXIT_CURRENT_HANDLER_LOOP_BIT: usize = 0x8000_0000;

pub fn process_function(
    handler: &Handler,
    thread_context: &mut ThreadContext,
    module_index: usize,
    function_public_index: usize,
    arguments: &[ForeignValue],
) -> Result<Vec<ForeignValue>, ProcessorError> {
    // reset the statck
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

    // the number of arguments does not match the specified funcion.
    if arguments.len() != params.len() {
        return Err(ProcessorError::new(
            ProcessorErrorType::ParametersAmountMissmatch,
        ));
    }

    // push arguments
    // --------------
    //
    // the first value will be inserted first, and placed at the stack bottom side:
    //
    // ```diagram
    // array [0, 1, 2] -> |  2  | <-- high addr
    //                    |  1  |
    //                    |  0  |
    //                    \-----/ <-- low addr
    // ```
    //
    // the reason why the arguments on the left side are pushed first onto the stack
    // is because the statck bottom is low address, thus make it easier to copy or move several consecutive arguments
    // to an array.
    //
    // e.g.
    //
    // ```c
    // stack[low_addr] = arg0
    // stack[low_addr + OPERAND_SIZE] = arg1
    // ...
    // stack[high_addr] = argn
    //
    // memory_copy(stack, array, N)
    // ```
    //
    // for simplicity, this procdure does not check the data type of arguments.

    for value in arguments {
        match value {
            ForeignValue::U32(value) => thread_context.stack.push_i32_u(*value),
            ForeignValue::U64(value) => thread_context.stack.push_i64_u(*value),
            ForeignValue::F32(value) => thread_context.stack.push_f32(*value),
            ForeignValue::F64(value) => thread_context.stack.push_f64(*value),
        }
    }

    // create function statck frame
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

                // set MSB of 'return module index' to '1' to indicate that it's the END of the
                // "function call path".
                module_index: EXIT_CURRENT_HANDLER_LOOP_BIT,
            }),
        )
        .map_err(|_| ProcessorError::new(ProcessorErrorType::StackOverflow))?;

    // set new PC
    thread_context.pc.module_index = target_function_object.module_index;
    thread_context.pc.function_internal_index = target_function_object.function_internal_index;
    thread_context.pc.instruction_address = function_info.code_offset;

    // start processing instructions
    if let Some(terminate_code) = process_continuous_instructions(handler, thread_context) {
        return Err(ProcessorError::new(ProcessorErrorType::Terminate(
            terminate_code,
        )));
    }

    // pop results
    // -----------
    //
    // similar to the pushing arguments,
    // the values on the stack top should be poped first and became
    // the LAST element of the result array.
    //
    // ```diagram
    // |  2  | -> array [0, 1, 2]
    // |  1  |
    // |  0  |
    // \-----/
    // ```

    // don't use the `pop_xxx` functions to pop the results,
    // because the `pop_xxx` requires a stack frame to work, however,
    // the stack has no frame after the entry function is finish.
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
    handler: &Handler,
    thread_context: &mut ThreadContext,
) -> Option<i32> /* terminate code */ {
    loop {
        let result = process_instruction(handler, thread_context);
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

                // break the instruction processing loop.
                break None;
            }
            HandleResult::Terminate(terminate_code) => {
                // break the instruction processing loop with terminate code.
                break Some(terminate_code);
            }
        }
    }
}

#[inline]
fn process_instruction(handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    let opcode_num = thread_context.get_opcode_num();
    let function = handler.handlers[opcode_num as usize]; //  unsafe { &INTERPRETERS[opcode_num as usize] };
    function(handler, thread_context)
}
