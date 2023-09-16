// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::thread::Thread;

use super::InterpretResult;

pub fn i32_eqz(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_i32_u();
    store_bool(thread, value == 0);
    InterpretResult::MoveOn(2)
}

pub fn i32_nez(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_i32_u();
    store_bool(thread, value != 0);
    InterpretResult::MoveOn(2)
}

pub fn i32_eq(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_bool(thread, left == right);
    InterpretResult::MoveOn(2)
}

pub fn i32_ne(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_bool(thread, left != right);
    InterpretResult::MoveOn(2)
}

pub fn i32_lt_s(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_s(thread);
    store_bool(thread, left < right);
    InterpretResult::MoveOn(2)
}

pub fn i32_lt_u(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_bool(thread, left < right);
    InterpretResult::MoveOn(2)
}

pub fn i32_gt_s(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_s(thread);
    store_bool(thread, left > right);
    InterpretResult::MoveOn(2)
}

pub fn i32_gt_u(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_bool(thread, left > right);
    InterpretResult::MoveOn(2)
}

pub fn i32_le_s(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_s(thread);
    store_bool(thread, left <= right);
    InterpretResult::MoveOn(2)
}

pub fn i32_le_u(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_bool(thread, left <= right);
    InterpretResult::MoveOn(2)
}

pub fn i32_ge_s(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_s(thread);
    store_bool(thread, left >= right);
    InterpretResult::MoveOn(2)
}

pub fn i32_ge_u(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_bool(thread, left >= right);
    InterpretResult::MoveOn(2)
}

pub fn i64_eqz(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_i64_u();
    store_bool(thread, value == 0);
    InterpretResult::MoveOn(2)
}

pub fn i64_nez(thread: &mut Thread) -> InterpretResult {
    let value = thread.stack.pop_i64_u();
    store_bool(thread, value != 0);
    InterpretResult::MoveOn(2)
}

pub fn i64_eq(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_bool(thread, left == right);
    InterpretResult::MoveOn(2)
}

pub fn i64_ne(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_bool(thread, left != right);
    InterpretResult::MoveOn(2)
}

pub fn i64_lt_s(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_s(thread);
    store_bool(thread, left < right);
    InterpretResult::MoveOn(2)
}

pub fn i64_lt_u(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_bool(thread, left < right);
    InterpretResult::MoveOn(2)
}

pub fn i64_gt_s(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_s(thread);
    store_bool(thread, left > right);
    InterpretResult::MoveOn(2)
}

pub fn i64_gt_u(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_bool(thread, left > right);
    InterpretResult::MoveOn(2)
}

pub fn i64_le_s(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_s(thread);
    store_bool(thread, left <= right);
    InterpretResult::MoveOn(2)
}

pub fn i64_le_u(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_bool(thread, left <= right);
    InterpretResult::MoveOn(2)
}

pub fn i64_ge_s(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_s(thread);
    store_bool(thread, left >= right);
    InterpretResult::MoveOn(2)
}

pub fn i64_ge_u(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_bool(thread, left >= right);
    InterpretResult::MoveOn(2)
}

pub fn f32_eq(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f32(thread);
    store_bool(thread, left == right);
    InterpretResult::MoveOn(2)
}

pub fn f32_ne(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f32(thread);
    store_bool(thread, left != right);
    InterpretResult::MoveOn(2)
}

pub fn f32_lt(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f32(thread);
    store_bool(thread, left < right);
    InterpretResult::MoveOn(2)
}

pub fn f32_gt(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f32(thread);
    store_bool(thread, left > right);
    InterpretResult::MoveOn(2)
}

pub fn f32_le(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f32(thread);
    store_bool(thread, left <= right);
    InterpretResult::MoveOn(2)
}

pub fn f32_ge(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f32(thread);
    store_bool(thread, left >= right);
    InterpretResult::MoveOn(2)
}

pub fn f64_eq(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f64(thread);
    store_bool(thread, left == right);
    InterpretResult::MoveOn(2)
}

pub fn f64_ne(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f64(thread);
    store_bool(thread, left != right);
    InterpretResult::MoveOn(2)
}

pub fn f64_lt(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f64(thread);
    store_bool(thread, left < right);
    InterpretResult::MoveOn(2)
}

pub fn f64_gt(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f64(thread);
    store_bool(thread, left > right);
    InterpretResult::MoveOn(2)
}

pub fn f64_le(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f64(thread);
    store_bool(thread, left <= right);
    InterpretResult::MoveOn(2)
}

pub fn f64_ge(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f64(thread);
    store_bool(thread, left >= right);
    InterpretResult::MoveOn(2)
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
fn store_bool(thread: &mut Thread, b: bool) {
    let v = if b { 1u32 } else { 0u32 };
    thread.stack.push_i32_u(v);
}
