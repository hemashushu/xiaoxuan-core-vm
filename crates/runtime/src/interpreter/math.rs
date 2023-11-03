// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_program::thread_context::ThreadContext;

use super::InterpretResult;

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

pub fn f32_pow(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f32(thread_context);
    store_f32(thread_context, left.powf(right));
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

pub fn f32_log(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f32(thread_context);
    store_f32(thread_context, left.log(right));
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

pub fn f64_pow(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f64(thread_context);
    store_f64(thread_context, left.powf(right));
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

pub fn f64_log(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f64(thread_context);
    store_f64(thread_context, left.log(right));
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
    fn test_interpreter_math_f32_a() {
        // numbers:
        //   - 0: 1.414
        //   - 1: -1.732
        //   - 2: 2.4
        //   - 3: 2.5
        //   - 4: 2.6
        //   - 5: -2.4
        //   - 6: -2.5
        //   - 7: -2.6
        //
        // functions:
        //   - abs      0   -> 1.414
        //   - abs      1   -> 1.732
        //   - neg      0   -> -1.414
        //   - neg      1   -> 1.732
        //
        //   - ceil     2   -> 3.0
        //   - ceil     4   -> 3.0
        //   - ceil     5   -> -2.0
        //   - ceil     7   -> -2.0
        //
        //   - floor    2   -> 2.0
        //   - floor    4   -> 2.0
        //   - floor    5   -> -3.0
        //   - floor    7   -> -3.0
        //
        //   - round_half_away_from_zero    2   -> 2.0
        //   - round_half_away_from_zero    3   -> 3.0
        //   - round_half_away_from_zero    4   -> 3.0
        //   - round_half_away_from_zero    5   -> -2.0
        //   - round_half_away_from_zero    6   -> -3.0
        //   - round_half_away_from_zero    7   -> -3.0

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_abs)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode(Opcode::f32_abs)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_neg)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode(Opcode::f32_neg)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 2)
            .append_opcode(Opcode::f32_ceil)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 4)
            .append_opcode(Opcode::f32_ceil)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 5)
            .append_opcode(Opcode::f32_ceil)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 7)
            .append_opcode(Opcode::f32_ceil)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 2)
            .append_opcode(Opcode::f32_floor)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 4)
            .append_opcode(Opcode::f32_floor)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 5)
            .append_opcode(Opcode::f32_floor)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 7)
            .append_opcode(Opcode::f32_floor)
            //
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
            ], // params
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
                ForeignValue::Float32(1.414),
                ForeignValue::Float32(-1.732),
                ForeignValue::Float32(2.4),
                ForeignValue::Float32(2.5),
                ForeignValue::Float32(2.6),
                ForeignValue::Float32(-2.4),
                ForeignValue::Float32(-2.5),
                ForeignValue::Float32(-2.6),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::Float32(1.414),
                ForeignValue::Float32(1.732),
                ForeignValue::Float32(-1.414),
                ForeignValue::Float32(1.732),
                //
                ForeignValue::Float32(3.0),
                ForeignValue::Float32(3.0),
                ForeignValue::Float32(-2.0),
                ForeignValue::Float32(-2.0),
                //
                ForeignValue::Float32(2.0),
                ForeignValue::Float32(2.0),
                ForeignValue::Float32(-3.0),
                ForeignValue::Float32(-3.0),
                //
                ForeignValue::Float32(2.0),
                ForeignValue::Float32(3.0),
                ForeignValue::Float32(3.0),
                ForeignValue::Float32(-2.0),
                ForeignValue::Float32(-3.0),
                ForeignValue::Float32(-3.0),
            ]
        );
    }

    #[test]
    fn test_interpreter_math_f32_b() {
        // numbers:
        //   - 0: 1.414
        //   - 1: 4.0
        //   - 2: 27.0
        //   - 3: 3.0
        //   - 4: 9.0
        //   - 5: 100.0
        //   - 6: 2.718281828
        //   - 7: 0.523598776   (deg 30)
        //
        // functions:
        //   - trunc   0        -> 1.0
        //   - fract   0        -> 0.414
        //   - sqrt    1        -> 2.0
        //   - cbrt    2        -> 3.0
        //
        //   - pow     1 3      -> 64.0 (4^3)
        //   - exp     3        -> 20.0855369232 (e^3)
        //   - exp2    4        -> 512.0
        //
        //   - ln      6        -> 1.0
        //   - log     4 3      -> 2.0 (log_3 9)
        //   - log2    1        -> 2.0 (log_2 4)
        //   - log10   5        -> 2.0 (log_10 100)
        //
        //   - sin     7        -> 0.5
        //   - cos     7        -> 0.866025403
        //   - tan     7        -> 0.577350269
        //   - asin    (0.5)    -> deg 30
        //   - acos    (0.86..) -> deg 30
        //   - atab    (0.57..) -> deg 30

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_trunc)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_fract)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode(Opcode::f32_sqrt)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 2)
            .append_opcode(Opcode::f32_cbrt)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 3)
            .append_opcode(Opcode::f32_pow)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 3)
            .append_opcode(Opcode::f32_exp)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 4)
            .append_opcode(Opcode::f32_exp2)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 6)
            .append_opcode(Opcode::f32_ln)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 3)
            .append_opcode(Opcode::f32_log)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode(Opcode::f32_log2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 5)
            .append_opcode(Opcode::f32_log10)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 7)
            .append_opcode(Opcode::f32_sin)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 7)
            .append_opcode(Opcode::f32_cos)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 7)
            .append_opcode(Opcode::f32_tan)
            .append_opcode_pesudo_f32(Opcode::f32_imm, 0.5)
            .append_opcode(Opcode::f32_asin)
            .append_opcode_pesudo_f32(Opcode::f32_imm, 0.866_025_4)
            .append_opcode(Opcode::f32_acos)
            .append_opcode_pesudo_f32(Opcode::f32_imm, 0.577_350_26)
            .append_opcode(Opcode::f32_atan)
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
            ], // params
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
                ForeignValue::Float32(1.414),
                ForeignValue::Float32(4.0),
                ForeignValue::Float32(27.0),
                ForeignValue::Float32(3.0),
                ForeignValue::Float32(9.0),
                ForeignValue::Float32(100.0),
                ForeignValue::Float32(std::f32::consts::E), // 2.718281828
                ForeignValue::Float32(std::f32::consts::FRAC_PI_6), // 0.523598776
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::Float32(1.0),
                ForeignValue::Float32(0.41400003),
                ForeignValue::Float32(2.0),
                ForeignValue::Float32(3.0),
                //
                ForeignValue::Float32(64.0),
                ForeignValue::Float32(20.085_537),
                ForeignValue::Float32(512.0),
                //
                ForeignValue::Float32(0.99999994), // 1.0
                ForeignValue::Float32(2.0),
                ForeignValue::Float32(2.0),
                ForeignValue::Float32(2.0),
                //
                ForeignValue::Float32(0.5),
                ForeignValue::Float32(0.8660254),
                ForeignValue::Float32(0.5773503),
                ForeignValue::Float32(std::f32::consts::FRAC_PI_6),
                ForeignValue::Float32(std::f32::consts::FRAC_PI_6),
                ForeignValue::Float32(std::f32::consts::FRAC_PI_6), // 0.5235988
            ]
        );
    }

    #[test]
    fn test_interpreter_math_f64_a() {
        // numbers:
        //   - 0: 1.414
        //   - 1: -1.732
        //   - 2: 2.4
        //   - 3: 2.5
        //   - 4: 2.6
        //   - 5: -2.4
        //   - 6: -2.5
        //   - 7: -2.6
        //
        // functions:
        //   - abs      0   -> 1.414
        //   - abs      1   -> 1.732
        //   - neg      0   -> -1.414
        //   - neg      1   -> 1.732
        //
        //   - ceil     2   -> 3.0
        //   - ceil     4   -> 3.0
        //   - ceil     5   -> -2.0
        //   - ceil     7   -> -2.0
        //
        //   - floor    2   -> 2.0
        //   - floor    4   -> 2.0
        //   - floor    5   -> -3.0
        //   - floor    7   -> -3.0
        //
        //   - round_half_away_from_zero    2   -> 2.0
        //   - round_half_away_from_zero    3   -> 3.0
        //   - round_half_away_from_zero    4   -> 3.0
        //   - round_half_away_from_zero    5   -> -2.0
        //   - round_half_away_from_zero    6   -> -3.0
        //   - round_half_away_from_zero    7   -> -3.0

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_abs)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode(Opcode::f64_abs)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_neg)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode(Opcode::f64_neg)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 2)
            .append_opcode(Opcode::f64_ceil)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            .append_opcode(Opcode::f64_ceil)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 5)
            .append_opcode(Opcode::f64_ceil)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 7)
            .append_opcode(Opcode::f64_ceil)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 2)
            .append_opcode(Opcode::f64_floor)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            .append_opcode(Opcode::f64_floor)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 5)
            .append_opcode(Opcode::f64_floor)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 7)
            .append_opcode(Opcode::f64_floor)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 2)
            .append_opcode(Opcode::f64_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 3)
            .append_opcode(Opcode::f64_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            .append_opcode(Opcode::f64_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 5)
            .append_opcode(Opcode::f64_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 6)
            .append_opcode(Opcode::f64_round_half_away_from_zero)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 7)
            .append_opcode(Opcode::f64_round_half_away_from_zero)
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
            ], // params
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
                ForeignValue::Float64(1.414),
                ForeignValue::Float64(-1.732),
                ForeignValue::Float64(2.4),
                ForeignValue::Float64(2.5),
                ForeignValue::Float64(2.6),
                ForeignValue::Float64(-2.4),
                ForeignValue::Float64(-2.5),
                ForeignValue::Float64(-2.6),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::Float64(1.414),
                ForeignValue::Float64(1.732),
                ForeignValue::Float64(-1.414),
                ForeignValue::Float64(1.732),
                //
                ForeignValue::Float64(3.0),
                ForeignValue::Float64(3.0),
                ForeignValue::Float64(-2.0),
                ForeignValue::Float64(-2.0),
                //
                ForeignValue::Float64(2.0),
                ForeignValue::Float64(2.0),
                ForeignValue::Float64(-3.0),
                ForeignValue::Float64(-3.0),
                //
                ForeignValue::Float64(2.0),
                ForeignValue::Float64(3.0),
                ForeignValue::Float64(3.0),
                ForeignValue::Float64(-2.0),
                ForeignValue::Float64(-3.0),
                ForeignValue::Float64(-3.0),
            ]
        );
    }

    #[test]
    fn test_interpreter_math_f64_b() {
        // numbers:
        //   - 0: 1.414
        //   - 1: 4.0
        //   - 2: 27.0
        //   - 3: 3.0
        //   - 4: 9.0
        //   - 5: 100.0
        //   - 6: 2.718281828               // std::f64::consts::E
        //   - 7: 0.523598776   (deg 30)    // std::f64::consts::FRAC_PI_6
        //
        // functions:
        //   - trunc   0        -> 1.0
        //   - fract   0        -> 0.414
        //   - sqrt    1        -> 2.0
        //   - cbrt    2        -> 3.0
        //
        //   - pow     1 3      -> 64.0 (4^3)
        //   - exp     3        -> 20.0855369232 (e^3)
        //   - exp2    4        -> 512.0
        //
        //   - ln      6        -> 1.0
        //   - log     4 3      -> 2.0 (log_3 9)
        //   - log2    1        -> 2.0 (log_2 4)
        //   - log10   5        -> 2.0 (log_10 100)
        //
        //   - sin     7        -> 0.5
        //   - cos     7        -> 0.866025403
        //   - tan     7        -> 0.577350269
        //   - asin    (0.5)    -> deg 30
        //   - acos    (0.86..) -> deg 30
        //   - atab    (0.57..) -> deg 30

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_trunc)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_fract)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode(Opcode::f64_sqrt)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 2)
            .append_opcode(Opcode::f64_cbrt)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 3)
            .append_opcode(Opcode::f64_pow)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 3)
            .append_opcode(Opcode::f64_exp)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            .append_opcode(Opcode::f64_exp2)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 6)
            .append_opcode(Opcode::f64_ln)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 3)
            .append_opcode(Opcode::f64_log)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode(Opcode::f64_log2)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 5)
            .append_opcode(Opcode::f64_log10)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 7)
            .append_opcode(Opcode::f64_sin)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 7)
            .append_opcode(Opcode::f64_cos)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 7)
            .append_opcode(Opcode::f64_tan)
            .append_opcode_pesudo_f64(Opcode::f64_imm, 0.5)
            .append_opcode(Opcode::f64_asin)
            .append_opcode_pesudo_f64(Opcode::f64_imm, 0.866025403)
            .append_opcode(Opcode::f64_acos)
            .append_opcode_pesudo_f64(Opcode::f64_imm, 0.577350269)
            .append_opcode(Opcode::f64_atan)
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
            ], // params
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
                ForeignValue::Float64(1.414),
                ForeignValue::Float64(4.0),
                ForeignValue::Float64(27.0),
                ForeignValue::Float64(3.0),
                ForeignValue::Float64(9.0),
                ForeignValue::Float64(100.0),
                ForeignValue::Float64(std::f64::consts::E),
                ForeignValue::Float64(std::f64::consts::FRAC_PI_6),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::Float64(1.0),
                ForeignValue::Float64(0.4139999999999999),
                ForeignValue::Float64(2.0),
                ForeignValue::Float64(3.0000000000000004),
                //
                ForeignValue::Float64(64.0),
                ForeignValue::Float64(20.085536923187668),
                ForeignValue::Float64(512.0),
                //
                ForeignValue::Float64(1.0),
                ForeignValue::Float64(2.0),
                ForeignValue::Float64(2.0),
                ForeignValue::Float64(2.0),
                //
                ForeignValue::Float64(0.5),
                ForeignValue::Float64(0.8660254037844386),
                ForeignValue::Float64(0.5773502691896258),
                ForeignValue::Float64(std::f64::consts::FRAC_PI_6),
                ForeignValue::Float64(0.5235987771671762),
                ForeignValue::Float64(0.5235987754560796),
            ]
        );
    }
}
