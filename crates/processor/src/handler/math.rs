// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::thread_context::ThreadContext;

use super::HandleResult;

pub fn abs_i32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = thread_context.stack.pop_i32_s();
    thread_context.stack.push_i32_s(v.abs());
    HandleResult::Move(2)
}

pub fn neg_i32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = thread_context.stack.pop_i32_s();
    thread_context.stack.push_i32_s(-v);
    HandleResult::Move(2)
}

pub fn abs_i64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = thread_context.stack.pop_i64_s();
    thread_context.stack.push_i64_s(v.abs());
    HandleResult::Move(2)
}

pub fn neg_i64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = thread_context.stack.pop_i64_s();
    thread_context.stack.push_i64_s(-v);
    HandleResult::Move(2)
}

pub fn abs_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.abs());
    HandleResult::Move(2)
}

pub fn neg_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, -v);
    HandleResult::Move(2)
}

pub fn ceil_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.ceil());
    HandleResult::Move(2)
}

pub fn floor_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.floor());
    HandleResult::Move(2)
}

pub fn round_half_away_from_zero_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.round());
    HandleResult::Move(2)
}

pub fn round_half_to_even_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);

    let toint_32: f32 = 1.0 / f32::EPSILON;

    let e = v.to_bits() >> 23 & 0xff;
    let r = if e >= 0x7f_u32 + 23 {
        v
    } else {
        (v.abs() + toint_32 - toint_32).copysign(v)
    };

    store_f32(thread_context, r);
    HandleResult::Move(2)
}

pub fn trunc_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.trunc());
    HandleResult::Move(2)
}

pub fn fract_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.fract());
    HandleResult::Move(2)
}

pub fn sqrt_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.sqrt());
    HandleResult::Move(2)
}

pub fn cbrt_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.cbrt());
    HandleResult::Move(2)
}

pub fn exp_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.exp());
    HandleResult::Move(2)
}

pub fn exp2_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.exp2());
    HandleResult::Move(2)
}

pub fn ln_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.ln());
    HandleResult::Move(2)
}

pub fn log2_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.log2());
    HandleResult::Move(2)
}

pub fn log10_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.log10());
    HandleResult::Move(2)
}

pub fn sin_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.sin());
    HandleResult::Move(2)
}

pub fn cos_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.cos());
    HandleResult::Move(2)
}

pub fn tan_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.tan());
    HandleResult::Move(2)
}

pub fn asin_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.asin());
    HandleResult::Move(2)
}

pub fn acos_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.acos());
    HandleResult::Move(2)
}

pub fn atan_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.atan());
    HandleResult::Move(2)
}

pub fn copysign_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let (num, sign) = load_operands_f32(thread_context);
    store_f32(thread_context, num.copysign(sign));
    HandleResult::Move(2)
}

pub fn pow_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f32(thread_context);
    store_f32(thread_context, left.powf(right));
    HandleResult::Move(2)
}

pub fn log_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f32(thread_context);
    store_f32(thread_context, left.log(right));
    HandleResult::Move(2)
}

pub fn min_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f32(thread_context);
    store_f32(thread_context, f32::min(left, right));
    HandleResult::Move(2)
}

pub fn max_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f32(thread_context);
    store_f32(thread_context, f32::max(left, right));
    HandleResult::Move(2)
}

pub fn abs_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.abs());
    HandleResult::Move(2)
}

pub fn neg_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, -v);
    HandleResult::Move(2)
}

pub fn ceil_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.ceil());
    HandleResult::Move(2)
}

pub fn floor_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.floor());
    HandleResult::Move(2)
}

pub fn round_half_away_from_zero_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.round());
    HandleResult::Move(2)
}

pub fn round_half_to_even_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);

    let toint_64: f64 = 1.0 / f64::EPSILON;

    let e = v.to_bits() >> 52 & 0x7ff_u64;
    let r = if e >= 0x3ff_u64 + 52 {
        v
    } else {
        (v.abs() + toint_64 - toint_64).copysign(v)
    };

    store_f64(thread_context, r);
    HandleResult::Move(2)
}

pub fn trunc_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.trunc());
    HandleResult::Move(2)
}

pub fn fract_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.fract());
    HandleResult::Move(2)
}

pub fn sqrt_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.sqrt());
    HandleResult::Move(2)
}

pub fn cbrt_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.cbrt());
    HandleResult::Move(2)
}

pub fn exp_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.exp());
    HandleResult::Move(2)
}

pub fn exp2_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.exp2());
    HandleResult::Move(2)
}

pub fn ln_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.ln());
    HandleResult::Move(2)
}

pub fn log2_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.log2());
    HandleResult::Move(2)
}

pub fn log10_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.log10());
    HandleResult::Move(2)
}

pub fn sin_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.sin());
    HandleResult::Move(2)
}

pub fn cos_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.cos());
    HandleResult::Move(2)
}

pub fn tan_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.tan());
    HandleResult::Move(2)
}

pub fn asin_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.asin());
    HandleResult::Move(2)
}

pub fn acos_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.acos());
    HandleResult::Move(2)
}

pub fn atan_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.atan());
    HandleResult::Move(2)
}

pub fn copysign_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let (num, sign) = load_operands_f64(thread_context);
    store_f64(thread_context, num.copysign(sign));
    HandleResult::Move(2)
}

pub fn pow_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f64(thread_context);
    store_f64(thread_context, left.powf(right));
    HandleResult::Move(2)
}

pub fn log_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f64(thread_context);
    store_f64(thread_context, left.log(right));
    HandleResult::Move(2)
}

pub fn min_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f64(thread_context);
    store_f64(thread_context, f64::min(left, right));
    HandleResult::Move(2)
}

pub fn max_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f64(thread_context);
    store_f64(thread_context, f64::max(left, right));
    HandleResult::Move(2)
}

#[inline]
fn load_operand_f32(thread_context: &mut ThreadContext) -> f32 {
    thread_context.stack.pop_f32()
}

#[inline]
fn load_operands_f32(thread_context: &mut ThreadContext) -> (f32, f32) {
    let right = thread_context.stack.pop_f32();
    let left = thread_context.stack.pop_f32();
    (left, right)
}

#[inline]
fn load_operand_f64(thread_context: &mut ThreadContext) -> f64 {
    thread_context.stack.pop_f64()
}

#[inline]
fn load_operands_f64(thread_context: &mut ThreadContext) -> (f64, f64) {
    let right = thread_context.stack.pop_f64();
    let left = thread_context.stack.pop_f64();
    (left, right)
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
    use crate::{
        handler::Handler, in_memory_resource::InMemoryResource, process::process_function,
    };
    use ancvm_context::resource::Resource;
    use ancvm_image::{
        bytecode_writer::BytecodeWriterHelper,
        utils::helper_build_module_binary_with_single_function,
    };
    use ancvm_isa::{opcode::Opcode, ForeignValue, OperandDataType};

    #[test]
    fn test_interpreter_math_i32() {
        // numbers:
        //   - 0: 11
        //   - 1: -11
        //
        // functions:
        //   - abs      0   -> 11
        //   - abs      1   -> 11
        //   - neg      0   -> -11
        //   - neg      1   -> 11
        //
        // (i32 i32) -> (i32 i32 i32 i32)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 0, 0)
            .append_opcode(Opcode::abs_i32)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 0, 1)
            .append_opcode(Opcode::abs_i32)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 0, 0)
            .append_opcode(Opcode::neg_i32)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 0, 1)
            .append_opcode(Opcode::neg_i32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![OperandDataType::I32, OperandDataType::I32], // params
            vec![
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
            ], // results
            vec![],                                           // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &handler,
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(11), ForeignValue::U32(-11i32 as u32)],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U32(11),
                ForeignValue::U32(11),
                ForeignValue::U32(-11i32 as u32),
                ForeignValue::U32(11),
            ]
        );
    }

    #[test]
    fn test_interpreter_math_i64() {
        // numbers:
        //   - 0: 11
        //   - 1: -11
        //
        // functions:
        //   - abs      0   -> 11
        //   - abs      1   -> 11
        //   - neg      0   -> -11
        //   - neg      1   -> 11
        //
        // (i64 i64) -> (i64 i64 i64 i64)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0)
            .append_opcode(Opcode::abs_i64)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode(Opcode::abs_i64)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0)
            .append_opcode(Opcode::neg_i64)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode(Opcode::neg_i64)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![OperandDataType::I64, OperandDataType::I64], // params
            vec![
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
            ], // results
            vec![],                                           // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &handler,
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U64(11), ForeignValue::U64(-11i64 as u64)],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U64(11),
                ForeignValue::U64(11),
                ForeignValue::U64(-11i64 as u64),
                ForeignValue::U64(11),
            ]
        );
    }

    #[test]
    fn test_interpreter_math_f32_part_a() {
        // numbers:
        //   - 0: 1.414
        //   - 1: -1.732
        //   - 2: 2.4
        //   - 3: 2.5
        //   - 4: 2.6
        //   - 5: 5.5
        //   - 6: -2.4
        //   - 7: -2.5
        //   - 8: -2.6
        //   - 9: -5.5
        //
        // functions:
        //   - abs      0   -> 1.414
        //   - abs      1   -> 1.732
        //   - neg      0   -> -1.414
        //   - neg      1   -> 1.732
        //
        //   - ceil     2   -> 3.0
        //   - ceil     4   -> 3.0
        //   - ceil     6   -> -2.0
        //   - ceil     8   -> -2.0
        //
        //   - floor    2   -> 2.0
        //   - floor    4   -> 2.0
        //   - floor    6   -> -3.0
        //   - floor    8   -> -3.0
        //
        //   - round_half_away_from_zero    2   -> 2.0
        //   - round_half_away_from_zero    3   -> 3.0
        //   - round_half_away_from_zero    4   -> 3.0
        //   - round_half_away_from_zero    5   -> 6.0
        //   - round_half_away_from_zero    6   -> -2.0
        //   - round_half_away_from_zero    7   -> -3.0
        //   - round_half_away_from_zero    8   -> -3.0
        //   - round_half_away_from_zero    9   -> -6.0
        //
        //   - round_half_to_even    2   -> 2.0
        //   - round_half_to_even    3   -> 2.0
        //   - round_half_to_even    4   -> 3.0
        //   - round_half_to_even    5   -> 6.0
        //   - round_half_to_even    6   -> -2.0
        //   - round_half_to_even    7   -> -2.0
        //   - round_half_to_even    8   -> -3.0
        //   - round_half_to_even    9   -> -6.0
        //
        // (f32 f32 f32 f32  f32 f32 f32 f32) ->
        // (f32 f32 f32 f32  f32 f32 f32 f32  f32 f32 f32 f32
        //  f32 f32 f32 f32 f32 f32 f32 f32
        //  f32 f32 f32 f32 f32 f32 f32 f32)

        let code0 = BytecodeWriterHelper::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode(Opcode::abs_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 1)
            .append_opcode(Opcode::abs_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode(Opcode::neg_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 1)
            .append_opcode(Opcode::neg_f32)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 2)
            .append_opcode(Opcode::ceil_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 4)
            .append_opcode(Opcode::ceil_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 6)
            .append_opcode(Opcode::ceil_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 8)
            .append_opcode(Opcode::ceil_f32)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 2)
            .append_opcode(Opcode::floor_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 4)
            .append_opcode(Opcode::floor_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 6)
            .append_opcode(Opcode::floor_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 8)
            .append_opcode(Opcode::floor_f32)
            // group 3
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 2)
            .append_opcode(Opcode::round_half_away_from_zero_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 3)
            .append_opcode(Opcode::round_half_away_from_zero_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 4)
            .append_opcode(Opcode::round_half_away_from_zero_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 5)
            .append_opcode(Opcode::round_half_away_from_zero_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 6)
            .append_opcode(Opcode::round_half_away_from_zero_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 7)
            .append_opcode(Opcode::round_half_away_from_zero_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 8)
            .append_opcode(Opcode::round_half_away_from_zero_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 9)
            .append_opcode(Opcode::round_half_away_from_zero_f32)
            // group 4
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 2)
            .append_opcode(Opcode::round_half_to_even_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 3)
            .append_opcode(Opcode::round_half_to_even_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 4)
            .append_opcode(Opcode::round_half_to_even_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 5)
            .append_opcode(Opcode::round_half_to_even_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 6)
            .append_opcode(Opcode::round_half_to_even_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 7)
            .append_opcode(Opcode::round_half_to_even_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 8)
            .append_opcode(Opcode::round_half_to_even_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 9)
            .append_opcode(Opcode::round_half_to_even_f32)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
            ], // params
            vec![
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                //
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                //
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                //
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                //
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
            ], // results
            vec![], // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &handler,
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::F32(1.414),
                ForeignValue::F32(-1.732),
                ForeignValue::F32(2.4),
                ForeignValue::F32(2.5),
                ForeignValue::F32(2.6),
                ForeignValue::F32(5.5),
                ForeignValue::F32(-2.4),
                ForeignValue::F32(-2.5),
                ForeignValue::F32(-2.6),
                ForeignValue::F32(-5.5),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::F32(1.414),
                ForeignValue::F32(1.732),
                ForeignValue::F32(-1.414),
                ForeignValue::F32(1.732),
                // group 1
                ForeignValue::F32(3.0),
                ForeignValue::F32(3.0),
                ForeignValue::F32(-2.0),
                ForeignValue::F32(-2.0),
                // group 2
                ForeignValue::F32(2.0),
                ForeignValue::F32(2.0),
                ForeignValue::F32(-3.0),
                ForeignValue::F32(-3.0),
                // group 3
                ForeignValue::F32(2.0),
                ForeignValue::F32(3.0),
                ForeignValue::F32(3.0),
                ForeignValue::F32(6.0),
                ForeignValue::F32(-2.0),
                ForeignValue::F32(-3.0),
                ForeignValue::F32(-3.0),
                ForeignValue::F32(-6.0),
                // group 4
                ForeignValue::F32(2.0),
                ForeignValue::F32(2.0),
                ForeignValue::F32(3.0),
                ForeignValue::F32(6.0),
                ForeignValue::F32(-2.0),
                ForeignValue::F32(-2.0),
                ForeignValue::F32(-3.0),
                ForeignValue::F32(-6.0),
            ]
        );
    }

    #[test]
    fn test_interpreter_math_f32_part_b() {
        // numbers:
        //   - 0: 1.414
        //   - 1: 4.0
        //   - 2: 27.0
        //   - 3: 3.0
        //   - 4: 9.0
        //   - 5  -3.0
        //   - 6: -9.0
        //   - 7: 100.0
        //   - 8: 2.718281828               // std::f32::consts::E
        //   - 9: 0.523598776   (deg 30)    // std::f32::consts::FRAC_PI_6
        //
        // functions:
        //   group 0:
        //   - trunc   0        -> 1.0
        //   - fract   0        -> 0.41400003
        //   - sqrt    1        -> 2.0
        //   - cbrt    2        -> 3.0
        //
        //   group 1:
        //   - exp     3        -> 20.085_537 (e^3)
        //   - exp2    4        -> 512.0
        //   - ln      8        -> 0.99999994
        //   - log2    1        -> 2.0 (log_2 4)
        //   - log10   7        -> 2.0 (log_10 100)
        //
        //   group 2:
        //   - sin     9        -> 0.5
        //   - cos     9        -> 0.866_025_4
        //   - tan     9        -> 0.577_350_3
        //   - asin    imm(0.5)     -> deg 30
        //   - acos    imm(0.86..)  -> deg 30
        //   - atab    imm(0.57..)  -> deg 30
        //
        //   group 3:
        //   - pow      1 3      -> 64.0 (4^3)
        //   - log      4 3      -> 2.0 (log_3 9)
        //
        //   group 4:
        //   - copysign 4 3      -> 9.0
        //   - copysign 4 5      -> -9.0
        //   - copysign 5 4      -> 3.0
        //   - copysign 5 6      -> -3.0
        //
        //   group 5:
        //   - min      3 4      -> 3.0
        //   - min      4 5      -> -3.0
        //   - max      4 5      -> 9.0
        //   - max      5 6      -> -3.0
        //
        // (f32 f32 f32 f32  f32 f32 f32 f32  f32 f32) ->
        // (f32 f32 f32 f32  f32 f32 f32 f32 f32  f32 f32 f32 f32 f32 f32
        //  f32 f32
        //  f32 f32 f32 f32
        //  f32 f32 f32 f32)

        let code0 = BytecodeWriterHelper::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode(Opcode::trunc_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode(Opcode::fract_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 1)
            .append_opcode(Opcode::sqrt_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 2)
            .append_opcode(Opcode::cbrt_f32)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 3)
            .append_opcode(Opcode::exp_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 4)
            .append_opcode(Opcode::exp2_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 8)
            .append_opcode(Opcode::ln_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 1)
            .append_opcode(Opcode::log2_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 7)
            .append_opcode(Opcode::log10_f32)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 9)
            .append_opcode(Opcode::sin_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 9)
            .append_opcode(Opcode::cos_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 9)
            .append_opcode(Opcode::tan_f32)
            .append_opcode_f32(Opcode::imm_f32, 0.5)
            .append_opcode(Opcode::asin_f32)
            .append_opcode_f32(Opcode::imm_f32, 0.866_025_4)
            .append_opcode(Opcode::acos_f32)
            .append_opcode_f32(Opcode::imm_f32, 0.577_350_3)
            .append_opcode(Opcode::atan_f32)
            // group 3
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 3)
            .append_opcode(Opcode::pow_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 3)
            .append_opcode(Opcode::log_f32)
            // group 4
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 3)
            .append_opcode(Opcode::copysign_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 5)
            .append_opcode(Opcode::copysign_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 5)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 4)
            .append_opcode(Opcode::copysign_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 5)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 6)
            .append_opcode(Opcode::copysign_f32)
            // group 5
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 3)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 4)
            .append_opcode(Opcode::min_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 5)
            .append_opcode(Opcode::min_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 5)
            .append_opcode(Opcode::max_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 5)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 6)
            .append_opcode(Opcode::max_f32)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                //
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
            ], // params
            vec![
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                //
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                //
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                //
                OperandDataType::F32,
                OperandDataType::F32,
                //
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                //
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
                OperandDataType::F32,
            ], // results
            vec![], // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &handler,
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::F32(1.414),
                ForeignValue::F32(4.0),
                ForeignValue::F32(27.0),
                ForeignValue::F32(3.0),
                ForeignValue::F32(9.0),
                ForeignValue::F32(-3.0),
                ForeignValue::F32(-9.0),
                ForeignValue::F32(100.0),
                ForeignValue::F32(std::f32::consts::E),
                ForeignValue::F32(std::f32::consts::FRAC_PI_6),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::F32(1.0),
                ForeignValue::F32(0.41400003),
                ForeignValue::F32(2.0),
                ForeignValue::F32(3.0),
                // group 1
                ForeignValue::F32(20.085_537),
                ForeignValue::F32(512.0),
                ForeignValue::F32(0.99999994), // 1.0
                ForeignValue::F32(2.0),
                ForeignValue::F32(2.0),
                // group 2
                ForeignValue::F32(0.5),
                ForeignValue::F32(0.866_025_4),
                ForeignValue::F32(0.577_350_3),
                ForeignValue::F32(std::f32::consts::FRAC_PI_6),
                ForeignValue::F32(std::f32::consts::FRAC_PI_6),
                ForeignValue::F32(std::f32::consts::FRAC_PI_6),
                // group 3
                ForeignValue::F32(64.0),
                ForeignValue::F32(2.0),
                // group 4
                ForeignValue::F32(9.0),
                ForeignValue::F32(-9.0),
                ForeignValue::F32(3.0),
                ForeignValue::F32(-3.0),
                // group 5
                ForeignValue::F32(3.0),
                ForeignValue::F32(-3.0),
                ForeignValue::F32(9.0),
                ForeignValue::F32(-3.0),
            ]
        );
    }

    #[test]
    fn test_interpreter_math_f64_part_a() {
        // numbers:
        //   - 0: 1.414
        //   - 1: -1.732
        //   - 2: 2.4
        //   - 3: 2.5
        //   - 4: 2.6
        //   - 5: 5.5
        //   - 6: -2.4
        //   - 7: -2.5
        //   - 8: -2.6
        //   - 9: -5.5
        //
        // functions:
        //   - abs      0   -> 1.414
        //   - abs      1   -> 1.732
        //   - neg      0   -> -1.414
        //   - neg      1   -> 1.732
        //
        //   - ceil     2   -> 3.0
        //   - ceil     4   -> 3.0
        //   - ceil     6   -> -2.0
        //   - ceil     8   -> -2.0
        //
        //   - floor    2   -> 2.0
        //   - floor    4   -> 2.0
        //   - floor    6   -> -3.0
        //   - floor    8   -> -3.0
        //
        //   - round_half_away_from_zero    2   -> 2.0
        //   - round_half_away_from_zero    3   -> 3.0
        //   - round_half_away_from_zero    4   -> 3.0
        //   - round_half_away_from_zero    5   -> 6.0
        //   - round_half_away_from_zero    6   -> -2.0
        //   - round_half_away_from_zero    7   -> -3.0
        //   - round_half_away_from_zero    8   -> -3.0
        //   - round_half_away_from_zero    9   -> -6.0
        //
        //   - round_half_to_even    2   -> 2.0
        //   - round_half_to_even    3   -> 2.0
        //   - round_half_to_even    4   -> 3.0
        //   - round_half_to_even    5   -> 6.0
        //   - round_half_to_even    6   -> -2.0
        //   - round_half_to_even    7   -> -2.0
        //   - round_half_to_even    8   -> -3.0
        //   - round_half_to_even    9   -> -6.0
        //
        // (f64 f64 f64 f64  f64 f64 f64 f64) ->
        // (f64 f64 f64 f64  f64 f64 f64 f64  f64 f64 f64 f64
        //  f64 f64 f64 f64 f64 f64 f64 f64
        //  f64 f64 f64 f64 f64 f64 f64 f64)

        let code0 = BytecodeWriterHelper::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode(Opcode::abs_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode(Opcode::abs_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode(Opcode::neg_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode(Opcode::neg_f64)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 2)
            .append_opcode(Opcode::ceil_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            .append_opcode(Opcode::ceil_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 6)
            .append_opcode(Opcode::ceil_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 8)
            .append_opcode(Opcode::ceil_f64)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 2)
            .append_opcode(Opcode::floor_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            .append_opcode(Opcode::floor_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 6)
            .append_opcode(Opcode::floor_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 8)
            .append_opcode(Opcode::floor_f64)
            // group 3
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 2)
            .append_opcode(Opcode::round_half_away_from_zero_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 3)
            .append_opcode(Opcode::round_half_away_from_zero_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            .append_opcode(Opcode::round_half_away_from_zero_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 5)
            .append_opcode(Opcode::round_half_away_from_zero_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 6)
            .append_opcode(Opcode::round_half_away_from_zero_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 7)
            .append_opcode(Opcode::round_half_away_from_zero_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 8)
            .append_opcode(Opcode::round_half_away_from_zero_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 9)
            .append_opcode(Opcode::round_half_away_from_zero_f64)
            // group 4
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 2)
            .append_opcode(Opcode::round_half_to_even_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 3)
            .append_opcode(Opcode::round_half_to_even_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            .append_opcode(Opcode::round_half_to_even_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 5)
            .append_opcode(Opcode::round_half_to_even_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 6)
            .append_opcode(Opcode::round_half_to_even_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 7)
            .append_opcode(Opcode::round_half_to_even_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 8)
            .append_opcode(Opcode::round_half_to_even_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 9)
            .append_opcode(Opcode::round_half_to_even_f64)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
            ], // params
            vec![
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                //
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                //
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                //
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                //
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
            ], // results
            vec![], // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &handler,
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::F64(1.414),
                ForeignValue::F64(-1.732),
                ForeignValue::F64(2.4),
                ForeignValue::F64(2.5),
                ForeignValue::F64(2.6),
                ForeignValue::F64(5.5),
                ForeignValue::F64(-2.4),
                ForeignValue::F64(-2.5),
                ForeignValue::F64(-2.6),
                ForeignValue::F64(-5.5),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::F64(1.414),
                ForeignValue::F64(1.732),
                ForeignValue::F64(-1.414),
                ForeignValue::F64(1.732),
                // group 1
                ForeignValue::F64(3.0),
                ForeignValue::F64(3.0),
                ForeignValue::F64(-2.0),
                ForeignValue::F64(-2.0),
                // group 2
                ForeignValue::F64(2.0),
                ForeignValue::F64(2.0),
                ForeignValue::F64(-3.0),
                ForeignValue::F64(-3.0),
                // group 3
                ForeignValue::F64(2.0),
                ForeignValue::F64(3.0),
                ForeignValue::F64(3.0),
                ForeignValue::F64(6.0),
                ForeignValue::F64(-2.0),
                ForeignValue::F64(-3.0),
                ForeignValue::F64(-3.0),
                ForeignValue::F64(-6.0),
                // group 4
                ForeignValue::F64(2.0),
                ForeignValue::F64(2.0),
                ForeignValue::F64(3.0),
                ForeignValue::F64(6.0),
                ForeignValue::F64(-2.0),
                ForeignValue::F64(-2.0),
                ForeignValue::F64(-3.0),
                ForeignValue::F64(-6.0),
            ]
        );
    }

    #[test]
    fn test_interpreter_math_f64_part_b() {
        // numbers:
        //   - 0: 1.414
        //   - 1: 4.0
        //   - 2: 27.0
        //   - 3: 3.0
        //   - 4: 9.0
        //   - 5: -3.0
        //   - 6: -9.0
        //   - 7: 100.0
        //   - 8: 2.718281828               // std::f64::consts::E
        //   - 9: 0.523598776   (deg 30)    // std::f64::consts::FRAC_PI_6
        //
        // functions:
        //   group 0:
        //   - trunc   0        -> 1.0
        //   - fract   0        -> 0.4139999999999999
        //   - sqrt    1        -> 2.0
        //   - cbrt    2        -> 3.0000000000000004
        //
        //   group 1:
        //   - exp     3        -> 20.085536923187668 (e^3)
        //   - exp2    4        -> 512.0
        //   - ln      8        -> 1.0
        //   - log2    1        -> 2.0 (log_2 4)
        //   - log10   7        -> 2.0 (log_10 100)
        //
        //   group 2:
        //   - sin     9        -> 0.5
        //   - cos     9        -> 0.866_025_403_784_438_6
        //   - tan     9        -> 0.577_350_269_189_625_8
        //   - asin    imm(0.5)     -> deg 30
        //   - acos    imm(0.86..)  -> deg 30
        //   - atab    imm(0.57..)  -> deg 30
        //
        //   group 3:
        //   - pow     1 3      -> 64.0 (4^3)
        //   - log     4 3      -> 2.0 (log_3 9)
        //
        //   group 4:
        //   - copysign 4 3      -> 9.0
        //   - copysign 4 5      -> -9.0
        //   - copysign 5 4      -> 3.0
        //   - copysign 5 6      -> -3.0
        //
        //   group 5:
        //   - min      3 4      -> 3.0
        //   - min      4 5      -> -3.0
        //   - max      4 5      -> 9.0
        //   - max      5 6      -> -3.0
        //
        // (f64 f64 f64 f64  f64 f64 f64 f64  f64 f64) ->
        // (f64 f64 f64 f64  f64 f64 f64 f64 f64  f64 f64 f64 f64 f64 f64
        //  f64 f64
        //  f64 f64 f64 f64
        //  f64 f64 f64 f64)

        let code0 = BytecodeWriterHelper::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode(Opcode::trunc_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode(Opcode::fract_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode(Opcode::sqrt_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 2)
            .append_opcode(Opcode::cbrt_f64)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 3)
            .append_opcode(Opcode::exp_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            .append_opcode(Opcode::exp2_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 8)
            .append_opcode(Opcode::ln_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode(Opcode::log2_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 7)
            .append_opcode(Opcode::log10_f64)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 9)
            .append_opcode(Opcode::sin_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 9)
            .append_opcode(Opcode::cos_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 9)
            .append_opcode(Opcode::tan_f64)
            .append_opcode_f64(Opcode::imm_f64, 0.5)
            .append_opcode(Opcode::asin_f64)
            .append_opcode_f64(Opcode::imm_f64, 0.8660254037844386)
            .append_opcode(Opcode::acos_f64)
            .append_opcode_f64(Opcode::imm_f64, 0.5773502691896258)
            .append_opcode(Opcode::atan_f64)
            // group 3
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 3)
            .append_opcode(Opcode::pow_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 3)
            .append_opcode(Opcode::log_f64)
            // group 4
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 3)
            .append_opcode(Opcode::copysign_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 5)
            .append_opcode(Opcode::copysign_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 5)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            .append_opcode(Opcode::copysign_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 5)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 6)
            .append_opcode(Opcode::copysign_f64)
            // group 5
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 3)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            .append_opcode(Opcode::min_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 5)
            .append_opcode(Opcode::min_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 5)
            .append_opcode(Opcode::max_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 5)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 6)
            .append_opcode(Opcode::max_f64)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                //
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
            ], // params
            vec![
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                //
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                //
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                //
                OperandDataType::F64,
                OperandDataType::F64,
                //
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                //
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
                OperandDataType::F64,
            ], // results
            vec![], // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &handler,
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::F64(1.414),
                ForeignValue::F64(4.0),
                ForeignValue::F64(27.0),
                ForeignValue::F64(3.0),
                ForeignValue::F64(9.0),
                ForeignValue::F64(-3.0),
                ForeignValue::F64(-9.0),
                ForeignValue::F64(100.0),
                ForeignValue::F64(std::f64::consts::E),
                ForeignValue::F64(std::f64::consts::FRAC_PI_6),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::F64(1.0),
                ForeignValue::F64(0.4139999999999999),
                ForeignValue::F64(2.0),
                ForeignValue::F64(3.0000000000000004),
                // group 1
                ForeignValue::F64(20.085536923187668),
                ForeignValue::F64(512.0),
                ForeignValue::F64(1.0),
                ForeignValue::F64(2.0),
                ForeignValue::F64(2.0),
                // group 2
                ForeignValue::F64(0.5),
                ForeignValue::F64(0.8660254037844386),
                ForeignValue::F64(0.5773502691896258),
                ForeignValue::F64(std::f64::consts::FRAC_PI_6),
                ForeignValue::F64(std::f64::consts::FRAC_PI_6),
                ForeignValue::F64(std::f64::consts::FRAC_PI_6),
                // group 3
                ForeignValue::F64(64.0),
                ForeignValue::F64(2.0),
                // group 4
                ForeignValue::F64(9.0),
                ForeignValue::F64(-9.0),
                ForeignValue::F64(3.0),
                ForeignValue::F64(-3.0),
                // group 5
                ForeignValue::F64(3.0),
                ForeignValue::F64(-3.0),
                ForeignValue::F64(9.0),
                ForeignValue::F64(-3.0),
            ]
        );
    }
}
