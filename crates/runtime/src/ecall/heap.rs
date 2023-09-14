// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::{resizeable_memory::ResizeableMemory, thread::Thread};

pub fn heap_capacity(thread: &mut Thread) -> Result<(), usize> {
    // `fn () -> pages:i64`
    let pages = thread.heap.get_capacity_in_pages();
    thread.stack.push_i64_u(pages as u64);
    Ok(())
}

pub fn heap_resize(thread: &mut Thread) -> Result<(), usize> {
    // `fn (pages:i64) -> new_pages:i64`
    let pages = thread.stack.pop_i64_u();
    let new_pages = thread.heap.resize(pages as usize);
    thread.stack.push_i64_u(new_pages as u64);
    Ok(())
}

pub fn heap_fill(thread: &mut Thread) -> Result<(), usize> {
    // `fn (address:i64, value:i8, count:i64)`
    let count = thread.stack.pop_i64_u() as usize;
    let value = thread.stack.pop_i32_u() as u8;
    let address = thread.stack.pop_i64_u() as usize;

    thread.heap.fill(address, value, count);

    Ok(())
}

pub fn heap_copy(thread: &mut Thread) -> Result<(), usize> {
    // `fn (dst_address:i64, src_address:i64, length_in_bytes:i64)`

    let length_in_bytes = thread.stack.pop_i64_u() as usize;
    let src_address = thread.stack.pop_i64_u() as usize;
    let dst_address = thread.stack.pop_i64_u() as usize;

    thread.heap.copy(dst_address, src_address, length_in_bytes);

    Ok(())
}
