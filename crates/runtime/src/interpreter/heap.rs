// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::{memory::Memory, thread::Thread};

use super::InterpretResult;

pub fn heap_load(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread.get_param_i16();
    let address = thread.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread.stack.push_from_memory();
    thread.heap.load_64(total_offset, dst_ptr);

    InterpretResult::MoveOn(4)
}

pub fn heap_load32(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread.get_param_i16();
    let address = thread.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread.stack.push_from_memory();
    thread.heap.load_32(total_offset, dst_ptr);

    InterpretResult::MoveOn(4)
}

pub fn heap_load32_i16_s(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread.get_param_i16();
    let address = thread.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread.stack.push_from_memory();
    thread.heap.load_32_extend_from_i16_s(total_offset, dst_ptr);

    InterpretResult::MoveOn(4)
}

pub fn heap_load32_i16_u(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread.get_param_i16();
    let address = thread.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread.stack.push_from_memory();
    thread.heap.load_32_extend_from_i16_u(total_offset, dst_ptr);

    InterpretResult::MoveOn(4)
}

pub fn heap_load32_i8_s(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread.get_param_i16();
    let address = thread.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread.stack.push_from_memory();
    thread.heap.load_32_extend_from_i8_s(total_offset, dst_ptr);

    InterpretResult::MoveOn(4)
}

pub fn heap_load32_i8_u(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread.get_param_i16();
    let address = thread.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread.stack.push_from_memory();
    thread.heap.load_32_extend_from_i8_u(total_offset, dst_ptr);

    InterpretResult::MoveOn(4)
}

pub fn heap_load_f64(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread.get_param_i16();
    let address = thread.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread.stack.push_from_memory();
    thread.heap.load_64_with_float_check(total_offset, dst_ptr);

    InterpretResult::MoveOn(4)
}

pub fn heap_load32_f32(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread.get_param_i16();
    let address = thread.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread.stack.push_from_memory();
    thread.heap.load_32_with_float_check(total_offset, dst_ptr);

    InterpretResult::MoveOn(4)
}

pub fn heap_store(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread.get_param_i16();
    let address = thread.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;

    let src_ptr = thread.stack.pop_to_memory();
    thread.heap.store_64(src_ptr, total_offset);
    InterpretResult::MoveOn(4)
}

pub fn heap_store32(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread.get_param_i16();
    let address = thread.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;

    let src_ptr = thread.stack.pop_to_memory();
    thread.heap.store_32(src_ptr, total_offset);
    InterpretResult::MoveOn(4)
}

pub fn heap_store16(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread.get_param_i16();
    let address = thread.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;

    let src_ptr = thread.stack.pop_to_memory();
    thread.heap.store_16(src_ptr, total_offset);
    InterpretResult::MoveOn(4)
}

pub fn heap_store8(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread.get_param_i16();
    let address = thread.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;

    let src_ptr = thread.stack.pop_to_memory();
    thread.heap.store_8(src_ptr, total_offset);
    InterpretResult::MoveOn(4)
}
