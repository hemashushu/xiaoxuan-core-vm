// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_program::thread_context::ThreadContext;

use super::InterpretResult;

pub fn i32_add(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left + right);
    InterpretResult::Move(2)
}

pub fn i32_sub(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left - right);
    InterpretResult::Move(2)
}

pub fn i32_mul(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread_context);
    store_i32_u(thread_context, left * right);
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

pub fn i32_add_imm(thread_context: &mut ThreadContext) -> InterpretResult {
    let amount = thread_context.get_param_i16();
    let value = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, value + amount as u32);
    InterpretResult::Move(4)
}

pub fn i32_sub_imm(thread_context: &mut ThreadContext) -> InterpretResult {
    let amount = thread_context.get_param_i16();
    let value = load_operand_i32_u(thread_context);
    store_i32_u(thread_context, value - amount as u32);
    InterpretResult::Move(4)
}

pub fn i64_add(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left + right);
    InterpretResult::Move(2)
}

pub fn i64_sub(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left - right);
    InterpretResult::Move(2)
}

pub fn i64_mul(thread_context: &mut ThreadContext) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread_context);
    store_i64_u(thread_context, left * right);
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

pub fn i64_add_imm(thread_context: &mut ThreadContext) -> InterpretResult {
    let amount = thread_context.get_param_i16();
    let value = load_operand_i64_u(thread_context);
    store_i64_u(thread_context, value + amount as u64);
    InterpretResult::Move(4)
}

pub fn i64_sub_imm(thread_context: &mut ThreadContext) -> InterpretResult {
    let amount = thread_context.get_param_i16();
    let value = load_operand_i64_u(thread_context);
    store_i64_u(thread_context, value - amount as u64);
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
    use ancvm_binary::utils::{build_module_binary_with_single_function, BytecodeWriter};
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    #[test]
    fn test_process_arithmetic_i32() {
        // numbers:
        //   - 0: 11
        //   - 1: 211
        //   - 2: -13

        // arithemtic:
        //   - add   0 1      -> 222
        //   - sub   1 0      -> 200
        //   - mul   0 1      -> 2321
        //   - div_s 1 2      -> -16
        //   - div_u 2 1      -> 20355295 (= 4294967283/211)
        //   - rem_s 1 2      -> 3
        //   - rem_u 2 1      -> 38
        //
        //   - inc   0 3      -> 14
        //   - dec   0 3      -> 8
        //   - inc   2 3      -> -10
        //   - dec   2 3      -> -16

        // note of the 'remainder':
        // (211 % -13) = 3
        //  ^      ^
        //  |      |divisor
        //  |dividend <--------- the result always takes the sign of the dividend.

        // bytecode
        //
        // 0x0000 local_load32         0 0 0
        // 0x0008 local_load32         0 0 1
        // 0x0010 i32_add
        // 0x0012 local_load32         0 0 1
        // 0x001a local_load32         0 0 0
        // 0x0022 i32_sub
        // 0x0024 local_load32         0 0 0
        // 0x002c local_load32         0 0 1
        // 0x0034 i32_mul
        // 0x0036 local_load32         0 0 1
        // 0x003e local_load32         0 0 2
        // 0x0046 i32_div_s
        // 0x0048 local_load32         0 0 2
        // 0x0050 local_load32         0 0 1
        // 0x0058 i32_div_u
        // 0x005a local_load32         0 0 1
        // 0x0062 local_load32         0 0 2
        // 0x006a i32_rem_s
        // 0x006c local_load32         0 0 2
        // 0x0074 local_load32         0 0 1
        // 0x007c i32_rem_u
        // 0x007e local_load32         0 0 0
        // 0x0086 i32_add_imm              3
        // 0x008a local_load32         0 0 0
        // 0x0092 i32_sub_imm              3
        // 0x0096 local_load32         0 0 2
        // 0x009e i32_add_imm              3
        // 0x00a2 local_load32         0 0 2
        // 0x00aa i32_sub_imm              3
        // 0x00ae end

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_add)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode(Opcode::i32_sub)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_mul)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode(Opcode::i32_div_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_div_u)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode(Opcode::i32_rem_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_rem_u)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode_i16(Opcode::i32_add_imm, 3)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode_i16(Opcode::i32_sub_imm, 3)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode_i16(Opcode::i32_add_imm, 3)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode_i16(Opcode::i32_sub_imm, 3)
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code0).to_text());

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32, DataType::I32], // params
            vec![
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                //
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
        let mut thread_context0 = program0.new_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &vec![
                ForeignValue::UInt32(11),
                ForeignValue::UInt32(211),
                ForeignValue::UInt32(-13i32 as u32),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(222),
                ForeignValue::UInt32(200),
                ForeignValue::UInt32(2321),
                ForeignValue::UInt32(-16i32 as u32),
                ForeignValue::UInt32(20355295),
                ForeignValue::UInt32(3),
                ForeignValue::UInt32(38),
                //
                ForeignValue::UInt32(14),
                ForeignValue::UInt32(8),
                ForeignValue::UInt32(-10i32 as u32),
                ForeignValue::UInt32(-16i32 as u32),
            ]
        );
    }

    #[test]
    fn test_process_arithmetic_i64() {
        //

        // numbers:
        //   - 0: 11
        //   - 1: 211
        //   - 2: -13

        // arithemtic:
        //   - add   0 1      -> 222
        //   - sub   1 0      -> 200
        //   - mul   0 1      -> 2321
        //   - div_s 1 2      -> -16
        //   - div_u 2 1      -> 87425327363552377 (= 18446744073709551603/211)
        //   - rem_s 1 2      -> 3
        //   - rem_u 2 1      -> 56
        //
        //   - inc   0 3      -> 14
        //   - dec   0 3      -> 8
        //   - inc   2 3      -> -10
        //   - dec   2 3      -> -16

        // note of the 'remainder':
        // (211 % -13) = 3
        //  ^      ^
        //  |      |divisor
        //  |dividend <--------- the result always takes the sign of the dividend.

        // bytecode
        //
        // 0x0000 local_load           0 0 0
        // 0x0008 local_load           0 0 1
        // 0x0010 i64_add
        // 0x0012 local_load           0 0 1
        // 0x001a local_load           0 0 0
        // 0x0022 i64_sub
        // 0x0024 local_load           0 0 0
        // 0x002c local_load           0 0 1
        // 0x0034 i64_mul
        // 0x0036 local_load           0 0 1
        // 0x003e local_load           0 0 2
        // 0x0046 i64_div_s
        // 0x0048 local_load           0 0 2
        // 0x0050 local_load           0 0 1
        // 0x0058 i64_div_u
        // 0x005a local_load           0 0 1
        // 0x0062 local_load           0 0 2
        // 0x006a i64_rem_s
        // 0x006c local_load           0 0 2
        // 0x0074 local_load           0 0 1
        // 0x007c i64_rem_u
        // 0x007e local_load           0 0 0
        // 0x0086 i64_add_imm              3
        // 0x008a local_load           0 0 0
        // 0x0092 i64_sub_imm              3
        // 0x0096 local_load           0 0 2
        // 0x009e i64_add_imm              3
        // 0x00a2 local_load           0 0 2
        // 0x00aa i64_sub_imm              3
        // 0x00ae end

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::i64_add)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 0)
            .write_opcode(Opcode::i64_sub)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::i64_mul)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode(Opcode::i64_div_s)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::i64_div_u)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode(Opcode::i64_rem_s)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::i64_rem_u)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 0)
            .write_opcode_i16(Opcode::i64_add_imm, 3)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 0)
            .write_opcode_i16(Opcode::i64_sub_imm, 3)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode_i16(Opcode::i64_add_imm, 3)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 2)
            .write_opcode_i16(Opcode::i64_sub_imm, 3)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code0).to_text());

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I64, DataType::I64, DataType::I64], // params
            vec![
                DataType::I64,
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
            ], // results
            vec![],                                            // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.new_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &vec![
                ForeignValue::UInt64(11),
                ForeignValue::UInt64(211),
                ForeignValue::UInt64(-13i64 as u64),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt64(222),
                ForeignValue::UInt64(200),
                ForeignValue::UInt64(2321),
                ForeignValue::UInt64(-16i64 as u64),
                ForeignValue::UInt64(87425327363552377),
                ForeignValue::UInt64(3),
                ForeignValue::UInt64(56),
                //
                ForeignValue::UInt64(14),
                ForeignValue::UInt64(8),
                ForeignValue::UInt64(-10i64 as u64),
                ForeignValue::UInt64(-16i64 as u64),
            ]
        );
    }

    #[test]
    fn test_process_arithmetic_f32() {
        //

        // numbers:
        //   - 0: 1.414
        //   - 1: 4.123

        // arithemtic:
        //   - add 0 1      -> 5.537
        //   - sub 1 0      -> 2.709
        //   - mul 0 1      -> 5.829922
        //   - div 1 0      -> 2.91584158416

        // bytecode
        //
        // 0x0000 local_load32_f32     0 0 0
        // 0x0008 local_load32_f32     0 0 1
        // 0x0010 f32_add
        // 0x0012 local_load32_f32     0 0 1
        // 0x001a local_load32_f32     0 0 0
        // 0x0022 f32_sub
        // 0x0024 local_load32_f32     0 0 0
        // 0x002c local_load32_f32     0 0 1
        // 0x0034 f32_mul
        // 0x0036 local_load32_f32     0 0 1
        // 0x003e local_load32_f32     0 0 0
        // 0x0046 f32_div
        // 0x0048 end

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .write_opcode(Opcode::f32_add)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode(Opcode::f32_sub)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .write_opcode(Opcode::f32_mul)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode(Opcode::f32_div)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code0).to_text());

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::F32, DataType::F32], // params
            vec![DataType::F32, DataType::F32, DataType::F32, DataType::F32], // results
            vec![],                             // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.new_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &vec![ForeignValue::Float32(1.414), ForeignValue::Float32(4.123)],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::Float32(5.537),
                ForeignValue::Float32(2.709),
                ForeignValue::Float32(5.829922),
                ForeignValue::Float32(2.91584158416),
            ]
        );
    }

    #[test]
    fn test_process_arithmetic_f64() {
        //

        // numbers:
        //   - 0: 1.414
        //   - 1: 4.123

        // arithemtic:
        //   - add 0 1      -> 5.537
        //   - sub 1 0      -> 2.709
        //   - mul 0 1      -> 5.829922
        //   - div 1 0      -> 2.91584158416

        // bytecode
        //
        // 0x0000 local_load_f64       0 0 0
        // 0x0008 local_load_f64       0 0 1
        // 0x0010 f64_add
        // 0x0012 local_load_f64       0 0 1
        // 0x001a local_load_f64       0 0 0
        // 0x0022 f64_sub
        // 0x0024 local_load_f64       0 0 0
        // 0x002c local_load_f64       0 0 1
        // 0x0034 f64_mul
        // 0x0036 local_load_f64       0 0 1
        // 0x003e local_load_f64       0 0 0
        // 0x0046 f64_div
        // 0x0048 end

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .write_opcode(Opcode::f64_add)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode(Opcode::f64_sub)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .write_opcode(Opcode::f64_mul)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode(Opcode::f64_div)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code0).to_text());

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::F64, DataType::F64], // params
            vec![DataType::F64, DataType::F64, DataType::F64, DataType::F64], // results
            vec![],                             // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.new_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &vec![ForeignValue::Float64(1.414), ForeignValue::Float64(4.123)],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::Float64(5.537),
                ForeignValue::Float64(2.7090000000000005),
                ForeignValue::Float64(5.829922),
                ForeignValue::Float64(2.915841584158416),
            ]
        );
    }
}
