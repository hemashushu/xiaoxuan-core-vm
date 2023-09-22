// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::thread::Thread;

use super::InterpretResult;

pub fn i32_add(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_i32_u(thread, left + right);
    InterpretResult::Move(2)
}

pub fn i32_sub(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_i32_u(thread, left - right);
    InterpretResult::Move(2)
}

pub fn i32_mul(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_i32_u(thread, left * right);
    InterpretResult::Move(2)
}

pub fn i32_div_s(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_s(thread);
    store_i32_s(thread, left / right);
    InterpretResult::Move(2)
}

pub fn i32_div_u(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_i32_u(thread, left / right);
    InterpretResult::Move(2)
}

pub fn i32_rem_s(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_s(thread);
    store_i32_s(thread, left % right);
    InterpretResult::Move(2)
}

pub fn i32_rem_u(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_i32_u(thread, left % right);
    InterpretResult::Move(2)
}

pub fn i64_add(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_i64_u(thread, left + right);
    InterpretResult::Move(2)
}

pub fn i64_sub(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_i64_u(thread, left - right);
    InterpretResult::Move(2)
}

pub fn i64_mul(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_i64_u(thread, left * right);
    InterpretResult::Move(2)
}

pub fn i64_div_s(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_s(thread);
    store_i64_s(thread, left / right);
    InterpretResult::Move(2)
}

pub fn i64_div_u(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_i64_u(thread, left / right);
    InterpretResult::Move(2)
}

pub fn i64_rem_s(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_s(thread);
    store_i64_s(thread, left % right);
    InterpretResult::Move(2)
}

pub fn i64_rem_u(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_i64_u(thread, left % right);
    InterpretResult::Move(2)
}

pub fn f32_add(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f32(thread);
    store_f32(thread, left + right);
    InterpretResult::Move(2)
}

pub fn f32_sub(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f32(thread);
    store_f32(thread, left - right);
    InterpretResult::Move(2)
}

pub fn f32_mul(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f32(thread);
    store_f32(thread, left * right);
    InterpretResult::Move(2)
}

pub fn f32_div(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f32(thread);
    store_f32(thread, left / right);
    InterpretResult::Move(2)
}

pub fn f64_add(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f64(thread);
    store_f64(thread, left + right);
    InterpretResult::Move(2)
}

pub fn f64_sub(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f64(thread);
    store_f64(thread, left - right);
    InterpretResult::Move(2)
}

pub fn f64_mul(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f64(thread);
    store_f64(thread, left * right);
    InterpretResult::Move(2)
}

pub fn f64_div(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f64(thread);
    store_f64(thread, left / right);
    InterpretResult::Move(2)
}

#[inline]
fn load_operands_i32_s(thread: &mut Thread) -> (i32, i32) {
    let right = thread.stack.pop_i32_s();
    let left = thread.stack.pop_i32_s();
    (left, right)
}

#[inline]
fn load_operands_i32_u(thread: &mut Thread) -> (u32, u32) {
    let right = thread.stack.pop_i32_u();
    let left = thread.stack.pop_i32_u();
    (left, right)
}

#[inline]
fn load_operands_i64_s(thread: &mut Thread) -> (i64, i64) {
    let right = thread.stack.pop_i64_s();
    let left = thread.stack.pop_i64_s();
    (left, right)
}

#[inline]
fn load_operands_i64_u(thread: &mut Thread) -> (u64, u64) {
    let right = thread.stack.pop_i64_u();
    let left = thread.stack.pop_i64_u();
    (left, right)
}

#[inline]
fn load_operands_f32(thread: &mut Thread) -> (f32, f32) {
    let right = thread.stack.pop_f32();
    let left = thread.stack.pop_f32();
    (left, right)
}

#[inline]
fn load_operands_f64(thread: &mut Thread) -> (f64, f64) {
    let right = thread.stack.pop_f64();
    let left = thread.stack.pop_f64();
    (left, right)
}

#[inline]
fn store_i32_s(thread: &mut Thread, v: i32) {
    thread.stack.push_i32_s(v);
}

#[inline]
fn store_i32_u(thread: &mut Thread, v: u32) {
    thread.stack.push_i32_u(v);
}

#[inline]
fn store_i64_s(thread: &mut Thread, v: i64) {
    thread.stack.push_i64_s(v);
}

#[inline]
fn store_i64_u(thread: &mut Thread, v: u64) {
    thread.stack.push_i64_u(v);
}

#[inline]
fn store_f32(thread: &mut Thread, v: f32) {
    thread.stack.push_f32(v);
}

#[inline]
fn store_f64(thread: &mut Thread, v: f64) {
    thread.stack.push_f64(v);
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        load_modules_binary,
        utils::{build_module_binary_with_single_function, BytecodeWriter},
    };
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    use crate::{init_runtime, interpreter::process_function, thread::Thread};

    #[test]
    fn test_process_arithmetic_i32() {
        init_runtime();

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

        // note of the 'remainder':
        // (211 % -13) = 3
        //  ^      ^
        //  |      |divisor
        //  |dividend <--------- the result always takes the sign of the dividend.

        // bytecode
        //
        // 0x0000 local_load32         0 0
        // 0x0008 local_load32         0 1
        // 0x0010 i32_add
        // 0x0012 nop
        // 0x0014 local_load32         0 1
        // 0x001c local_load32         0 0
        // 0x0024 i32_sub
        // 0x0026 nop
        // 0x0028 local_load32         0 0
        // 0x0030 local_load32         0 1
        // 0x0038 i32_mul
        // 0x003a nop
        // 0x003c local_load32         0 1
        // 0x0044 local_load32         0 2
        // 0x004c i32_div_s
        // 0x004e nop
        // 0x0050 local_load32         0 2
        // 0x0058 local_load32         0 1
        // 0x0060 i32_div_u
        // 0x0062 nop
        // 0x0064 local_load32         0 1
        // 0x006c local_load32         0 2
        // 0x0074 i32_rem_s
        // 0x0076 nop
        // 0x0078 local_load32         0 2
        // 0x0080 local_load32         0 1
        // 0x0088 i32_rem_u
        // 0x008a end

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::i32_add)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode(Opcode::i32_sub)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::i32_mul)
            //
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 2)
            .write_opcode(Opcode::i32_div_s)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 2)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::i32_div_u)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 2)
            .write_opcode(Opcode::i32_rem_s)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 2)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::i32_rem_u)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

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
            ], // results
            code0,
            vec![], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = process_function(
            &mut thread0,
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
            ]
        );
    }

    #[test]
    fn test_process_arithmetic_i64() {
        init_runtime();

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

        // note of the 'remainder':
        // (211 % -13) = 3
        //  ^      ^
        //  |      |divisor
        //  |dividend <--------- the result always takes the sign of the dividend.

        // bytecode
        //
        // 0x0000 local_load           0 0
        // 0x0008 local_load           0 1
        // 0x0010 i64_add
        // 0x0012 nop
        // 0x0014 local_load           0 1
        // 0x001c local_load           0 0
        // 0x0024 i64_sub
        // 0x0026 nop
        // 0x0028 local_load           0 0
        // 0x0030 local_load           0 1
        // 0x0038 i64_mul
        // 0x003a nop
        // 0x003c local_load           0 1
        // 0x0044 local_load           0 2
        // 0x004c i64_div_s
        // 0x004e nop
        // 0x0050 local_load           0 2
        // 0x0058 local_load           0 1
        // 0x0060 i64_div_u
        // 0x0062 nop
        // 0x0064 local_load           0 1
        // 0x006c local_load           0 2
        // 0x0074 i64_rem_s
        // 0x0076 nop
        // 0x0078 local_load           0 2
        // 0x0080 local_load           0 1
        // 0x0088 i64_rem_u
        // 0x008a end

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::local_load, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load, 0, 1)
            .write_opcode(Opcode::i64_add)
            .write_opcode_i16_i32(Opcode::local_load, 0, 1)
            .write_opcode_i16_i32(Opcode::local_load, 0, 0)
            .write_opcode(Opcode::i64_sub)
            .write_opcode_i16_i32(Opcode::local_load, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load, 0, 1)
            .write_opcode(Opcode::i64_mul)
            //
            .write_opcode_i16_i32(Opcode::local_load, 0, 1)
            .write_opcode_i16_i32(Opcode::local_load, 0, 2)
            .write_opcode(Opcode::i64_div_s)
            .write_opcode_i16_i32(Opcode::local_load, 0, 2)
            .write_opcode_i16_i32(Opcode::local_load, 0, 1)
            .write_opcode(Opcode::i64_div_u)
            .write_opcode_i16_i32(Opcode::local_load, 0, 1)
            .write_opcode_i16_i32(Opcode::local_load, 0, 2)
            .write_opcode(Opcode::i64_rem_s)
            .write_opcode_i16_i32(Opcode::local_load, 0, 2)
            .write_opcode_i16_i32(Opcode::local_load, 0, 1)
            .write_opcode(Opcode::i64_rem_u)
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
            ], // results
            code0,
            vec![], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = process_function(
            &mut thread0,
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
            ]
        );
    }

    #[test]
    fn test_process_arithmetic_f32() {
        init_runtime();

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
        // 0x0000 local_load32_f32     0 0
        // 0x0008 local_load32_f32     0 1
        // 0x0010 f32_add
        // 0x0012 nop
        // 0x0014 local_load32_f32     0 1
        // 0x001c local_load32_f32     0 0
        // 0x0024 f32_sub
        // 0x0026 nop
        // 0x0028 local_load32_f32     0 0
        // 0x0030 local_load32_f32     0 1
        // 0x0038 f32_mul
        // 0x003a nop
        // 0x003c local_load32_f32     0 1
        // 0x0044 local_load32_f32     0 0
        // 0x004c f32_div
        // 0x004e end

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 1)
            .write_opcode(Opcode::f32_add)
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 1)
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 0)
            .write_opcode(Opcode::f32_sub)
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 1)
            .write_opcode(Opcode::f32_mul)
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 1)
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 0)
            .write_opcode(Opcode::f32_div)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code0).to_text());

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::F32, DataType::F32], // params
            vec![DataType::F32, DataType::F32, DataType::F32, DataType::F32], // results
            code0,
            vec![], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = process_function(
            &mut thread0,
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
        init_runtime();

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
        // 0x0000 local_load_f64       0 0
        // 0x0008 local_load_f64       0 1
        // 0x0010 f64_add
        // 0x0012 nop
        // 0x0014 local_load_f64       0 1
        // 0x001c local_load_f64       0 0
        // 0x0024 f64_sub
        // 0x0026 nop
        // 0x0028 local_load_f64       0 0
        // 0x0030 local_load_f64       0 1
        // 0x0038 f64_mul
        // 0x003a nop
        // 0x003c local_load_f64       0 1
        // 0x0044 local_load_f64       0 0
        // 0x004c f64_div
        // 0x004e end

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 1)
            .write_opcode(Opcode::f64_add)
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 1)
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 0)
            .write_opcode(Opcode::f64_sub)
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 1)
            .write_opcode(Opcode::f64_mul)
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 1)
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 0)
            .write_opcode(Opcode::f64_div)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code0).to_text());

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::F64, DataType::F64], // params
            vec![DataType::F64, DataType::F64, DataType::F64, DataType::F64], // results
            code0,
            vec![], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = process_function(
            &mut thread0,
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
