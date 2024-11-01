// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::thread_context::ThreadContext;

use super::HandleResult;

pub fn i32_and(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left & right);
    HandleResult::Move(2)
}

pub fn i32_or(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left | right);
    HandleResult::Move(2)
}

pub fn i32_xor(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left ^ right);
    HandleResult::Move(2)
}

pub fn i32_not(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, !v);
    HandleResult::Move(2)
}

pub fn i32_leading_zeros(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, v.leading_zeros());
    HandleResult::Move(2)
}

pub fn i32_leading_ones(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, v.leading_ones());
    HandleResult::Move(2)
}

pub fn i32_trailing_zeros(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, v.trailing_zeros());
    HandleResult::Move(2)
}

pub fn i32_count_ones(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, v.count_ones());
    HandleResult::Move(2)
}

pub fn i32_shift_left(thread_context: &mut ThreadContext) -> HandleResult {
    let bits = load_operand_i32_u(thread_context);
    let number = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, number << bits);
    HandleResult::Move(2)
}

pub fn i32_shift_right_s(thread_context: &mut ThreadContext) -> HandleResult {
    let bits = load_operand_i32_u(thread_context);
    let number = load_operand_i32_s(thread_context);
    store_i32_s(thread_context, number >> bits);
    HandleResult::Move(2)
}

pub fn i32_shift_right_u(thread_context: &mut ThreadContext) -> HandleResult {
    let bits = load_operand_i32_u(thread_context);
    let number = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, number >> bits);
    HandleResult::Move(2)
}

pub fn i32_rotate_left(thread_context: &mut ThreadContext) -> HandleResult {
    let bits = load_operand_i32_u(thread_context);
    let number = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, number.rotate_left(bits));
    HandleResult::Move(2)
}

pub fn i32_rotate_right(thread_context: &mut ThreadContext) -> HandleResult {
    let bits = load_operand_i32_u(thread_context);
    let number = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, number.rotate_right(bits));
    HandleResult::Move(2)
}

pub fn i64_and(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left & right);
    HandleResult::Move(2)
}

pub fn i64_or(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left | right);
    HandleResult::Move(2)
}

pub fn i64_xor(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left ^ right);
    HandleResult::Move(2)
}

pub fn i64_not(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_i64_u(thread_context);
    store_i64_u(thread_context, !v);
    HandleResult::Move(2)
}

pub fn i64_leading_zeros(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_i64_u(thread_context);
    store_i32_u(thread_context, v.leading_zeros()); // the result of 'clz' is u32
    HandleResult::Move(2)
}

pub fn i64_leading_ones(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_i64_u(thread_context);
    store_i32_u(thread_context, v.leading_ones()); // the result of 'cls' is u32
    HandleResult::Move(2)
}

pub fn i64_trailing_zeros(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_i64_u(thread_context);
    store_i32_u(thread_context, v.trailing_zeros()); // the result of 'ctz' is u32
    HandleResult::Move(2)
}

pub fn i64_count_ones(thread_context: &mut ThreadContext) -> HandleResult {
    let v = load_operand_i64_u(thread_context);
    store_i32_u(thread_context, v.count_ones()); // the result of 'popcnt' is u32
    HandleResult::Move(2)
}

pub fn i64_shift_left(thread_context: &mut ThreadContext) -> HandleResult {
    let bits = load_operand_i32_u(thread_context); // the type of 'bits' is u32
    let number = load_operand_i64_u(thread_context);
    store_i64_u(thread_context, number << bits);
    HandleResult::Move(2)
}

pub fn i64_shift_right_s(thread_context: &mut ThreadContext) -> HandleResult {
    let bits = load_operand_i32_u(thread_context); // the type of 'bits' is u32
    let number = load_operand_i64_s(thread_context);
    store_i64_s(thread_context, number >> bits);
    HandleResult::Move(2)
}

pub fn i64_shift_right_u(thread_context: &mut ThreadContext) -> HandleResult {
    let bits = load_operand_i32_u(thread_context); // the type of 'bits' is u32
    let number = load_operand_i64_u(thread_context);
    store_i64_u(thread_context, number >> bits);
    HandleResult::Move(2)
}

pub fn i64_rotate_left(thread_context: &mut ThreadContext) -> HandleResult {
    let bits = load_operand_i32_u(thread_context); // the type of 'bits' is u32
    let number = load_operand_i64_u(thread_context);
    store_i64_u(thread_context, number.rotate_left(bits));
    HandleResult::Move(2)
}

pub fn i64_rotate_right(thread_context: &mut ThreadContext) -> HandleResult {
    let bits = load_operand_i32_u(thread_context); // the type of 'bits' is u32
    let number = load_operand_i64_u(thread_context);
    store_i64_u(thread_context, number.rotate_right(bits));
    HandleResult::Move(2)
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
    use crate::{in_memory_program_resource::InMemoryProgramResource, interpreter::process_function};
    use ancvm_binary::{
        bytecode_writer::BytecodeWriter, utils::helper_build_module_binary_with_single_function,
    };
    use ancvm_context::program_resource::ProgramResource;
    use ancvm_isa::{opcode::Opcode, OperandDataType, ForeignValue};

    #[test]
    fn test_interpreter_bitwise_i32() {
        // numbers:
        //   - 0: 0xff0000ff
        //   - 1: 0xf0f000ff
        //   - 2: 0x00f00000
        //   - 3: 0x80000000

        // arithemtic:
        //   group 0:
        //   - and       0 1      -> 0xf00000ff
        //   - or        0 1      -> 0xfff000ff
        //   - xor       0 1      -> 0x0ff00000
        //
        //   group 1:
        //   - shift_l   2 imm:4    -> 0x0f000000
        //   - shift_r_s 3 imm:16   -> 0xffff8000
        //   - shift_r_u 3 imm:16   -> 0x00008000
        //
        //   group 2:
        //   - shift_l   2 imm:24   -> 0x00000000
        //   - rotate_l  2 imm:24   -> 0x0000f000
        //   - shift_r_u 2 imm:28   -> 0x00000000
        //   - rotate_r  2 imm:28   -> 0x0f000000
        //
        //   group 3:
        //   - not       0        -> 0x00ffff00
        //   - cls       0        -> 8
        //   - cls       1        -> 4
        //   - clz       2        -> 8
        //   - ctz       2        -> 20
        //   - ones      2        -> 4
        //
        // (i32 i32 i32 i32) -> (i32 i32 i32  i32 i32 i32  i32 i32 i32 i32  i32 i32 i32 i32 i32 i32)

        let code0 = BytecodeWriterHelper::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_and)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode(Opcode::i32_or)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_xor)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i32(Opcode::imm_i32, 4)
            .append_opcode(Opcode::i32_shift_left)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 3)
            .append_opcode_i32(Opcode::imm_i32, 16)
            .append_opcode(Opcode::i32_shift_right_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 3)
            .append_opcode_i32(Opcode::imm_i32, 16)
            .append_opcode(Opcode::i32_shift_right_u)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i32(Opcode::imm_i32, 24)
            .append_opcode(Opcode::i32_shift_left)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i32(Opcode::imm_i32, 24)
            .append_opcode(Opcode::i32_rotate_left)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i32(Opcode::imm_i32, 28)
            .append_opcode(Opcode::i32_shift_right_u)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i32(Opcode::imm_i32, 28)
            .append_opcode(Opcode::i32_rotate_right)
            // group 3
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode(Opcode::i32_not)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode(Opcode::i32_leading_ones)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_leading_ones)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode(Opcode::i32_leading_zeros)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode(Opcode::i32_trailing_zeros)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode(Opcode::i32_count_ones)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![OperandDataType::I32, OperandDataType::I32, OperandDataType::I32, OperandDataType::I32], // params
            vec![
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                //
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                //
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                //
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                //
            ], // results
            vec![],                                                           // local vars
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::U32(0xff0000ff),
                ForeignValue::U32(0xf0f000ff),
                ForeignValue::U32(0x00f00000),
                ForeignValue::U32(0x80000000),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::U32(0xf00000ff),
                ForeignValue::U32(0xfff000ff),
                ForeignValue::U32(0x0ff00000),
                // group 1
                ForeignValue::U32(0x0f000000),
                ForeignValue::U32(0xffff8000),
                ForeignValue::U32(0x00008000),
                // group 2
                ForeignValue::U32(0x00000000),
                ForeignValue::U32(0x0000f000),
                ForeignValue::U32(0x00000000),
                ForeignValue::U32(0x0f000000),
                // group 3
                ForeignValue::U32(0x00ffff00),
                ForeignValue::U32(8),
                ForeignValue::U32(4),
                ForeignValue::U32(8),
                ForeignValue::U32(20),
                ForeignValue::U32(4),
            ]
        );
    }

    #[test]
    fn test_interpreter_bitwise_i64() {
        // numbers:
        //   - 0: 0xff00ff00_00ff00ff
        //   - 1: 0xf0f00f0f_00ff00ff
        //   - 2: 0x0000ff00_00000000
        //   - 3: 0x80000000_00000000

        // arithemtic:
        //   group 0:
        //   - and       0 1      -> 0xf0000f00_00ff00ff
        //   - or        0 1      -> 0xfff0ff0f_00ff00ff
        //   - xor       0 1      -> 0x0ff0f00f_00000000
        //
        //   group 1:
        //   - shift_l   2 8      -> 0x00ff0000_00000000
        //   - shift_r_s 3 16     -> 0xffff8000_00000000
        //   - shift_r_u 3 16     -> 0x00008000_00000000
        //
        //   group 2:
        //   - shift_l   2 32     -> 0x00000000_00000000
        //   - rotate_l  2 32     -> 0x00000000_0000ff00
        //   - shift_r_u 2 56     -> 0x00000000_00000000
        //   - rotate_r  2 56     -> 0x00ff0000_00000000
        //
        //   group 3:
        //   - not       0        -> 0x00ff00ff_ff00ff00
        //   - cls       0        -> 8
        //   - cls       1        -> 4
        //   - clz       2        -> 16
        //   - ctz       2        -> 40
        //   - ones      2        -> 8
        //
        // (i64 i64 i64 i64) -> (i64 i64 i64  i64 i64 i64  i64 i64 i64 i64  i64 i32 i32 i32 i32 i32)

        let code0 = BytecodeWriterHelper::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_and)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_or)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_xor)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode_i32(Opcode::imm_i32, 8)
            .append_opcode(Opcode::i64_shift_left)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 3)
            .append_opcode_i32(Opcode::imm_i32, 16)
            .append_opcode(Opcode::i64_shift_right_s)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 3)
            .append_opcode_i32(Opcode::imm_i32, 16)
            .append_opcode(Opcode::i64_shift_right_u)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode_i32(Opcode::imm_i32, 32)
            .append_opcode(Opcode::i64_shift_left)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode_i32(Opcode::imm_i32, 32)
            .append_opcode(Opcode::i64_rotate_left)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode_i32(Opcode::imm_i32, 56)
            .append_opcode(Opcode::i64_shift_right_u)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode_i32(Opcode::imm_i32, 56)
            .append_opcode(Opcode::i64_rotate_right)
            // group 3
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0)
            .append_opcode(Opcode::i64_not)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0)
            .append_opcode(Opcode::i64_leading_ones)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_leading_ones)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode(Opcode::i64_leading_zeros)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode(Opcode::i64_trailing_zeros)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode(Opcode::i64_count_ones)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![OperandDataType::I64, OperandDataType::I64, OperandDataType::I64, OperandDataType::I64], // params
            vec![
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                //
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
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                //
            ], // results
            vec![],                                                           // local vars
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::U64(0xff00ff00_00ff00ff),
                ForeignValue::U64(0xf0f00f0f_00ff00ff),
                ForeignValue::U64(0x0000ff00_00000000),
                ForeignValue::U64(0x80000000_00000000),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::U64(0xf0000f00_00ff00ff),
                ForeignValue::U64(0xfff0ff0f_00ff00ff),
                ForeignValue::U64(0x0ff0f00f_00000000),
                // group 1
                ForeignValue::U64(0x00ff0000_00000000),
                ForeignValue::U64(0xffff8000_00000000),
                ForeignValue::U64(0x00008000_00000000),
                // group 2
                ForeignValue::U64(0x00000000_00000000),
                ForeignValue::U64(0x00000000_0000ff00),
                ForeignValue::U64(0x00000000_00000000),
                ForeignValue::U64(0x00ff0000_00000000),
                // group 3
                ForeignValue::U64(0x00ff00ff_ff00ff00),
                ForeignValue::U32(8),
                ForeignValue::U32(4),
                ForeignValue::U32(16),
                ForeignValue::U32(40),
                ForeignValue::U32(8),
            ]
        );
    }
}
