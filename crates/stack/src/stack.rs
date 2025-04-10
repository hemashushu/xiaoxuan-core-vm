// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_memory::MemoryError;

use crate::{FrameType, ProgramCounter, StackError};

pub trait Stack {
    fn push_i64_s(&mut self, value: i64);
    fn push_i64_u(&mut self, value: u64);
    fn push_i32_s(&mut self, value: i32);
    fn push_i32_u(&mut self, value: u32);
    fn push_f64(&mut self, value: f64);
    fn push_f32(&mut self, value: f32);

    // fast write operands from memory to stack.
    //
    // notes:
    // - this function does not interpret the data type of the data.
    // - the caller should write data to the pointer immediately after calling this function.
    //
    // e.g.
    //
    // ```rust
    // let ptr = stack.prepare_pushing_operand_from_memory();
    // some_memory.load_64(address, ptr);
    // ```
    //
    fn prepare_pushing_operand_from_memory(&mut self) -> *mut u8;
    fn prepare_pushing_operands_from_memory(&mut self, count: usize) -> *mut u8;

    fn peek_i64_s(&self) -> i64;
    fn peek_i64_u(&self) -> u64;
    fn peek_i32_s(&self) -> i32;
    fn peek_i32_u(&self) -> u32;
    fn peek_f64(&self) -> Result<f64, MemoryError>;
    fn peek_f32(&self) -> Result<f32, MemoryError>;

    fn pop_i64_s(&mut self) -> i64;
    fn pop_i64_u(&mut self) -> u64;
    fn pop_i32_s(&mut self) -> i32;
    fn pop_i32_u(&mut self) -> u32;
    fn pop_f64(&mut self) -> Result<f64, MemoryError>;
    fn pop_f32(&mut self) -> Result<f32, MemoryError>;

    // fast read operands from stack to memory.
    //
    // notes:
    // - this function does not interpret the data type of the data.
    // - the caller should read data from the pointer immediately after calling this function.
    //
    // e.g.
    //
    // ```rust
    // let ptr = stack.prepare_popping_operand_to_memory();
    // some_memory.store_64(ptr, address);
    // ```
    fn prepare_popping_operand_to_memory(&mut self) -> *const u8;
    fn prepare_popping_operands_to_memory(&mut self, count: usize) -> *const u8;

    fn create_frame(
        &mut self,
        params_count: u16,
        results_count: u16,
        local_variable_list_index: u32,
        local_variables_allocate_bytes: u32,

        // pass None if creating block frame.
        optional_return_pc: Option<ProgramCounter>,
    ) -> Result<(), StackError>;

    fn remove_frames(&mut self, reversed_index: u16) -> Option<ProgramCounter>;

    fn reset_frames(&mut self, reversed_index: u16) -> FrameType;
}
