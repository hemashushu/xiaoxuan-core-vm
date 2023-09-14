// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::thread::Thread;

use super::InterpretResult;

pub fn i32_imm(thread: &mut Thread) -> InterpretResult {
    let value = thread.get_param_i32();
    thread.stack.push_i32_u(value);
    InterpretResult::MoveOn(8)
}

pub fn i64_imm(thread: &mut Thread) -> InterpretResult {
    let (low, high) = thread.get_param_i32_i32();
    let mut value: u64 = high as u64;
    value = value << 32;
    value = value | (low as u64);

    thread.stack.push_i64_u(value);
    InterpretResult::MoveOn(12)
}

pub fn f32_imm(thread: &mut Thread) -> InterpretResult {
    let i32_value = thread.get_param_i32();
    let value = unsafe { std::mem::transmute::<u32, f32>(i32_value) };

    thread.stack.push_f32(value);
    InterpretResult::MoveOn(8)
}

pub fn f64_imm(thread: &mut Thread) -> InterpretResult {
    let (low, high) = thread.get_param_i32_i32();

    let mut bytes = [0u8; 8];
    {
        let (p0, p1) = bytes.split_at_mut(4);
        p0.copy_from_slice(&low.to_le_bytes());
        p1.copy_from_slice(&high.to_le_bytes());
    }

    let value = f64::from_le_bytes(bytes);

    thread.stack.push_f64(value);
    InterpretResult::MoveOn(12)
}
