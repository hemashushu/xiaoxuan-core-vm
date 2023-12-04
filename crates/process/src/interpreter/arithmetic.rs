// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_program::thread_context::ThreadContext;

use super::InterpretResult;

pub fn i32_add(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left.wrapping_add(right));
    InterpretResult::Move(2)
}

pub fn i32_sub(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left.wrapping_sub(right));
    InterpretResult::Move(2)
}

pub fn i32_mul(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left.wrapping_mul(right));
    InterpretResult::Move(2)
}

pub fn i32_div_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_s(thread_context);
    store_i32_s(thread_context, left / right);
    InterpretResult::Move(2)
}

pub fn i32_div_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left / right);
    InterpretResult::Move(2)
}

pub fn i32_rem_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_s(thread_context);
    store_i32_s(thread_context, left % right);
    InterpretResult::Move(2)
}

pub fn i32_rem_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left % right);
    InterpretResult::Move(2)
}

pub fn i32_inc(thread_context: &mut ThreadContext) -> InterpretResult {
    let amount = thread_context.get_param_i16();
    let value = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, value.wrapping_add(amount as u32));
    InterpretResult::Move(4)
}

pub fn i32_dec(thread_context: &mut ThreadContext) -> InterpretResult {
    let amount = thread_context.get_param_i16();
    let value = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, value.wrapping_sub(amount as u32));
    InterpretResult::Move(4)
}

pub fn i64_add(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left.wrapping_add(right));
    InterpretResult::Move(2)
}

pub fn i64_sub(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left.wrapping_sub(right));
    InterpretResult::Move(2)
}

pub fn i64_mul(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left.wrapping_mul(right));
    InterpretResult::Move(2)
}

pub fn i64_div_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_s(thread_context);
    store_i64_s(thread_context, left / right);
    InterpretResult::Move(2)
}

pub fn i64_div_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left / right);
    InterpretResult::Move(2)
}

pub fn i64_rem_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_s(thread_context);
    store_i64_s(thread_context, left % right);
    InterpretResult::Move(2)
}

pub fn i64_rem_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left % right);
    InterpretResult::Move(2)
}

pub fn i64_inc(thread_context: &mut ThreadContext) -> InterpretResult {
    let amount = thread_context.get_param_i16();
    let value = load_operand_i64_u(thread_context);
    store_i64_u(thread_context, value.wrapping_add(amount as u64));
    InterpretResult::Move(4)
}

pub fn i64_dec(thread_context: &mut ThreadContext) -> InterpretResult {
    let amount = thread_context.get_param_i16();
    let value = load_operand_i64_u(thread_context);
    store_i64_u(thread_context, value.wrapping_sub(amount as u64));
    InterpretResult::Move(4)
}

pub fn f32_add(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f32(thread_context);
    store_f32(thread_context, left + right);
    InterpretResult::Move(2)
}

pub fn f32_sub(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f32(thread_context);
    store_f32(thread_context, left - right);
    InterpretResult::Move(2)
}

pub fn f32_mul(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f32(thread_context);
    store_f32(thread_context, left * right);
    InterpretResult::Move(2)
}

pub fn f32_div(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f32(thread_context);
    store_f32(thread_context, left / right);
    InterpretResult::Move(2)
}

pub fn f64_add(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f64(thread_context);
    store_f64(thread_context, left + right);
    InterpretResult::Move(2)
}

pub fn f64_sub(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f64(thread_context);
    store_f64(thread_context, left - right);
    InterpretResult::Move(2)
}

pub fn f64_mul(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f64(thread_context);
    store_f64(thread_context, left * right);
    InterpretResult::Move(2)
}

pub fn f64_div(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f64(thread_context);
    store_f64(thread_context, left / right);
    InterpretResult::Move(2)
}

#[inline]
fn load_operands_i32_s(thread_context: &mut ThreadContext) -> (i32, i32) {
    let right = thread_context.stack.pop_i32_s();
    let left = thread_context.stack.pop_i32_s();
    (left, right)
}

#[inline]
fn load_operand_i32_u(thread_context: &mut ThreadContext) -> u32 {
    thread_context.stack.pop_i32_u()
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
fn load_operand_i64_u(thread_context: &mut ThreadContext) -> u64 {
    thread_context.stack.pop_i64_u()
}

#[inline]
fn load_operands_f32(thread_context: &mut ThreadContext) -> (f32, f32) {
    let right = thread_context.stack.pop_f32();
    let left = thread_context.stack.pop_f32();
    (left, right)
}

#[inline]
fn load_operands_f64(thread_context: &mut ThreadContext) -> (f64, f64) {
    let right = thread_context.stack.pop_f64();
    let left = thread_context.stack.pop_f64();
    (left, right)
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
    use crate::{in_memory_program_source::InMemoryProgramSource, interpreter::process_function};
    use ancvm_binary::{
        bytecode_writer::BytecodeWriter, utils::helper_build_module_binary_with_single_function,
    };
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    #[test]
    fn test_interpreter_arithmetic_i32() {
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
        //   - inc   0 amount:3     -> 14
        //   - dec   0 amount:3     -> 8
        //   - inc   2 amount:3     -> -10
        //   - dec   2 amount:3     -> -16
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

        let code0 = BytecodeWriter::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_add)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode(Opcode::i32_sub)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_sub)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_mul)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode(Opcode::i32_div_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode(Opcode::i32_div_u)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_div_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_div_u)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode(Opcode::i32_rem_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_rem_u)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16(Opcode::i32_inc, 3)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16(Opcode::i32_dec, 3)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i16(Opcode::i32_inc, 3)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i16(Opcode::i32_dec, 3)
            // group 3
            .append_opcode_i32(Opcode::i32_imm, 0xffff_ffff)
            .append_opcode_i32(Opcode::i32_imm, 0x2)
            .append_opcode(Opcode::i32_add)
            .append_opcode_i32(Opcode::i32_imm, 0xf0e0_d0c0)
            .append_opcode_i32(Opcode::i32_imm, 0x2)
            .append_opcode(Opcode::i32_mul)
            .append_opcode_i32(Opcode::i32_imm, 0xffff_ffff)
            .append_opcode_i16(Opcode::i32_inc, 2)
            .append_opcode_i32(Opcode::i32_imm, 0x1)
            .append_opcode_i16(Opcode::i32_dec, 2)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32, DataType::I32], // params
            vec![
                // group 0
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                // group 1
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                // group 2
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                // group 3
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
            ], // results
            vec![],                                            // local vars
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
    fn test_interpreter_arithmetic_i64() {
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
        //   - inc   0 amount:3     -> 14
        //   - dec   0 amount:3     -> 8
        //   - inc   2 amount:3     -> -10
        //   - dec   2 amount:3     -> -16
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

        let code0 = BytecodeWriter::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_add)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 0)
            .append_opcode(Opcode::i64_sub)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_sub)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_mul)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode(Opcode::i64_div_s)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode(Opcode::i64_div_u)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_div_s)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_div_u)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode(Opcode::i64_rem_s)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_rem_u)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 0)
            .append_opcode_i16(Opcode::i64_inc, 3)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 0)
            .append_opcode_i16(Opcode::i64_dec, 3)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode_i16(Opcode::i64_inc, 3)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode_i16(Opcode::i64_dec, 3)
            // group 3
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0xffff_ffff_ffff_ffff)
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x2)
            .append_opcode(Opcode::i64_add)
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0xf0e0_d0c0_b0a0_9080)
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x2)
            .append_opcode(Opcode::i64_mul)
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0xffff_ffff_ffff_ffff)
            .append_opcode_i16(Opcode::i64_inc, 2)
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x1)
            .append_opcode_i16(Opcode::i64_dec, 2)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![DataType::I64, DataType::I64, DataType::I64], // params
            vec![
                //
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                //
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                //
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                //
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
            ], // results
            vec![],                                            // local vars
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
    fn test_interpreter_arithmetic_f32() {
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

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode(Opcode::f32_add)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_sub)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode(Opcode::f32_mul)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_div)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![DataType::F32, DataType::F32], // params
            vec![DataType::F32, DataType::F32, DataType::F32, DataType::F32], // results
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
    fn test_interpreter_arithmetic_f64() {
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

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode(Opcode::f64_add)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_sub)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode(Opcode::f64_mul)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_div)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![DataType::F64, DataType::F64], // params
            vec![DataType::F64, DataType::F64, DataType::F64, DataType::F64], // results
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
