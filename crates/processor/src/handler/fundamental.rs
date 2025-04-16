// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;

use super::{HandleResult, Handler};

pub fn nop(_handler: &Handler, _thread: &mut ThreadContext) -> HandleResult {
    HandleResult::Move(2)
}

pub fn imm_i32(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // note:
    // all i32 will be signed-extended to i64
    let value = thread_context.get_param_i32();
    thread_context.stack.push_i32_u(value);
    HandleResult::Move(8)
}

pub fn imm_i64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    let (low, high) = thread_context.get_param_i32_i32();
    let mut value: u64 = high as u64;
    value <<= 32;
    value |= low as u64;

    thread_context.stack.push_i64_u(value);
    HandleResult::Move(12)
}

pub fn imm_f32(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    let i32_value = thread_context.get_param_i32();
    let value = f32::from_bits(i32_value);

    thread_context.stack.push_f32(value);
    HandleResult::Move(8)
}

pub fn imm_f64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    let (low, high) = thread_context.get_param_i32_i32();

    let mut bytes = [0u8; 8];
    {
        let (p0, p1) = bytes.split_at_mut(4);
        p0.copy_from_slice(&low.to_le_bytes());
        p1.copy_from_slice(&high.to_le_bytes());
    }

    let value = f64::from_le_bytes(bytes);

    thread_context.stack.push_f64(value);
    HandleResult::Move(12)
}

#[cfg(test)]
mod tests {
    use crate::{
        handler::Handler, in_memory_program_source::InMemoryProgramSource,
        process::process_function,
    };

    use anc_context::program_source::ProgramSource;
    use anc_image::{
        bytecode_writer::BytecodeWriterHelper,
        utils::helper_build_module_binary_with_single_function,
    };
    use anc_isa::{opcode::Opcode, ForeignValue, OperandDataType};

    #[test]
    fn test_handler_fundamental_nop() {
        // () -> ()
        let code0 = BytecodeWriterHelper::new()
            .append_opcode(Opcode::nop)
            .append_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", format_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function(
            &[], // params
            &[], // results
            &[], // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        assert!(result0.is_ok());
    }

    #[test]
    fn test_handler_fundamental_immediate_integer() {
        // () -> (i32, i64, i32, i64)
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 23)
            .append_opcode_i64(Opcode::imm_i64, 0x29313741_43475359u64)
            .append_opcode_i32(Opcode::imm_i32, (-223_i32) as u32)
            .append_opcode_i64(Opcode::imm_i64, (-227_i32) as u64)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[], // params
            &[
                OperandDataType::I32,
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I64,
            ], // results
            &[], // local variables
            code0,
        );

        let interpreter = Handler::new();
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&interpreter, &mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U32(23),
                ForeignValue::U64(0x29313741_43475359u64),
                ForeignValue::U32((-223_i32) as u32),
                ForeignValue::U64((-227_i64) as u64)
            ]
        );
    }

    #[test]
    fn test_handler_fundamental_immediate_float() {
        // () -> (f32, f64, f32, f64)
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_f32(Opcode::imm_f32, std::f32::consts::PI)
            .append_opcode_f64(Opcode::imm_f64, std::f64::consts::SQRT_2)
            .append_opcode_f32(Opcode::imm_f32, -std::f32::consts::E)
            .append_opcode_f64(Opcode::imm_f64, -std::f64::consts::FRAC_PI_6)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[], // params
            &[
                OperandDataType::F32,
                OperandDataType::F64,
                OperandDataType::F32,
                OperandDataType::F64,
            ], // results
            &[], // local variables
            code0,
        );

        let interpreter = Handler::new();
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&interpreter, &mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::F32(std::f32::consts::PI),
                ForeignValue::F64(std::f64::consts::SQRT_2),
                ForeignValue::F32(-std::f32::consts::E),
                ForeignValue::F64(-std::f64::consts::FRAC_PI_6),
            ]
        );
    }
}
