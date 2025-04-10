// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::fmt::Display;

pub mod simple_stack;
pub mod stack;

// the initial size of the stack.
// note that the stack will be enlarge when the free size is less than
// the half of the stack size.
pub const INIT_STACK_SIZE_IN_BYTES: usize = 64 * 1024; // 64KB

// the maximum size of the stack. it's identical to the maximum size of the
// Linux x86_64 stack size.
pub const MAX_STACK_SIZE_IN_BYTES: usize = 8 * 1024 * 1024; // 8MB

/// The location of next instruction to be executed.
///
/// On the real hardware platform, the PC (program counter) or IP (instruction pointer) is
/// a register in CPU that contains the address of the next instruction to be executed.
/// a single address is sufficient since all the code (includes application and shared libraries)
/// are loaded into one memory space.
///
/// However, in XiaoXuan Core VM, modules are individual objects and functions are accessed
/// using the function index. The PC is not a single number,
/// but a tuple of `(module index, function index, instruction address)`.
#[derive(Debug, PartialEq)]
pub struct ProgramCounter {
    // address of the next instruction to be executed.
    // it's the offset of all functions code area in the "FunctionSection".
    pub instruction_address: usize,

    // the index of module where the function is defined.
    pub module_index: usize,

    // internal index of the next function to be executed.
    // it's redundant since the instruction address can be used to
    // calculate the function index, but it's kept here for debugging.
    pub function_internal_index: usize,
}

/// The type of the frame in the stack.
#[derive(Debug, PartialEq)]
pub enum FrameType {
    /// Function stack frame.
    Function,

    /// Block stack frame.
    Block,
}

#[derive(Debug)]
pub enum StackErrorType {
    StackOverflow,
}

#[derive(Debug)]
pub struct StackError {
    pub error_type: StackErrorType,
}

impl StackError {
    pub fn new(error_type: StackErrorType) -> Self {
        StackError { error_type }
    }
}

impl Display for StackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.error_type {
            StackErrorType::StackOverflow => write!(f, "Insufficient stack space."),
        }
    }
}

impl std::error::Error for StackError {}
