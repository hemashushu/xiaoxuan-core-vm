// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

// mod env;
// mod host;
// mod multithread;
// mod random;
// mod regex;
mod runtime;
// mod timer;

use anc_context::thread_context::ThreadContext;
use anc_image::bytecode_reader::format_bytecode_as_text;

use crate::{
    envcall_num::{EnvCallNum, MAX_ENVCALL_CODE_NUMBER},
    handler::Handler,
};

pub type EnvCallHandlerFunc = fn(&Handler, &mut ThreadContext);

fn envcall_unreachable_handler(_handler: &Handler, thread_context: &mut ThreadContext) {
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

pub fn generate_envcall_handlers() -> [EnvCallHandlerFunc; MAX_ENVCALL_CODE_NUMBER] {
    let mut handlers: [EnvCallHandlerFunc; MAX_ENVCALL_CODE_NUMBER] =
        [envcall_unreachable_handler; MAX_ENVCALL_CODE_NUMBER];

    // runtime info
    handlers[EnvCallNum::runtime_edition as usize] = runtime::runtime_edition;
    handlers[EnvCallNum::runtime_version as usize] = runtime::runtime_version;

//     // timer
//     handlers[EnvCallNum::time_now as usize] = timer::time_now;
//
//     // multiple thread
//     handlers[EnvCallNum::thread_id as usize] = multithread::thread_id;
//     handlers[EnvCallNum::thread_start_data_length as usize] = multithread::thread_start_data_length;
//     handlers[EnvCallNum::thread_start_data_read as usize] = multithread::thread_start_data_read;
//     handlers[EnvCallNum::thread_create as usize] = multithread::thread_create;
//     handlers[EnvCallNum::thread_wait_and_collect as usize] = multithread::thread_wait_and_collect;
//     handlers[EnvCallNum::thread_running_status as usize] = multithread::thread_running_status;
//     handlers[EnvCallNum::thread_terminate as usize] = multithread::thread_terminate;
//     handlers[EnvCallNum::thread_receive_msg_from_parent as usize] =
//         multithread::thread_receive_msg_from_parent;
//     handlers[EnvCallNum::thread_send_msg_to_parent as usize] =
//         multithread::thread_send_msg_to_parent;
//     handlers[EnvCallNum::thread_receive_msg as usize] = multithread::thread_receive_msg;
//     handlers[EnvCallNum::thread_send_msg as usize] = multithread::thread_send_msg;
//     handlers[EnvCallNum::thread_msg_length as usize] = multithread::thread_msg_length;
//     handlers[EnvCallNum::thread_msg_read as usize] = multithread::thread_msg_read;
//     handlers[EnvCallNum::thread_sleep as usize] = multithread::thread_sleep;

    handlers
}
