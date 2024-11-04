// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::thread_context::ThreadContext;
use syscall_util::call::{
    syscall_with_1_arg, syscall_with_2_args, syscall_with_3_args, syscall_with_4_args,
    syscall_with_5_args, syscall_with_6_args, syscall_without_args,
};

use crate::handler::Handler;

pub type SysCallHandlerFunc = fn(
    &Handler,
    &mut ThreadContext,
    /* call_number */ usize,
) -> Result</* err_no */ usize, /* ret value */ usize>;

// 1 type no args + 6 types with args = 7 types
pub const MAX_SYSCALL_TYPE_NUMBER: usize = 1 + 6;

pub fn generate_syscall_handlers() -> [SysCallHandlerFunc; MAX_SYSCALL_TYPE_NUMBER] {
    let mut handlers: [SysCallHandlerFunc; MAX_SYSCALL_TYPE_NUMBER] =
        [syscall_unreachable_handler; MAX_SYSCALL_TYPE_NUMBER];
    handlers[0] = handle_syscall_without_args;
    handlers[1] = handle_syscall_with_1_arg;
    handlers[2] = handle_syscall_with_2_args;
    handlers[3] = handle_syscall_with_3_args;
    handlers[4] = handle_syscall_with_4_args;
    handlers[5] = handle_syscall_with_5_args;
    handlers[6] = handle_syscall_with_6_args;
    handlers
}

fn syscall_unreachable_handler(
    _handler: &Handler,
    _thread_context: &mut ThreadContext,
    _number: usize,
) -> Result<usize, usize> {
    unreachable!()
}

fn handle_syscall_without_args(
    _handler: &Handler,
    _thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    unsafe { syscall_without_args(number) }
}

fn handle_syscall_with_1_arg(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 1;
    let args_u8 = thread_context.stack.pop_operands(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_u8.as_ptr() as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_1_arg(number, args[0]) }
}

fn handle_syscall_with_2_args(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 2;
    let args_u8 = thread_context.stack.pop_operands(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_u8.as_ptr() as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_2_args(number, args[0], args[1]) }
}

fn handle_syscall_with_3_args(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 3;
    let args_u8 = thread_context.stack.pop_operands(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_u8.as_ptr() as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_3_args(number, args[0], args[1], args[2]) }
}

fn handle_syscall_with_4_args(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 4;
    let args_u8 = thread_context.stack.pop_operands(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_u8.as_ptr() as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_4_args(number, args[0], args[1], args[2], args[3]) }
}

fn handle_syscall_with_5_args(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 5;
    let args_u8 = thread_context.stack.pop_operands(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_u8.as_ptr() as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_5_args(number, args[0], args[1], args[2], args[3], args[4]) }
}

fn handle_syscall_with_6_args(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 6;
    let args_u8 = thread_context.stack.pop_operands(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_u8.as_ptr() as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_6_args(number, args[0], args[1], args[2], args[3], args[4], args[5]) }
}
