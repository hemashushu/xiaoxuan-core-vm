// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;
use syscall_util::call::{
    syscall_with_1_arg, syscall_with_2_args, syscall_with_3_args, syscall_with_4_args,
    syscall_with_5_args, syscall_with_6_args, syscall_without_args,
};

pub type SysCallHandlerFunc =
    fn(
        &mut ThreadContext,
        /* call_number */ usize,
    ) -> Result</* error_number */ usize, /* return value */ usize>;

/// 1 type without args + 6 types with args = total 7 types
/// pub const MAX_SYSCALL_TYPE_NUMBER: usize = 1 + 6;
pub fn get_syscall_handler(arg_count: usize) -> SysCallHandlerFunc {
    match arg_count {
        0 => handle_syscall_without_args,
        1 => handle_syscall_with_1_arg,
        2 => handle_syscall_with_2_args,
        3 => handle_syscall_with_3_args,
        4 => handle_syscall_with_4_args,
        5 => handle_syscall_with_5_args,
        6 => handle_syscall_with_6_args,
        _ => syscall_unreachable_handler,
    }
}

fn syscall_unreachable_handler(
    _thread_context: &mut ThreadContext,
    _number: usize,
) -> Result<usize, usize> {
    unreachable!()
}

fn handle_syscall_without_args(
    _thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    unsafe { syscall_without_args(number) }
}

fn handle_syscall_with_1_arg(
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 1;
    let args_ptr = thread_context.stack.pop_operands_to_memory(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_ptr as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_1_arg(number, args[0]) }
}

fn handle_syscall_with_2_args(
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 2;
    let args_ptr = thread_context.stack.pop_operands_to_memory(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_ptr as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_2_args(number, args[0], args[1]) }
}

fn handle_syscall_with_3_args(
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 3;
    let args_ptr = thread_context.stack.pop_operands_to_memory(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_ptr as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_3_args(number, args[0], args[1], args[2]) }
}

fn handle_syscall_with_4_args(
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 4;
    let args_ptr = thread_context.stack.pop_operands_to_memory(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_ptr as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_4_args(number, args[0], args[1], args[2], args[3]) }
}

fn handle_syscall_with_5_args(
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 5;
    let args_ptr = thread_context.stack.pop_operands_to_memory(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_ptr as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_5_args(number, args[0], args[1], args[2], args[3], args[4]) }
}

fn handle_syscall_with_6_args(
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 6;
    let args_ptr = thread_context.stack.pop_operands_to_memory(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_ptr as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_6_args(number, args[0], args[1], args[2], args[3], args[4], args[5]) }
}
