// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// mod multithread;
mod runtime_info;
mod timer;

use ancvm_context::thread_context::ThreadContext;
use ancvm_image::bytecode_reader::format_bytecode_as_text;

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
        "Invalid EnvCall number: 0x{:04x}
Module index: {}
Function index: {}
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
    handlers[EnvCallNum::runtime_name as usize] = runtime_info::runtime_name;
    handlers[EnvCallNum::runtime_version as usize] = runtime_info::runtime_version;

    // timer
    handlers[EnvCallNum::time_now as usize] = timer::time_now;

    //     // multiple thread
    //     handlers[EnvCallCode::thread_id as usize] = multithread::thread_id;
    //     handlers[EnvCallCode::thread_start_data_length as usize] =
    //         multithread::thread_start_data_length;
    //     handlers[EnvCallCode::thread_start_data_read as usize] = multithread::thread_start_data_read;
    //     handlers[EnvCallCode::thread_create as usize] = multithread::thread_create;
    //     handlers[EnvCallCode::thread_wait_and_collect as usize] = multithread::thread_wait_and_collect;
    //     handlers[EnvCallCode::thread_running_status as usize] = multithread::thread_running_status;
    //     handlers[EnvCallCode::thread_terminate as usize] = multithread::thread_terminate;
    //     handlers[EnvCallCode::thread_receive_msg_from_parent as usize] =
    //         multithread::thread_receive_msg_from_parent;
    //     handlers[EnvCallCode::thread_send_msg_to_parent as usize] =
    //         multithread::thread_send_msg_to_parent;
    //     handlers[EnvCallCode::thread_receive_msg as usize] = multithread::thread_receive_msg;
    //     handlers[EnvCallCode::thread_send_msg as usize] = multithread::thread_send_msg;
    //     handlers[EnvCallCode::thread_msg_length as usize] = multithread::thread_msg_length;
    //     handlers[EnvCallCode::thread_msg_read as usize] = multithread::thread_msg_read;
    //     handlers[EnvCallCode::thread_sleep as usize] = multithread::time_sleep;

    handlers
}
