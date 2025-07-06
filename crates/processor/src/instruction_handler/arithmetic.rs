// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;
use anc_memory::MemoryError;

use crate::TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS;

use super::HandleResult;

pub fn add_i32(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand left:i32 right:i32) -> i32
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left.wrapping_add(right));
    HandleResult::Move(2)
}

pub fn sub_i32(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left.wrapping_sub(right));
    HandleResult::Move(2)
}

pub fn add_imm_i32(thread_context: &mut ThreadContext) -> HandleResult {
    let imm = thread_context.get_param_i16();
    let value = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, value.wrapping_add(imm as u32));
    HandleResult::Move(4)
}

pub fn sub_imm_i32(thread_context: &mut ThreadContext) -> HandleResult {
    let imm = thread_context.get_param_i16();
    let value = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, value.wrapping_sub(imm as u32));
    HandleResult::Move(4)
}

pub fn mul_i32(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left.wrapping_mul(right));
    HandleResult::Move(2)
}

pub fn div_i32_s(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_s(thread_context);
    store_i32_s(thread_context, left / right);
    HandleResult::Move(2)
}

pub fn div_i32_u(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left / right);
    HandleResult::Move(2)
}

pub fn rem_i32_s(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_s(thread_context);
    store_i32_s(thread_context, left % right);
    HandleResult::Move(2)
}

pub fn rem_i32_u(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left % right);
    HandleResult::Move(2)
}

pub fn add_i64(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left.wrapping_add(right));
    HandleResult::Move(2)
}

pub fn sub_i64(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left.wrapping_sub(right));
    HandleResult::Move(2)
}

pub fn add_imm_i64(thread_context: &mut ThreadContext) -> HandleResult {
    let imm = thread_context.get_param_i16();
    let value = load_operand_i64_u(thread_context);
    store_i64_u(thread_context, value.wrapping_add(imm as u64));
    HandleResult::Move(4)
}

pub fn sub_imm_i64(thread_context: &mut ThreadContext) -> HandleResult {
    let imm = thread_context.get_param_i16();
    let value = load_operand_i64_u(thread_context);
    store_i64_u(thread_context, value.wrapping_sub(imm as u64));
    HandleResult::Move(4)
}

pub fn mul_i64(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left.wrapping_mul(right));
    HandleResult::Move(2)
}

pub fn div_i64_s(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_s(thread_context);
    store_i64_s(thread_context, left / right);
    HandleResult::Move(2)
}

pub fn div_i64_u(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left / right);
    HandleResult::Move(2)
}

pub fn rem_i64_s(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_s(thread_context);
    store_i64_s(thread_context, left % right);
    HandleResult::Move(2)
}

pub fn rem_i64_u(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left % right);
    HandleResult::Move(2)
}

pub fn add_f32(thread_context: &mut ThreadContext) -> HandleResult {
    match load_operands_f32(thread_context) {
        Ok((left, right)) => {
            store_f32(thread_context, left + right);
            HandleResult::Move(2)
        }
        Err(_) => HandleResult::Terminate(TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS),
    }
}

pub fn sub_f32(thread_context: &mut ThreadContext) -> HandleResult {
    match load_operands_f32(thread_context) {
        Ok((left, right)) => {
            store_f32(thread_context, left - right);
            HandleResult::Move(2)
        }
        Err(_) => HandleResult::Terminate(TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS),
    }
}

pub fn mul_f32(thread_context: &mut ThreadContext) -> HandleResult {
    match load_operands_f32(thread_context) {
        Ok((left, right)) => {
            store_f32(thread_context, left * right);
            HandleResult::Move(2)
        }
        Err(_) => HandleResult::Terminate(TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS),
    }
}

pub fn div_f32(thread_context: &mut ThreadContext) -> HandleResult {
    match load_operands_f32(thread_context) {
        Ok((left, right)) => {
            store_f32(thread_context, left / right);
            HandleResult::Move(2)
        }
        Err(_) => HandleResult::Terminate(TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS),
    }
}

pub fn add_f64(thread_context: &mut ThreadContext) -> HandleResult {
    match load_operands_f64(thread_context) {
        Ok((left, right)) => {
            store_f64(thread_context, left + right);
            HandleResult::Move(2)
        }
        Err(_) => HandleResult::Terminate(TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS),
    }
}

pub fn sub_f64(thread_context: &mut ThreadContext) -> HandleResult {
    match load_operands_f64(thread_context) {
        Ok((left, right)) => {
            store_f64(thread_context, left - right);
            HandleResult::Move(2)
        }
        Err(_) => HandleResult::Terminate(TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS),
    }
}

pub fn mul_f64(thread_context: &mut ThreadContext) -> HandleResult {
    match load_operands_f64(thread_context) {
        Ok((left, right)) => {
            store_f64(thread_context, left * right);
            HandleResult::Move(2)
        }
        Err(_) => HandleResult::Terminate(TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS),
    }
}

pub fn div_f64(thread_context: &mut ThreadContext) -> HandleResult {
    match load_operands_f64(thread_context) {
        Ok((left, right)) => {
            store_f64(thread_context, left / right);
            HandleResult::Move(2)
        }
        Err(_) => HandleResult::Terminate(TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS),
    }
}

#[inline]
fn load_operand_i32_u(thread_context: &mut ThreadContext) -> u32 {
    thread_context.stack.pop_i32_u()
}

#[inline]
fn load_operand_i64_u(thread_context: &mut ThreadContext) -> u64 {
    thread_context.stack.pop_i64_u()
}

#[inline]
fn load_operands_i32_s(thread_context: &mut ThreadContext) -> (i32, i32) {
    let right = thread_context.stack.pop_i32_s();
    let left = thread_context.stack.pop_i32_s();
    (left, right)
}

#[inline]
fn load_operands_i32_u(thread_context: &mut ThreadContext) -> (u32, u32) {
    let right = thread_context.stack.pop_i32_u();
    let left = thread_context.stack.pop_i32_u();
    (left, right)
}

#[inline]
fn load_operands_i64_s(thread_context: &mut ThreadContext) -> (i64, i64) {
    let right = thread_context.stack.pop_i64_s();
    let left = thread_context.stack.pop_i64_s();
    (left, right)
}

#[inline]
fn load_operands_i64_u(thread_context: &mut ThreadContext) -> (u64, u64) {
    let right = thread_context.stack.pop_i64_u();
    let left = thread_context.stack.pop_i64_u();
    (left, right)
}

#[inline]
fn load_operands_f32(thread_context: &mut ThreadContext) -> Result<(f32, f32), MemoryError> {
    let right = thread_context.stack.pop_f32()?;
    let left = thread_context.stack.pop_f32()?;
    Ok((left, right))
}

#[inline]
fn load_operands_f64(thread_context: &mut ThreadContext) -> Result<(f64, f64), MemoryError> {
    let right = thread_context.stack.pop_f64()?;
    let left = thread_context.stack.pop_f64()?;
    Ok((left, right))
}

#[inline]
fn store_i32_s(thread_context: &mut ThreadContext, v: i32) {
    thread_context.stack.push_i32_s(v);
}

#[inline]
fn store_i32_u(thread_context: &mut ThreadContext, v: u32) {
    thread_context.stack.push_i32_u(v);
}

#[inline]
fn store_i64_s(thread_context: &mut ThreadContext, v: i64) {
    thread_context.stack.push_i64_s(v);
}

#[inline]
fn store_i64_u(thread_context: &mut ThreadContext, v: u64) {
    thread_context.stack.push_i64_u(v);
}

#[inline]
fn store_f32(thread_context: &mut ThreadContext, v: f32) {
    thread_context.stack.push_f32(v);
}

#[inline]
fn store_f64(thread_context: &mut ThreadContext, v: f64) {
    thread_context.stack.push_f64(v);
}

#[cfg(test)]
mod tests {
    use crate::{in_memory_program_source::InMemoryProgramSource, process::process_function};

    use anc_context::program_source::ProgramSource;
    use anc_image::{
        bytecode_writer::BytecodeWriterHelper,
        utils::helper_build_module_binary_with_single_function,
    };
    use anc_isa::{opcode::Opcode, ForeignValue, OperandDataType};

    #[test]
    fn test_handler_arithmetic_i32() {
        // numbers:
        //   - 0: 11
        //   - 1: 211
        //   - 2: -13

        // arithemtic:
        //   group 0:
        //   - add   0 1      -> 222
        //   - sub   1 0      -> 200
        //   - sub   0 1      -> -200
        //   - mul   0 1      -> 2321
        //
        //   group 1:
        //   - div_s 1 2      -> -16
        //   - div_u 1 2      -> 0
        //   - div_s 2 1      -> 0
        //   - div_u 2 1      -> 20355295 (= 4294967283/211)
        //   - rem_s 1 2      -> 3
        //   - rem_u 2 1      -> 38
        //
        //   group 2:
        //   - inc   0 imm:3     -> 14
        //   - dec   0 imm:3     -> 8
        //   - inc   2 imm:3     -> -10
        //   - dec   2 imm:3     -> -16
        //
        //   group 3:
        //   - add 0xffff_ffff 0x2  -> 0x1                  ;; -1 + 2 = 1
        //   - mul 0xf0e0_d0c0 0x2  -> 0xf0e0_d0c0 << 1
        //   - inc 0xffff_ffff 0x2  -> 0x1
        //   - dec 0x1         0x2  -> 0xffff_ffff
        //
        // (i32 i32 i32) -> (i32 i32 i32 i32  i32 i32 i32 i32 i32 i32  i32 i32 i32 i32  i32 i32 i32 i32)

        // note of the 'remainder':
        // (211 % -13) = 3
        //  ^      ^
        //  |      |divisor
        //  |dividend <--------- the result always takes the sign of the dividend.

        let code0 = BytecodeWriterHelper::new()
            // group 0
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode(Opcode::add_i32)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode(Opcode::sub_i32)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode(Opcode::sub_i32)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode(Opcode::mul_i32)
            // group 1
            .append_opcode_i16_i32(Opcode::local_load_i32_s, 0, 1)
            .append_opcode_i16_i32(Opcode::local_load_i32_s, 0, 2)
            .append_opcode(Opcode::div_i32_s)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 2)
            .append_opcode(Opcode::div_i32_u)
            .append_opcode_i16_i32(Opcode::local_load_i32_s, 0, 2)
            .append_opcode_i16_i32(Opcode::local_load_i32_s, 0, 1)
            .append_opcode(Opcode::div_i32_s)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 2)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode(Opcode::div_i32_u)
            .append_opcode_i16_i32(Opcode::local_load_i32_s, 0, 1)
            .append_opcode_i16_i32(Opcode::local_load_i32_s, 0, 2)
            .append_opcode(Opcode::rem_i32_s)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 2)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1)
            .append_opcode(Opcode::rem_i32_u)
            // group 2
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i16(Opcode::add_imm_i32, 3)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
            .append_opcode_i16(Opcode::sub_imm_i32, 3)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 2)
            .append_opcode_i16(Opcode::add_imm_i32, 3)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 2)
            .append_opcode_i16(Opcode::sub_imm_i32, 3)
            // group 3
            .append_opcode_i32(Opcode::imm_i32, 0xffff_ffff)
            .append_opcode_i32(Opcode::imm_i32, 0x2)
            .append_opcode(Opcode::add_i32)
            .append_opcode_i32(Opcode::imm_i32, 0xf0e0_d0c0)
            .append_opcode_i32(Opcode::imm_i32, 0x2)
            .append_opcode(Opcode::mul_i32)
            .append_opcode_i32(Opcode::imm_i32, 0xffff_ffff)
            .append_opcode_i16(Opcode::add_imm_i32, 2)
            .append_opcode_i32(Opcode::imm_i32, 0x1)
            .append_opcode_i16(Opcode::sub_imm_i32, 2)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
            ], // params
            &[
                // group 0
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                // group 1
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                // group 2
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                // group 3
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
            ], // results
            &[], // local variables
            code0,
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::U32(11),
                ForeignValue::U32(211),
                ForeignValue::U32(-13i32 as u32),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::U32(222),
                ForeignValue::U32(200),
                ForeignValue::U32(-200i32 as u32),
                ForeignValue::U32(2321),
                // group 1
                ForeignValue::U32(-16i32 as u32),
                ForeignValue::U32(0),
                ForeignValue::U32(0),
                ForeignValue::U32(20355295),
                ForeignValue::U32(3),
                ForeignValue::U32(38),
                // group 2
                ForeignValue::U32(14),
                ForeignValue::U32(8),
                ForeignValue::U32(-10i32 as u32),
                ForeignValue::U32(-16i32 as u32),
                // group 3
                ForeignValue::U32(0x1),
                ForeignValue::U32(0xf0e0_d0c0 << 1),
                ForeignValue::U32(0x1),
                ForeignValue::U32(0xffff_ffff),
            ]
        );
    }

    #[test]
    fn test_handler_arithmetic_i64() {
        // numbers:
        //   - 0: 11
        //   - 1: 211
        //   - 2: -13

        // arithemtic:
        //   group 0:
        //   - add   0 1      -> 222
        //   - sub   1 0      -> 200
        //   - sub   0 1      -> -200
        //   - mul   0 1      -> 2321
        //
        //   group 1:
        //   - div_s 1 2      -> -16
        //   - div_u 1 2      -> 0
        //   - div_s 2 1      -> 0
        //   - div_u 2 1      -> 87425327363552377 (= 18446744073709551603/211)
        //   - rem_s 1 2      -> 3
        //   - rem_u 2 1      -> 56
        //
        //   group 2:
        //   - inc   0 imm:3     -> 14
        //   - dec   0 imm:3     -> 8
        //   - inc   2 imm:3     -> -10
        //   - dec   2 imm:3     -> -16
        //
        //   group 3:
        //   - add 0xffff_ffff_ffff_ffff 0x2  -> 0x1    ;; -1 + 2 = 1
        //   - mul 0xf0e0_d0c0_b0a0_9080 0x2  -> 0xf0e0_d0c0_b0a0_9080 << 1
        //   - inc 0xffff_ffff_ffff_ffff 0x2  -> 0x1
        //   - dec 0x1                   0x2  -> 0xffff_ffff_ffff_ffff
        //
        // (i64 i64 i64) -> (i64 i64 i64 i64  i64 i64 i64 i64 i64 i64  i64 i64 i64 i64  i64 i64 i64 i64)

        // note of the 'remainder':
        // (211 % -13) = 3
        //  ^      ^
        //  |      |divisor
        //  |dividend <--------- the result always takes the sign of the dividend.

        let code0 = BytecodeWriterHelper::new()
            // group 0
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 1)
            .append_opcode(Opcode::add_i64)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 1)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0)
            .append_opcode(Opcode::sub_i64)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 1)
            .append_opcode(Opcode::sub_i64)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 1)
            .append_opcode(Opcode::mul_i64)
            // group 1
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 1)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 2)
            .append_opcode(Opcode::div_i64_s)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 1)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 2)
            .append_opcode(Opcode::div_i64_u)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 1)
            .append_opcode(Opcode::div_i64_s)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 1)
            .append_opcode(Opcode::div_i64_u)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 1)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 2)
            .append_opcode(Opcode::rem_i64_s)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 1)
            .append_opcode(Opcode::rem_i64_u)
            // group 2
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0)
            .append_opcode_i16(Opcode::add_imm_i64, 3)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0)
            .append_opcode_i16(Opcode::sub_imm_i64, 3)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 2)
            .append_opcode_i16(Opcode::add_imm_i64, 3)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 2)
            .append_opcode_i16(Opcode::sub_imm_i64, 3)
            // group 3
            .append_opcode_i64(Opcode::imm_i64, 0xffff_ffff_ffff_ffff)
            .append_opcode_i64(Opcode::imm_i64, 0x2)
            .append_opcode(Opcode::add_i64)
            .append_opcode_i64(Opcode::imm_i64, 0xf0e0_d0c0_b0a0_9080)
            .append_opcode_i64(Opcode::imm_i64, 0x2)
            .append_opcode(Opcode::mul_i64)
            .append_opcode_i64(Opcode::imm_i64, 0xffff_ffff_ffff_ffff)
            .append_opcode_i16(Opcode::add_imm_i64, 2)
            .append_opcode_i64(Opcode::imm_i64, 0x1)
            .append_opcode_i16(Opcode::sub_imm_i64, 2)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
            ], // params
            &[
                //
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                //
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                //
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                //
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
            ], // results
            &[], // local variables
            code0,
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::U64(11),
                ForeignValue::U64(211),
                ForeignValue::U64(-13i64 as u64),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::U64(222),
                ForeignValue::U64(200),
                ForeignValue::U64(-200_i64 as u64),
                ForeignValue::U64(2321),
                // group 1
                ForeignValue::U64(-16i64 as u64),
                ForeignValue::U64(0),
                ForeignValue::U64(0),
                ForeignValue::U64(87425327363552377),
                ForeignValue::U64(3),
                ForeignValue::U64(56),
                // group 2
                ForeignValue::U64(14),
                ForeignValue::U64(8),
                ForeignValue::U64(-10i64 as u64),
                ForeignValue::U64(-16i64 as u64),
                // group 3
                ForeignValue::U64(0x1),
                ForeignValue::U64(0xf0e0_d0c0_b0a0_9080 << 1),
                ForeignValue::U64(0x1),
                ForeignValue::U64(0xffff_ffff_ffff_ffff),
            ]
        );
    }

    #[test]
    fn test_handler_arithmetic_f32() {
        // numbers:
        //   - 0: 1.414
        //   - 1: 4.123

        // arithemtic:
        //   - add 0 1      -> 5.537
        //   - sub 1 0      -> 2.709
        //   - mul 0 1      -> 5.829922
        //   - div 1 0      -> 2.91584158416
        //
        // (f32 f32) -> (f32 f32 f32 f32)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::local_load_f32, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_f32, 0, 1)
            .append_opcode(Opcode::add_f32)
            .append_opcode_i16_i32(Opcode::local_load_f32, 0, 1)
            .append_opcode_i16_i32(Opcode::local_load_f32, 0, 0)
            .append_opcode(Opcode::sub_f32)
            .append_opcode_i16_i32(Opcode::local_load_f32, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_f32, 0, 1)
            .append_opcode(Opcode::mul_f32)
            .append_opcode_i16_i32(Opcode::local_load_f32, 0, 1)
            .append_opcode_i16_i32(Opcode::local_load_f32, 0, 0)
            .append_opcode(Opcode::div_f32)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[OperandDataType::F32, OperandDataType::F32], // params
            &[
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
            ], // results
            &[],                                           // local variables
            code0,
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::F32(1.414), ForeignValue::F32(4.123)],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::F32(5.537),
                ForeignValue::F32(2.709),
                ForeignValue::F32(5.829922),
                ForeignValue::F32(2.915_841_6),
            ]
        );
    }

    #[test]
    fn test_handler_arithmetic_f64() {
        // numbers:
        //   - 0: 1.414
        //   - 1: 4.123

        // arithemtic:
        //   - add 0 1      -> 5.537
        //   - sub 1 0      -> 2.709
        //   - mul 0 1      -> 5.829922
        //   - div 1 0      -> 2.91584158416
        //
        // (f64 f64) -> (f64 f64 f64 f64)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::local_load_f64, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_f64, 0, 1)
            .append_opcode(Opcode::add_f64)
            .append_opcode_i16_i32(Opcode::local_load_f64, 0, 1)
            .append_opcode_i16_i32(Opcode::local_load_f64, 0, 0)
            .append_opcode(Opcode::sub_f64)
            .append_opcode_i16_i32(Opcode::local_load_f64, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_f64, 0, 1)
            .append_opcode(Opcode::mul_f64)
            .append_opcode_i16_i32(Opcode::local_load_f64, 0, 1)
            .append_opcode_i16_i32(Opcode::local_load_f64, 0, 0)
            .append_opcode(Opcode::div_f64)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[OperandDataType::F64, OperandDataType::F64], // params
            &[
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
            ], // results
            &[],                                           // local variables
            code0,
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::F64(1.414), ForeignValue::F64(4.123)],
        );

        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::F64(5.537),
                ForeignValue::F64(2.7090000000000005),
                ForeignValue::F64(5.829922),
                ForeignValue::F64(2.915841584158416),
            ]
        );
    }
}
