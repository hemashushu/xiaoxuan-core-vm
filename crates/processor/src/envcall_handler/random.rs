// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;

// See:
// - https://docs.rs/rand/latest/rand/fn.random.html
// - https://docs.rs/rand/latest/rand/fn.random_range.html
// - https://docs.rs/rand/latest/rand/fn.fill.html

pub fn random_i32(thread_context: &mut ThreadContext) {
    // `fn () -> i32`
    let value = rand::random::<i32>();
    thread_context.stack.push_i32_u(value as u32);
}

pub fn random_i64(thread_context: &mut ThreadContext) {
    // `fn () -> i64`
    let value = rand::random::<i64>();
    thread_context.stack.push_i64_u(value as u64);
}

pub fn random_f32(thread_context: &mut ThreadContext) {
    // `fn () -> f32`
    let value = rand::random::<f32>();
    thread_context.stack.push_f32(value);
}

pub fn random_f64(thread_context: &mut ThreadContext) {
    // `fn () -> f64`
    let value = rand::random::<f64>();
    thread_context.stack.push_f64(value);
}

pub fn random_range_i32(thread_context: &mut ThreadContext) {
    // `fn (start: i32, end_exclusive: i32) -> i32`
    let start = thread_context.stack.pop_i32_u() as i32;
    let end_exclusive = thread_context.stack.pop_i32_u() as i32;
    let value = rand::random_range(start..end_exclusive);
    thread_context.stack.push_i32_u(value as u32);
}

pub fn random_range_i64(thread_context: &mut ThreadContext) {
    // `fn (start: i64, end_exclusive: i64) -> i64`
    let start = thread_context.stack.pop_i64_u() as i64;
    let end_exclusive = thread_context.stack.pop_i64_u() as i64;
    let value = rand::random_range(start..end_exclusive);
    thread_context.stack.push_i64_u(value as u64);
}

pub fn random_range_f32(thread_context: &mut ThreadContext) {
    // `fn (start: f32, end_exclusive: f32) -> f32`
    let start = thread_context
        .stack
        .pop_f32()
        .expect("Failed to pop `start` f32 value in envcall `random_range_f32`");

    let end_exclusive = thread_context
        .stack
        .pop_f32()
        .expect("Failed to pop `end_exclusive` f32 value in envcall `random_range_f32`");

    let value = rand::random_range(start..end_exclusive);
    thread_context.stack.push_f32(value);
}

pub fn random_range_f64(thread_context: &mut ThreadContext) {
    // `fn (start: f64, end_exclusive: f64) -> f64`
    let start = thread_context
        .stack
        .pop_f64()
        .expect("Failed to pop `start` f64 value in envcall `random_range_f64`");

    let end_exclusive = thread_context
        .stack
        .pop_f64()
        .expect("Failed to pop `end_exclusive` f64 value in envcall `random_range_f64`");

    let value = rand::random_range(start..end_exclusive);
    thread_context.stack.push_f64(value);
}

pub fn random_fill(thread_context: &mut ThreadContext) {
    // `fn (module_index: i32, data_access_index: i64, data_length_in_bytes:i32) -> ()`
    let data_length_in_bytes = thread_context.stack.pop_i32_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();

    let target_data_object = thread_context.get_target_data_object(
        module_index as usize,
        data_access_index as usize,
        0,
        data_length_in_bytes as usize,
    );

    let mut buf = vec![0u8; data_length_in_bytes as usize];
    rand::fill(&mut buf[..]);

    target_data_object.accessor.write_idx(
        buf.as_ptr(),
        data_access_index as usize,
        0,
        data_length_in_bytes as usize,
    );
}
