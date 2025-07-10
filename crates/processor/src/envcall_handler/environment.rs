// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;

pub fn program_path_length(thread_context: &mut ThreadContext) {
    // `fn () -> i32`
    let process_property = thread_context.process_property.lock().unwrap();
    let size = process_property.program_path.to_str().unwrap().len();
    thread_context.stack.push_i32_u(size as u32);
}

pub fn program_path_read(thread_context: &mut ThreadContext) {
    // `fn (module_index: i32, data_access_index: i64, expected_data_length_in_bytes:i32) -> i32`
    let expected_data_length_in_bytes = thread_context.stack.pop_i32_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();

    let process_property = thread_context.process_property.lock().unwrap();
    let content = process_property.program_path.to_str().unwrap();
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

pub fn program_source_type(thread_context: &mut ThreadContext) {
    // `fn () -> i32`
    let process_property = thread_context.process_property.lock().unwrap();
    let size = process_property.program_source_type as u32;

    thread_context.stack.push_i32_u(size as u32);
}

pub fn arguments_length(thread_context: &mut ThreadContext) {
    // `fn () -> i32`
    let process_property = thread_context.process_property.lock().unwrap();

    if process_property.arguments.is_empty() {
        thread_context.stack.push_i32_u(0);
    } else {
        let mut content_length = process_property
            .arguments
            .iter()
            .fold(0, |acc, arg| acc + arg.as_bytes().len());

        // Add the number of null terminators
        content_length += process_property.arguments.len() - 1;
        thread_context.stack.push_i32_u(content_length as u32);
    }
}

pub fn arguments_read(thread_context: &mut ThreadContext) {
    // `fn (module_index: i32, data_access_index: i64, expected_data_length_in_bytes:i32) -> i32`
    let expected_data_length_in_bytes = thread_context.stack.pop_i32_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();

    let process_property = thread_context.process_property.lock().unwrap();
    let content = process_property.arguments.join("\0");
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

pub fn environment_variables_length(thread_context: &mut ThreadContext) {
    // `fn () -> i32`
    let process_property = thread_context.process_property.lock().unwrap();

    if process_property.environments.is_empty() {
        thread_context.stack.push_i32_u(0);
    } else {
        let mut content_length = process_property
            .environments
            .iter()
            .fold(0, |acc, (key, value)| {
                acc + key.as_bytes().len() + value.as_bytes().len()
            });

        // Add the number of null terminators
        content_length += process_property.environments.len() - 1;
        thread_context.stack.push_i32_u(content_length as u32);
    }
}

pub fn environment_variables_read(thread_context: &mut ThreadContext) {
    // `fn (module_index: i32, data_access_index: i64, expected_data_length_in_bytes:i32) -> i32`
    let expected_data_length_in_bytes = thread_context.stack.pop_i32_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();

    let process_property = thread_context.process_property.lock().unwrap();
    let content = process_property
        .environments
        .iter()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect::<Vec<String>>()
        .join("\0");
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

pub fn environment_variable_find(thread_context: &mut ThreadContext) {
    // `fn (module_index: i32, data_access_index: i64, data_length_in_bytes: i32) -> i32`

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

    let process_property = thread_context.process_property.lock().unwrap();
    let position = process_property
        .environments
        .iter()
        .position(|(name, _)| name == content);

    if let Some(pos) = position {
        // If the environment variable is found, return its index.
        thread_context.stack.push_i32_u(pos as u32);
    } else {
        // If not found, return -1.
        thread_context.stack.push_i32_u(u32::MAX);
    }
}

pub fn environment_variable_item_length(thread_context: &mut ThreadContext) {
    // `fn (environment_variable_index: i32) -> i32`

    let environment_variable_index = thread_context.stack.pop_i32_u();

    let process_property = thread_context.process_property.lock().unwrap();
    let opt_environment_variable = process_property
        .environments
        .get(environment_variable_index as usize);

    if let Some((_, value)) = opt_environment_variable {
        // If the environment variable exists, return its value length.
        let value_length = value.as_bytes().len();
        thread_context.stack.push_i32_u(value_length as u32);
    } else {
        // If the environment variable does not exist, return 0.
        thread_context.stack.push_i32_u(0);
    }
}

pub fn environment_variable_item_read(thread_context: &mut ThreadContext) {
    // `fn (environment_variable_index: i32, module_index: i32, data_access_index: i64, expected_data_length_in_bytes:i32) -> i32`

    let expected_data_length_in_bytes = thread_context.stack.pop_i32_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();
    let environment_variable_index = thread_context.stack.pop_i32_u();

    let process_property = thread_context.process_property.lock().unwrap();
    let opt_environment_variable = process_property
        .environments
        .get(environment_variable_index as usize);

    if let Some((_, content)) = opt_environment_variable {
        // If the environment variable exists, read its value and
        // return the actual length of the data read.

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
    } else {
        // If the environment variable does not exist, return 0.
        thread_context.stack.push_i32_u(0);
    }
}

pub fn environment_variable_set(thread_context: &mut ThreadContext) {
    // `fn (module_index: i32, data_access_index: i64, data_length_in_bytes: i32)`
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

    let (name, value) = content
        .split_once('=')
        .map(|(n, v)| (n.to_string(), v.to_string()))
        .unwrap_or((content.to_string(), String::new()));

    let mut process_property = thread_context.process_property.lock().unwrap();
    let position = process_property
        .environments
        .iter()
        .position(|(var_name, _)| var_name == &name);

    if let Some(pos) = position {
        // If the environment variable is found, replace its value.
        process_property.environments[pos] = (name, value);
    } else {
        // If not found, add a new environment variable.
        process_property.environments.push((name, value));
    }
}

pub fn environment_variable_remove(thread_context: &mut ThreadContext) {
    // `fn (module_index: i32, data_access_index: i64, data_length_in_bytes: i32)`
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

    let mut process_property = thread_context.process_property.lock().unwrap();
    let position = process_property
        .environments
        .iter()
        .position(|(var_name, _)| var_name == content);

    if let Some(pos) = position {
        // If the environment variable is found, remove it.
        process_property.environments.remove(pos);
    }
}
