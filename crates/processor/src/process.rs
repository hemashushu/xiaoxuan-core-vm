// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::{ProgramCounter, ThreadContext};
use anc_isa::{ForeignValue, OperandDataType, OPERAND_SIZE_IN_BYTES};

use crate::{
    handler::{HandleResult, Handler},
    FunctionEntryError, FunctionEntryErrorType,
};

// the `EXIT_CURRENT_HANDLER_LOOP_BIT` flag is used to indicated
// the current function stack is nested, for example,
// it's within the callback function call.
//
// if the value of the MSB of 'return module index' is '1',
// it indicates that the `process_continuous_instructions()` should be terminated
// instead of returns to caller.
pub const EXIT_CURRENT_HANDLER_LOOP_BIT: usize = 0x8000_0000;

// note:
// the value of 'function public index' includes the amount of imported functions,
// it equals to 'amount of imported functions' + 'function internal index'
pub fn process_function(
    handler: &Handler,
    thread_context: &mut ThreadContext,
    module_index: usize,
    function_public_index: usize,
    arguments: &[ForeignValue],
) -> Result<Vec<ForeignValue>, FunctionEntryError> {
    // reset the statck
    thread_context.stack.reset();

    // find the code start address
    let (target_module_index, function_internal_index) = thread_context
        .get_function_target_module_index_and_internal_index(module_index, function_public_index);
    let (type_index, local_variable_list_index, code_offset, local_variables_allocate_bytes) =
        thread_context
            .get_function_type_and_local_variable_list_index_and_code_offset_and_local_variables_allocate_bytes(
                target_module_index,
                function_internal_index,
            );

    let (params, results) = {
        let pars = thread_context.module_common_instances[target_module_index]
            .type_section
            .get_item_params_and_results(type_index);
        (pars.0.to_vec(), pars.1.to_vec())
    };

    // the number of arguments does not match the specified funcion.
    if arguments.len() != params.len() {
        return Err(FunctionEntryError::new(
            FunctionEntryErrorType::ParametersAmountMissmatch,
        ));
    }

    // push arguments
    // --------------
    //
    // the first value will be inserted first, and placed at the stack bottom side:
    //
    // array [0, 1, 2] -> |  2  | <-- high addr
    //                    |  1  |
    //                    |  0  |
    //                    \-----/ <-- low addr
    //
    // the reason why the arguments on the left side are pushed first onto the stack
    // is to make it easier to copy or move several consecutive arguments.
    //
    // e.g.
    // stack[low_addr] = arg 0
    // stack[low_addr + OPERAND_SIZE] = arg 1
    // ...
    // stack[high_addr] = arg n
    //
    // thus:
    // stack[low_addr:high_addr] = args from 0 to n
    //
    // note that for simplicity, this procdure does not check the data type of arguments.

    for value in arguments {
        match value {
            ForeignValue::U32(value) => thread_context.stack.push_i32_u(*value),
            ForeignValue::U64(value) => thread_context.stack.push_i64_u(*value),
            ForeignValue::F32(value) => thread_context.stack.push_f32(*value),
            ForeignValue::F64(value) => thread_context.stack.push_f64(*value),
        }
    }

    // create function statck frame
    thread_context.stack.create_frame(
        params.len() as u16,
        results.len() as u16,
        local_variable_list_index as u32,
        local_variables_allocate_bytes,
        Some(ProgramCounter {
            instruction_address: 0,
            function_internal_index: 0,

            // set MSB of 'return module index' to '1' to indicate that it's the END of the
            // current function call.
            module_index: EXIT_CURRENT_HANDLER_LOOP_BIT,
        }),
    );

    // set new PC
    thread_context.pc.module_index = target_module_index;
    thread_context.pc.function_internal_index = function_internal_index;
    thread_context.pc.instruction_address = code_offset;

    // start processing instructions
    if let Some(terminate_code) = process_continuous_instructions(handler, thread_context) {
        return Err(FunctionEntryError::new(FunctionEntryErrorType::Terminate(
            terminate_code,
        )));
    }

    // pop the results from the stack
    //
    // the values on the stack top will be poped first and became
    // the LAST element of the array
    //
    // |  2  | -> array [0, 1, 2]
    // |  1  |
    // |  0  |
    // \-----/
    let result_operands = thread_context
        .stack
        .pop_operands_without_bound_check(results.len());
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

                // break the instruction processing loop
                // break Ok(());
                break None;
            }
            HandleResult::Terminate(terminate_code) => {
                // break Err(HandlerError::new(HandleErrorType::Terminate(code))),
                // std::process::exit(terminate_code)
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
