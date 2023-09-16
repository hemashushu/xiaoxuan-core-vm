// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::thread::Thread;

use super::InterpretResult;

pub fn i32_and(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_i32_u(thread, left & right);
    InterpretResult::MoveOn(2)
}

pub fn i32_or(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_i32_u(thread, left | right);
    InterpretResult::MoveOn(2)
}

pub fn i32_xor(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_i32_u(thread, left ^ right);
    InterpretResult::MoveOn(2)
}

pub fn i32_not(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_i32_u(thread);
    store_i32_u(thread, !v);
    InterpretResult::MoveOn(2)
}

pub fn i32_leading_zeros(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_i32_u(thread);
    store_i32_u(thread, v.leading_zeros());
    InterpretResult::MoveOn(2)
}

pub fn i32_trailing_zeros(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_i32_u(thread);
    store_i32_u(thread, v.trailing_zeros());
    InterpretResult::MoveOn(2)
}

pub fn i32_count_ones(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_i32_u(thread);
    store_i32_u(thread, v.count_ones());
    InterpretResult::MoveOn(2)
}

pub fn i32_shl(thread: &mut Thread) -> InterpretResult {
    let number = load_operand_i32_u(thread);
    let bits = load_operand_i32_u(thread);
    store_i32_u(thread, number << (bits & 32));
    InterpretResult::MoveOn(2)
}

pub fn i32_shr_s(thread: &mut Thread) -> InterpretResult {
    let number = load_operand_i32_s(thread);
    let bits = load_operand_i32_u(thread);
    store_i32_s(thread, number >> (bits & 32));
    InterpretResult::MoveOn(2)
}

pub fn i32_shr_u(thread: &mut Thread) -> InterpretResult {
    let number = load_operand_i32_u(thread);
    let bits = load_operand_i32_u(thread);
    store_i32_u(thread, number >> (bits & 32));
    InterpretResult::MoveOn(2)
}

pub fn i32_rotl(thread: &mut Thread) -> InterpretResult {
    let number = load_operand_i32_u(thread);
    let bits = load_operand_i32_u(thread);
    store_i32_u(thread, number.rotate_left(bits));
    InterpretResult::MoveOn(2)
}

pub fn i32_rotr(thread: &mut Thread) -> InterpretResult {
    let number = load_operand_i32_u(thread);
    let bits = load_operand_i32_u(thread);
    store_i32_u(thread, number.rotate_right(bits));
    InterpretResult::MoveOn(2)
}

pub fn i64_and(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_i64_u(thread, left & right);
    InterpretResult::MoveOn(2)
}

pub fn i64_or(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_i64_u(thread, left | right);
    InterpretResult::MoveOn(2)
}

pub fn i64_xor(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_i64_u(thread, left ^ right);
    InterpretResult::MoveOn(2)
}

pub fn i64_not(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_i64_u(thread);
    store_i64_u(thread, !v);
    InterpretResult::MoveOn(2)
}

pub fn i64_leading_zeros(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_i64_u(thread);
    store_i32_u(thread, v.leading_zeros()); // the result of 'clz' is u32
    InterpretResult::MoveOn(2)
}

pub fn i64_trailing_zeros(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_i64_u(thread);
    store_i32_u(thread, v.trailing_zeros()); // the result of 'ctz' is u32
    InterpretResult::MoveOn(2)
}

pub fn i64_count_ones(thread: &mut Thread) -> InterpretResult {
    let v = load_operand_i64_u(thread);
    store_i32_u(thread, v.count_ones()); // the result of 'popcnt' is u32
    InterpretResult::MoveOn(2)
}

pub fn i64_shl(thread: &mut Thread) -> InterpretResult {
    let number = load_operand_i64_u(thread);
    let bits = load_operand_i32_u(thread); // the type of 'bits' is u32
    store_i64_u(thread, number << (bits & 32));
    InterpretResult::MoveOn(2)
}

pub fn i64_shr_s(thread: &mut Thread) -> InterpretResult {
    let number = load_operand_i64_s(thread);
    let bits = load_operand_i32_u(thread); // the type of 'bits' is u32
    store_i64_s(thread, number >> (bits & 32));
    InterpretResult::MoveOn(2)
}

pub fn i64_shr_u(thread: &mut Thread) -> InterpretResult {
    let number = load_operand_i64_u(thread);
    let bits = load_operand_i32_u(thread); // the type of 'bits' is u32
    store_i64_u(thread, number >> (bits & 32));
    InterpretResult::MoveOn(2)
}

pub fn i64_rotl(thread: &mut Thread) -> InterpretResult {
    let number = load_operand_i64_u(thread);
    let bits = load_operand_i32_u(thread); // the type of 'bits' is u32
    store_i64_u(thread, number.rotate_left(bits));
    InterpretResult::MoveOn(2)
}

pub fn i64_rotr(thread: &mut Thread) -> InterpretResult {
    let number = load_operand_i64_u(thread);
    let bits = load_operand_i32_u(thread);  // the type of 'bits' is u32
    store_i64_u(thread, number.rotate_right(bits));
    InterpretResult::MoveOn(2)
}

#[inline]
fn load_operand_i32_s(thread: &mut Thread) -> i32 {
    let v = thread.stack.pop_i32_s();
    v
}

#[inline]
fn load_operand_i32_u(thread: &mut Thread) -> u32 {
    let v = thread.stack.pop_i32_u();
    v
}

#[inline]
fn load_operands_i32_u(thread: &mut Thread) -> (u32, u32) {
    let right = thread.stack.pop_i32_u();
    let left = thread.stack.pop_i32_u();
    (left, right)
}

#[inline]
fn load_operand_i64_u(thread: &mut Thread) -> u64 {
    let v = thread.stack.pop_i64_u();
    v
}

#[inline]
fn load_operand_i64_s(thread: &mut Thread) -> i64 {
    let v = thread.stack.pop_i64_s();
    v
}

#[inline]
fn load_operands_i64_u(thread: &mut Thread) -> (u64, u64) {
    let right = thread.stack.pop_i64_u();
    let left = thread.stack.pop_i64_u();
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