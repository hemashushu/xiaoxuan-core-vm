// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

pub mod simple_stack;
pub mod stack;

// the stack will be enlarge when the free size of stack is less than this value
pub const INIT_STACK_SIZE_IN_BYTES: usize = 64 * 1024; // 64KB
pub const MAX_STACK_SIZE_IN_BYTES: usize = 64 * 1024 * 1024; // 64MB

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
