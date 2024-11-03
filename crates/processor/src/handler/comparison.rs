// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::thread_context::ThreadContext;

use super::HandleResult;

pub fn eqz_i32(thread_context: &mut ThreadContext) -> HandleResult {
    let value = thread_context.stack.pop_i32_u();
    store_bool(thread_context, value == 0);
    HandleResult::Move(2)
}

pub fn nez_i32(thread_context: &mut ThreadContext) -> HandleResult {
    let value = thread_context.stack.pop_i32_u();
    store_bool(thread_context, value != 0);
    HandleResult::Move(2)
}

pub fn eq_i32(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_bool(thread_context, left == right);
    HandleResult::Move(2)
}

pub fn ne_i32(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_bool(thread_context, left != right);
    HandleResult::Move(2)
}

pub fn lt_i32_s(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_s(thread_context);
    store_bool(thread_context, left < right);
    HandleResult::Move(2)
}

pub fn lt_i32_u(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_bool(thread_context, left < right);
    HandleResult::Move(2)
}

pub fn gt_i32_s(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_s(thread_context);
    store_bool(thread_context, left > right);
    HandleResult::Move(2)
}

pub fn gt_i32_u(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_bool(thread_context, left > right);
    HandleResult::Move(2)
}

pub fn le_i32_s(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_s(thread_context);
    store_bool(thread_context, left <= right);
    HandleResult::Move(2)
}

pub fn le_i32_u(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_bool(thread_context, left <= right);
    HandleResult::Move(2)
}

pub fn ge_i32_s(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_s(thread_context);
    store_bool(thread_context, left >= right);
    HandleResult::Move(2)
}

pub fn ge_i32_u(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_bool(thread_context, left >= right);
    HandleResult::Move(2)
}

pub fn eqz_i64(thread_context: &mut ThreadContext) -> HandleResult {
    let value = thread_context.stack.pop_i64_u();
    store_bool(thread_context, value == 0);
    HandleResult::Move(2)
}

pub fn nez_i64(thread_context: &mut ThreadContext) -> HandleResult {
    let value = thread_context.stack.pop_i64_u();
    store_bool(thread_context, value != 0);
    HandleResult::Move(2)
}

pub fn eq_i64(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_bool(thread_context, left == right);
    HandleResult::Move(2)
}

pub fn ne_i64(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_bool(thread_context, left != right);
    HandleResult::Move(2)
}

pub fn lt_i64_s(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_s(thread_context);
    store_bool(thread_context, left < right);
    HandleResult::Move(2)
}

pub fn lt_i64_u(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_bool(thread_context, left < right);
    HandleResult::Move(2)
}

pub fn gt_i64_s(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_s(thread_context);
    store_bool(thread_context, left > right);
    HandleResult::Move(2)
}

pub fn gt_i64_u(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_bool(thread_context, left > right);
    HandleResult::Move(2)
}

pub fn le_i64_s(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_s(thread_context);
    store_bool(thread_context, left <= right);
    HandleResult::Move(2)
}

pub fn le_i64_u(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_bool(thread_context, left <= right);
    HandleResult::Move(2)
}

pub fn ge_i64_s(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_s(thread_context);
    store_bool(thread_context, left >= right);
    HandleResult::Move(2)
}

pub fn ge_i64_u(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_bool(thread_context, left >= right);
    HandleResult::Move(2)
}

pub fn eq_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f32(thread_context);
    store_bool(thread_context, left == right);
    HandleResult::Move(2)
}

pub fn ne_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f32(thread_context);
    store_bool(thread_context, left != right);
    HandleResult::Move(2)
}

pub fn lt_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f32(thread_context);
    store_bool(thread_context, left < right);
    HandleResult::Move(2)
}

pub fn gt_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f32(thread_context);
    store_bool(thread_context, left > right);
    HandleResult::Move(2)
}

pub fn le_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f32(thread_context);
    store_bool(thread_context, left <= right);
    HandleResult::Move(2)
}

pub fn ge_f32(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f32(thread_context);
    store_bool(thread_context, left >= right);
    HandleResult::Move(2)
}

pub fn eq_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f64(thread_context);
    store_bool(thread_context, left == right);
    HandleResult::Move(2)
}

pub fn ne_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f64(thread_context);
    store_bool(thread_context, left != right);
    HandleResult::Move(2)
}

pub fn lt_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f64(thread_context);
    store_bool(thread_context, left < right);
    HandleResult::Move(2)
}

pub fn gt_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f64(thread_context);
    store_bool(thread_context, left > right);
    HandleResult::Move(2)
}

pub fn le_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f64(thread_context);
    store_bool(thread_context, left <= right);
    HandleResult::Move(2)
}

pub fn ge_f64(thread_context: &mut ThreadContext) -> HandleResult {
    let (left, right) = load_operands_f64(thread_context);
    store_bool(thread_context, left >= right);
    HandleResult::Move(2)
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

        let code0 = BytecodeWriterHelper::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 0)
            .append_opcode(Opcode::eqz_i32)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1)
            .append_opcode(Opcode::eqz_i32)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 0)
            .append_opcode(Opcode::nez_i32)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1)
            .append_opcode(Opcode::nez_i32)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 2)
            .append_opcode(Opcode::eq_i32)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 2)
            .append_opcode(Opcode::ne_i32)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1)
            .append_opcode(Opcode::eq_i32)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1)
            .append_opcode(Opcode::ne_i32)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 0, 3)
            .append_opcode(Opcode::lt_i32_s)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 3)
            .append_opcode(Opcode::lt_i32_u)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 0, 3)
            .append_opcode(Opcode::gt_i32_s)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 3)
            .append_opcode(Opcode::gt_i32_u)
            // group 3
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 0, 1)
            .append_opcode(Opcode::le_i32_s)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1)
            .append_opcode(Opcode::le_i32_u)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 0, 1)
            .append_opcode(Opcode::le_i32_s)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1)
            .append_opcode(Opcode::le_i32_u)
            // group 4
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 0, 2)
            .append_opcode(Opcode::ge_i32_s)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 2)
            .append_opcode(Opcode::ge_i32_u)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 0, 1)
            .append_opcode(Opcode::ge_i32_s)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1)
            .append_opcode(Opcode::ge_i32_u)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
            ], // params
            vec![
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
                // group 4
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
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
                ForeignValue::U32(0),
                ForeignValue::U32(11),
                ForeignValue::U32(13),
                ForeignValue::U32(-7i32 as u32),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::U32(1),
                ForeignValue::U32(0),
                ForeignValue::U32(0),
                ForeignValue::U32(1),
                // group 1
                ForeignValue::U32(0),
                ForeignValue::U32(1),
                ForeignValue::U32(1),
                ForeignValue::U32(0),
                // group 2
                ForeignValue::U32(0),
                ForeignValue::U32(1),
                ForeignValue::U32(1),
                ForeignValue::U32(0),
                // group 3
                ForeignValue::U32(0),
                ForeignValue::U32(0),
                ForeignValue::U32(1),
                ForeignValue::U32(1),
                // group 4
                ForeignValue::U32(0),
                ForeignValue::U32(0),
                ForeignValue::U32(1),
                ForeignValue::U32(1),
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

        let code0 = BytecodeWriterHelper::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0)
            .append_opcode(Opcode::eqz_i64)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode(Opcode::eqz_i64)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0)
            .append_opcode(Opcode::nez_i64)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode(Opcode::nez_i64)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode(Opcode::eq_i64)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode(Opcode::ne_i64)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode(Opcode::eq_i64)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode(Opcode::ne_i64)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 3)
            .append_opcode(Opcode::lt_i64_s)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 3)
            .append_opcode(Opcode::lt_i64_u)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 3)
            .append_opcode(Opcode::gt_i64_s)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 3)
            .append_opcode(Opcode::gt_i64_u)
            // group 3
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode(Opcode::le_i64_s)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode(Opcode::le_i64_u)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode(Opcode::le_i64_s)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode(Opcode::le_i64_u)
            // group 4
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode(Opcode::ge_i64_s)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode(Opcode::ge_i64_u)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode(Opcode::ge_i64_s)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode(Opcode::ge_i64_u)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
            ], // params
            vec![
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
                // group 4
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
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
                ForeignValue::U64(0),
                ForeignValue::U64(11),
                ForeignValue::U64(13),
                ForeignValue::U64(-7i64 as u64),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::U32(1),
                ForeignValue::U32(0),
                ForeignValue::U32(0),
                ForeignValue::U32(1),
                // group 1
                ForeignValue::U32(0),
                ForeignValue::U32(1),
                ForeignValue::U32(1),
                ForeignValue::U32(0),
                // group 2
                ForeignValue::U32(0),
                ForeignValue::U32(1),
                ForeignValue::U32(1),
                ForeignValue::U32(0),
                // group 3
                ForeignValue::U32(0),
                ForeignValue::U32(0),
                ForeignValue::U32(1),
                ForeignValue::U32(1),
                // group 4
                ForeignValue::U32(0),
                ForeignValue::U32(0),
                ForeignValue::U32(1),
                ForeignValue::U32(1),
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

        let code0 = BytecodeWriterHelper::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 1)
            .append_opcode(Opcode::eq_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 1)
            .append_opcode(Opcode::ne_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode(Opcode::eq_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode(Opcode::ne_f32)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 1)
            .append_opcode(Opcode::lt_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode(Opcode::lt_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode(Opcode::lt_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 1)
            .append_opcode(Opcode::gt_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode(Opcode::gt_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode(Opcode::gt_f32)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode(Opcode::le_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode(Opcode::le_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 1)
            .append_opcode(Opcode::ge_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode(Opcode::ge_f32)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![OperandDataType::F32, OperandDataType::F32], // params
            vec![
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
            &[ForeignValue::F32(1.414f32), ForeignValue::F32(1.732f32)],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::U32(0),
                ForeignValue::U32(1),
                ForeignValue::U32(1),
                ForeignValue::U32(0),
                // group 1
                ForeignValue::U32(1),
                ForeignValue::U32(0),
                ForeignValue::U32(0),
                ForeignValue::U32(0),
                ForeignValue::U32(1),
                ForeignValue::U32(0),
                // group 2
                ForeignValue::U32(0),
                ForeignValue::U32(1),
                ForeignValue::U32(0),
                ForeignValue::U32(1),
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

        let code0 = BytecodeWriterHelper::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode(Opcode::eq_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode(Opcode::ne_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode(Opcode::eq_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode(Opcode::ne_f64)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode(Opcode::lt_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode(Opcode::lt_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode(Opcode::lt_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode(Opcode::gt_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode(Opcode::gt_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode(Opcode::gt_f64)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode(Opcode::le_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode(Opcode::le_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode(Opcode::ge_f64)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .append_opcode(Opcode::ge_f64)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![OperandDataType::F64, OperandDataType::F64], // params
            vec![
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
            &[ForeignValue::F64(1.414f64), ForeignValue::F64(1.732f64)],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::U32(0),
                ForeignValue::U32(1),
                ForeignValue::U32(1),
                ForeignValue::U32(0),
                // group 1
                ForeignValue::U32(1),
                ForeignValue::U32(0),
                ForeignValue::U32(0),
                ForeignValue::U32(0),
                ForeignValue::U32(1),
                ForeignValue::U32(0),
                // group 2
                ForeignValue::U32(0),
                ForeignValue::U32(1),
                ForeignValue::U32(0),
                ForeignValue::U32(1),
            ]
        );
    }
}
