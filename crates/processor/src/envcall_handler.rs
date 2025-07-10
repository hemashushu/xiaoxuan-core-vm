// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

mod environment;
mod host;
mod multithread;
mod random;
mod regex;
mod runtime;
mod time;

use anc_context::thread_context::ThreadContext;
use anc_image::bytecode_reader::format_bytecode_as_text;

use crate::envcall_num::EnvCallNum;

pub type EnvCallHandlerFunc = fn(&mut ThreadContext);

fn envcall_unreachable_handler(thread_context: &mut ThreadContext) {
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
                EnvCallNum::program_path_length => environment::program_path_length,
                EnvCallNum::program_path_read => environment::program_path_read,
                EnvCallNum::program_source_type => environment::program_source_type,
                EnvCallNum::arguments_length => environment::arguments_length,
                EnvCallNum::arguments_read => environment::arguments_read,
                EnvCallNum::environment_variables_length => {
                    environment::environment_variables_length
                }
                EnvCallNum::environment_variables_read => environment::environment_variables_read,
                EnvCallNum::environment_variable_find => environment::environment_variable_find,
                EnvCallNum::environment_variable_item_length => {
                    environment::environment_variable_item_length
                }
                EnvCallNum::environment_variable_item_read => {
                    environment::environment_variable_item_read
                }
                EnvCallNum::environment_variable_set => environment::environment_variable_set,
                EnvCallNum::environment_variable_remove => environment::environment_variable_remove,
                _ => envcall_unreachable_handler,
            }
        }
        0x0004 => {
            // Category: Timer
            match envcall_num {
                EnvCallNum::time_now => time::time_now,
                _ => envcall_unreachable_handler,
            }
        }
        0x0005 => {
            // Category: Random number generation
            match envcall_num {
                EnvCallNum::random_i32 => random::random_i32,
                EnvCallNum::random_i64 => random::random_i64,
                EnvCallNum::random_f32 => random::random_f32,
                EnvCallNum::random_f64 => random::random_f64,
                EnvCallNum::random_range_i32 => random::random_range_i32,
                EnvCallNum::random_range_i64 => random::random_range_i64,
                EnvCallNum::random_range_f32 => random::random_range_f32,
                EnvCallNum::random_range_f64 => random::random_range_f64,
                EnvCallNum::random_fill => random::random_fill,
                _ => envcall_unreachable_handler,
            }
        }
        0x0006 => {
            // Category: Regular expressions
            match envcall_num {
                EnvCallNum::regex_create => regex::regex_create,
                EnvCallNum::regex_capture_group_count => regex::regex_capture_group_count,
                EnvCallNum::regex_capture_group_names_length => {
                    regex::regex_capture_group_names_length
                }
                EnvCallNum::regex_capture_group_names_read => regex::regex_capture_group_names_read,
                EnvCallNum::regex_match => regex::regex_match,
                EnvCallNum::regex_last_captures_read => regex::regex_last_captures_read,
                EnvCallNum::regex_remove => regex::regex_remove,
                _ => envcall_unreachable_handler,
            }
        }
        0x0007 => {
            // Category: Thread
            match envcall_num {
                EnvCallNum::thread_id => multithread::thread_id,
                EnvCallNum::thread_create => multithread::thread_create,
                EnvCallNum::thread_start_data_length => multithread::thread_start_data_length,
                EnvCallNum::thread_start_data_read => multithread::thread_start_data_read,
                EnvCallNum::thread_wait_and_collect => multithread::thread_wait_and_collect,
                EnvCallNum::thread_running_status => multithread::thread_running_status,
                EnvCallNum::thread_terminate => multithread::thread_terminate,
                EnvCallNum::thread_receive_msg_from_parent => {
                    multithread::thread_receive_msg_from_parent
                }
                EnvCallNum::thread_send_msg_to_parent => multithread::thread_send_msg_to_parent,
                EnvCallNum::thread_receive_msg => multithread::thread_receive_msg,
                EnvCallNum::thread_send_msg => multithread::thread_send_msg,
                EnvCallNum::thread_msg_length => multithread::thread_msg_length,
                EnvCallNum::thread_msg_read => multithread::thread_msg_read,
                EnvCallNum::thread_sleep => multithread::thread_sleep,
                _ => envcall_unreachable_handler,
            }
        }
        0x0008 => {
            // Category: Capabilities
            match envcall_num {
                EnvCallNum::capable_syscall => envcall_unreachable_handler,
                EnvCallNum::capable_extcall => envcall_unreachable_handler,
                EnvCallNum::capable_shell_execute => envcall_unreachable_handler,
                EnvCallNum::capable_shell_execute_all => envcall_unreachable_handler,
                EnvCallNum::capable_file_execute => envcall_unreachable_handler,
                EnvCallNum::capable_file_execute_all => envcall_unreachable_handler,
                EnvCallNum::capable_personal_dir_access => envcall_unreachable_handler,
                EnvCallNum::capable_application_dir_access => envcall_unreachable_handler,
                EnvCallNum::capable_file_access => envcall_unreachable_handler,
                EnvCallNum::capable_file_access_all => envcall_unreachable_handler,
                _ => envcall_unreachable_handler,
            }
        }
        0x0009 => {
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
        0x000A => {
            // Category: File system
            match envcall_num {
                EnvCallNum::fs_get_current_dir => envcall_unreachable_handler,
                EnvCallNum::fs_get_temporary_dir => envcall_unreachable_handler,
                EnvCallNum::fs_get_application_dir => envcall_unreachable_handler,
                EnvCallNum::fs_list_dir => envcall_unreachable_handler,
                EnvCallNum::fs_list_dir_read => envcall_unreachable_handler,
                EnvCallNum::fs_create_dir => envcall_unreachable_handler,
                EnvCallNum::fs_remove_dir => envcall_unreachable_handler,
                EnvCallNum::fs_remove_file => envcall_unreachable_handler,
                EnvCallNum::fs_rename => envcall_unreachable_handler,
                EnvCallNum::fs_exists => envcall_unreachable_handler,
                _ => envcall_unreachable_handler,
            }
        }
        _ => envcall_unreachable_handler,
    }
}
