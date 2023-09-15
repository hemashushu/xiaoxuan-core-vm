// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::thread::Thread;

use super::InterpretResult;

// demote i64 to i32
// discard the high 32 bits of an i64 number directly
pub fn i32_demote_i64(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_i64_u();
    thread.stack.push_i32_u(value as u32);
    InterpretResult::MoveOn(2)
}

// promote i32 to i64
pub fn i64_promote_i32_s(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_i32_s();
    thread.stack.push_i64_s(value as i64);
    InterpretResult::MoveOn(2)
}

pub fn i64_promote_i32_u(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_i32_u();
    thread.stack.push_i64_u(value as u64);
    InterpretResult::MoveOn(2)
}

// demote f64 to f32
pub fn f32_demote_f64(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_f64();
    thread.stack.push_f32(value as f32);
    InterpretResult::MoveOn(2)
}

// promote f32 to f64
pub fn f64_promote_f32(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_f32();
    thread.stack.push_f64(value as f64);
    InterpretResult::MoveOn(2)
}

// convert float to int
// truncate fractional part
pub fn i32_trunc_f32_s(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_f32();
    thread.stack.push_i32_s(value as i32);
    InterpretResult::MoveOn(2)
}

pub fn i32_trunc_f32_u(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_f32();
    thread.stack.push_i32_u(value as u32);
    InterpretResult::MoveOn(2)
}

pub fn i32_trunc_f64_s(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_f64();
    thread.stack.push_i32_s(value as i32);
    InterpretResult::MoveOn(2)
}

pub fn i32_trunc_f64_u(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_f64();
    thread.stack.push_i32_u(value as u32);
    InterpretResult::MoveOn(2)
}

pub fn i64_trunc_f32_s(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_f32();
    thread.stack.push_i64_s(value as i64);
    InterpretResult::MoveOn(2)
}

pub fn i64_trunc_f32_u(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_f32();
    thread.stack.push_i64_u(value as u64);
    InterpretResult::MoveOn(2)
}

pub fn i64_trunc_f64_s(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_f64();
    thread.stack.push_i64_s(value as i64);
    InterpretResult::MoveOn(2)
}

pub fn i64_trunc_f64_u(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_f64();
    thread.stack.push_i64_u(value as u64);
    InterpretResult::MoveOn(2)
}

// convert int to float
pub fn f32_convert_i32_s(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_i32_s();
    thread.stack.push_f32(value as f32);
    InterpretResult::MoveOn(2)
}

pub fn f32_convert_i32_u(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_i32_u();
    thread.stack.push_f32(value as f32);
    InterpretResult::MoveOn(2)
}

pub fn f32_convert_i64_s(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_i64_s();
    thread.stack.push_f32(value as f32);
    InterpretResult::MoveOn(2)
}

pub fn f32_convert_i64_u(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_i64_u();
    thread.stack.push_f32(value as f32);
    InterpretResult::MoveOn(2)
}

pub fn f64_convert_i32_s(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_i32_s();
    thread.stack.push_f64(value as f64);
    InterpretResult::MoveOn(2)
}

pub fn f64_convert_i32_u(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_i32_u();
    thread.stack.push_f64(value as f64);
    InterpretResult::MoveOn(2)
}

pub fn f64_convert_i64_s(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_i64_s();
    thread.stack.push_f64(value as f64);
    InterpretResult::MoveOn(2)
}

pub fn f64_convert_i64_u(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_i64_u();
    thread.stack.push_f64(value as f64);
    InterpretResult::MoveOn(2)
}
