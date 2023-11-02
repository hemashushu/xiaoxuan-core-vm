// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_binary::bytecode_reader::print_bytecode_as_text;
use ancvm_program::thread_context::ThreadContext;
use ancvm_types::envcallcode::{EnvCallCode, MAX_ECALLCODE_NUMBER};

use crate::interpreter::InterpretResult;

pub mod multithread;
pub mod runtime_info;

type EnvCallHandlerFunc = fn(&mut ThreadContext);

fn unreachable(thread_context: &mut ThreadContext) {
    let pc = &thread_context.pc;
    let func_item = &thread_context.program_context.program_modules[pc.module_index]
        .func_section
        .items[pc.function_internal_index];
    let codes = &thread_context.program_context.program_modules[pc.module_index]
        .func_section
        .codes_data
        [func_item.code_offset as usize..(func_item.code_offset + func_item.code_length) as usize];
    let code_text = print_bytecode_as_text(codes);

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

    // multiple thread
    handlers[EnvCallCode::thread_id as usize] = multithread::thread_id;
    handlers[EnvCallCode::thread_start_data_read as usize] = multithread::thread_start_data_read;
    handlers[EnvCallCode::thread_create as usize] = multithread::thread_create;
    handlers[EnvCallCode::thread_wait_for_finish as usize] = multithread::thread_wait_for_finish;
}

pub fn envcall(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param env_func_num:i32)

    let env_func_num = thread_context.get_param_i32();
    let func = unsafe { &HANDLERS[env_func_num as usize] };
    func(thread_context);
    InterpretResult::Move(8)
}
