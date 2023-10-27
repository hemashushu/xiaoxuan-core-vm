// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_program::thread_context::ThreadContext;

use super::InterpretResult;

pub fn i32_and(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left & right);
    InterpretResult::Move(2)
}

pub fn i32_or(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left | right);
    InterpretResult::Move(2)
}

pub fn i32_xor(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left ^ right);
    InterpretResult::Move(2)
}

pub fn i32_not(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, !v);
    InterpretResult::Move(2)
}

pub fn i32_leading_zeros(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, v.leading_zeros());
    InterpretResult::Move(2)
}

pub fn i32_trailing_zeros(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, v.trailing_zeros());
    InterpretResult::Move(2)
}

pub fn i32_count_ones(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, v.count_ones());
    InterpretResult::Move(2)
}

pub fn i32_shift_left(thread_context: &mut ThreadContext) -> InterpretResult {
    let bits = load_operand_i32_u(thread_context);
    let number = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, number << bits);
    InterpretResult::Move(2)
}

pub fn i32_shift_right_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let bits = load_operand_i32_u(thread_context);
    let number = load_operand_i32_s(thread_context);
    store_i32_s(thread_context, number >> bits);
    InterpretResult::Move(2)
}

pub fn i32_shift_right_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let bits = load_operand_i32_u(thread_context);
    let number = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, number >> bits);
    InterpretResult::Move(2)
}

pub fn i32_rotate_left(thread_context: &mut ThreadContext) -> InterpretResult {
    let bits = load_operand_i32_u(thread_context);
    let number = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, number.rotate_left(bits));
    InterpretResult::Move(2)
}

pub fn i32_rotate_right(thread_context: &mut ThreadContext) -> InterpretResult {
    let bits = load_operand_i32_u(thread_context);
    let number = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, number.rotate_right(bits));
    InterpretResult::Move(2)
}

pub fn i64_and(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left & right);
    InterpretResult::Move(2)
}

pub fn i64_or(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left | right);
    InterpretResult::Move(2)
}

pub fn i64_xor(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left ^ right);
    InterpretResult::Move(2)
}

pub fn i64_not(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_i64_u(thread_context);
    store_i64_u(thread_context, !v);
    InterpretResult::Move(2)
}

pub fn i64_leading_zeros(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_i64_u(thread_context);
    store_i32_u(thread_context, v.leading_zeros()); // the result of 'clz' is u32
    InterpretResult::Move(2)
}

pub fn i64_trailing_zeros(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_i64_u(thread_context);
    store_i32_u(thread_context, v.trailing_zeros()); // the result of 'ctz' is u32
    InterpretResult::Move(2)
}

pub fn i64_count_ones(thread_context: &mut ThreadContext) -> InterpretResult {
    let v = load_operand_i64_u(thread_context);
    store_i32_u(thread_context, v.count_ones()); // the result of 'popcnt' is u32
    InterpretResult::Move(2)
}

pub fn i64_shift_left(thread_context: &mut ThreadContext) -> InterpretResult {
    let bits = load_operand_i32_u(thread_context); // the type of 'bits' is u32
    let number = load_operand_i64_u(thread_context);
    store_i64_u(thread_context, number << bits);
    InterpretResult::Move(2)
}

pub fn i64_shift_right_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let bits = load_operand_i32_u(thread_context); // the type of 'bits' is u32
    let number = load_operand_i64_s(thread_context);
    store_i64_s(thread_context, number >> bits);
    InterpretResult::Move(2)
}

pub fn i64_shift_right_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let bits = load_operand_i32_u(thread_context); // the type of 'bits' is u32
    let number = load_operand_i64_u(thread_context);
    store_i64_u(thread_context, number >> bits);
    InterpretResult::Move(2)
}

pub fn i64_rotate_left(thread_context: &mut ThreadContext) -> InterpretResult {
    let bits = load_operand_i32_u(thread_context); // the type of 'bits' is u32
    let number = load_operand_i64_u(thread_context);
    store_i64_u(thread_context, number.rotate_left(bits));
    InterpretResult::Move(2)
}

pub fn i64_rotate_right(thread_context: &mut ThreadContext) -> InterpretResult {
    let bits = load_operand_i32_u(thread_context); // the type of 'bits' is u32
    let number = load_operand_i64_u(thread_context);
    store_i64_u(thread_context, number.rotate_right(bits));
    InterpretResult::Move(2)
}

#[inline]
fn load_operand_i32_s(thread_context: &mut ThreadContext) -> i32 {
    thread_context.stack.pop_i32_s()
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
fn load_operand_i64_u(thread_context: &mut ThreadContext) -> u64 {
    thread_context.stack.pop_i64_u()
}

#[inline]
fn load_operand_i64_s(thread_context: &mut ThreadContext) -> i64 {
    thread_context.stack.pop_i64_s()
}

#[inline]
fn load_operands_i64_u(thread_context: &mut ThreadContext) -> (u64, u64) {
    let right = thread_context.stack.pop_i64_u();
    let left = thread_context.stack.pop_i64_u();
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

#[cfg(test)]
mod tests {
    use crate::{in_memory_program_source::InMemoryProgramSource, interpreter::process_function};
    use ancvm_binary::utils::{build_module_binary_with_single_function, BytecodeWriter};
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    #[test]
    fn test_process_bitwise_i32() {
        // numbers:
        //   - 0: 0xff0000ff
        //   - 1: 0xf0f000ff
        //   - 2: 0x00f00000
        //   - 3: 0x80000000

        // arithemtic:
        //   - and       0 1      -> 0xf00000ff
        //   - or        0 1      -> 0xfff000ff
        //   - xor       0 1      -> 0x0ff00000
        //   - not       0        -> 0x00ffff00
        //
        //   - lz        2        -> 8
        //   - tz        2        -> 20
        //   - ones      2        -> 4
        //
        //   - shift_l   2 4      -> 0x0f000000
        //   - shift_r_s 3 16     -> 0xffff8000
        //   - shift_r_u 3 16     -> 0x00008000
        //
        //   - shift_l   2 24     -> 0x00000000
        //   - rotate_l  2 24     -> 0x0000f000
        //   - shift_r_u 2 28     -> 0x00000000
        //   - rotate_r  2 28     -> 0x0f000000

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_and)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode(Opcode::i32_or)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_xor)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode(Opcode::i32_not)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode(Opcode::i32_leading_zeros)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode(Opcode::i32_trailing_zeros)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode(Opcode::i32_count_ones)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode_i32(Opcode::i32_imm, 4)
            .write_opcode(Opcode::i32_shift_left)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 3)
            .write_opcode_i32(Opcode::i32_imm, 16)
            .write_opcode(Opcode::i32_shift_right_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 3)
            .write_opcode_i32(Opcode::i32_imm, 16)
            .write_opcode(Opcode::i32_shift_right_u)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode_i32(Opcode::i32_imm, 24)
            .write_opcode(Opcode::i32_shift_left)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode_i32(Opcode::i32_imm, 24)
            .write_opcode(Opcode::i32_rotate_left)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode_i32(Opcode::i32_imm, 28)
            .write_opcode(Opcode::i32_shift_right_u)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode_i32(Opcode::i32_imm, 28)
            .write_opcode(Opcode::i32_rotate_right)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code0).to_text());

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32, DataType::I32, DataType::I32], // params
            vec![
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                //
                DataType::I32,
                DataType::I32,
                DataType::I32,
                //
                DataType::I32,
                DataType::I32,
                DataType::I32,
                //
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                //
            ], // results
            vec![],                                                           // local vars
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
                ForeignValue::UInt32(0xff0000ff),
                ForeignValue::UInt32(0xf0f000ff),
                ForeignValue::UInt32(0x00f00000),
                ForeignValue::UInt32(0x80000000),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(0xf00000ff),
                ForeignValue::UInt32(0xfff000ff),
                ForeignValue::UInt32(0x0ff00000),
                ForeignValue::UInt32(0x00ffff00),
                //
                ForeignValue::UInt32(8),
                ForeignValue::UInt32(20),
                ForeignValue::UInt32(4),
                //
                ForeignValue::UInt32(0x0f000000),
                ForeignValue::UInt32(0xffff8000),
                ForeignValue::UInt32(0x00008000),
                //
                ForeignValue::UInt32(0x00000000),
                ForeignValue::UInt32(0x0000f000),
                ForeignValue::UInt32(0x00000000),
                ForeignValue::UInt32(0x0f000000),
            ]
        );
    }

    #[test]
    fn test_process_bitwise_i64() {
        // numbers:
        //   - 0: 0xff00ff00_00ff00ff
        //   - 1: 0xf0f00f0f_00ff00ff
        //   - 2: 0x0000ff00_00000000
        //   - 3: 0x80000000_00000000

        // arithemtic:
        //   - and       0 1      -> 0xf0000f00_00ff00ff
        //   - or        0 1      -> 0xfff0ff0f_00ff00ff
        //   - xor       0 1      -> 0x0ff0f00f_00000000
        //   - not       0        -> 0x00ff00ff_ff00ff00
        //
        //   - lz        2        -> 16
        //   - tz        2        -> 40
        //   - ones      2        -> 8
        //
        //   - shift_l   2 8      -> 0x00ff0000_00000000
        //   - shift_r_s 3 16     -> 0xffff8000_00000000
        //   - shift_r_u 3 16     -> 0x00008000_00000000
        //
        //   - shift_l   2 32     -> 0x00000000_00000000
        //   - rotate_l  2 32     -> 0x00000000_0000ff00
        //   - shift_r_u 2 56     -> 0x00000000_00000000
        //   - rotate_r  2 56     -> 0x00ff0000_00000000

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::i64_and)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::i64_or)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::i64_xor)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 0)
            .write_opcode(Opcode::i64_not)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode(Opcode::i64_leading_zeros)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode(Opcode::i64_trailing_zeros)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode(Opcode::i64_count_ones)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode_i32(Opcode::i32_imm, 8)
            .write_opcode(Opcode::i64_shift_left)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 3)
            .write_opcode_i32(Opcode::i32_imm, 16)
            .write_opcode(Opcode::i64_shift_right_s)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 3)
            .write_opcode_i32(Opcode::i32_imm, 16)
            .write_opcode(Opcode::i64_shift_right_u)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode_i32(Opcode::i32_imm, 32)
            .write_opcode(Opcode::i64_shift_left)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode_i32(Opcode::i32_imm, 32)
            .write_opcode(Opcode::i64_rotate_left)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode_i32(Opcode::i32_imm, 56)
            .write_opcode(Opcode::i64_shift_right_u)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode_i32(Opcode::i32_imm, 56)
            .write_opcode(Opcode::i64_rotate_right)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code0).to_text());

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I64, DataType::I64, DataType::I64, DataType::I64], // params
            vec![
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                //
                DataType::I32,
                DataType::I32,
                DataType::I32,
                //
                DataType::I64,
                DataType::I64,
                DataType::I64,
                //
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                //
            ], // results
            vec![],                                                           // local vars
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
                ForeignValue::UInt64(0xff00ff00_00ff00ff),
                ForeignValue::UInt64(0xf0f00f0f_00ff00ff),
                ForeignValue::UInt64(0x0000ff00_00000000),
                ForeignValue::UInt64(0x80000000_00000000),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt64(0xf0000f00_00ff00ff),
                ForeignValue::UInt64(0xfff0ff0f_00ff00ff),
                ForeignValue::UInt64(0x0ff0f00f_00000000),
                ForeignValue::UInt64(0x00ff00ff_ff00ff00),
                //
                ForeignValue::UInt32(16),
                ForeignValue::UInt32(40),
                ForeignValue::UInt32(8),
                //
                ForeignValue::UInt64(0x00ff0000_00000000),
                ForeignValue::UInt64(0xffff8000_00000000),
                ForeignValue::UInt64(0x00008000_00000000),
                //
                ForeignValue::UInt64(0x00000000_00000000),
                ForeignValue::UInt64(0x00000000_0000ff00),
                ForeignValue::UInt64(0x00000000_00000000),
                ForeignValue::UInt64(0x00ff0000_00000000),
            ]
        );
    }
}
