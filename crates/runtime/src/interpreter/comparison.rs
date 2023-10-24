// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_program::thread_context::ThreadContext;

use super::InterpretResult;

pub fn i32_eqz(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_i32_u();
    store_bool(thread_context, value == 0);
    InterpretResult::Move(2)
}

pub fn i32_nez(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_i32_u();
    store_bool(thread_context, value != 0);
    InterpretResult::Move(2)
}

pub fn i32_eq(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_bool(thread_context, left == right);
    InterpretResult::Move(2)
}

pub fn i32_ne(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_bool(thread_context, left != right);
    InterpretResult::Move(2)
}

pub fn i32_lt_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_s(thread_context);
    store_bool(thread_context, left < right);
    InterpretResult::Move(2)
}

pub fn i32_lt_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_bool(thread_context, left < right);
    InterpretResult::Move(2)
}

pub fn i32_gt_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_s(thread_context);
    store_bool(thread_context, left > right);
    InterpretResult::Move(2)
}

pub fn i32_gt_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_bool(thread_context, left > right);
    InterpretResult::Move(2)
}

pub fn i32_le_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_s(thread_context);
    store_bool(thread_context, left <= right);
    InterpretResult::Move(2)
}

pub fn i32_le_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_bool(thread_context, left <= right);
    InterpretResult::Move(2)
}

pub fn i32_ge_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_s(thread_context);
    store_bool(thread_context, left >= right);
    InterpretResult::Move(2)
}

pub fn i32_ge_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_bool(thread_context, left >= right);
    InterpretResult::Move(2)
}

pub fn i64_eqz(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_i64_u();
    store_bool(thread_context, value == 0);
    InterpretResult::Move(2)
}

pub fn i64_nez(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_i64_u();
    store_bool(thread_context, value != 0);
    InterpretResult::Move(2)
}

pub fn i64_eq(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_bool(thread_context, left == right);
    InterpretResult::Move(2)
}

pub fn i64_ne(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_bool(thread_context, left != right);
    InterpretResult::Move(2)
}

pub fn i64_lt_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_s(thread_context);
    store_bool(thread_context, left < right);
    InterpretResult::Move(2)
}

pub fn i64_lt_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_bool(thread_context, left < right);
    InterpretResult::Move(2)
}

pub fn i64_gt_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_s(thread_context);
    store_bool(thread_context, left > right);
    InterpretResult::Move(2)
}

pub fn i64_gt_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_bool(thread_context, left > right);
    InterpretResult::Move(2)
}

pub fn i64_le_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_s(thread_context);
    store_bool(thread_context, left <= right);
    InterpretResult::Move(2)
}

pub fn i64_le_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_bool(thread_context, left <= right);
    InterpretResult::Move(2)
}

pub fn i64_ge_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_s(thread_context);
    store_bool(thread_context, left >= right);
    InterpretResult::Move(2)
}

pub fn i64_ge_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_bool(thread_context, left >= right);
    InterpretResult::Move(2)
}

pub fn f32_eq(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f32(thread_context);
    store_bool(thread_context, left == right);
    InterpretResult::Move(2)
}

pub fn f32_ne(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f32(thread_context);
    store_bool(thread_context, left != right);
    InterpretResult::Move(2)
}

pub fn f32_lt(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f32(thread_context);
    store_bool(thread_context, left < right);
    InterpretResult::Move(2)
}

pub fn f32_gt(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f32(thread_context);
    store_bool(thread_context, left > right);
    InterpretResult::Move(2)
}

pub fn f32_le(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f32(thread_context);
    store_bool(thread_context, left <= right);
    InterpretResult::Move(2)
}

pub fn f32_ge(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f32(thread_context);
    store_bool(thread_context, left >= right);
    InterpretResult::Move(2)
}

pub fn f64_eq(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f64(thread_context);
    store_bool(thread_context, left == right);
    InterpretResult::Move(2)
}

pub fn f64_ne(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f64(thread_context);
    store_bool(thread_context, left != right);
    InterpretResult::Move(2)
}

pub fn f64_lt(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f64(thread_context);
    store_bool(thread_context, left < right);
    InterpretResult::Move(2)
}

pub fn f64_gt(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f64(thread_context);
    store_bool(thread_context, left > right);
    InterpretResult::Move(2)
}

pub fn f64_le(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f64(thread_context);
    store_bool(thread_context, left <= right);
    InterpretResult::Move(2)
}

pub fn f64_ge(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_f64(thread_context);
    store_bool(thread_context, left >= right);
    InterpretResult::Move(2)
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
fn store_bool(thread_context: &mut ThreadContext, b: bool) {
    let v = if b { 1u32 } else { 0u32 };
    thread_context.stack.push_i32_u(v);
}

#[cfg(test)]
mod tests {
    use crate::{in_memory_program_source::InMemoryProgramSource, interpreter::process_function};
    use ancvm_binary::utils::{build_module_binary_with_single_function, BytecodeWriter};
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    #[test]
    fn test_process_comparison_i32() {
        //

        // numbers:
        //   - 0: 0
        //   - 1: 11
        //   - 2: 13
        //   - 3: -7
        // comparison:
        //   - eqz  0         -> 1
        //   - eqz  11        -> 0
        //   - nez  0         -> 0
        //   - nez  11        -> 1
        //   - eq   11 13     -> 0
        //   - ne   11 13     -> 1
        //   - eq   11 11     -> 1
        //   - ne   11 11     -> 0
        //   - lt_s 13 -7     -> 0
        //   - lt_u 13 -7     -> 1
        //   - gt_s 13 -7     -> 1
        //   - gt_u 13 -7     -> 0
        //   - le_s 13 11     -> 0
        //   - le_u 13 11     -> 0
        //   - le_s 11 11     -> 1
        //   - le_u 11 11     -> 1
        //   - ge_s 11 13     -> 0
        //   - ge_u 11 13     -> 0
        //   - ge_s 11 11     -> 1
        //   - ge_u 11 11     -> 1

        // bytecode
        //
        // 0x0000 local_load32         0 0
        // 0x0008 i32_eqz
        // 0x000a nop
        // 0x000c local_load32         0 1
        // 0x0014 i32_eqz
        // 0x0016 nop
        // 0x0018 local_load32         0 0
        // 0x0020 i32_nez
        // 0x0022 nop
        // 0x0024 local_load32         0 1
        // 0x002c i32_nez
        // 0x002e nop
        // 0x0030 local_load32         0 1
        // 0x0038 local_load32         0 2
        // 0x0040 i32_eq
        // 0x0042 nop
        // 0x0044 local_load32         0 1
        // 0x004c local_load32         0 2
        // 0x0054 i32_ne
        // 0x0056 nop
        // 0x0058 local_load32         0 1
        // 0x0060 local_load32         0 1
        // 0x0068 i32_eq
        // 0x006a nop
        // 0x006c local_load32         0 1
        // 0x0074 local_load32         0 1
        // 0x007c i32_ne
        // 0x007e nop
        // 0x0080 local_load32         0 2
        // 0x0088 local_load32         0 3
        // 0x0090 i32_lt_s
        // 0x0092 nop
        // 0x0094 local_load32         0 2
        // 0x009c local_load32         0 3
        // 0x00a4 i32_lt_u
        // 0x00a6 nop
        // 0x00a8 local_load32         0 2
        // 0x00b0 local_load32         0 3
        // 0x00b8 i32_gt_s
        // 0x00ba nop
        // 0x00bc local_load32         0 2
        // 0x00c4 local_load32         0 3
        // 0x00cc i32_gt_u
        // 0x00ce nop
        // 0x00d0 local_load32         0 2
        // 0x00d8 local_load32         0 1
        // 0x00e0 i32_le_s
        // 0x00e2 nop
        // 0x00e4 local_load32         0 2
        // 0x00ec local_load32         0 1
        // 0x00f4 i32_le_u
        // 0x00f6 nop
        // 0x00f8 local_load32         0 1
        // 0x0100 local_load32         0 1
        // 0x0108 i32_le_s
        // 0x010a nop
        // 0x010c local_load32         0 1
        // 0x0114 local_load32         0 1
        // 0x011c i32_le_u
        // 0x011e nop
        // 0x0120 local_load32         0 1
        // 0x0128 local_load32         0 2
        // 0x0130 i32_ge_s
        // 0x0132 nop
        // 0x0134 local_load32         0 1
        // 0x013c local_load32         0 2
        // 0x0144 i32_ge_u
        // 0x0146 nop
        // 0x0148 local_load32         0 1
        // 0x0150 local_load32         0 1
        // 0x0158 i32_ge_s
        // 0x015a nop
        // 0x015c local_load32         0 1
        // 0x0164 local_load32         0 1
        // 0x016c i32_ge_u
        // 0x016e end

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode(Opcode::i32_eqz)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_eqz)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode(Opcode::i32_nez)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_nez)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode(Opcode::i32_eq)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode(Opcode::i32_ne)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_eq)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_ne)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 3)
            .write_opcode(Opcode::i32_lt_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 3)
            .write_opcode(Opcode::i32_lt_u)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 3)
            .write_opcode(Opcode::i32_gt_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 3)
            .write_opcode(Opcode::i32_gt_u)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_le_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_le_u)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_le_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_le_u)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode(Opcode::i32_ge_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode(Opcode::i32_ge_u)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_ge_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_ge_u)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32, DataType::I32, DataType::I32], // params
            vec![
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
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
            &vec![
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(11),
                ForeignValue::UInt32(13),
                ForeignValue::UInt32(-7i32 as u32),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
            ]
        );
    }

    #[test]
    fn test_process_comparison_i64() {
        //

        // numbers:
        //   - 0: 0
        //   - 1: 11
        //   - 2: 13
        //   - 3: -7
        // comparison:
        //   - eqz  0         -> 1
        //   - eqz  11        -> 0
        //   - nez  0         -> 0
        //   - nez  11        -> 1
        //   - eq   11 13     -> 0
        //   - ne   11 13     -> 1
        //   - eq   11 11     -> 1
        //   - ne   11 11     -> 0
        //   - lt_s 13 -7     -> 0
        //   - lt_u 13 -7     -> 1
        //   - gt_s 13 -7     -> 1
        //   - gt_u 13 -7     -> 0
        //   - le_s 13 11     -> 0
        //   - le_u 13 11     -> 0
        //   - le_s 11 11     -> 1
        //   - le_u 11 11     -> 1
        //   - ge_s 11 13     -> 0
        //   - ge_u 11 13     -> 0
        //   - ge_s 11 11     -> 1
        //   - ge_u 11 11     -> 1

        // bytecode
        //
        // 0x0000 local_load           0 0
        // 0x0008 i64_eqz
        // 0x000a nop
        // 0x000c local_load           0 1
        // 0x0014 i64_eqz
        // 0x0016 nop
        // 0x0018 local_load           0 0
        // 0x0020 i64_nez
        // 0x0022 nop
        // 0x0024 local_load           0 1
        // 0x002c i64_nez
        // 0x002e nop
        // 0x0030 local_load           0 1
        // 0x0038 local_load           0 2
        // 0x0040 i64_eq
        // 0x0042 nop
        // 0x0044 local_load           0 1
        // 0x004c local_load           0 2
        // 0x0054 i64_ne
        // 0x0056 nop
        // 0x0058 local_load           0 1
        // 0x0060 local_load           0 1
        // 0x0068 i64_eq
        // 0x006a nop
        // 0x006c local_load           0 1
        // 0x0074 local_load           0 1
        // 0x007c i64_ne
        // 0x007e nop
        // 0x0080 local_load           0 2
        // 0x0088 local_load           0 3
        // 0x0090 i64_lt_s
        // 0x0092 nop
        // 0x0094 local_load           0 2
        // 0x009c local_load           0 3
        // 0x00a4 i64_lt_u
        // 0x00a6 nop
        // 0x00a8 local_load           0 2
        // 0x00b0 local_load           0 3
        // 0x00b8 i64_gt_s
        // 0x00ba nop
        // 0x00bc local_load           0 2
        // 0x00c4 local_load           0 3
        // 0x00cc i64_gt_u
        // 0x00ce nop
        // 0x00d0 local_load           0 2
        // 0x00d8 local_load           0 1
        // 0x00e0 i64_le_s
        // 0x00e2 nop
        // 0x00e4 local_load           0 2
        // 0x00ec local_load           0 1
        // 0x00f4 i64_le_u
        // 0x00f6 nop
        // 0x00f8 local_load           0 1
        // 0x0100 local_load           0 1
        // 0x0108 i64_le_s
        // 0x010a nop
        // 0x010c local_load           0 1
        // 0x0114 local_load           0 1
        // 0x011c i64_le_u
        // 0x011e nop
        // 0x0120 local_load           0 1
        // 0x0128 local_load           0 2
        // 0x0130 i64_ge_s
        // 0x0132 nop
        // 0x0134 local_load           0 1
        // 0x013c local_load           0 2
        // 0x0144 i64_ge_u
        // 0x0146 nop
        // 0x0148 local_load           0 1
        // 0x0150 local_load           0 1
        // 0x0158 i64_ge_s
        // 0x015a nop
        // 0x015c local_load           0 1
        // 0x0164 local_load           0 1
        // 0x016c i64_ge_u
        // 0x016e end

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 0)
            .write_opcode(Opcode::i64_eqz)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::i64_eqz)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 0)
            .write_opcode(Opcode::i64_nez)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::i64_nez)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode(Opcode::i64_eq)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode(Opcode::i64_ne)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::i64_eq)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::i64_ne)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 3)
            .write_opcode(Opcode::i64_lt_s)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 3)
            .write_opcode(Opcode::i64_lt_u)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 3)
            .write_opcode(Opcode::i64_gt_s)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 3)
            .write_opcode(Opcode::i64_gt_u)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::i64_le_s)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::i64_le_u)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::i64_le_s)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::i64_le_u)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode(Opcode::i64_ge_s)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode(Opcode::i64_ge_u)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::i64_ge_s)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::i64_ge_u)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I64, DataType::I64, DataType::I64, DataType::I64], // params
            vec![
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
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
            &vec![
                ForeignValue::UInt64(0),
                ForeignValue::UInt64(11),
                ForeignValue::UInt64(13),
                ForeignValue::UInt64(-7i64 as u64),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
            ]
        );
    }

    #[test]
    fn test_process_comparison_f32() {
        //

        // numbers:
        //   - 0: 1.414
        //   - 1: 1.732
        // comparison:
        //   - eq 0 1         -> 0
        //   - ne 0 1         -> 1
        //   - eq 0 0         -> 1
        //   - ne 0 0         -> 0
        //   - lt 0 1         -> 1
        //   - lt 1 0         -> 0
        //   - lt 0 0         -> 0
        //   - gt 0 1         -> 0
        //   - gt 1 0         -> 1
        //   - gt 0 0         -> 0
        //   - le 1 0         -> 0
        //   - le 0 0         -> 1
        //   - ge 0 1         -> 0
        //   - ge 0 0         -> 1

        // bytecode
        //
        // 0x0000 local_load32_f32     0 0
        // 0x0008 local_load32_f32     0 1
        // 0x0010 f32_eq
        // 0x0012 nop
        // 0x0014 local_load32_f32     0 0
        // 0x001c local_load32_f32     0 1
        // 0x0024 f32_ne
        // 0x0026 nop
        // 0x0028 local_load32_f32     0 0
        // 0x0030 local_load32_f32     0 0
        // 0x0038 f32_eq
        // 0x003a nop
        // 0x003c local_load32_f32     0 0
        // 0x0044 local_load32_f32     0 0
        // 0x004c f32_ne
        // 0x004e nop
        // 0x0050 local_load32_f32     0 0
        // 0x0058 local_load32_f32     0 1
        // 0x0060 f32_lt
        // 0x0062 nop
        // 0x0064 local_load32_f32     0 1
        // 0x006c local_load32_f32     0 0
        // 0x0074 f32_lt
        // 0x0076 nop
        // 0x0078 local_load32_f32     0 0
        // 0x0080 local_load32_f32     0 0
        // 0x0088 f32_lt
        // 0x008a nop
        // 0x008c local_load32_f32     0 0
        // 0x0094 local_load32_f32     0 1
        // 0x009c f32_gt
        // 0x009e nop
        // 0x00a0 local_load32_f32     0 1
        // 0x00a8 local_load32_f32     0 0
        // 0x00b0 f32_gt
        // 0x00b2 nop
        // 0x00b4 local_load32_f32     0 0
        // 0x00bc local_load32_f32     0 0
        // 0x00c4 f32_gt
        // 0x00c6 nop
        // 0x00c8 local_load32_f32     0 1
        // 0x00d0 local_load32_f32     0 0
        // 0x00d8 f32_le
        // 0x00da nop
        // 0x00dc local_load32_f32     0 0
        // 0x00e4 local_load32_f32     0 0
        // 0x00ec f32_le
        // 0x00ee nop
        // 0x00f0 local_load32_f32     0 0
        // 0x00f8 local_load32_f32     0 1
        // 0x0100 f32_ge
        // 0x0102 nop
        // 0x0104 local_load32_f32     0 0
        // 0x010c local_load32_f32     0 0
        // 0x0114 f32_ge
        // 0x0116 end

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .write_opcode(Opcode::f32_eq)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .write_opcode(Opcode::f32_ne)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode(Opcode::f32_eq)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode(Opcode::f32_ne)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .write_opcode(Opcode::f32_lt)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode(Opcode::f32_lt)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode(Opcode::f32_lt)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .write_opcode(Opcode::f32_gt)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode(Opcode::f32_gt)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode(Opcode::f32_gt)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode(Opcode::f32_le)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode(Opcode::f32_le)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .write_opcode(Opcode::f32_ge)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode(Opcode::f32_ge)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::F32, DataType::F32], // params
            vec![
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
            ], // results
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
            &vec![
                ForeignValue::Float32(1.414f32),
                ForeignValue::Float32(1.732f32),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
            ]
        );
    }

    #[test]
    fn test_process_comparison_f64() {
        //

        // numbers:
        //   - 0: 1.414
        //   - 1: 1.732
        // comparison:
        //   - eq 0 1         -> 0
        //   - ne 0 1         -> 1
        //   - eq 0 0         -> 1
        //   - ne 0 0         -> 0
        //   - lt 0 1         -> 1
        //   - lt 1 0         -> 0
        //   - lt 0 0         -> 0
        //   - gt 0 1         -> 0
        //   - gt 1 0         -> 1
        //   - gt 0 0         -> 0
        //   - le 1 0         -> 0
        //   - le 0 0         -> 1
        //   - ge 0 1         -> 0
        //   - ge 0 0         -> 1

        // bytecode
        //
        // 0x0000 local_load_f64       0 0
        // 0x0008 local_load_f64       0 1
        // 0x0010 f64_eq
        // 0x0012 nop
        // 0x0014 local_load_f64       0 0
        // 0x001c local_load_f64       0 1
        // 0x0024 f64_ne
        // 0x0026 nop
        // 0x0028 local_load_f64       0 0
        // 0x0030 local_load_f64       0 0
        // 0x0038 f64_eq
        // 0x003a nop
        // 0x003c local_load_f64       0 0
        // 0x0044 local_load_f64       0 0
        // 0x004c f64_ne
        // 0x004e nop
        // 0x0050 local_load_f64       0 0
        // 0x0058 local_load_f64       0 1
        // 0x0060 f64_lt
        // 0x0062 nop
        // 0x0064 local_load_f64       0 1
        // 0x006c local_load_f64       0 0
        // 0x0074 f64_lt
        // 0x0076 nop
        // 0x0078 local_load_f64       0 0
        // 0x0080 local_load_f64       0 0
        // 0x0088 f64_lt
        // 0x008a nop
        // 0x008c local_load_f64       0 0
        // 0x0094 local_load_f64       0 1
        // 0x009c f64_gt
        // 0x009e nop
        // 0x00a0 local_load_f64       0 1
        // 0x00a8 local_load_f64       0 0
        // 0x00b0 f64_gt
        // 0x00b2 nop
        // 0x00b4 local_load_f64       0 0
        // 0x00bc local_load_f64       0 0
        // 0x00c4 f64_gt
        // 0x00c6 nop
        // 0x00c8 local_load_f64       0 1
        // 0x00d0 local_load_f64       0 0
        // 0x00d8 f64_le
        // 0x00da nop
        // 0x00dc local_load_f64       0 0
        // 0x00e4 local_load_f64       0 0
        // 0x00ec f64_le
        // 0x00ee nop
        // 0x00f0 local_load_f64       0 0
        // 0x00f8 local_load_f64       0 1
        // 0x0100 f64_ge
        // 0x0102 nop
        // 0x0104 local_load_f64       0 0
        // 0x010c local_load_f64       0 0
        // 0x0114 f64_ge
        // 0x0116 end

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .write_opcode(Opcode::f64_eq)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .write_opcode(Opcode::f64_ne)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode(Opcode::f64_eq)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode(Opcode::f64_ne)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .write_opcode(Opcode::f64_lt)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode(Opcode::f64_lt)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode(Opcode::f64_lt)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .write_opcode(Opcode::f64_gt)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode(Opcode::f64_gt)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode(Opcode::f64_gt)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode(Opcode::f64_le)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode(Opcode::f64_le)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .write_opcode(Opcode::f64_ge)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode(Opcode::f64_ge)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code0).to_text());

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::F64, DataType::F64], // params
            vec![
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
            ], // results
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
            &vec![
                ForeignValue::Float32(1.414f32),
                ForeignValue::Float32(1.732f32),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
            ]
        );
    }
}
