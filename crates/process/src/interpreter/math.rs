// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_program::thread_context::ThreadContext;

use super::InterpretResult;

pub fn i32_abs(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = thread_context.stack.pop_i32_s();
    thread_context.stack.push_i32_s(v.abs());
    InterpretResult::Move(2)
}

pub fn i32_neg(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = thread_context.stack.pop_i32_s();
    thread_context.stack.push_i32_s(-v);
    InterpretResult::Move(2)
}

pub fn i64_abs(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = thread_context.stack.pop_i64_s();
    thread_context.stack.push_i64_s(v.abs());
    InterpretResult::Move(2)
}

pub fn i64_neg(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = thread_context.stack.pop_i64_s();
    thread_context.stack.push_i64_s(-v);
    InterpretResult::Move(2)
}

pub fn f32_abs(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.abs());
    InterpretResult::Move(2)
}

pub fn f32_neg(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, -v);
    InterpretResult::Move(2)
}

pub fn f32_ceil(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.ceil());
    InterpretResult::Move(2)
}

pub fn f32_floor(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.floor());
    InterpretResult::Move(2)
}

pub fn f32_round_half_away_from_zero(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.round());
    InterpretResult::Move(2)
}

pub fn f32_round_half_to_even(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);

    let toint_32: f32 = 1.0 / f32::EPSILON;

    let e = v.to_bits() >> 23 & 0xff;
    let r = if e >= 0x7f_u32 + 23 {
        v
    } else {
        (v.abs() + toint_32 - toint_32).copysign(v)
    };

    store_f32(thread_context, r);
    InterpretResult::Move(2)
}

pub fn f32_trunc(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.trunc());
    InterpretResult::Move(2)
}

pub fn f32_fract(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.fract());
    InterpretResult::Move(2)
}

pub fn f32_sqrt(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.sqrt());
    InterpretResult::Move(2)
}

pub fn f32_cbrt(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.cbrt());
    InterpretResult::Move(2)
}

pub fn f32_exp(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.exp());
    InterpretResult::Move(2)
}

pub fn f32_exp2(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.exp2());
    InterpretResult::Move(2)
}

pub fn f32_ln(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.ln());
    InterpretResult::Move(2)
}

pub fn f32_log2(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.log2());
    InterpretResult::Move(2)
}

pub fn f32_log10(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.log10());
    InterpretResult::Move(2)
}

pub fn f32_sin(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.sin());
    InterpretResult::Move(2)
}

pub fn f32_cos(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.cos());
    InterpretResult::Move(2)
}

pub fn f32_tan(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.tan());
    InterpretResult::Move(2)
}

pub fn f32_asin(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.asin());
    InterpretResult::Move(2)
}

pub fn f32_acos(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.acos());
    InterpretResult::Move(2)
}

pub fn f32_atan(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f32(thread_context);
    store_f32(thread_context, v.atan());
    InterpretResult::Move(2)
}

pub fn f32_copysign(thread_context: &mut ThreadContext) -> InterpretResult {
    let (num, sign) = load_operands_f32(thread_context);
    store_f32(thread_context, num.copysign(sign));
    InterpretResult::Move(2)
}

pub fn f32_pow(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f32(thread_context);
    store_f32(thread_context, left.powf(right));
    InterpretResult::Move(2)
}

pub fn f32_log(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f32(thread_context);
    store_f32(thread_context, left.log(right));
    InterpretResult::Move(2)
}

pub fn f32_min(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f32(thread_context);
    store_f32(thread_context, f32::min(left, right));
    InterpretResult::Move(2)
}

pub fn f32_max(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f32(thread_context);
    store_f32(thread_context, f32::max(left, right));
    InterpretResult::Move(2)
}

pub fn f64_abs(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.abs());
    InterpretResult::Move(2)
}

pub fn f64_neg(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, -v);
    InterpretResult::Move(2)
}

pub fn f64_ceil(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.ceil());
    InterpretResult::Move(2)
}

pub fn f64_floor(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.floor());
    InterpretResult::Move(2)
}

pub fn f64_round_half_away_from_zero(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.round());
    InterpretResult::Move(2)
}

pub fn f64_round_half_to_even(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);

    let toint_64: f64 = 1.0 / f64::EPSILON;

    let e = v.to_bits() >> 52 & 0x7ff_u64;
    let r = if e >= 0x3ff_u64 + 52 {
        v
    } else {
        (v.abs() + toint_64 - toint_64).copysign(v)
    };

    store_f64(thread_context, r);
    InterpretResult::Move(2)
}

pub fn f64_trunc(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.trunc());
    InterpretResult::Move(2)
}

pub fn f64_fract(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.fract());
    InterpretResult::Move(2)
}

pub fn f64_sqrt(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.sqrt());
    InterpretResult::Move(2)
}

pub fn f64_cbrt(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.cbrt());
    InterpretResult::Move(2)
}

pub fn f64_exp(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.exp());
    InterpretResult::Move(2)
}

pub fn f64_exp2(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.exp2());
    InterpretResult::Move(2)
}

pub fn f64_ln(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.ln());
    InterpretResult::Move(2)
}

pub fn f64_log2(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.log2());
    InterpretResult::Move(2)
}

pub fn f64_log10(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.log10());
    InterpretResult::Move(2)
}

pub fn f64_sin(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.sin());
    InterpretResult::Move(2)
}

pub fn f64_cos(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.cos());
    InterpretResult::Move(2)
}

pub fn f64_tan(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.tan());
    InterpretResult::Move(2)
}

pub fn f64_asin(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.asin());
    InterpretResult::Move(2)
}

pub fn f64_acos(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.acos());
    InterpretResult::Move(2)
}

pub fn f64_atan(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_f64(thread_context);
    store_f64(thread_context, v.atan());
    InterpretResult::Move(2)
}

pub fn f64_copysign(thread_context: &mut ThreadContext) -> InterpretResult {
    let (num, sign) = load_operands_f64(thread_context);
    store_f64(thread_context, num.copysign(sign));
    InterpretResult::Move(2)
}

pub fn f64_pow(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f64(thread_context);
    store_f64(thread_context, left.powf(right));
    InterpretResult::Move(2)
}

pub fn f64_log(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f64(thread_context);
    store_f64(thread_context, left.log(right));
    InterpretResult::Move(2)
}

pub fn f64_min(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f64(thread_context);
    store_f64(thread_context, f64::min(left, right));
    InterpretResult::Move(2)
}

pub fn f64_max(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f64(thread_context);
    store_f64(thread_context, f64::max(left, right));
    InterpretResult::Move(2)
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
    use crate::{in_memory_program_source::InMemoryProgramSource, interpreter::process_function};
    use ancvm_binary::{
        bytecode_writer::BytecodeWriter, utils::helper_build_module_binary_with_single_function,
    };
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

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

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode(Opcode::i32_abs)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_abs)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode(Opcode::i32_neg)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_neg)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32], // params
            vec![DataType::I32, DataType::I32, DataType::I32, DataType::I32], // results
            vec![],                             // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
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

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 0)
            .append_opcode(Opcode::i64_abs)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_abs)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 0)
            .append_opcode(Opcode::i64_neg)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_neg)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![DataType::I64, DataType::I64], // params
            vec![DataType::I64, DataType::I64, DataType::I64, DataType::I64], // results
            vec![],                             // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
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

        let code0 = BytecodeWriter::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_abs)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode(Opcode::f32_abs)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_neg)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode(Opcode::f32_neg)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 2)
            .append_opcode(Opcode::f32_ceil)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 4)
            .append_opcode(Opcode::f32_ceil)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 6)
            .append_opcode(Opcode::f32_ceil)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 8)
            .append_opcode(Opcode::f32_ceil)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 2)
            .append_opcode(Opcode::f32_floor)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 4)
            .append_opcode(Opcode::f32_floor)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 6)
            .append_opcode(Opcode::f32_floor)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 8)
            .append_opcode(Opcode::f32_floor)
            // group 3
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 2)
            .append_opcode(Opcode::f32_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 3)
            .append_opcode(Opcode::f32_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 4)
            .append_opcode(Opcode::f32_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 5)
            .append_opcode(Opcode::f32_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 6)
            .append_opcode(Opcode::f32_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 7)
            .append_opcode(Opcode::f32_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 8)
            .append_opcode(Opcode::f32_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 9)
            .append_opcode(Opcode::f32_round_half_away_from_zero)
            // group 4
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 2)
            .append_opcode(Opcode::f32_round_half_to_even)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 3)
            .append_opcode(Opcode::f32_round_half_to_even)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 4)
            .append_opcode(Opcode::f32_round_half_to_even)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 5)
            .append_opcode(Opcode::f32_round_half_to_even)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 6)
            .append_opcode(Opcode::f32_round_half_to_even)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 7)
            .append_opcode(Opcode::f32_round_half_to_even)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 8)
            .append_opcode(Opcode::f32_round_half_to_even)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 9)
            .append_opcode(Opcode::f32_round_half_to_even)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
            ], // params
            vec![
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                //
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                //
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                //
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                //
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
            ], // results
            vec![], // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
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

        let code0 = BytecodeWriter::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_trunc)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_fract)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode(Opcode::f32_sqrt)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 2)
            .append_opcode(Opcode::f32_cbrt)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 3)
            .append_opcode(Opcode::f32_exp)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 4)
            .append_opcode(Opcode::f32_exp2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 8)
            .append_opcode(Opcode::f32_ln)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode(Opcode::f32_log2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 7)
            .append_opcode(Opcode::f32_log10)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 9)
            .append_opcode(Opcode::f32_sin)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 9)
            .append_opcode(Opcode::f32_cos)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 9)
            .append_opcode(Opcode::f32_tan)
            .append_opcode_pesudo_f32(Opcode::f32_imm, 0.5)
            .append_opcode(Opcode::f32_asin)
            .append_opcode_pesudo_f32(Opcode::f32_imm, 0.866_025_4)
            .append_opcode(Opcode::f32_acos)
            .append_opcode_pesudo_f32(Opcode::f32_imm, 0.577_350_3)
            .append_opcode(Opcode::f32_atan)
            // group 3
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 3)
            .append_opcode(Opcode::f32_pow)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 3)
            .append_opcode(Opcode::f32_log)
            // group 4
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 3)
            .append_opcode(Opcode::f32_copysign)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 5)
            .append_opcode(Opcode::f32_copysign)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 5)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 4)
            .append_opcode(Opcode::f32_copysign)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 5)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 6)
            .append_opcode(Opcode::f32_copysign)
            // group 5
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 3)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 4)
            .append_opcode(Opcode::f32_min)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 5)
            .append_opcode(Opcode::f32_min)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 5)
            .append_opcode(Opcode::f32_max)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 5)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 6)
            .append_opcode(Opcode::f32_max)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                //
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
            ], // params
            vec![
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                //
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                //
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                //
                DataType::F32,
                DataType::F32,
                //
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
                //
                DataType::F32,
                DataType::F32,
                DataType::F32,
                DataType::F32,
            ], // results
            vec![], // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
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

        let code0 = BytecodeWriter::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_abs)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode(Opcode::f64_abs)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_neg)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode(Opcode::f64_neg)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 2)
            .append_opcode(Opcode::f64_ceil)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 4)
            .append_opcode(Opcode::f64_ceil)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 6)
            .append_opcode(Opcode::f64_ceil)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 8)
            .append_opcode(Opcode::f64_ceil)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 2)
            .append_opcode(Opcode::f64_floor)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 4)
            .append_opcode(Opcode::f64_floor)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 6)
            .append_opcode(Opcode::f64_floor)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 8)
            .append_opcode(Opcode::f64_floor)
            // group 3
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 2)
            .append_opcode(Opcode::f64_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 3)
            .append_opcode(Opcode::f64_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 4)
            .append_opcode(Opcode::f64_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 5)
            .append_opcode(Opcode::f64_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 6)
            .append_opcode(Opcode::f64_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 7)
            .append_opcode(Opcode::f64_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 8)
            .append_opcode(Opcode::f64_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 9)
            .append_opcode(Opcode::f64_round_half_away_from_zero)
            // group 4
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 2)
            .append_opcode(Opcode::f64_round_half_to_even)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 3)
            .append_opcode(Opcode::f64_round_half_to_even)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 4)
            .append_opcode(Opcode::f64_round_half_to_even)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 5)
            .append_opcode(Opcode::f64_round_half_to_even)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 6)
            .append_opcode(Opcode::f64_round_half_to_even)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 7)
            .append_opcode(Opcode::f64_round_half_to_even)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 8)
            .append_opcode(Opcode::f64_round_half_to_even)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 9)
            .append_opcode(Opcode::f64_round_half_to_even)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
            ], // params
            vec![
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                //
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                //
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                //
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                //
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
            ], // results
            vec![], // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
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

        let code0 = BytecodeWriter::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_trunc)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_fract)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode(Opcode::f64_sqrt)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 2)
            .append_opcode(Opcode::f64_cbrt)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 3)
            .append_opcode(Opcode::f64_exp)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 4)
            .append_opcode(Opcode::f64_exp2)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 8)
            .append_opcode(Opcode::f64_ln)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode(Opcode::f64_log2)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 7)
            .append_opcode(Opcode::f64_log10)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 9)
            .append_opcode(Opcode::f64_sin)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 9)
            .append_opcode(Opcode::f64_cos)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 9)
            .append_opcode(Opcode::f64_tan)
            .append_opcode_pesudo_f64(Opcode::f64_imm, 0.5)
            .append_opcode(Opcode::f64_asin)
            .append_opcode_pesudo_f64(Opcode::f64_imm, 0.8660254037844386)
            .append_opcode(Opcode::f64_acos)
            .append_opcode_pesudo_f64(Opcode::f64_imm, 0.5773502691896258)
            .append_opcode(Opcode::f64_atan)
            // group 3
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 3)
            .append_opcode(Opcode::f64_pow)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 3)
            .append_opcode(Opcode::f64_log)
            // group 4
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 3)
            .append_opcode(Opcode::f64_copysign)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 5)
            .append_opcode(Opcode::f64_copysign)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 5)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 4)
            .append_opcode(Opcode::f64_copysign)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 5)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 6)
            .append_opcode(Opcode::f64_copysign)
            // group 5
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 3)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 4)
            .append_opcode(Opcode::f64_min)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 5)
            .append_opcode(Opcode::f64_min)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 5)
            .append_opcode(Opcode::f64_max)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 5)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 6)
            .append_opcode(Opcode::f64_max)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                //
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
            ], // params
            vec![
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                //
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                //
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                //
                DataType::F64,
                DataType::F64,
                //
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
                //
                DataType::F64,
                DataType::F64,
                DataType::F64,
                DataType::F64,
            ], // results
            vec![], // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
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
