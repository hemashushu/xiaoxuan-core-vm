// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

// XiaoXuan Core Calling Convention
// --------------------------------
// The following diagrams illustrate how the stack changes when entering and exiting a function or block.
//
// 1. Function 1 is preparing to call Function 2 by setting up the arguments.
//
// ```diagram
// |         |
// |         |
// |  arg 1  | <-- Operands of Function 1, used as arguments for Function 2.
// |  arg 0  |
// |   ###   | <-- Other operands of Function 1.
// |   ###   |
// |---------| <-- Stack frame pointer for Function 1 (current FP).
// |   ...   |
// \---------/ <-- Stack start.
// ```
//
// 2. Function 2 is called.
//
// ```diagram
// |         |
// | local 1 |
// | local 0 | <-- Local variable slots allocated for Function 2.
// |  arg 1  | <-- Arguments from Function 1 are moved here.
// |  arg 0  |
// |---------|
// |   ...   | <-- Stack frame information, including the previous FP, return PC,
// |   ...   |     and function metadata (e.g., type, index).
// |   ...   |
// |---------| <-- Stack frame pointer for Function 2 (current FP).
// |   ###   |
// |   ###   | <-- Remaining operands of Function 1.
// |---------| <-- Stack frame pointer for Function 1.
// |   ...   |
// \---------/ <-- Stack start.
// ```
//
// 3. Function 2 finishes and prepares to return to Function 1.
//
// ```diagram
// |         |
// | resul 1 |
// | resul 0 | <-- Results of Function 2.
// |   ###   | <-- Remaining operands of Function 2.
// |   ###   |
// |---------|
// | local 1 |
// | local 0 |
// |  arg 1  |
// |  arg 0  |
// |---------|
// |   ...   |
// |   ...   |
// |   ...   |
// |---------| <-- Stack frame pointer for Function 2 (current FP).
// |   ###   |
// |   ###   | <-- Remaining operands of Function 1.
// |---------| <-- Stack frame pointer for Function 1.
// |   ...   |
// \---------/ <-- Stack start.
// ```
//
// 4. Returning to Function 1.
//
// ```diagram
// |         |
// | resul 1 | <-- Frame of Function 2 is removed,
// | resul 0 |     and results of Function 2 are moved here.
// |   ###   |
// |   ###   | <-- Remaining operands of Function 1.
// |---------| <-- Stack frame pointer for Function 1 (current FP).
// |   ...   |
// \---------/ <-- Stack start.
// ```

// Returning Multiple Values
// -------------------------
// On most architectures, only one or two values can be returned by a function (e.g.,
// registers `rax/rdx` on x86_64, `x0/x1` on AArch64, `a0/a1` on RISC-V). However, the XiaoXuan Core VM
// allows functions to return multiple values.
//
// When interoperating with other programs (e.g., written in C), it is recommended to limit
// function returns to a single value for compatibility.

use anc_memory::{memory_access::MemoryAccess, MemoryError};

use crate::{FrameType, ProgramCounter, StackError};

pub trait Stack: MemoryAccess {
    fn push_i64_s(&mut self, value: i64);
    fn push_i64_u(&mut self, value: u64);
    fn push_i32_s(&mut self, value: i32);
    fn push_i32_u(&mut self, value: u32);
    fn push_f64(&mut self, value: f64);
    fn push_f32(&mut self, value: f32);

    // Prepares the stack to write a single operand from memory.
    //
    // Notes:
    // - This function does not interpret the data type of the operand.
    // - The caller must write data to the returned pointer immediately after calling this function.
    //
    // Example:
    // ```rust
    // let ptr = stack.prepare_pushing_operand_from_memory();
    // some_memory.load_64(address, ptr);
    // ```
    fn prepare_pushing_operand_from_memory(&mut self) -> *mut u8;

    // Prepares the stack to write multiple operands from memory.
    //
    // Notes:
    // - This function does not interpret the data type of the operands.
    // - The caller must write data to the returned pointer immediately after calling this function.
    //
    // Example:
    // ```rust
    // let ptr = stack.prepare_pushing_operands_from_memory(count);
    // some_memory.load_multiple(address, ptr, count);
    // ```
    fn prepare_pushing_operands_from_memory(&mut self, count: usize) -> *mut u8;

    fn peek_i64_s(&self) -> i64;
    fn peek_i64_u(&self) -> u64;
    fn peek_i32_s(&self) -> i32;
    fn peek_i32_u(&self) -> u32;

    // Retrieves the top f64 value from the stack.
    // Returns an error if the operation fails due to memory issues.
    fn peek_f64(&self) -> Result<f64, MemoryError>;

    // Retrieves the top f32 value from the stack.
    // Returns an error if the operation fails due to memory issues.
    fn peek_f32(&self) -> Result<f32, MemoryError>;

    fn pop_i64_s(&mut self) -> i64;
    fn pop_i64_u(&mut self) -> u64;
    fn pop_i32_s(&mut self) -> i32;
    fn pop_i32_u(&mut self) -> u32;

    // Removes and returns the top f64 value from the stack.
    // Returns an error if the operation fails due to memory issues.
    fn pop_f64(&mut self) -> Result<f64, MemoryError>;

    // Removes and returns the top f32 value from the stack.
    // Returns an error if the operation fails due to memory issues.
    fn pop_f32(&mut self) -> Result<f32, MemoryError>;

    // Prepares the stack to read a single operand to memory.
    //
    // Notes:
    // - This function does not interpret the data type of the operand.
    // - The caller must read data from the returned pointer immediately after calling this function.
    //
    // Example:
    // ```rust
    // let ptr = stack.prepare_popping_operand_to_memory();
    // some_memory.store_64(ptr, address);
    // ```
    fn prepare_popping_operand_to_memory(&mut self) -> *const u8;

    // Prepares the stack to read multiple operands to memory.
    //
    // Notes:
    // - This function does not interpret the data type of the operands.
    // - The caller must read data from the returned pointer immediately after calling this function.
    //
    // Example:
    // ```rust
    // let ptr = stack.prepare_popping_operands_to_memory(count);
    // some_memory.store_multiple(ptr, address, count);
    // ```
    fn prepare_popping_operands_to_memory(&mut self, count: usize) -> *const u8;

    // Creates a new stack frame.
    //
    // Parameters:
    // - `params_count`: The number of parameters for the frame.
    // - `results_count`: The number of results for the frame.
    // - `local_variable_list_index`: The index of the local variable list.
    // - `local_variables_with_arguments_allocated_bytes`: The number of bytes to allocate for local variables (includes arguments).
    // - `optional_return_pc`: The return program counter, or `None` if creating a block frame.
    //
    // Returns:
    // - `Ok(())` if the frame is successfully created.
    // - `Err(StackError)` if an error occurs.
    fn create_frame(
        &mut self,
        params_count: u16,
        results_count: u16,
        local_variable_list_index: u32,
        local_variables_with_arguments_allocated_bytes: u32,
        optional_return_pc: Option<ProgramCounter>,
    ) -> Result<(), StackError>;

    // Removes frames from the stack up to the specified reversed index.
    //
    // Parameters:
    // - `reversed_index`: The reversed index of the frame to remove up to.
    //
    // Returns:
    // - The program counter of the frame that was removed, if any.
    fn remove_frames(&mut self, reversed_index: u16) -> Option<ProgramCounter>;

    // Resets frames on the stack up to the specified reversed index.
    //
    // Parameters:
    // - `reversed_index`: The reversed index of the frame to reset up to.
    //
    // Returns:
    // - The type of the frame that was reset.
    fn reset_frames(&mut self, reversed_index: u16) -> FrameType;

    fn get_frame_local_variable_list_index_and_start_address_by_reversed_index(
        &self,
        reversed_index: u16,
    ) -> (usize, usize);

    fn push_first_operands(&mut self, data: &[u8]);

    /// Pops the specified number of operands from the stack without boundary checks.
    ///
    /// This method is used when the stack is empty or when boundary checks are not required,
    /// such as returning from the "entry" function of application.
    fn pop_last_operands(&mut self, count: usize) -> &[u8];
}
