// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::fmt::Display;

pub mod simple_stack;
pub mod stack;

// The initial size of the stack in bytes.
// The stack will automatically grow when the available free space
// is less than half of the current stack size.
pub const INIT_STACK_SIZE_IN_BYTES: usize = 64 * 1024; // 64KB

// The maximum size of the stack in bytes.
// This value matches the maximum stack size for Linux x86_64 systems.
pub const MAX_STACK_SIZE_IN_BYTES: usize = 8 * 1024 * 1024; // 8MB

/// Represents the location of the next instruction to be executed.
///
/// On real hardware platforms, the program counter (PC) or instruction pointer (IP)
/// is a CPU register that holds the address of the next instruction to execute.
/// A single address is sufficient since all code (including applications and shared libraries)
/// resides in a unified memory space.
///
/// In the XiaoXuan Core VM, however, modules are independent objects, and functions
/// are accessed using their function index. As a result, the program counter is represented
/// as a tuple: `(module index, function index, instruction address)`.
#[derive(Debug, PartialEq)]
pub struct ProgramCounter {
    // The offset of the next instruction to be executed within the "FunctionSection".
    pub instruction_address: usize,

    // The index of the module where the function is defined.
    pub module_index: usize,

    // The internal index of the next function to be executed.
    // This is redundant because the instruction address can be used to
    // calculate the function index, but it is retained for debugging purposes.
    pub function_internal_index: usize,
}

/// Represents the type of a frame in the stack.
#[derive(Debug, PartialEq)]
pub enum FrameType {
    /// A stack frame for a function.
    Function,

    /// A stack frame for a block.
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
