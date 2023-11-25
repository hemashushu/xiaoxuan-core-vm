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
    use ancvm_binary::{
        bytecode_writer::BytecodeWriter, utils::helper_build_module_binary_with_single_function,
    };
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    #[test]
    fn test_interpreter_comparison_i32() {
        // numbers:
        //   - 0: 0
        //   - 1: 11
        //   - 2: 13
        //   - 3: -7
        // comparison:
        //   group 0:
        //   - eqz  0         -> 1
        //   - eqz  1         -> 0
        //   - nez  0         -> 0
        //   - nez  1         -> 1
        //
        //   group 1:
        //   - eq   1  2      -> 0
        //   - ne   1  2      -> 1
        //   - eq   1  1      -> 1
        //   - ne   1  1      -> 0
        //
        //   group 2:
        //   - lt_s 2  3      -> 0
        //   - lt_u 2  3      -> 1
        //   - gt_s 2  3      -> 1
        //   - gt_u 2  3      -> 0
        //
        //   group 3:
        //   - le_s 2  1      -> 0
        //   - le_u 2  1      -> 0
        //   - le_s 1  1      -> 1
        //   - le_u 1  1      -> 1
        //
        //   group 4:
        //   - ge_s 1  2      -> 0
        //   - ge_u 1  2      -> 0
        //   - ge_s 1  1      -> 1
        //   - ge_u 1  1      -> 1
        //
        // (i32 i32 i32 i32) -> (i32 i32 i32 i32  i32 i32 i32 i32  i32 i32 i32 i32  i32 i32 i32 i32)

        let code0 = BytecodeWriter::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode(Opcode::i32_eqz)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_eqz)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode(Opcode::i32_nez)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_nez)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode(Opcode::i32_eq)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode(Opcode::i32_ne)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_eq)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_ne)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 3)
            .append_opcode(Opcode::i32_lt_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 3)
            .append_opcode(Opcode::i32_lt_u)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 3)
            .append_opcode(Opcode::i32_gt_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 3)
            .append_opcode(Opcode::i32_gt_u)
            // group 3
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_le_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_le_u)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_le_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_le_u)
            // group 4
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode(Opcode::i32_ge_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode(Opcode::i32_ge_u)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_ge_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_ge_u)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32, DataType::I32, DataType::I32], // params
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
                // group 4
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
            &[
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(11),
                ForeignValue::UInt32(13),
                ForeignValue::UInt32(-7i32 as u32),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                // group 1
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                // group 2
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                // group 3
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
                // group 4
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
            ]
        );
    }

    #[test]
    fn test_interpreter_comparison_i64() {
        // numbers:
        //   - 0: 0
        //   - 1: 11
        //   - 2: 13
        //   - 3: -7
        // comparison:
        //   group 0:
        //   - eqz  0         -> 1
        //   - eqz  1         -> 0
        //   - nez  0         -> 0
        //   - nez  1         -> 1
        //
        //   group 1:
        //   - eq   1  2      -> 0
        //   - ne   1  2      -> 1
        //   - eq   1  1      -> 1
        //   - ne   1  1      -> 0
        //
        //   group 2:
        //   - lt_s 2  3      -> 0
        //   - lt_u 2  3      -> 1
        //   - gt_s 2  3      -> 1
        //   - gt_u 2  3      -> 0
        //
        //   group 3:
        //   - le_s 2  1      -> 0
        //   - le_u 2  1      -> 0
        //   - le_s 1  1      -> 1
        //   - le_u 1  1      -> 1
        //
        //   group 4:
        //   - ge_s 1  2      -> 0
        //   - ge_u 1  2      -> 0
        //   - ge_s 1  1      -> 1
        //   - ge_u 1  1      -> 1
        //
        // (i64 i64 i64 i64) -> (i32 i32 i32 i32  i32 i32 i32 i32  i32 i32 i32 i32  i32 i32 i32 i32)

        let code0 = BytecodeWriter::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 0)
            .append_opcode(Opcode::i64_eqz)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_eqz)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 0)
            .append_opcode(Opcode::i64_nez)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_nez)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode(Opcode::i64_eq)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode(Opcode::i64_ne)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_eq)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_ne)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 3)
            .append_opcode(Opcode::i64_lt_s)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 3)
            .append_opcode(Opcode::i64_lt_u)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 3)
            .append_opcode(Opcode::i64_gt_s)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 3)
            .append_opcode(Opcode::i64_gt_u)
            // group 3
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_le_s)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_le_u)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_le_s)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_le_u)
            // group 4
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode(Opcode::i64_ge_s)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 2)
            .append_opcode(Opcode::i64_ge_u)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_ge_s)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::i64_ge_u)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![DataType::I64, DataType::I64, DataType::I64, DataType::I64], // params
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
                // group 4
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
            &[
                ForeignValue::UInt64(0),
                ForeignValue::UInt64(11),
                ForeignValue::UInt64(13),
                ForeignValue::UInt64(-7i64 as u64),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                // group 1
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                // group 2
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                // group 3
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
                // group 4
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
            ]
        );
    }

    #[test]
    fn test_interpreter_comparison_f32() {
        // numbers:
        //   - 0: 1.414
        //   - 1: 1.732
        // comparison:
        //   group 0:
        //   - eq 0  1        -> 0
        //   - ne 0  1        -> 1
        //   - eq 0  0        -> 1
        //   - ne 0  0        -> 0
        //
        //   group 1:
        //   - lt 0  1        -> 1
        //   - lt 1  0        -> 0
        //   - lt 0  0        -> 0
        //   - gt 0  1        -> 0
        //   - gt 1  0        -> 1
        //   - gt 0  0        -> 0
        //
        //   group 2:
        //   - le 1  0        -> 0
        //   - le 0  0        -> 1
        //   - ge 0  1        -> 0
        //   - ge 0  0        -> 1
        //
        // (f32 f32) -> (i32 i32 i32 i32  i32 i32 i32 i32 i32 i32  i32 i32 i32 i32)

        let code0 = BytecodeWriter::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode(Opcode::f32_eq)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode(Opcode::f32_ne)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_eq)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_ne)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode(Opcode::f32_lt)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_lt)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_lt)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode(Opcode::f32_gt)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_gt)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_gt)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_le)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_le)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode(Opcode::f32_ge)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::f32_ge)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![DataType::F32, DataType::F32], // params
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
            &[
                ForeignValue::Float32(1.414f32),
                ForeignValue::Float32(1.732f32),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                // group 1
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                // group 2
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
            ]
        );
    }

    #[test]
    fn test_interpreter_comparison_f64() {
        // numbers:
        //   - 0: 1.414
        //   - 1: 1.732
        // comparison:
        //   group 0:
        //   - eq 0  1        -> 0
        //   - ne 0  1        -> 1
        //   - eq 0  0        -> 1
        //   - ne 0  0        -> 0
        //
        //   group 1:
        //   - lt 0  1        -> 1
        //   - lt 1  0        -> 0
        //   - lt 0  0        -> 0
        //   - gt 0  1        -> 0
        //   - gt 1  0        -> 1
        //   - gt 0  0        -> 0
        //
        //   group 2:
        //   - le 1  0        -> 0
        //   - le 0  0        -> 1
        //   - ge 0  1        -> 0
        //   - ge 0  0        -> 1
        //
        // (f64 f64) -> (i32 i32 i32 i32  i32 i32 i32 i32 i32 i32  i32 i32 i32 i32)

        let code0 = BytecodeWriter::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode(Opcode::f64_eq)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode(Opcode::f64_ne)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_eq)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_ne)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode(Opcode::f64_lt)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_lt)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_lt)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode(Opcode::f64_gt)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_gt)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_gt)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_le)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_le)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode(Opcode::f64_ge)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode(Opcode::f64_ge)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![DataType::F64, DataType::F64], // params
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
            &[
                ForeignValue::Float64(1.414f64),
                ForeignValue::Float64(1.732f64),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                // group 1
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                // group 2
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
                ForeignValue::UInt32(0),
                ForeignValue::UInt32(1),
            ]
        );
    }
}
