// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_binary::utils::format_bytecodes;
use ancvm_program::thread_context::ThreadContext;
use ancvm_types::ecallcode::{ECallCode, MAX_ECALLCODE_NUMBER};

use crate::interpreter::InterpretResult;

use self::syscall::init_syscall_handlers;

pub mod callback;
pub mod extcall;
pub mod heap;
pub mod info;
pub mod syscall;

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
    let code_text = format_bytecodes(codes);

    unreachable!(
        "Invalid environment call number: 0x{:04x}
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
    // init the syscall handlers
    init_syscall_handlers();

    let handlers = unsafe { &mut HANDLERS };

    // info
    handlers[ECallCode::runtime_name as usize] = info::runtime_name;
    handlers[ECallCode::runtime_version as usize] = info::runtime_version;
    handlers[ECallCode::features as usize] = info::features;
    handlers[ECallCode::check_feature as usize] = info::check_feature;

    // heap
    handlers[ECallCode::heap_fill as usize] = heap::heap_fill;
    handlers[ECallCode::heap_copy as usize] = heap::heap_copy;
    handlers[ECallCode::heap_capacity as usize] = heap::heap_capacity;
    handlers[ECallCode::heap_resize as usize] = heap::heap_resize;

    // system
    handlers[ECallCode::syscall as usize] = syscall::syscall;
    handlers[ECallCode::extcall as usize] = extcall::extcall;
}

pub fn ecall(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param env_func_num:i32)

    let env_func_num = thread_context.get_param_i32();
    let func = unsafe { &HANDLERS[env_func_num as usize] };
    func(thread_context);
    InterpretResult::Move(8)
}
