// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_binary::bytecode_reader::format_bytecode_as_text;
use ancvm_context::thread_context::ThreadContext;
use ancvm_isa::envcallcode::{EnvCallCode, MAX_ECALLCODE_NUMBER};

use crate::interpreter::HandleResult;

// mod initialization;
mod multithread;
mod runtime_info;
mod time;

type EnvCallHandlerFunc = fn(&mut ThreadContext);

fn unreachable(thread_context: &mut ThreadContext) {
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

static mut HANDLERS: [EnvCallHandlerFunc; MAX_ECALLCODE_NUMBER] =
    [unreachable; MAX_ECALLCODE_NUMBER];

// note:
//
// ensure this initialization is only called once
pub fn init_ecall_handlers() {
    // other initializations

    let handlers = unsafe { &mut HANDLERS };

    // runtime info
    handlers[EnvCallCode::runtime_name as usize] = runtime_info::runtime_name;
    handlers[EnvCallCode::runtime_version as usize] = runtime_info::runtime_version;

    // // initialization
    // handlers[EnvCallCode::count_start_function as usize] = initialization::count_start_function;
    // handlers[EnvCallCode::count_exit_function as usize] = initialization::count_exit_function;
    // handlers[EnvCallCode::get_start_function_item as usize] =
    //     initialization::get_start_function_item;
    // handlers[EnvCallCode::get_exit_function_item as usize] = initialization::get_exit_function_item;

    // multiple thread
    handlers[EnvCallCode::thread_id as usize] = multithread::thread_id;
    handlers[EnvCallCode::thread_start_data_length as usize] =
        multithread::thread_start_data_length;
    handlers[EnvCallCode::thread_start_data_read as usize] = multithread::thread_start_data_read;
    handlers[EnvCallCode::thread_create as usize] = multithread::thread_create;
    handlers[EnvCallCode::thread_wait_and_collect as usize] = multithread::thread_wait_and_collect;
    handlers[EnvCallCode::thread_running_status as usize] = multithread::thread_running_status;
    handlers[EnvCallCode::thread_terminate as usize] = multithread::thread_terminate;
    handlers[EnvCallCode::thread_receive_msg_from_parent as usize] =
        multithread::thread_receive_msg_from_parent;
    handlers[EnvCallCode::thread_send_msg_to_parent as usize] =
        multithread::thread_send_msg_to_parent;
    handlers[EnvCallCode::thread_receive_msg as usize] = multithread::thread_receive_msg;
    handlers[EnvCallCode::thread_send_msg as usize] = multithread::thread_send_msg;
    handlers[EnvCallCode::thread_msg_length as usize] = multithread::thread_msg_length;
    handlers[EnvCallCode::thread_msg_read as usize] = multithread::thread_msg_read;
    handlers[EnvCallCode::thread_sleep as usize] = multithread::time_sleep;

    // time
    handlers[EnvCallCode::time_now as usize] = time::time_now;
}

pub fn envcall(thread_context: &mut ThreadContext) -> HandleResult {
    // (param env_func_num:i32)

    let env_func_num = thread_context.get_param_i32();
    let func = unsafe { &HANDLERS[env_func_num as usize] };
    func(thread_context);
    HandleResult::Move(8)
}
