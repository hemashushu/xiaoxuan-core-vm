// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::fmt::Display;

pub mod nostd_stack;
pub mod simple_stack;
pub mod stack;

// The Program Counter (PC)
// ------------------------
//
// On real hardware platforms, the program counter (PC), also known as the instruction pointer (IP),
// is a CPU register that holds the memory address of the next instruction to execute.
// A single address is sufficient because all code (including applications and shared libraries)
// resides in a unified memory space.
//
// In the XiaoXuan Core VM, however, modules are independent entities, and functions
// are accessed using their function index. Consequently, the program counter is represented
// as a tuple: `(module index, function index, instruction address)`.

// The `module_index` in `ProgramCounter` when returning from function calls
// -------------------------------------------------------------------------
//
// When returning from a function call, if the most significant bit (MSB) of `module_index`
// in the `ProgramCounter` is set to 1, it indicates that this is the first frame in a new
// "function call path." A stack can have multiple "function call paths" because each callback
// creates a new path.
//
// The following diagram illustrates the concept:
//
// ```diagram
//              stack                      external
//           |         |                   functions
// calling | | frame 1 |    callback
//  path 2 | | frame 0 | <--------------  /--------\
//           |         |                  |        |
//           |         |  call external   |  fn 1  |
//           |         | -------------->  \--------/
//         | | frame 4 |    function
//         | | frame 3 |
//         | | frame 2 |
// calling | | frame 1 |    callback
//  path 1 | | frame 0 | <--------------  /--------\
//           |         |                  |        |
//           |         |  call external   |  fn 0  |
//           |         | -------------->  \--------/
// calling | | frame 1 |    function
//  path 0 | | frame 0 |
//           \---------/ <-- stack start
// ```

/// Represents the location of the next instruction to be executed.
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
