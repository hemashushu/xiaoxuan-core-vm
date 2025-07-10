// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;
use regex_anre::Regex;

pub fn regex_create(thread_context: &mut ThreadContext) {
    // `fn (module_index: i32, data_access_index: i64, data_length_in_bytes:i32, flavour:i32) -> i32`
    // Returns `regex_index` if the compilation is successful, or -1 if it fails.

    let flavour = thread_context.stack.pop_i32_u();
    let data_length_in_bytes = thread_context.stack.pop_i32_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();

    let target_data_object = thread_context.get_target_data_object(
        module_index as usize,
        data_access_index as usize,
        0,
        data_length_in_bytes as usize,
    );

    let content_address = target_data_object
        .accessor
        .get_start_address_by_index(data_access_index as usize);
    let content_ptr = target_data_object.accessor.get_ptr(content_address, 0);
    let content_bytes =
        unsafe { std::slice::from_raw_parts(content_ptr, data_length_in_bytes as usize) };
    let content = unsafe { str::from_utf8_unchecked(content_bytes) };

    // Parameter `flavour` represents the syntax of the regular expression:
    // 0 for traditional, 1 for the "XiaoXuan Regular Expression (ANRE)."
    let result_regex = if flavour == 1 {
        Regex::from_anre(content)
    } else {
        Regex::new(content)
    };

    match result_regex {
        Ok(regex) => {
            let regex_index = thread_context.thread_resources.add_regex(regex);
            thread_context.stack.push_i32_u(regex_index as u32);
        }
        Err(_) => {
            // If the regex creation fails, push an error code.
            thread_context.stack.push_i32_u(u32::MAX);
        }
    }
}

pub fn regex_capture_group_count(thread_context: &mut ThreadContext) {
    // `fn (regex_index: i32) -> i32`
    // Returns -1 if the regex object does not exist.

    let regex_index = thread_context.stack.pop_i32_u();

    let regex = match thread_context
        .thread_resources
        .get_regex(regex_index as usize)
    {
        Some(regex) => regex,
        None => {
            // If the regex object does not exist, return -1.
            thread_context.stack.push_i32_u(u32::MAX);
            return;
        }
    };

    let count = regex.object_file.capture_group_names.len();
    thread_context.stack.push_i32_u(count as u32);
}

pub fn regex_capture_group_names_length(thread_context: &mut ThreadContext) {
    // `fn (regex_index: i32) -> i32`
    // Returns -1 if the regex object does not exist.

    let regex_index = thread_context.stack.pop_i32_u();

    let regex = match thread_context
        .thread_resources
        .get_regex(regex_index as usize)
    {
        Some(regex) => regex,
        None => {
            // If the regex object does not exist, return -1.
            thread_context.stack.push_i32_u(u32::MAX);
            return;
        }
    };

    if regex.object_file.capture_group_names.is_empty() {
        // If there are no capture group names, return 0.
        thread_context.stack.push_i32_u(0);
    } else {
        let mut total_length = regex
            .object_file
            .capture_group_names
            .iter()
            .fold(0, |acc, name| {
                if let Some(n) = name {
                    acc + n.as_bytes().len()
                } else {
                    acc
                }
            });

        // Add the number of separator characters ('\0') between names.
        total_length += regex.object_file.capture_group_names.len() - 1;

        thread_context.stack.push_i32_u(total_length as u32);
    }
}

pub fn regex_capture_group_names_read(thread_context: &mut ThreadContext) {
    // `fn (regex_index: i32, module_index: i32, data_access_index: i64, expected_data_length_in_bytes:i32) -> i32`

    let expected_data_length_in_bytes = thread_context.stack.pop_i32_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();
    let regex_index = thread_context.stack.pop_i32_u();

    let regex = match thread_context
        .thread_resources
        .get_regex(regex_index as usize)
    {
        Some(regex) => regex,
        None => {
            // If the regex object does not exist, return -1.
            thread_context.stack.push_i32_u(u32::MAX);
            return;
        }
    };

    if regex.object_file.capture_group_names.is_empty() {
        // If there are no capture group names, return 0.
        thread_context.stack.push_i32_u(0);
    } else {
        let names = regex
            .object_file
            .capture_group_names
            .iter()
            .map(|name| {
                if let Some(n) = name {
                    n.to_owned()
                } else {
                    String::new()
                }
            })
            .collect::<Vec<String>>();

        let content = names.join("\0");
        let content_bytes = content.as_bytes();
        let content_length = content_bytes.len();

        let actual_read_length = if content_length > expected_data_length_in_bytes as usize {
            expected_data_length_in_bytes as usize
        } else {
            content_length
        };

        let target_data_object = thread_context.get_target_data_object(
            module_index as usize,
            data_access_index as usize,
            0,
            actual_read_length,
        );

        let src_ptr = content_bytes.as_ptr();
        target_data_object.accessor.write_idx(
            src_ptr,
            data_access_index as usize,
            0,
            actual_read_length,
        );

        thread_context.stack.push_i32_u(actual_read_length as u32);
    }
}

pub fn regex_match(thread_context: &mut ThreadContext) {
    // ```
    // fn (
    //   regex_index: i32,
    //   module_index: i32,
    //   data_access_index: i64,
    //   data_length_in_bytes:i32,
    //   start_offset_in_bytes:i32) -> (match_start:i32, match_end_exclusive:i32)
    // ```
    //
    // Returns:
    // - `(match_start:i32, match_end_exclusive:i32)` if a match is found,
    // - `(0, 0)` if no match is found.
    // -  `(0, -1)` if the regex object does not exist.

    let start_offset_in_bytes = thread_context.stack.pop_i32_u();
    let data_length_in_bytes = thread_context.stack.pop_i32_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();
    let regex_index = thread_context.stack.pop_i32_u();

    let target_data_object = thread_context.get_target_data_object(
        module_index as usize,
        data_access_index as usize,
        0,
        data_length_in_bytes as usize,
    );

    let content_address = target_data_object
        .accessor
        .get_start_address_by_index(data_access_index as usize);
    let content_ptr = target_data_object.accessor.get_ptr(content_address, 0);
    let content_bytes =
        unsafe { std::slice::from_raw_parts(content_ptr, data_length_in_bytes as usize) };
    // let content = unsafe { str::from_utf8_unchecked(content_bytes) };

    let regex = match thread_context
        .thread_resources
        .get_regex(regex_index as usize)
    {
        Some(regex) => regex,
        None => {
            // If the regex object does not exist, return (0, -1).
            thread_context.stack.push_i32_u(0);
            thread_context.stack.push_i32_u(u32::MAX);
            return;
        }
    };

    let number_of_capture_groups = regex.object_file.capture_group_names.len();
    let mut context =
        regex_anre::context::Context::from_bytes(content_bytes, number_of_capture_groups);

    if !regex_anre::process::start_process(
        &mut context,
        &regex.object_file,
        start_offset_in_bytes as usize,
    ) {
        // no match found, return `(0, 0)`
        thread_context.stack.push_i32_u(0);
        thread_context.stack.push_i32_u(0);
        return;
    }

    let range = context.match_ranges[0].clone();

    // store match ranges
    thread_context
        .thread_resources
        .set_last_captures(context.match_ranges.clone());

    // Returns `(match_start:i32, match_end_exclusive:i32)`
    thread_context.stack.push_i32_u(range.start as u32);
    thread_context.stack.push_i32_u(range.end as u32);
}

pub fn regex_last_captures_read(thread_context: &mut ThreadContext) {
    // `fn (module_index: i32, data_access_index: i64, data_length_in_bytes:i32) -> i32`
    let expected_data_length_in_bytes = thread_context.stack.pop_i32_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();

    let last_captures = thread_context.thread_resources.get_last_captures();
    let content_bytes = last_captures
        .iter()
        .flat_map(|range| {
            let mut data = range.start.to_le_bytes().to_vec();
            data.extend_from_slice(&range.end.to_le_bytes());
            data
        })
        .collect::<Vec<_>>();

    let content_length = content_bytes.len();

    let actual_read_length = if content_length > expected_data_length_in_bytes as usize {
        expected_data_length_in_bytes as usize
    } else {
        content_length
    };

    let target_data_object = thread_context.get_target_data_object(
        module_index as usize,
        data_access_index as usize,
        0,
        actual_read_length,
    );

    let src_ptr = content_bytes.as_ptr();
    target_data_object.accessor.write_idx(
        src_ptr,
        data_access_index as usize,
        0,
        actual_read_length,
    );

    thread_context.stack.push_i32_u(actual_read_length as u32);
}

pub fn regex_remove(thread_context: &mut ThreadContext) {
    // `fn (regex_index: i32)`

    let regex_index = thread_context.stack.pop_i32_u();

    thread_context
        .thread_resources
        .remove_regex(regex_index as usize);
}
