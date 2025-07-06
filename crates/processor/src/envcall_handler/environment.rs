// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;

pub fn program_path_length(thread_context: &mut ThreadContext) {
    // `fn () -> i32`
    let size = thread_context
        .process_property
        .program_path
        .to_str()
        .unwrap()
        .len();
    thread_context.stack.push_i32_u(size as u32);
}

pub fn program_path_read(thread_context: &mut ThreadContext) {
    // `fn (module_index: i32, data_access_index: i64) -> i32`
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();

    let content = thread_context
        .process_property
        .program_path
        .to_str()
        .unwrap();
    let content_bytes = content.as_bytes();
    let content_length = content_bytes.len();

    let target_data_object = thread_context.get_target_data_object(
        module_index as usize,
        data_access_index as usize,
        0,
        content_length,
    );

    let src_ptr = content_bytes.as_ptr();
    target_data_object
        .accessor
        .write_idx(src_ptr, data_access_index as usize, 0, content_length);

    thread_context.stack.push_i32_u(content_length as u32);
}

pub fn program_source_type(thread_context: &mut ThreadContext) {
    // `fn () -> i32`
    let size = thread_context.process_property.program_source_type as u32;

    thread_context.stack.push_i32_u(size as u32);
}

pub fn argument_length(thread_context: &mut ThreadContext) {
    todo!()
}

pub fn argument_read(thread_context: &mut ThreadContext) {
    todo!()
}

pub fn environment_variable_count(thread_context: &mut ThreadContext) {
    todo!()
}

pub fn environment_variable_item_length(thread_context: &mut ThreadContext) {
    todo!()
}

pub fn environment_variable_item_read(thread_context: &mut ThreadContext) {
    todo!()
}

pub fn environment_variable_set(thread_context: &mut ThreadContext) {
    todo!()
}

pub fn environment_variable_remove(thread_context: &mut ThreadContext) {
    todo!()
}
