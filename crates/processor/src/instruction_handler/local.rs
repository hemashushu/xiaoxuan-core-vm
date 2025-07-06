// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;

use crate::TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS;

use super::HandleResult;

const DATA_LENGTH_IN_BYTES_64_BIT: usize = 8;
const DATA_LENGTH_IN_BYTES_32_BIT: usize = 4;
const DATA_LENGTH_IN_BYTES_16_BIT: usize = 2;
const DATA_LENGTH_IN_BYTES_8_BIT: usize = 1;

pub fn local_load_i64(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (params: layers: i16, local_variable_index: i32) -> i64
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();

    // There are two approaches to transfer data from memory to the stack:
    // 1. Read data (integer or floating-point number) from memory into a temporary variable,
    //    and then push the variable onto the stack. For example:
    //
    //    ```rust
    //    let num = stack.read_u64(data_address, offset);
    //    stack.push_u64(num);
    //    ```
    //
    // 2. Use "memory copy," which is more efficient as it avoids data conversion.
    //    This method is used here.

    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_64_BIT,
    );
    thread_context.stack.read_i64(data_address, 0, dst_ptr as *mut u64);

    HandleResult::Move(8)
}

pub fn local_load_i32_s(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 local_variable_index:i32) -> i32
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_32_BIT,
    );
    thread_context.stack.read_i32_s_to_i64(data_address, 0, dst_ptr as *mut i64);

    HandleResult::Move(8)
}

pub fn local_load_i32_u(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 local_variable_index:i32) -> i32
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_32_BIT,
    );
    thread_context.stack.read_i32_u_to_u64(data_address, 0, dst_ptr as *mut u64);

    HandleResult::Move(8)
}

pub fn local_load_i16_s(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 local_variable_index:i32) -> i16
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_16_BIT,
    );
    thread_context.stack.read_i16_s_to_i64(data_address, 0, dst_ptr as *mut i64);

    HandleResult::Move(8)
}

pub fn local_load_i16_u(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 local_variable_index:i32) -> i16
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_16_BIT,
    );
    thread_context.stack.read_i16_u_to_u64(data_address, 0, dst_ptr as *mut u64);

    HandleResult::Move(8)
}

pub fn local_load_i8_s(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 local_variable_index:i32) -> i8
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_8_BIT,
    );
    thread_context.stack.read_i8_s_to_i64(data_address, 0, dst_ptr as *mut i64);

    HandleResult::Move(8)
}

pub fn local_load_i8_u(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 local_variable_index:i32) -> i8
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_8_BIT,
    );
    thread_context.stack.read_i8_u_to_u64(data_address, 0, dst_ptr as *mut u64);

    HandleResult::Move(8)
}

pub fn local_load_f32(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 local_variable_index:i32) -> f32
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_32_BIT,
    );

    // Handle potential errors when reading floating-point data.
    match thread_context.stack.read_f32(data_address, 0, dst_ptr as *mut f32) {
        Ok(_) => HandleResult::Move(8),
        Err(_) => HandleResult::Terminate(TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS),
    }
}

pub fn local_load_f64(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 local_variable_index:i32) -> f64
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_64_BIT,
    );

    match thread_context.stack.read_f64(data_address, 0, dst_ptr as *mut f64) {
        Ok(_) => HandleResult::Move(8),
        Err(_) => HandleResult::Terminate(TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS),
    }
}

pub fn local_store_i64(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 local_variable_index:i32) (operand value:i64) -> (remain_values)
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_64_BIT,
    );
    thread_context.stack.write_i64(src_ptr, data_address, 0);

    HandleResult::Move(8)
}

pub fn local_store_i32(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 local_variable_index:i32) (operand value:i32) -> (remain_values)
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_32_BIT,
    );
    thread_context.stack.write_i32(src_ptr, data_address, 0);

    HandleResult::Move(8)
}

pub fn local_store_i16(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 local_variable_index:i32) (operand value:i32) -> (remain_values)
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_16_BIT,
    );
    thread_context.stack.write_i16(src_ptr, data_address, 0);

    HandleResult::Move(8)
}

pub fn local_store_i8(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 local_variable_index:i32) (operand value:i32) -> (remain_values)
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_8_BIT,
    );
    thread_context.stack.write_i8(src_ptr, data_address, 0);

    HandleResult::Move(8)
}

// All tests here ignore the `layers` parameter because it depends on
// the `block` instruction.
// The `layers` parameter will be tested in the module `interpreter/control_flow`.
#[cfg(test)]
mod tests {
    use anc_context::program_source::ProgramSource;
    use anc_image::{
        bytecode_writer::BytecodeWriterHelper,
        utils::helper_build_module_binary_with_single_function,
    };
    use anc_isa::{opcode::Opcode, ForeignValue, OperandDataType};

    use crate::{
        in_memory_program_source::InMemoryProgramSource,
        process::process_function, ProcessorError, ProcessorErrorType,
        TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS,
    };

    #[test]
    fn test_handler_local_load_and_store() {
        // Test case for loading and storing local variables.
        // Arguments (also local variables): 0, 1
        // Data types: f32, f64
        //
        //       |low address                                         high address|
        // local |                                                                |
        // index |2             3      4      5                         6      7  |
        //  type |i32-------|  |f32|  |f64|  |i64-------------------|  |i64|  |i64|
        //
        // write aa bb cc dd   f32    f64    11 13 17 19 23 29 31 37    ^        ^
        //       |imm          |ld32  |ld64  ^imm                       |ld_i32u |ld_i32s
        //       |store32      |st32  |st64  |store64                   |store64 |store64
        //       |step0        |step1 |step2 |step3                     |step4   |step5
        //       |             |      |      |                          |        |
        //       |                                                      |        |
        //       \---------------->---------------------->--------------/--------/
        //
        //  read aa bb cc dd   f32    f64    11 13 17 19 23 29 31 37    i64     i64
        //       |              |      |      |                          |       |
        //       |           loadf32   |      |                          |       |
        //       |                  loadf64   |                          |       |
        //    loadi32u                      loadi64                    loadi64   |
        //    loadi32s                                                        loadi64
        //
        // (f32, f64) -> (i32,i32, f32,f64, i64,i64,i64)

        let code0 = BytecodeWriterHelper::new()
            // Step 0: Store an i32 value into local variable 2.
            .append_opcode_i32(Opcode::imm_i32, 0xaabbccdd)
            .append_opcode_i16_i32(Opcode::local_store_i32, 0, 2)
            // Step 1: Load f32 from local variable 0 and store it in variable 3.
            .append_opcode_i16_i32(Opcode::local_load_f32, 0, 0)
            .append_opcode_i16_i32(Opcode::local_store_f32, 0, 3)
            // Step 2: Load f64 from local variable 1 and store it in variable 4.
            .append_opcode_i16_i32(Opcode::local_load_f64, 0, 1)
            .append_opcode_i16_i32(Opcode::local_store_f64, 0, 4)
            // Step 3: Store an i64 value into local variable 5.
            .append_opcode_i64(Opcode::imm_i64, 0x11131719_23293137)
            .append_opcode_i16_i32(Opcode::local_store_i64, 0, 5)
            // Step 4: Load i32 (unsigned) from variable 2 and store it in variable 6.
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 2)
            .append_opcode_i16_i32(Opcode::local_store_i64, 0, 6)
            // Step 5: Load i32 (signed) from variable 2 and store it in variable 7.
            .append_opcode_i16_i32(Opcode::local_load_i32_s, 0, 2)
            .append_opcode_i16_i32(Opcode::local_store_i64, 0, 7)
            // Group 0: Load i32 (unsigned and signed) from variable 2.
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 2)
            .append_opcode_i16_i32(Opcode::local_load_i32_s, 0, 2)
            // Group 1: Load f32 and f64 from variables 3 and 4.
            .append_opcode_i16_i32(Opcode::local_load_f32, 0, 3)
            .append_opcode_i16_i32(Opcode::local_load_f64, 0, 4)
            // Group 2: Load i64 from variables 5, 6, and 7.
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 5)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 6)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 7)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[OperandDataType::F32, OperandDataType::F64], // params
            &[
                // group 0
                OperandDataType::I32,
                OperandDataType::I32,
                // group 1
                OperandDataType::F32,
                OperandDataType::F64,
                // group 2
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
            ], // results
            &[
                OperandDataType::I32,
                OperandDataType::F32,
                OperandDataType::F64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
            ], // local variables
            code0,
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
            &[
                ForeignValue::F32(std::f32::consts::PI),
                ForeignValue::F64(std::f64::consts::E),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::U32(0xaabbccdd),
                ForeignValue::U32(0xaabbccdd),
                // group 1
                ForeignValue::F32(std::f32::consts::PI),
                ForeignValue::F64(std::f64::consts::E),
                // group 2
                ForeignValue::U64(0x11131719_23293137),
                ForeignValue::U64(0x00000000_aabbccdd),
                ForeignValue::U64(0xffffffff_aabbccdd),
            ]
        );
    }

    #[test]
    fn test_handler_local_bounds_check_data_length_exceeded() {
        // Testing: Attempt to load an `i32` variable using the `local_load_i64` instruction.
        // This should fail because the data length exceeds the expected size.

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // Load i64 from local variable 0.
            .append_opcode(Opcode::end) // End of bytecode.
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[],                     // No parameters.
            &[],                     // No results.
            &[OperandDataType::I32], // Local variables: one `i32`.
            code0,
        );

        let prev_hook = std::panic::take_hook(); // Silence panic output.
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            /* let handler = Handler::new(); */
            let resource0 = InMemoryProgramSource::new(vec![binary0]);
            let process_context0 = resource0.create_process_context().unwrap();
            let mut thread_context0 = process_context0.create_thread_context();
            // Error: Attempting to load `i64` from an `i32` variable (data length exceeded).
            let _ = process_function( /* &handler, */ &mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook); // Restore the original panic hook.

        assert!(result.is_err()); // Assert that the operation results in an error.
    }

    #[test]
    fn test_handler_local_bounds_check_index_out_of_range() {
        // Testing: Attempt to store an `i32` value into a non-existent local variable (index 2).
        // This should fail because the index is out of range.

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 11) // Push the value `11` onto the stack.
            .append_opcode_i16_i32(Opcode::local_store_i32, 0, 2) // Attempt to store it in local variable index 2.
            .append_opcode(Opcode::end) // End of bytecode.
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[],                     // No parameters.
            &[],                     // No results.
            &[OperandDataType::I32], // Local variables: one `i32`.
            code0,
        );

        let prev_hook = std::panic::take_hook(); // Silence panic output.
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            /* let handler = Handler::new(); */
            let resource0 = InMemoryProgramSource::new(vec![binary0]);
            let process_context0 = resource0.create_process_context().unwrap();

            let mut thread_context0 = process_context0.create_thread_context();
            // Error: Attempting to access a non-existent local variable (index out of range).
            let _ = process_function( /* &handler, */ &mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook); // Restore the original panic hook.

        assert!(result.is_err()); // Assert that the operation results in an error.
    }

    #[test]
    fn test_handler_local_unsupported_floating_point_variant() {
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::local_load_f32, 0, 0) // Attempt to store it in local variable index 2.
            .append_opcode(Opcode::end) // End of bytecode.
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[OperandDataType::F32], // No parameters.
            &[OperandDataType::F32],
            &[OperandDataType::F32],
            code0,
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();

        let mut thread_context0 = process_context0.create_thread_context();
        // Error: Attempting to access an unsupported floating-point variant.
        let result = process_function(
            /* &handler, */
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::F32(std::f32::NAN)],
        );

        assert!(matches!(
            result,
            Err(ProcessorError {
                error_type: ProcessorErrorType::Terminate(
                    TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS
                )
            })
        ));
    }
}
