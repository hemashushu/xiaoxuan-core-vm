// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;

use crate::TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS;

use super::{HandleResult, Handler};

const DATA_LENGTH_IN_BYTES_64_BIT: usize = 8;
const DATA_LENGTH_IN_BYTES_32_BIT: usize = 4;
const DATA_LENGTH_IN_BYTES_16_BIT: usize = 2;
const DATA_LENGTH_IN_BYTES_8_BIT: usize = 1;

pub fn local_load_i64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 local_variable_index:i32) -> i64
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();

    // there are two approachs to transfer data from memory to stack, one
    // is to read data (integer or floating point number) from memory to
    // a temporary variable, and then push the variable onto the stack, e.g.
    //
    // ```rust
    // let num = stack.read_u64(data_address, offset);
    // stack.push_u64(num);
    // ```
    //
    // the another approach is using "memory copy",
    // which has a higher efficiency because it eliminates data conversion,
    // the second method is adopted here.

    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_64_BIT,
    );
    thread_context.stack.read_i64(data_address, 0, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load_i32_s(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 offset_bytes:i16 local_variable_index:i16)
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_32_BIT,
    );
    thread_context.stack.read_i32_s(data_address, 0, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load_i32_u(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 offset_bytes:i16 local_variable_index:i16)
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_32_BIT,
    );
    thread_context.stack.read_i32_u(data_address, 0, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load_i16_s(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 offset_bytes:i16 local_variable_index:i16)
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_16_BIT,
    );
    thread_context.stack.read_i16_s(data_address, 0, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load_i16_u(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 offset_bytes:i16 local_variable_index:i16)
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_16_BIT,
    );
    thread_context.stack.read_i16_u(data_address, 0, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load_i8_s(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 offset_bytes:i16 local_variable_index:i16)
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_8_BIT,
    );
    thread_context.stack.read_i8_s(data_address, 0, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load_i8_u(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 offset_bytes:i16 local_variable_index:i16)
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_8_BIT,
    );
    thread_context.stack.read_i8_u(data_address, 0, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load_f32(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 offset_bytes:i16 local_variable_index:i16)
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_32_BIT,
    );

    match thread_context.stack.read_f32(data_address, 0, dst_ptr) {
        Ok(_) => HandleResult::Move(8),
        Err(_) => HandleResult::Terminate(TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS),
    }
}

pub fn local_load_f64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 offset_bytes:i16 local_variable_index:i16)
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_64_BIT,
    );

    match thread_context.stack.read_f64(data_address, 0, dst_ptr) {
        Ok(_) => HandleResult::Move(8),
        Err(_) => HandleResult::Terminate(TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS),
    }
}

pub fn local_store_i64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 offset_bytes:i16 local_variable_index:i16)
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_64_BIT,
    );
    thread_context.stack.write_i64(src_ptr, data_address, 0);

    HandleResult::Move(8)
}

pub fn local_store_i32(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 offset_bytes:i16 local_variable_index:i16)
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_32_BIT,
    );
    thread_context.stack.write_i32(src_ptr, data_address, 0);

    HandleResult::Move(8)
}

pub fn local_store_i16(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 offset_bytes:i16 local_variable_index:i16)
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_16_BIT,
    );
    thread_context.stack.write_i16(src_ptr, data_address, 0);

    HandleResult::Move(8)
}

pub fn local_store_i8(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param layers:i16 offset_bytes:i16 local_variable_index:i16)
    let (layers, local_variable_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    let data_address = thread_context.get_local_variable_start_address(
        layers,
        local_variable_index as usize,
        DATA_LENGTH_IN_BYTES_8_BIT,
    );
    thread_context.stack.write_i8(src_ptr, data_address, 0);

    HandleResult::Move(8)
}

// all testing here are ignore the `layers` parameter because it relies on
// the instruction `block`.
// the `layers` will be tested on the module 'interpreter/control_flow'.
#[cfg(test)]
mod tests {
    use anc_context::program_source::ProgramSource;
    use anc_image::{
        bytecode_writer::BytecodeWriterHelper,
        utils::helper_build_module_binary_with_single_function,
    };
    use anc_isa::{opcode::Opcode, ForeignValue, OperandDataType};

    use crate::{
        handler::Handler, in_memory_program_source::InMemoryProgramSource,
        process::process_function,
    };

    #[test]
    fn test_handler_local_load_and_store() {
        // args (also local vars): 0, 1
        // data type: f32, f64
        //
        //       |low address                                         high address|
        // local |                                                                |
        // index |2             3      4      5                         6      7  |
        //  type |i32-------|  |f32|  |f64|  |i64-------------------|  |i64|  |i64|
        //
        //  data aa bb cc dd   f32    f64    11 13 17 19 23 29 31 37    ^        ^
        //       |imm          |ld32  |ld64  ^imm                       |ld_i32u |ld_i32s
        //       |store32      |st32  |st64  |store64                   |store64 |store64
        //       |step0        |step1 |step2 |step3                     |step4   |step5
        //       |             |      |      |                          |        |
        //       |                                                      |        |
        //       \---------------->---------------------->--------------/--------/
        //
        //       aa bb cc dd   f32    f64    11 13 17 19 23 29 31 37    i64     i64
        //       |              |      |      |                          |       |
        //       |           loadf32   |      |                          |       |
        //       |                  loadf64   |                          |       |
        //    loadi32u                      loadi64                    loadi64   |
        //    loadi32s                                                        loadi64
        //
        // (f32, f64) -> (i32,i32, f32,f64, i64,i64,i64)

        let code0 = BytecodeWriterHelper::new()
            // step 0
            .append_opcode_i32(Opcode::imm_i32, 0xaabbccdd)
            .append_opcode_i16_i32(Opcode::local_store_i32, 0, 2)
            // step 1
            .append_opcode_i16_i32(Opcode::local_load_f32, 0, 0)
            .append_opcode_i16_i32(Opcode::local_store_f32, 0, 3)
            // step 2
            .append_opcode_i16_i32(Opcode::local_load_f64, 0, 1)
            .append_opcode_i16_i32(Opcode::local_store_f64, 0, 4)
            // step 3
            .append_opcode_i64(Opcode::imm_i64, 0x11131719_23293137)
            .append_opcode_i16_i32(Opcode::local_store_i64, 0, 5)
            // step 4
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 2)
            .append_opcode_i16_i32(Opcode::local_store_i64, 0, 6)
            // step 5
            .append_opcode_i16_i32(Opcode::local_load_i32_s, 0, 2)
            .append_opcode_i16_i32(Opcode::local_store_i64, 0, 7)
            // group 0
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 2)
            .append_opcode_i16_i32(Opcode::local_load_i32_s, 0, 2)
            // group 1
            .append_opcode_i16_i32(Opcode::local_load_f32, 0, 3)
            .append_opcode_i16_i32(Opcode::local_load_f64, 0, 4)
            // group 2
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 5)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 6)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 7)
            //
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

        let handler = Handler::new();
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &handler,
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
        // tesing: load i32 variable with `local_load_i64` instruction

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[],                     // params
            &[],                     // results
            &[OperandDataType::I32], // local variables
            code0,
        );

        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let handler = Handler::new();
            let resource0 = InMemoryProgramSource::new(vec![binary0]);
            let process_context0 = resource0.create_process_context().unwrap();
            let mut thread_context0 = process_context0.create_thread_context();
            // err: data length exceeded
            let _ = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_handler_local_bounds_check_index_out_of_range() {
        // testing: store a i32 data to local variable index 2 (which does not exist)
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 11)
            .append_opcode_i16_i32(Opcode::local_store_i32, 0, 2)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[],                     // params
            &[],                     // results
            &[OperandDataType::I32], // local variables
            code0,
        );

        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let handler = Handler::new();
            let resource0 = InMemoryProgramSource::new(vec![binary0]);
            let process_context0 = resource0.create_process_context().unwrap();

            let mut thread_context0 = process_context0.create_thread_context();
            // err: access non-exist index local variable
            let _ = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }
}
