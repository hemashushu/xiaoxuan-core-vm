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
    InterpretResult::MoveOn(2)
}

pub fn i32_sub(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_i32_u(thread, left - right);
    InterpretResult::MoveOn(2)
}

pub fn i32_mul(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_i32_u(thread, left * right);
    InterpretResult::MoveOn(2)
}

pub fn i32_div_s(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_s(thread);
    store_i32_s(thread, left / right);
    InterpretResult::MoveOn(2)
}

pub fn i32_div_u(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_i32_u(thread, left / right);
    InterpretResult::MoveOn(2)
}

pub fn i32_rem_s(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_s(thread);
    store_i32_s(thread, left % right);
    InterpretResult::MoveOn(2)
}

pub fn i32_rem_u(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i32_u(thread);
    store_i32_u(thread, left % right);
    InterpretResult::MoveOn(2)
}

pub fn i64_add(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_i64_u(thread, left + right);
    InterpretResult::MoveOn(2)
}

pub fn i64_sub(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_i64_u(thread, left - right);
    InterpretResult::MoveOn(2)
}

pub fn i64_mul(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_i64_u(thread, left * right);
    InterpretResult::MoveOn(2)
}

pub fn i64_div_s(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_s(thread);
    store_i64_s(thread, left / right);
    InterpretResult::MoveOn(2)
}

pub fn i64_div_u(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_i64_u(thread, left / right);
    InterpretResult::MoveOn(2)
}

pub fn i64_rem_s(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_s(thread);
    store_i64_s(thread, left % right);
    InterpretResult::MoveOn(2)
}

pub fn i64_rem_u(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_i64_u(thread);
    store_i64_u(thread, left % right);
    InterpretResult::MoveOn(2)
}

pub fn f32_add(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f32(thread);
    store_f32(thread, left + right);
    InterpretResult::MoveOn(2)
}

pub fn f32_sub(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f32(thread);
    store_f32(thread, left - right);
    InterpretResult::MoveOn(2)
}

pub fn f32_mul(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f32(thread);
    store_f32(thread, left * right);
    InterpretResult::MoveOn(2)
}

pub fn f32_div(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f32(thread);
    store_f32(thread, left / right);
    InterpretResult::MoveOn(2)
}

pub fn f64_add(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f64(thread);
    store_f64(thread, left + right);
    InterpretResult::MoveOn(2)
}

pub fn f64_sub(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f64(thread);
    store_f64(thread, left - right);
    InterpretResult::MoveOn(2)
}

pub fn f64_mul(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f64(thread);
    store_f64(thread, left * right);
    InterpretResult::MoveOn(2)
}

pub fn f64_div(thread: &mut Thread) -> InterpretResult {
    let (left, right) = load_operands_f64(thread);
    store_f64(thread, left / right);
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
