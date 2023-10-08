// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_program::{resizeable_memory::ResizeableMemory, thread_context::ThreadContext};

pub fn heap_capacity(thread_context: &mut ThreadContext) {
    // `fn () -> pages:i64`
    let pages = thread_context.heap.get_capacity_in_pages();
    thread_context.stack.push_i64_u(pages as u64);
}

pub fn heap_resize(thread_context: &mut ThreadContext) {
    // `fn (pages:i64) -> new_pages:i64`
    let pages = thread_context.stack.pop_i64_u();
    let new_pages = thread_context.heap.resize(pages as usize);
    thread_context.stack.push_i64_u(new_pages as u64);
}

pub fn heap_fill(thread_context: &mut ThreadContext) {
    // `fn (address:i64, value:i8, count:i64)`
    let count = thread_context.stack.pop_i64_u() as usize;
    let value = thread_context.stack.pop_i32_u() as u8;
    let address = thread_context.stack.pop_i64_u() as usize;

    thread_context.heap.fill(address, value, count);
}

pub fn heap_copy(thread_context: &mut ThreadContext) {
    // `fn (dst_address:i64, src_address:i64, length_in_bytes:i64)`

    let length_in_bytes = thread_context.stack.pop_i64_u() as usize;
    let src_address = thread_context.stack.pop_i64_u() as usize;
    let dst_address = thread_context.stack.pop_i64_u() as usize;

    thread_context.heap.copy(dst_address, src_address, length_in_bytes);
}
