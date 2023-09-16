// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::thread::Thread;

use super::InterpretResult;

pub fn f32_abs(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.abs());
    InterpretResult::MoveOn(2)
}

pub fn f32_neg(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, -v);
    InterpretResult::MoveOn(2)
}

pub fn f32_ceil(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.ceil());
    InterpretResult::MoveOn(2)
}

pub fn f32_floor(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.floor());
    InterpretResult::MoveOn(2)
}

pub fn f32_round_half_away_from_zero(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.round());
    InterpretResult::MoveOn(2)
}

pub fn f32_trunc(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.trunc());
    InterpretResult::MoveOn(2)
}

pub fn f32_fract(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.fract());
    InterpretResult::MoveOn(2)
}

pub fn f32_sqrt(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.sqrt());
    InterpretResult::MoveOn(2)
}

pub fn f32_cbrt(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.cbrt());
    InterpretResult::MoveOn(2)
}

pub fn f32_pow(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f32(thread);
    store_f32(thread, left.powf(right));
    InterpretResult::MoveOn(2)
}

pub fn f32_exp(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.exp());
    InterpretResult::MoveOn(2)
}

pub fn f32_exp2(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.exp2());
    InterpretResult::MoveOn(2)
}

pub fn f32_ln(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.ln());
    InterpretResult::MoveOn(2)
}

pub fn f32_log(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f32(thread);
    store_f32(thread, left.log(right));
    InterpretResult::MoveOn(2)
}

pub fn f32_log2(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.log2());
    InterpretResult::MoveOn(2)
}

pub fn f32_log10(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.log10());
    InterpretResult::MoveOn(2)
}

pub fn f32_sin(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.sin());
    InterpretResult::MoveOn(2)
}

pub fn f32_cos(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.cos());
    InterpretResult::MoveOn(2)
}

pub fn f32_tan(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.tan());
    InterpretResult::MoveOn(2)
}

pub fn f32_asin(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.asin());
    InterpretResult::MoveOn(2)
}

pub fn f32_acos(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.acos());
    InterpretResult::MoveOn(2)
}

pub fn f32_atan(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f32(thread);
    store_f32(thread, v.atan());
    InterpretResult::MoveOn(2)
}

pub fn f32_copysign(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f32(thread);
    store_f32(thread, left.copysign(right));
    InterpretResult::MoveOn(2)
}

pub fn f64_abs(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.abs());
    InterpretResult::MoveOn(2)
}

pub fn f64_neg(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, -v);
    InterpretResult::MoveOn(2)
}

pub fn f64_ceil(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.ceil());
    InterpretResult::MoveOn(2)
}

pub fn f64_floor(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.floor());
    InterpretResult::MoveOn(2)
}

pub fn f64_round_half_away_from_zero(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.round());
    InterpretResult::MoveOn(2)
}

pub fn f64_trunc(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.trunc());
    InterpretResult::MoveOn(2)
}

pub fn f64_fract(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.fract());
    InterpretResult::MoveOn(2)
}

pub fn f64_sqrt(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.sqrt());
    InterpretResult::MoveOn(2)
}

pub fn f64_cbrt(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.cbrt());
    InterpretResult::MoveOn(2)
}

pub fn f64_pow(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f64(thread);
    store_f64(thread, left.powf(right));
    InterpretResult::MoveOn(2)
}

pub fn f64_exp(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.exp());
    InterpretResult::MoveOn(2)
}

pub fn f64_exp2(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.exp2());
    InterpretResult::MoveOn(2)
}

pub fn f64_ln(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.ln());
    InterpretResult::MoveOn(2)
}

pub fn f64_log(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f64(thread);
    store_f64(thread, left.log(right));
    InterpretResult::MoveOn(2)
}

pub fn f64_log2(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.log2());
    InterpretResult::MoveOn(2)
}

pub fn f64_log10(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.log10());
    InterpretResult::MoveOn(2)
}

pub fn f64_sin(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.sin());
    InterpretResult::MoveOn(2)
}

pub fn f64_cos(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.cos());
    InterpretResult::MoveOn(2)
}

pub fn f64_tan(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.tan());
    InterpretResult::MoveOn(2)
}

pub fn f64_asin(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.asin());
    InterpretResult::MoveOn(2)
}

pub fn f64_acos(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.acos());
    InterpretResult::MoveOn(2)
}

pub fn f64_atan(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_f64(thread);
    store_f64(thread, v.atan());
    InterpretResult::MoveOn(2)
}

pub fn f64_copysign(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f64(thread);
    store_f64(thread, left.copysign(right));
    InterpretResult::MoveOn(2)
}

#[inline]
fn load_operand_f32(thread: &mut Thread) -> f32 {
    thread.stack.pop_f32()
}

#[inline]
fn load_operands_f32(thread: &mut Thread) -> (f32, f32) {
    let right = thread.stack.pop_f32();
    let left = thread.stack.pop_f32();
    (left, right)
}

#[inline]
fn load_operand_f64(thread: &mut Thread) -> f64 {
    thread.stack.pop_f64()
}

#[inline]
fn load_operands_f64(thread: &mut Thread) -> (f64, f64) {
    let right = thread.stack.pop_f64();
    let left = thread.stack.pop_f64();
    (left, right)
}

#[inline]
fn store_f32(thread: &mut Thread, v: f32) {
    thread.stack.push_f32(v);
}

#[inline]
fn store_f64(thread: &mut Thread, v: f64) {
    thread.stack.push_f64(v);
}
