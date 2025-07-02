// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

// mod program;
mod host;
// mod multithread;
// mod random;
// mod regex;
mod runtime;
// mod timer;

use anc_context::thread_context::ThreadContext;
use anc_image::bytecode_reader::format_bytecode_as_text;

use crate::envcall_num::EnvCallNum;

pub type EnvCallHandlerFunc = fn(&mut ThreadContext);

fn envcall_unreachable_handler(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    let pc = &thread_context.pc;
    let func_item = &thread_context.module_common_instances[pc.module_index]
        .function_section
        .items[pc.function_internal_index];
    let codes = &thread_context.module_common_instances[pc.module_index]
        .function_section
        .codes_data
        [func_item.code_offset as usize..(func_item.code_offset + func_item.code_length) as usize];
    let code_text = format_bytecode_as_text(codes);

    unreachable!(
        "\
Invalid EnvCall number: 0x{:04x}
Module index: {}
Function internal index: {}
Instruction address: 0x{:04x}
Bytecode:
{}",
        thread_context.get_param_i32(),
        pc.module_index,
        pc.function_internal_index,
        pc.instruction_address,
        code_text
    );
}

#[inline]
pub fn get_envcall_handlers(envcall_num_integer: u32) -> EnvCallHandlerFunc {
    let envcall_num = unsafe { std::mem::transmute::<u32, EnvCallNum>(envcall_num_integer) };
    let category = envcall_num_integer >> 16;

    match category {
        0x0001 => {
            // Category: Runtime information
            match envcall_num {
                EnvCallNum::runtime_edition => runtime::runtime_edition,
                EnvCallNum::runtime_version => runtime::runtime_version,
                _ => envcall_unreachable_handler,
            }
        }
        0x0002 => {
            // Category: Host Information
            match envcall_num {
                EnvCallNum::host_arch => host::host_arch,
                EnvCallNum::host_os => host::host_os,
                EnvCallNum::host_family => host::host_family,
                EnvCallNum::host_endian => host::host_endian,
                EnvCallNum::host_memory_width => host::host_memory_width,
                _ => envcall_unreachable_handler,
            }
        }
        0x0003 => {
            // Category: Process environment
            match envcall_num {
                EnvCallNum::program_path_size => envcall_unreachable_handler,
                EnvCallNum::program_path_text => envcall_unreachable_handler,
                EnvCallNum::program_source_type => envcall_unreachable_handler,
                EnvCallNum::argument_size => envcall_unreachable_handler,
                EnvCallNum::argument_text => envcall_unreachable_handler,
                EnvCallNum::environment_variable_count => envcall_unreachable_handler,
                EnvCallNum::environment_variable_item_size => envcall_unreachable_handler,
                EnvCallNum::environment_variable_item_text => envcall_unreachable_handler,
                EnvCallNum::environment_variable_set => envcall_unreachable_handler,
                EnvCallNum::environment_variable_remove => envcall_unreachable_handler,
                _ => envcall_unreachable_handler,
            }
        }
        0x0004 => {
            // Category: Time
            match envcall_num {
                EnvCallNum::time_now => envcall_unreachable_handler,
                _ => envcall_unreachable_handler,
            }
        }
        0x0005 => {
            // Category: Random number generation
            match envcall_num {
                EnvCallNum::random_i32 => envcall_unreachable_handler,
                EnvCallNum::random_i64 => envcall_unreachable_handler,
                EnvCallNum::random_f32 => envcall_unreachable_handler,
                EnvCallNum::random_f64 => envcall_unreachable_handler,
                EnvCallNum::random_range_i32 => envcall_unreachable_handler,
                EnvCallNum::random_range_i64 => envcall_unreachable_handler,
                EnvCallNum::random_range_f32 => envcall_unreachable_handler,
                EnvCallNum::random_range_f64 => envcall_unreachable_handler,
                _ => envcall_unreachable_handler,
            }
        }
        0x0006 => {
            // Category: I/O
            match envcall_num {
                EnvCallNum::file_open => envcall_unreachable_handler,
                EnvCallNum::file_read => envcall_unreachable_handler,
                EnvCallNum::file_write => envcall_unreachable_handler,
                EnvCallNum::file_seek => envcall_unreachable_handler,
                EnvCallNum::file_flush => envcall_unreachable_handler,
                EnvCallNum::file_close => envcall_unreachable_handler,
                EnvCallNum::file_is_terminal => envcall_unreachable_handler,
                _ => envcall_unreachable_handler,
            }
        }
        0x0007 => {
            // Category: File system
            match envcall_num {
                EnvCallNum::fs_open_dir => envcall_unreachable_handler,
                EnvCallNum::fs_read_dir => envcall_unreachable_handler,
                EnvCallNum::fs_create_dir => envcall_unreachable_handler,
                EnvCallNum::fs_remove_dir => envcall_unreachable_handler,
                EnvCallNum::fs_remove_file => envcall_unreachable_handler,
                EnvCallNum::fs_rename => envcall_unreachable_handler,
                EnvCallNum::fs_exists => envcall_unreachable_handler,
                _ => envcall_unreachable_handler,
            }
        }
        0x0008 => {
            // Category: Thread
            match envcall_num {
                EnvCallNum::thread_index => envcall_unreachable_handler,
                EnvCallNum::thread_create => envcall_unreachable_handler,
                EnvCallNum::thread_start_data_length => envcall_unreachable_handler,
                EnvCallNum::thread_start_data_text => envcall_unreachable_handler,
                EnvCallNum::thread_wait_and_collect => envcall_unreachable_handler,
                EnvCallNum::thread_running_status => envcall_unreachable_handler,
                EnvCallNum::thread_terminate => envcall_unreachable_handler,
                EnvCallNum::thread_receive_msg_from_parent => envcall_unreachable_handler,
                EnvCallNum::thread_send_msg_to_parent => envcall_unreachable_handler,
                EnvCallNum::thread_receive_msg => envcall_unreachable_handler,
                EnvCallNum::thread_send_msg => envcall_unreachable_handler,
                EnvCallNum::thread_msg_length => envcall_unreachable_handler,
                EnvCallNum::thread_msg_read => envcall_unreachable_handler,
                EnvCallNum::thread_sleep => envcall_unreachable_handler,
                _ => envcall_unreachable_handler,
            }
        }
        0x0009 => {
            // Category: Regular expressions
            match envcall_num {
                EnvCallNum::regex_create => envcall_unreachable_handler,
                EnvCallNum::regex_capture_group_count => envcall_unreachable_handler,
                EnvCallNum::regex_capture_group_names_length => envcall_unreachable_handler,
                EnvCallNum::regex_capture_group_names_text => envcall_unreachable_handler,
                EnvCallNum::regex_match => envcall_unreachable_handler,
                EnvCallNum::regex_capture_groups => envcall_unreachable_handler,
                EnvCallNum::regex_remove => envcall_unreachable_handler,
                _ => envcall_unreachable_handler,
            }
        }
        _ => envcall_unreachable_handler,
    }
}
