// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

// A Rust `no_std` stack implementation
// ------------------------------------
//
// This stack consists of contiguous stack frames. Each frame contains
// frame information, local variables (include arguments), and operands.
//
// Layout of a stack frame:
//
// ```diagram
// | ...                     |                           | ...                     |
// | operand N               |                           | operand N               |
// | operand 1               |                           | operand 1               |
// | operand 0               | <-- operands              | operand 0               |
// |-------------------------|                           |-------------------------|
// | local var 1 (idx 3)     |                           | local var 1 (idx 3)     |
// | local var 0 (idx 2)     | <-- local variables       | local var 0 (idx 2)     |
// | arg 1 (local var idx 1) |                           | arg 1 (local var idx 1) |
// | arg 0 (local var idx 0) | <-- args from caller      | arg 0 (local var idx 0) |
// |-------------------------|                           |-------------------------|
// | return instruction address        |                           | 0                       | <-- 0
// | return function internal idx         |                           | 0                       | <-- 0
// | return module idx       |                           | 0                       | <-- 0
// | local vars alloc bytes  | <-- frame info            | local vars alloc bytes  |
// | local vars list index   |                           | local vars list index   |
// | params/results count    |                           | params/results count    |
// | function FP             |                           | function FP             |
// | previous FP             |                           | previous FP             |
// |=========================| <-- frame start (FP)      |=========================|
// | ...                     |                           | ...                     |
// | ...                     |                           | ...                     |
// \-------------------------/ <-- stack start           \-------------------------/
//       function frame                                          block frame
// ```

// The `previous_frame_address` in `FrameInfo`
// -------------------------------------------
//
// Since frame lengths are variable, each stack frame contains a `previous_frame_address` pointer.
// These pointers form a singly linked list.
//
// Frame chain:
//
//             |             |
//             |-------------| <----------------------- stack.SP
//             | ...         |
//             | func FP     | -----------------\
//  func frame | previous FP | ---\             |
//             |-------------|    |     <-------/ <---- stack.FP
//             |             |    |
//             |             |    | <-- crossing functions
//             | ...         |    |
//             | func FP     | ---|-------------\
// block frame | previous FP | ---|--\          | All "func FP" of blocks point to the
//             |-------------| <--/  |          | start position of the current function frame (FP).
//             | ...         |       |          |
//             | func FP     | ------|--------->\
// block frame | previous FP | ---\  |          |
//             |-------------| <--|--/          |
//             | ...         |    |             |
//             | func FP     | ---|------------>\
// block frame | previous FP | ---|--\          |
//             |-------------| <--/  |          |
//             | ...         |       |          |
//             | func FP     | ------|--------->\ The value of "func FP" of the current function frame
//  func frame | previous FP | ---\  |          | is the FP of the frame itself.
//             |-------------| <--|--/  <-------/
//             |             |    |
//             | ...         |   ...
//             |             |
//             \-------------/ <-- stack start

use std::mem::size_of;

use anc_isa::OPERAND_SIZE_IN_BYTES;
use anc_memory::{
    memory_access::MemoryAccess, primitive_memory_access::PrimitiveMemoryAccess, MemoryError,
};

use crate::{
    stack::{CallingStack, LocalVariablesStack, OperandStack, Stack},
    FrameType, ProgramCounter, StackError,
};

// The size of the swap area in bytes.
const SWAP_SIZE_IN_BYTES: usize = 32 * 8; // length of 32 operands

// The total size of the stack in bytes.
const STACK_SIZE_IN_BYTES: usize = 16 * 1024; // 16KB

pub struct NostdStack {
    // The stack data is stored in a contiguous memory area.
    // The stack pointer (SP) points to the end of the stack,
    // while the frame pointer (FP) points to the start of the current frame.
    data: [u8; STACK_SIZE_IN_BYTES],

    // The end position of the stack (stack pointer).
    pub sp: usize,

    // The start position of the current frame (frame pointer).
    pub fp: usize,

    // A temporary memory area used for swapping operands.
    //
    // When a new stack frame is created:
    // 1. Move the arguments (operands at the top of the stack) from the stack to the swap area.
    // 2. Create the new frame (this includes creating frame metadata and allocating space for local variables).
    // 3. Restore the arguments from the swap area back to the stack.
    //
    // When exiting a stack frame:
    // 1. Move the results (operands at the top of the stack) from the stack to the swap area.
    // 2. Remove the stack frame and all operands that follow it.
    // 3. Restore the results from the swap area back to the stack.
    swap: [u8; SWAP_SIZE_IN_BYTES],
}

impl Default for NostdStack {
    fn default() -> Self {
        Self::new()
    }
}

/// Data structure insided a stack frame
#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct FrameInfoData {
    pub previous_frame_address: u32, // <-- Address low
    pub function_frame_address: u32,
    pub params_count: u16,
    pub results_count: u16,
    pub local_variable_list_index: u32,
    pub local_variables_with_arguments_allocated_bytes: u32,
    pub return_module_index: u32,
    pub return_function_internal_index: u32,
    pub return_instruction_address: u32, // <-- Address high
}

#[derive(Debug, PartialEq)]
pub struct FrameInfo<'a> {
    pub address: usize,
    pub info_data: &'a FrameInfoData,
}

impl<'a> FrameInfo<'a> {
    pub fn new(address: usize, info_data: &'a FrameInfoData) -> Self {
        Self { address, info_data }
    }

    pub fn get_frame_type(&self) -> FrameType {
        if self.info_data.function_frame_address as usize == self.address {
            FrameType::Function
        } else {
            FrameType::Block
        }
    }
}

impl MemoryAccess for NostdStack {
    #[inline]
    fn get_ptr(&self, address: usize, offset_in_bytes: usize) -> *const u8 {
        unsafe { self.data[address..].as_ptr().add(offset_in_bytes) }
    }

    #[inline]
    fn get_mut_ptr(&mut self, address: usize, offset_in_bytes: usize) -> *mut u8 {
        unsafe { self.data[address..].as_mut_ptr().add(offset_in_bytes) }
    }
}

impl LocalVariablesStack for NostdStack {
    //
}

impl PrimitiveMemoryAccess for NostdStack {
    //
}

impl OperandStack for NostdStack {
    fn push_i64_s(&mut self, value: i64) {
        self.write_primitive_i64_s(self.sp, 0, value);
        self.sp += OPERAND_SIZE_IN_BYTES;
    }

    fn push_i64_u(&mut self, value: u64) {
        self.write_primitive_i64_u(self.sp, 0, value);
        self.sp += OPERAND_SIZE_IN_BYTES;
    }

    fn push_i32_s(&mut self, value: i32) {
        // sign-extend i32 to i64
        self.write_primitive_i64_s(self.sp, 0, value as i64);
        self.sp += OPERAND_SIZE_IN_BYTES;
    }

    fn push_i32_u(&mut self, value: u32) {
        // zero-extend u32 to u64
        self.write_primitive_i64_u(self.sp, 0, value as u64);
        self.sp += OPERAND_SIZE_IN_BYTES;
    }

    fn push_f64(&mut self, value: f64) {
        self.write_primitive_f64(self.sp, 0, value);
        self.sp += OPERAND_SIZE_IN_BYTES;
    }

    fn push_f32(&mut self, value: f32) {
        self.write_primitive_f32(self.sp, 0, value);
        self.sp += OPERAND_SIZE_IN_BYTES;
    }

    fn prepare_pushing_operand_from_memory(&mut self) -> *mut u8 {
        let ptr = self.get_mut_ptr(self.sp, 0);
        self.sp += OPERAND_SIZE_IN_BYTES;
        ptr
    }

    fn prepare_pushing_operands_from_memory(&mut self, count: usize) -> *mut u8 {
        let ptr = self.get_mut_ptr(self.sp, 0);
        self.sp += OPERAND_SIZE_IN_BYTES * count;
        ptr
    }

    fn peek_i64_s(&self) -> i64 {
        self.read_primitive_i64_s(self.sp - OPERAND_SIZE_IN_BYTES, 0)
    }

    fn peek_i64_u(&self) -> u64 {
        self.read_primitive_i64_u(self.sp - OPERAND_SIZE_IN_BYTES, 0)
    }

    fn peek_i32_s(&self) -> i32 {
        self.read_primitive_i32_s(self.sp - OPERAND_SIZE_IN_BYTES, 0)
    }

    fn peek_i32_u(&self) -> u32 {
        self.read_primitive_i32_u(self.sp - OPERAND_SIZE_IN_BYTES, 0)
    }

    fn peek_f64(&self) -> Result<f64, MemoryError> {
        self.read_primitive_f64(self.sp - OPERAND_SIZE_IN_BYTES, 0)
    }

    fn peek_f32(&self) -> Result<f32, MemoryError> {
        self.read_primitive_f32(self.sp - OPERAND_SIZE_IN_BYTES, 0)
    }

    fn pop_i64_s(&mut self) -> i64 {
        self.check_if_sufficient_operands_to_pop(1);

        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_primitive_i64_s(self.sp, 0)
    }

    fn pop_i64_u(&mut self) -> u64 {
        self.check_if_sufficient_operands_to_pop(1);

        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_primitive_i64_u(self.sp, 0)
    }

    fn pop_i32_s(&mut self) -> i32 {
        self.check_if_sufficient_operands_to_pop(1);

        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_primitive_i32_s(self.sp, 0)
    }

    fn pop_i32_u(&mut self) -> u32 {
        self.check_if_sufficient_operands_to_pop(1);

        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_primitive_i32_u(self.sp, 0)
    }

    fn pop_f64(&mut self) -> Result<f64, MemoryError> {
        self.check_if_sufficient_operands_to_pop(1);

        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_primitive_f64(self.sp, 0)
    }

    fn pop_f32(&mut self) -> Result<f32, MemoryError> {
        self.check_if_sufficient_operands_to_pop(1);

        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_primitive_f32(self.sp, 0)
    }

    fn prepare_popping_operand_to_memory(&mut self) -> *const u8 {
        self.check_if_sufficient_operands_to_pop(1);

        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.get_ptr(self.sp, 0)
    }

    fn prepare_popping_operands_to_memory(&mut self, count: usize) -> *const u8 {
        self.check_if_sufficient_operands_to_pop(count);

        let length = count * OPERAND_SIZE_IN_BYTES;
        self.sp -= length;
        self.get_ptr(self.sp, 0)
    }

    fn push_first_operands(&mut self, data: &[u8]) {
        self.data[0..data.len()].copy_from_slice(data);
        self.sp += data.len()
    }

    fn pop_last_operands(&mut self, count: usize) -> &[u8] {
        let length = count * OPERAND_SIZE_IN_BYTES;
        self.sp -= length;
        &self.data[self.sp..]
    }
}

impl CallingStack for NostdStack {
    /// Create a new stack frame.
    ///
    /// parameter `optional_return_pc` should be `None` when creating a 'block frame'.
    fn create_frame(
        &mut self,
        params_count: u16,
        results_count: u16,
        local_variable_list_index: u32,

        // includes the length of arguments and local variables
        local_variables_with_arguments_allocated_bytes: u32,
        optional_return_pc: Option<ProgramCounter>,
    ) -> Result<(), StackError> {
        // self.check_and_increase_stack_capacity()?;

        // move the arguments to swap
        self.move_operands_to_swap(params_count as usize);

        let previous_fp = self.fp;
        let next_fp = self.sp;

        let function_fp = if optional_return_pc.is_some() {
            // the `FunctionFramePointer` point to the new frame FP itself when creating a function frame.
            next_fp as u32
        } else {
            // the `FunctionFramePointer` point is inherited from the previous frame when creating a block frame.
            let frame_info_data = self.get_frame_info_data(previous_fp);
            frame_info_data.function_frame_address
        };

        // create new frame (full of random data)
        let frame_info_data = self.get_frame_info_data_mutable(next_fp);

        // write values
        frame_info_data.previous_frame_address = previous_fp as u32;
        frame_info_data.function_frame_address = function_fp;
        frame_info_data.params_count = params_count;
        frame_info_data.results_count = results_count;
        frame_info_data.local_variable_list_index = local_variable_list_index;
        frame_info_data.local_variables_with_arguments_allocated_bytes =
            local_variables_with_arguments_allocated_bytes;

        if let Some(return_pc) = optional_return_pc {
            frame_info_data.return_module_index = return_pc.module_index as u32;
            frame_info_data.return_function_internal_index =
                return_pc.function_internal_index as u32;
            frame_info_data.return_instruction_address = return_pc.instruction_address as u32;
        } else {
            frame_info_data.return_module_index = 0;
            frame_info_data.return_function_internal_index = 0;
            frame_info_data.return_instruction_address = 0;
        }

        // update sp and fp
        self.sp += size_of::<FrameInfoData>();
        self.fp = next_fp;

        // restore the arguments from swap
        self.restore_operands_from_swap(params_count as usize);

        // clean the actually local variables slots.
        //
        // note that can not use `local_variables_with_arguments_allocated_bytes` directly because its value
        // includes the length of arguments:
        //
        // ```diagram
        //       |                     |
        // ----- |---------------------| <----
        //  ^    | local var 2 (idx 4) |    ^  the actual local variables slots, length:
        //  |    | local var 1 (idx 3) |    |  'local_variables_with_arguments_allocated_bytes -
        // local | local var 0 (idx 2) |    v   params_count * OPERAND_SIZE_IN_BYTES'
        // vars  |---------------------|------
        // area  | arg 1 (local idx 1) |    ^   params_count * OPERAND_SIZE_IN_BYTES'
        //  v    | arg 0 (local idx 0) |    v
        // ----- |---------------------| <----
        //       |                     |
        //       \---------------------/ <---- stack start
        // ```
        //
        let local_variables_allocate_bytes_without_args =
            local_variables_with_arguments_allocated_bytes as usize
                - params_count as usize * OPERAND_SIZE_IN_BYTES;

        self.data[self.sp..(self.sp + local_variables_allocate_bytes_without_args)].fill(0);
        self.sp += local_variables_allocate_bytes_without_args;

        Ok(())
    }

    /// remove the specified frame and all frames that follows this frame.
    ///
    /// returns:
    /// - None: when the target frame is block frame.
    /// - Some(ProgramCounter): when the target frame is function frame.
    fn remove_frames(&mut self, layers: u16) -> Option<ProgramCounter> {
        let (sp, fp, is_function_frame, results_count, return_pc) = {
            let frame_info = self.get_frame_info_by_layers(layers);
            let is_function_frame = frame_info.get_frame_type() == FrameType::Function;
            (
                frame_info.address, // current frame start address
                frame_info.info_data.previous_frame_address as usize, // previous FP
                is_function_frame,
                frame_info.info_data.results_count,
                ProgramCounter {
                    instruction_address: frame_info.info_data.return_instruction_address as usize,
                    function_internal_index: frame_info.info_data.return_function_internal_index
                        as usize,
                    module_index: frame_info.info_data.return_module_index as usize,
                },
            )
        };

        // move the specified number of operands to swap as return values
        self.move_operands_to_swap(results_count as usize);

        self.sp = sp;
        self.fp = fp;

        // restore parameters from swap
        self.restore_operands_from_swap(results_count as usize);

        if is_function_frame {
            Some(return_pc)
        } else {
            None
        }
    }

    /// reset the specified function frame or block frame.
    fn reset_frames(&mut self, layers: u16) -> FrameType {
        let (
            is_function_frame,
            frame_addr,
            params_count,
            local_variables_with_arguments_allocated_bytes,
        ) = {
            let frame_info = self.get_frame_info_by_layers(layers);
            let is_function_frame = frame_info.get_frame_type() == FrameType::Function;
            (
                is_function_frame,
                frame_info.address,
                frame_info.info_data.params_count,
                frame_info
                    .info_data
                    .local_variables_with_arguments_allocated_bytes as usize,
            )
        };

        // there is an optimization for case of "looping in the current frame", but requires some conditions:
        // - the target frame is the current frame itself.
        // - there is no other operands than the local vars and parameters on the top of stack
        //
        // ```diagram
        //
        //                      operands that are about to
        //                      become arguments in recur.              move as args
        //                                   |                       /---------------\
        //                                   |                       |               |
        //        |            |       |     v      |          |     |               |
        //        |            |       |------------| <-- SP   |     |      |        |
        //        |            |       | results    |          | x x x      |        |
        // SP --> |------------|       |------------|          |------------| <-- SP |
        //        | local vars |       | local vars |          | local vars | <------|-- reset
        //        |------------|  ==>  |------------|  ==>     |------------|        |
        //        | arg 1      |       | arg 1      |          | arg 1      |        |
        //        | arg 0      |       | arg 0      |          | arg 0      | <------/
        //        |------------|       |------------|          |------------|
        //        | info       |       | info       |          | info       |
        // FP --> |============|       |============|          |============| <-- FP
        //        |            |       |            |          |            |
        //        \------------/       \------------/          \------------/
        //         before recur         going to recur           after recur
        // ```
        //
        // when the conditions above are met, there is no need to move the
        // operands (as argurments) to the "swap" and move back again, just
        // simply reset the local variable slots to '0' and
        // move (is memory copy actually) the results to argument slots.

        let params_bytes = params_count as usize * OPERAND_SIZE_IN_BYTES;
        if (layers == 0)
            && (self.sp
                == self.fp
                    + size_of::<FrameInfoData>()
                    + local_variables_with_arguments_allocated_bytes
                    + params_bytes)
        {
            // move (memory copy) the results to argument slots
            unsafe {
                std::ptr::copy(
                    self.data[(self.sp - params_bytes)..].as_ptr(),
                    self.data[self.fp + size_of::<FrameInfoData>()..].as_mut_ptr(),
                    params_bytes,
                );
            }

            self.sp -= params_bytes;

            // reset the local variable slots to 0
            let local_variables_addr_start = self.fp + size_of::<FrameInfoData>() + params_bytes;
            let local_variables_allocate_bytes_without_args =
                local_variables_with_arguments_allocated_bytes - params_bytes;
            self.data[local_variables_addr_start
                ..(local_variables_addr_start + local_variables_allocate_bytes_without_args)]
                .fill(0);

            if is_function_frame {
                FrameType::Function
            } else {
                FrameType::Block
            }
        } else {
            // move the specified number of operands to swap (they will become arguments)
            self.move_operands_to_swap(params_count as usize);

            // remove all operands and frames which follows the current frame
            //
            // |            |
            // | ...        |
            // |------------| <-- move SP to here
            // | local vars |
            // |------------|
            // | frame info |
            // |------------| <-- move FP to here
            // | ...        |
            // \------------/

            self.fp = frame_addr;
            self.sp = frame_addr + size_of::<FrameInfoData>();

            // restore parameters from swap
            self.restore_operands_from_swap(params_count as usize);

            // clean the actually local variables slots.
            //
            // note that can not use `local_variables_with_arguments_allocated_bytes` directly because its value
            // includes the length of arguments:
            //
            // ```diagram
            //       |                     |
            // ----- |---------------------| <----
            //  ^    | local var 2 (idx 4) |    ^  the actual local variables slots, length:
            //  |    | local var 1 (idx 3) |    |  'local_variables_with_arguments_allocated_bytes -
            // local | local var 0 (idx 2) |    v   params_count * OPERAND_SIZE_IN_BYTES'
            // vars  |---------------------|------
            // area  | arg 1 (local idx 1) |    ^   params_count * OPERAND_SIZE_IN_BYTES'
            //  v    | arg 0 (local idx 0) |    v
            // ----- |---------------------| <----
            //       |                     |
            //       \---------------------/ <---- stack start
            // ```

            let local_variables_allocate_bytes_without_args =
                local_variables_with_arguments_allocated_bytes
                    - params_count as usize * OPERAND_SIZE_IN_BYTES;
            self.data[self.sp..(self.sp + local_variables_allocate_bytes_without_args)].fill(0);
            self.sp += local_variables_allocate_bytes_without_args;

            if is_function_frame {
                FrameType::Function
            } else {
                FrameType::Block
            }
        }
    }

    /// Calculates the start address of the local variables area for a frame
    /// identified by the given layers.
    ///
    /// The address is computed as `frame pointer + size of FrameInfoData`.
    /// This method always returns the calculated address, even if no local variables exist.
    fn get_local_variable_list_index_and_start_address_by_layers(
        &self,
        layers: u16,
    ) -> (usize, usize) {
        // ```diagram
        // |                 |
        // | local vars      |
        // |-----------------|
        // | args            |
        // |-----------------| <-- local vars start address
        // | frame info data |
        // |-----------------| <-- frame pointer
        // | ...             |
        // \-----------------/
        // ```

        let frame_info = self.get_frame_info_by_layers(layers);

        (
            frame_info.info_data.local_variable_list_index as usize,
            frame_info.address + size_of::<FrameInfoData>(),
        )
    }

    fn reset(&mut self) {
        self.data = [0u8; STACK_SIZE_IN_BYTES];
        self.swap = [0u8; SWAP_SIZE_IN_BYTES];
        self.fp = 0;
        self.sp = 0;
    }
}

impl Stack for NostdStack {}

impl NostdStack {
    /// Creates a new `SimpleStack` instance with initialized stack and swap areas.
    pub fn new() -> Self {
        let data = [0u8; STACK_SIZE_IN_BYTES];
        let swap = [0u8; SWAP_SIZE_IN_BYTES];
        Self {
            data,
            swap,
            sp: 0,
            fp: 0,
        }
    }

    //    /// Returns the current capacity of the stack in bytes.
    //    fn get_stack_capacity_in_bytes(&self) -> usize {
    //        self.data.len()
    //    }
    //
    //     /// Doubles the stack capacity if it does not exceed the maximum allowed size.
    //     /// Returns the new capacity or an error if the maximum size is exceeded.
    //     fn increase_stack_capacity(&mut self) -> Result<usize, StackError> {
    //         let new_size_in_bytes = self.get_stack_capacity_in_bytes() * 2;
    //         if new_size_in_bytes > STACK_SIZE_IN_BYTES {
    //             return Err(StackError::new(StackErrorType::StackOverflow));
    //         }
    //
    //         self.data.resize(new_size_in_bytes, 0);
    //         Ok(new_size_in_bytes)
    //     }
    //
    //     /// Ensures there is enough space for a new stack frame.
    //     /// If the stack pointer exceeds half the current capacity, the stack is resized.
    //     fn check_and_increase_stack_capacity(&mut self) -> Result<usize, StackError> {
    //         let stack_size_in_bytes = self.get_stack_capacity_in_bytes();
    //         let new_size_in_bytes = if self.sp > stack_size_in_bytes / 2 {
    //             self.increase_stack_capacity()?
    //         } else {
    //             stack_size_in_bytes
    //         };
    //         Ok(new_size_in_bytes)
    //     }

    /// Retrieves a reference to `FrameInfoData` at the specified frame pointer (FP).
    fn get_frame_info_data(&self, frame_pointer: usize) -> &FrameInfoData {
        let ptr = self.data[frame_pointer..].as_ptr();
        unsafe { &*(ptr as *const FrameInfoData) }
    }

    /// Retrieves a mutable reference to `FrameInfoData` at the specified frame pointer (FP).
    fn get_frame_info_data_mutable(&mut self, addr: usize) -> &mut FrameInfoData {
        let ptr = self.data[addr..].as_mut_ptr();
        unsafe { &mut *(ptr as *mut FrameInfoData) }
    }

    /// Retrieves `FrameInfo` by the given layers.
    ///
    /// The layers specifies the depth of the frame relative to the current frame.
    /// For example:
    /// - `0` retrieves the current frame.
    /// - `1` retrieves the parent frame.
    /// - `n` retrieves the nth parent frame.
    ///
    /// ```diagram
    /// fn {
    ///   ;; frame 0 (function frame)
    ///   block
    ///     ;; frame 1 (block frame)
    ///     block
    ///       ;; frame 2 (block frame)
    ///       block
    ///         ;; frame 3 (block frame)
    ///         ;; assuming this is the current stack frame, then:
    ///         ;; - to get frame 3: layers = 0
    ///         ;; - to get frame 2: layers = 1
    ///         ;; - to get frame 0: layers = 3
    ///       end
    ///     end
    ///   end
    /// }
    /// ```
    ///
    /// Panics if the number of layer exceeds the available frames or crosses function boundaries.
    fn get_frame_info_by_layers(&self, layers: u16) -> FrameInfo {
        // the `FP` chain:
        //
        // ```diagram
        //           |         |           |         |           |         |
        //           |---------|           |---------|           |---------|
        // FrameInfo | prev FP |----\      | prev FP |----\      | ...     |
        //      Data | ...     |    |      | ...     |    |      | ...     |
        //     FP -> |---------|    \----> |---------|    \----> |---------|
        //           | ...     |           | ...     |           | ...     |
        //           \---------/           \---------/           \---------/
        //             layers 0              layers 1              layers 2
        // ```

        let mut remains = layers;
        let mut fp = self.fp;
        let mut frame_info_data = self.get_frame_info_data(fp);
        let mut is_function_frame = fp == frame_info_data.function_frame_address as usize;

        while remains > 0 {
            if is_function_frame {
                // crossing function is not allowed
                panic!(
                    "The layers is out of bounds when retrieving stack frame information.
FP: {}, SP: {}, layers: {}.",
                    self.fp, self.sp, layers
                )
            }

            fp = frame_info_data.previous_frame_address as usize;
            frame_info_data = self.get_frame_info_data(fp);
            is_function_frame = fp == frame_info_data.function_frame_address as usize;
            remains -= 1;
        }

        FrameInfo::new(fp, frame_info_data)
    }

    //     /// Retrieves `FrameInfo` for the current function frame.
    //     ///
    //     /// Function frames and block frames are distinguished by the `function_frame_address` field
    //     /// in `FrameInfoData`. This method identifies the function frame associated with the current frame.
    //     fn get_function_frame_info(&self) -> FrameInfo {
    //         // Function frames and block frames
    //         // --------------------------------
    //         // There are two types of stack frames: block frames and function frames.
    //         // Both types include the `FunctionFramePointer` field in the `FrameInfoData` structure.
    //         // This field makes it straightforward to identify the function frame associated with a given frame.
    //         //
    //         // Example 1: A block frame followed by a function frame
    //         //
    //         // ```diagram
    //         //               |         |
    //         //               |---------|
    //         //               | ...     |
    //         //   block frame | Func FP | ---\
    //         //               | prev FP |    |
    //         // current FP -> |---------|    |
    //         //               | ...     |    |
    //         //               | Func FP |    |
    //         //    func frame | prev FP |    |
    //         //               |---------| <--/
    //         //               | ...     |
    //         //               \---------/
    //         // ```
    //         //
    //         // Example 2: A standalone function frame
    //         //
    //         // ```diagram
    //         //               |         |
    //         //               |---------|
    //         //               | ...     |
    //         //    func frame | Func FP | ---\
    //         //               | prev FP |    |
    //         // current FP -> |---------| <--/
    //         //               | ...     |
    //         //               \---------/
    //         // ```
    //         //
    //         // Explanation:
    //         // - The `Func FP` field in the block frame points to the start of the function frame.
    //         // - The `prev FP` field in the block frame points to the previous frame (if any).
    //         // - The function frame contains its own `Func FP` field, which points to itself.
    //
    //         let frame_info_data = self.get_frame_info_data(self.fp);
    //         if frame_info_data.function_frame_address as usize == self.fp {
    //             // the current frame itself is function frame.
    //             FrameInfo::new(self.fp, frame_info_data)
    //         } else {
    //             // the current frame is a block frame, get the function frame with the
    //             // `FunctionFramePointer` field in the `FrameInfoData`
    //             let function_fp = frame_info_data.function_frame_address as usize;
    //             let function_frame_info_data = self.get_frame_info_data(function_fp);
    //             FrameInfo::new(function_fp, function_frame_info_data)
    //         }
    //     }

    /// Moves the specified number of operands from the stack to the swap area.
    ///
    /// This method is used in scenarios such as:
    /// - Function calls: Move arguments to the swap area before creating a new frame.
    /// - Function returns: Move results to the swap area before removing a frame.
    /// - Tail call optimization (TCO): Move results to the swap area before resetting the frame.
    fn move_operands_to_swap(&mut self, operands_count: usize) {
        if operands_count == 0 {
            return;
        }

        // Note:
        // The function `check_if_sufficient_operands_to_pop` requires a stack frame to operate correctly.
        // However, there is no definitive way to determine whether a stack frame is present,
        // especially since the stack pointer (SP) being zero does not necessarily indicate the absence of a frame.
        // This is because when the frame pointer (FP) is zero, it is possible that the first stack frame
        // is located at address 0.
        #[cfg(feature = "bounds_check")]
        {
            if self.fp == 0 {
                if self.sp < operands_count * OPERAND_SIZE_IN_BYTES {
                    panic!(
                        "Insufficient operands on the stack for function arguments.
FP: {}, SP: {}, expected operands count: {} (length in bytes: {}).",
                        self.fp,
                        self.sp,
                        operands_count,
                        operands_count * OPERAND_SIZE_IN_BYTES
                    )
                }
            } else {
                self.check_if_sufficient_operands_to_pop(operands_count);
            }
        }

        let size_in_bytes = operands_count * OPERAND_SIZE_IN_BYTES;
        let offset = self.sp - size_in_bytes;

        // memory copy from stack to swap
        let src = self.data[offset..].as_ptr();
        let dst = self.swap.as_mut_ptr();
        unsafe {
            std::ptr::copy(src, dst, size_in_bytes);
        }

        // update the SP
        self.sp = offset;
    }

    /// Restores the specified number of operands from the swap area back to the stack.
    fn restore_operands_from_swap(&mut self, operands_count: usize) {
        if operands_count == 0 {
            return;
        }

        let size_in_bytes = operands_count * OPERAND_SIZE_IN_BYTES;

        // Copy memory from swap area to stack.
        let src = self.swap.as_ptr();
        let dst = self.data[self.sp..].as_mut_ptr();
        unsafe {
            std::ptr::copy(src, dst, size_in_bytes);
        }

        // update the SP
        self.sp += size_in_bytes;
    }

    /// Checks if there are sufficient operands on the stack to pop the specified count.
    ///
    /// This method ensures that popping operands does not violate stack boundaries.
    #[inline]
    fn check_if_sufficient_operands_to_pop(&self, count: usize) {
        #[cfg(feature = "bounds_check")]
        {
            let frame_info = self.get_frame_info_data(self.fp);
            let local_variables_with_arguments_allocated_bytes =
                frame_info.local_variables_with_arguments_allocated_bytes as usize;

            if self.sp - (count * OPERAND_SIZE_IN_BYTES)
                < self.fp
                    + size_of::<FrameInfoData>()
                    + local_variables_with_arguments_allocated_bytes
            {
                panic!(
                    "Insufficient operands on the stack for popping.
Expected: SP > (FP + frame info length + local variables length + popping length).
SP: {}, FP: {}, frame info length (in bytes): {}, local variables area length (in bytes): {},
expected popping operands count: {} (length in bytes: {}).",
                    self.sp,
                    self.fp,
                    size_of::<FrameInfoData>(),
                    local_variables_with_arguments_allocated_bytes,
                    count,
                    count * OPERAND_SIZE_IN_BYTES
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use anc_isa::OPERAND_SIZE_IN_BYTES;
    use anc_memory::{memory_access::MemoryAccess, primitive_memory_access::PrimitiveMemoryAccess};

    use crate::{
        nostd_stack::FrameInfo,
        stack::{CallingStack, OperandStack},
        FrameType, ProgramCounter,
    };

    use super::{FrameInfoData, NostdStack};

    // Helper functions for unit tests
    impl NostdStack {
        /// Creates an empty frame for testing purposes.
        fn create_empty_frame(&mut self) {
            self.create_frame(0, 0, 0, 0, None).unwrap();
        }
    }

    #[test]
    fn test_push_pop_and_peek() {
        let mut stack = NostdStack::new();

        // `pop_xxx` functions require a stack frame to operate.
        stack.create_empty_frame();

        const FRAME_INFO_DATA_SIZE_IN_BYTES: usize = size_of::<FrameInfoData>();
        const INITIAL_SP: usize = FRAME_INFO_DATA_SIZE_IN_BYTES;

        // Test pushing, peeking, and popping values
        stack.push_i32_u(11);
        stack.push_i64_u(13);
        stack.push_f32(std::f32::consts::PI);
        stack.push_f64(std::f64::consts::E);

        assert_eq!(stack.sp, INITIAL_SP + OPERAND_SIZE_IN_BYTES * 4);

        assert_eq!(stack.peek_f64().unwrap(), std::f64::consts::E);
        assert_eq!(stack.pop_f64().unwrap(), std::f64::consts::E);

        assert_eq!(stack.peek_f32().unwrap(), std::f32::consts::PI);
        assert_eq!(stack.pop_f32().unwrap(), std::f32::consts::PI);

        assert_eq!(stack.peek_i64_u(), 13);
        assert_eq!(stack.pop_i64_u(), 13);

        assert_eq!(stack.peek_i32_u(), 11);
        assert_eq!(stack.pop_i32_u(), 11);

        assert_eq!(stack.sp, INITIAL_SP);
    }

    #[test]
    fn test_operand_signed_extend() {
        let mut stack = NostdStack::new();

        // `pop_xxx` functions require a stack frame to operate.
        stack.create_empty_frame();

        // Test signed and unsigned extension of i32 to i64
        stack.push_i32_s(0x8000_0000_u32 as i32);
        stack.push_i32_u(0x8000_0000_u32);
        assert_eq!(stack.peek_i64_s(), 0x0000_0000_8000_0000_u64 as i64);
        assert_eq!(stack.pop_i64_u(), 0x0000_0000_8000_0000_u64);
        assert_eq!(stack.peek_i64_s(), 0xffff_ffff_8000_0000_u64 as i64);
        assert_eq!(stack.pop_i64_u(), 0xffff_ffff_8000_0000_u64);
    }

    #[test]
    fn test_operand_stack_boundary_check() {
        let mut stack = NostdStack::new();

        // `pop_xxx` functions require a stack frame to operate.
        stack.create_empty_frame();

        stack.push_i32_u(11);
        stack.push_i32_u(13);

        assert_eq!(stack.pop_i32_u(), 13);
        assert_eq!(stack.pop_i32_u(), 11);

        // Test boundary check by attempting to pop from an empty stack
        let prev_hook = std::panic::take_hook(); // Silence panic output
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            stack.pop_i32_u(); // This should panic
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_frame_depth_boundary_check() {
        let mut stack = NostdStack::new();

        // create frames:
        //
        // ```diagram
        // | block frame 2 | depth: 3
        // | block frame 1 | depth: 2
        // | block frame 0 | depth: 1
        // | func frame 1  | depth: 0
        // | func frame 0  | depth: 0
        // \---------------/
        // ```

        // function frame
        stack
            .create_frame(
                0,
                0,
                0,
                0,
                Some(ProgramCounter {
                    instruction_address: 0,
                    module_index: 0,
                    function_internal_index: 0,
                }),
            )
            .unwrap();

        stack
            .create_frame(
                0,
                0,
                1,
                8,
                Some(ProgramCounter {
                    instruction_address: 0,
                    module_index: 0,
                    function_internal_index: 0,
                }),
            )
            .unwrap();

        stack.create_frame(0, 0, 2, 16, None).unwrap();
        stack.create_frame(0, 0, 3, 24, None).unwrap();
        stack.create_frame(0, 0, 4, 32, None).unwrap();

        // Verify frame information
        let frame_info0 = stack.get_frame_info_by_layers(0);
        assert_eq!(frame_info0.info_data.local_variable_list_index, 4);

        let frame_info1 = stack.get_frame_info_by_layers(1);
        assert_eq!(frame_info1.info_data.local_variable_list_index, 3);

        let frame_info2 = stack.get_frame_info_by_layers(2);
        assert_eq!(frame_info2.info_data.local_variable_list_index, 2);

        let frame_info3 = stack.get_frame_info_by_layers(3);
        assert_eq!(frame_info3.info_data.local_variable_list_index, 1);

        // Test boundary check by attempting to access a non-existent frame
        let prev_hook = std::panic::take_hook(); // Silence panic output
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            stack.get_frame_info_by_layers(4); // This should panic
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_floating_point_variants_check() {
        let mut stack = NostdStack::new();

        // `pop_xxx` functions require a stack frame to operate.
        stack.create_empty_frame();

        // Test pushing and popping floating-point values
        stack.push_f32(std::f32::consts::PI);
        stack.push_f64(std::f64::consts::E);
        assert_eq!(stack.pop_f64().unwrap(), std::f64::consts::E);
        assert_eq!(stack.pop_f32().unwrap(), std::f32::consts::PI);

        // Test handling of NaN and infinity values
        stack.push_i32_u(0xffc0_0000); // NaN
        assert!(stack.peek_f32().is_err());
        assert!(stack.pop_f32().is_err());

        stack.push_i32_u(0x7f80_0000); // +Inf
        assert!(stack.peek_f32().is_err());
        assert!(stack.pop_f32().is_err());

        stack.push_i32_u(0xff80_0000); // -Inf
        assert!(stack.peek_f32().is_err());
        assert!(stack.pop_f32().is_err());
    }

    #[test]
    fn test_local_variables_host_address() {
        let mut stack = NostdStack::new();

        // Push arguments onto the stack
        stack.push_i32_u(11);
        stack.push_i32_u(13);
        stack.push_i32_u(17);
        stack.push_i32_u(19);

        // Create a frame with arguments and local variables
        stack.create_frame(4, 0, 0, 32 + 16, None).unwrap();

        let (_, local_start) =
            stack.get_local_variable_list_index_and_start_address_by_layers(0);

        // Write to local variables
        stack.write_primitive_i32_u(local_start, 4 * 8, 23);
        stack.write_primitive_i32_u(local_start, 5 * 8, 29);

        // Verify local variables
        assert_eq!(stack.read_primitive_i32_u(local_start, 0), 11);
        assert_eq!(stack.read_primitive_i32_u(local_start, 8), 13);
        assert_eq!(stack.read_primitive_i32_u(local_start, 2 * 8), 17);
        assert_eq!(stack.read_primitive_i32_u(local_start, 3 * 8), 19);
        assert_eq!(stack.read_primitive_i32_u(local_start, 4 * 8), 23);
        assert_eq!(stack.read_primitive_i32_u(local_start, 5 * 8), 29);

        // Verify memory addresses of local variables
        let ptr0 = stack.get_ptr(local_start, 0);
        let ptr1 = stack.get_ptr(local_start, 8);
        let ptr2 = stack.get_ptr(local_start, 2 * 8);
        let ptr3 = stack.get_ptr(local_start, 3 * 8);
        let ptr4 = stack.get_ptr(local_start, 4 * 8);
        let ptr5 = stack.get_ptr(local_start, 5 * 8);

        let read_i32 = |addr: *const u8| -> u32 {
            let ptr = addr as *const u32;
            unsafe { std::ptr::read(ptr) }
        };

        // Each operand occupies 8 bytes
        assert_eq!(read_i32(ptr0), 11);
        assert_eq!(read_i32(ptr1), 13);
        assert_eq!(read_i32(ptr2), 17);
        assert_eq!(read_i32(ptr3), 19);
        assert_eq!(read_i32(ptr4), 23);
        assert_eq!(read_i32(ptr5), 29);
    }

    #[test]
    fn test_create_frames() {
        // tasks:
        //
        // 1. create function frame (f0) .. with 2 args + 2 local vars, 0 results.
        // 2. create block frame (f1) ..... with 1 args + 0 local vars, 2 results.
        // 3. create block frame (f2) ..... with 0 args + 0 local vars, 0 results.
        // 4. create function frame (f3) .. with 1 args + 0 local vars, 3 results.
        // 5. remove `f3`.
        // 6. remove `f2` and `f1` at once.
        // 7. remove `f0`

        let mut stack = NostdStack::new();

        // the arguments for the first functon call
        stack.push_i32_u(23);
        stack.push_i32_u(29);
        stack.push_i32_u(31);
        stack.push_i32_u(37);

        // the stack data layout:
        //
        // ```diagram
        // SP    0d0032 |        |
        //       0d0024 | 37     |
        //       0d0016 | 31     |
        //       0d0008 | 29     |
        // FP    0d0000 | 23     |
        //              \--------/
        // ```

        // tasks:
        //
        // > 1. create function frame (f0) .. with 2 args + 2 local vars, 0 results.
        //   2. create block frame (f1) ..... with 1 args + 0 local vars, 2 results.
        //   3. create block frame (f2) ..... with 0 args + 0 local vars, 0 results.
        //   4. create function frame (f3) .. with 1 args + 0 local vars, 3 results.
        //   5. remove `f3`.
        //   6. remove `f2` and `f1` at once.
        //   7. remove `f0`

        stack
            .create_frame(
                2, // params count
                0, // results count
                401,
                16 + 16, // local vars length: 2 args + 2 locals
                Some(ProgramCounter {
                    module_index: 503,            // return module idx
                    function_internal_index: 509, // return function internal idx
                    instruction_address: 521,     // return instruction address
                }),
            )
            .unwrap();

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0080 |        |
        //       0d0072 | 0      |
        //       0d0064 | 0      | <-- local vars 0
        //              |--------|
        //       0d0056 | 37     |
        //       0d0048 | 31     | <-- args 0
        //              |--------|
        //       0d0044 | 521    | return instruction address
        //       0d0040 | 509    | return function internal idx
        //       0d0036 | 503    | return module idx
        //       0d0032 | 32     | local vars length
        //       0d0028 | 401    | local vars list idx
        //       0d0024 | 2/0    | params/results count
        //       0d0020 | 16     | func FP
        // FP--> 0d0016 | 0      | prev FP
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/
        // ```

        // check remain operands
        assert_eq!(stack.read_primitive_i64_u(0, 0), 23);
        assert_eq!(stack.read_primitive_i64_u(8, 0), 29);

        // check FP and SP
        let fp0 = 16;
        assert_eq!(stack.sp, 80);
        assert_eq!(stack.fp, fp0);

        // check frame info
        let frame_info_0 = stack.get_frame_info_by_layers(0);
        let expected_frame_info_0 = FrameInfo {
            address: fp0,
            info_data: &FrameInfoData {
                previous_frame_address: 0,
                function_frame_address: fp0 as u32,
                params_count: 2,
                results_count: 0,
                local_variable_list_index: 401,
                local_variables_with_arguments_allocated_bytes: 32,
                return_module_index: 503,
                return_function_internal_index: 509,
                return_instruction_address: 521,
            },
        };

        assert_eq!(frame_info_0, expected_frame_info_0);

        // check local variables

        // the stack data layout:
        //
        // ```diagram
        //
        //       0d0072 | 0      |
        //       0d0064 | 0      | <-- local vars 0
        //              |--------|
        //       0d0056 | 37     |
        //       0d0048 | 31     | <-- args 0
        //              |--------|
        //              | info 0 |
        // ```

        let local_start_0 = fp0 + size_of::<FrameInfoData>();
        assert_eq!(
            stack.get_local_variable_list_index_and_start_address_by_layers(0),
            (401, local_start_0),
        );

        assert_eq!(stack.read_primitive_i32_u(local_start_0, 0), 31);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 8), 37);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 2 * 8), 0);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 3 * 8), 0);

        // update local variables
        stack.write_primitive_i32_u(local_start_0, 2 * 8, 211);
        stack.write_primitive_i32_u(local_start_0, 3 * 8, 223);

        // check local variables again
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 2 * 8), 211);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 3 * 8), 223);

        // add more operands
        stack.push_i32_u(41);
        stack.push_i32_u(43);
        stack.push_i32_u(47);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0104 |        |
        //       0d0096 | 47     |
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //              |--------|
        //       0d0072 | 223    |
        //       0d0064 | 211    | <-- local vars 0
        //              |--------|
        //       0d0056 | 37     |
        //       0d0048 | 31     | <-- args 0, also local vars 0
        //              |--------|
        //              | info 0 |
        // ```

        // check FP and SP
        assert_eq!(stack.fp, fp0);
        assert_eq!(stack.sp, 104);

        // tasks:
        //
        //   1. create function frame (f0) .. with 2 args + 2 local vars, 0 results.
        // > 2. create block frame (f1) ..... with 1 args + 0 local vars, 2 results.
        //   3. create block frame (f2) ..... with 0 args + 0 local vars, 0 results.
        //   4. create function frame (f3) .. with 1 args + 0 local vars, 3 results.
        //   5. remove `f3`.
        //   6. remove `f2` and `f1` at once.
        //   7. remove `f0`

        stack
            .create_frame(
                1, // params count
                2, // results count
                419, 8, // local vars length, 1 arg + 0 local
                None,
            )
            .unwrap();

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0136 |        |
        //       0d0128 | 47     | <-- args 1, also local vars 1
        //              |--------|
        //       0d0124 | 0      | return instruction address
        //       0d0120 | 0      | return function internal idx
        //       0d0116 | 0      | return module idx
        //       0d0112 | 8      | local vars length
        //       0d0108 | 419    | local vars list idx
        //       0d0104 | 1/2    | params/results count
        //       0d0100 | 16     | func FP
        // FP--> 0d0096 | 16     | prev FP
        //              |--------| <-- fp1
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //              |--------|
        //       0d0072 | 223    |
        //       0d0064 | 211    | <-- local vars 0
        //              |--------|
        //       0d0056 | 37     |
        //       0d0048 | 31     | <-- args 0, also local vars 0
        //              |--------|
        //              | info 0 |
        // ```

        let fp1 = 96;

        // check FP and SP
        assert_eq!(stack.fp, fp1);
        assert_eq!(stack.sp, 136);

        // check remain operands
        assert_eq!(stack.read_primitive_i32_u(88, 0), 43);
        assert_eq!(stack.read_primitive_i32_u(80, 0), 41);

        let frame_info_1 = stack.get_frame_info_by_layers(0);
        let expected_frame_info_1 = FrameInfo {
            address: fp1,
            info_data: &FrameInfoData {
                previous_frame_address: 16,
                function_frame_address: fp0 as u32,
                params_count: 1,
                results_count: 2,
                local_variable_list_index: 419,
                local_variables_with_arguments_allocated_bytes: 8,
                return_module_index: 0,
                return_function_internal_index: 0,
                return_instruction_address: 0,
            },
        };

        assert_eq!(frame_info_1, expected_frame_info_1);

        assert_eq!(
            stack.get_frame_info_by_layers(1),
            expected_frame_info_0
        );

        // check local variables
        let local_start_1 = fp1 + size_of::<FrameInfoData>();

        assert_eq!(
            (419, local_start_1),
            stack.get_local_variable_list_index_and_start_address_by_layers(0),
        );

        assert_eq!(
            (401, local_start_0),
            stack.get_local_variable_list_index_and_start_address_by_layers(1),
        );

        // check local variables, frame 1
        assert_eq!(stack.read_primitive_i32_u(local_start_1, 0), 47);

        // check local variables, frame 0
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 0), 31);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 8), 37);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 2 * 8), 211);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 3 * 8), 223);

        // update and check local variables, frame 1
        stack.write_primitive_i32_u(local_start_1, 0, 227);
        assert_eq!(stack.read_primitive_i32_u(local_start_1, 0), 227);

        // update and check local variables, frame 0
        stack.write_primitive_i32_u(local_start_0, 0, 229);
        stack.write_primitive_i32_u(local_start_0, 8, 233);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 0), 229);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 8), 233);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0136 |        |
        //       0d0128 | 227    | <-- args 1, also local vars 1
        //              |--------|
        //       0d0124 | 0      | return instruction address
        //       0d0120 | 0      | return function internal idx
        //       0d0116 | 0      | return module idx
        //       0d0112 | 8      | local vars length
        //       0d0108 | 419    | local vars list idx
        //       0d0104 | 1/2    | params/results count
        //       0d0100 | 16     | func FP
        // FP--> 0d0096 | 16     | prev FP
        //              |--------| <-- fp1
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //              |--------|
        //       0d0072 | 223    |
        //       0d0064 | 211    | <-- local vars 0
        //              |--------|
        //       0d0056 | 233    |
        //       0d0048 | 229    | <-- args 0, also local vars 0
        //              |--------|
        //              | info 0 |
        // ```

        // tasks:
        //
        //   1. create function frame (f0) .. with 2 args + 2 local vars, 0 results.
        //   2. create block frame (f1) ..... with 1 args + 0 local vars, 2 results.
        // > 3. create block frame (f2) ..... with 0 args + 0 local vars, 0 results.
        //   4. create function frame (f3) .. with 1 args + 0 local vars, 3 results.
        //   5. remove `f3`.
        //   6. remove `f2` and `f1` at once.
        //   7. remove `f0`

        stack.create_frame(0, 0, 421, 0, None).unwrap();

        // the stack data layout:
        //
        // ```diagram
        //
        // FP--> 0d0136 | info 2 |
        //              |--------|
        //       0d0128 | 227    | <-- args 1, also local vars 1
        //              |--------|
        //              | info 1 |
        //              |--------| <-- fp1
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //       0d0072 | 223    |
        //       0d0064 | 211    | <-- local vars 0
        //       0d0056 | 233    |
        //       0d0048 | 229    | <-- args 0, also local vars 0
        //              |--------|
        //              | info 0 |
        // ```

        // check FP and SP
        let fp2 = fp1 + size_of::<FrameInfoData>() + 8; // 1 args in the 1st block frame
        assert_eq!(stack.fp, fp2);
        assert_eq!(stack.sp, fp2 + size_of::<FrameInfoData>());

        // check frame info
        let frame_info_2 = stack.get_frame_info_by_layers(0);
        let expected_frame_info_2 = FrameInfo {
            address: fp2,
            info_data: &FrameInfoData {
                previous_frame_address: fp1 as u32,
                function_frame_address: fp0 as u32,
                params_count: 0,
                results_count: 0,
                local_variable_list_index: 421,
                local_variables_with_arguments_allocated_bytes: 0,
                return_module_index: 0,
                return_function_internal_index: 0,
                return_instruction_address: 0,
            },
        };

        assert_eq!(frame_info_2, expected_frame_info_2);

        assert_eq!(
            stack.get_frame_info_by_layers(1),
            expected_frame_info_1
        );

        assert_eq!(
            stack.get_frame_info_by_layers(2),
            expected_frame_info_0
        );

        // check local variables
        let local_start_2 = fp2 + size_of::<FrameInfoData>();
        assert_eq!(
            (421, local_start_2),
            stack.get_local_variable_list_index_and_start_address_by_layers(0),
        );

        assert_eq!(
            (419, local_start_1),
            stack.get_local_variable_list_index_and_start_address_by_layers(1),
        );

        assert_eq!(
            (401, local_start_0),
            stack.get_local_variable_list_index_and_start_address_by_layers(2),
        );

        // check local variables, frame 2
        // frame 2 has no local vars

        // check local variables, frame 1
        assert_eq!(stack.read_primitive_i32_u(local_start_1, 0), 227);

        // check local variables, frame 0
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 0), 229);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 8), 233);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 2 * 8), 211);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 3 * 8), 223);

        // update and check local variables, frame 1
        stack.write_primitive_i32_u(local_start_1, 0, 239);
        assert_eq!(stack.read_primitive_i32_u(local_start_1, 0), 239);

        // update and check local variables, frame 0
        stack.write_primitive_i32_u(local_start_0, 2 * 8, 241);
        stack.write_primitive_i32_u(local_start_0, 3 * 8, 251);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 2 * 8), 241);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 3 * 8), 251);

        // add operands
        stack.push_i32_u(47);
        stack.push_i32_u(53);

        // the stack data layout:
        //
        // ```diagram
        //              | 53     |
        //              | 47     |
        //              |--------|
        // FP--> 0d0136 | info 2 |
        //              |--------|
        //       0d0128 | 239    | <-- args 1, also local vars 1
        //              |--------|
        //              | info 1 |
        //              |--------| <-- fp1
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //       0d0072 | 251    |
        //       0d0064 | 241    | <-- local vars 0
        //       0d0056 | 233    |
        //       0d0048 | 229    | <-- args 0, also local vars 0
        //              |--------|
        //              | info 0 |
        // ```

        // tasks:
        //
        //   1. create function frame (f0) .. with 2 args + 2 local vars, 0 results.
        //   2. create block frame (f1) ..... with 1 args + 0 local vars, 2 results.
        //   3. create block frame (f2) ..... with 0 args + 0 local vars, 0 results.
        // > 4. create function frame (f3) .. with 1 args + 0 local vars, 3 results.
        //   5. remove `f3`.
        //   6. remove `f2` and `f1` at once.
        //   7. remove `f0`

        stack
            .create_frame(
                1, // params count
                3, // results count
                431,
                8, // local vars length
                Some(ProgramCounter {
                    module_index: 47,            // return module idx
                    function_internal_index: 43, // return function internal idx
                    instruction_address: 53,     // return instructon address
                }),
            )
            .unwrap();

        // the stack data layout:
        //
        // ```diagram
        //
        //              | 53     | <-- args 3, also local vars 3
        //              |--------|
        // FP-->        | info 3 |
        //              |--------|
        //              | 47     | <-- operands 2
        //              |--------|
        //       0d0136 | info 2 |
        //              |--------|
        //       0d0128 | 239    | <-- args 1, also local vars 1
        //              |--------|
        //              | info 1 |
        //              |--------|
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //       0d0072 | 251    |
        //       0d0064 | 241    | <-- local vars 0
        //       0d0056 | 233    |
        //       0d0048 | 229    | <-- args 0, also local vars 0
        //              |--------|
        //              | info 0 |
        // ```

        // check FP and SP
        let fp3 = fp2 + size_of::<FrameInfoData>() + 8; // 1 operand in the 2nd block frame
        assert_eq!(stack.fp, fp3);
        assert_eq!(stack.sp, fp3 + size_of::<FrameInfoData>() + 8); // 1 args in the current frame

        let frame_info_3 = stack.get_frame_info_by_layers(0);
        let expected_frame_info_3 = FrameInfo {
            address: fp3,
            info_data: &FrameInfoData {
                previous_frame_address: fp2 as u32,
                function_frame_address: fp3 as u32,
                params_count: 1,
                results_count: 3,
                local_variable_list_index: 431,
                local_variables_with_arguments_allocated_bytes: 8,
                return_module_index: 47,
                return_function_internal_index: 43,
                return_instruction_address: 53,
            },
        };

        assert_eq!(frame_info_3, expected_frame_info_3);

        // check local variables
        let local_start_3 = fp3 + size_of::<FrameInfoData>();
        assert_eq!(
            (431, local_start_3),
            stack.get_local_variable_list_index_and_start_address_by_layers(0),
        );

        assert_eq!(stack.read_primitive_i32_u(local_start_3, 0), 53);

        // update and check local varibles
        stack.write_primitive_i32_u(local_start_3, 0, 257);
        assert_eq!(stack.read_primitive_i32_u(local_start_3, 0), 257);

        // push some oparnds first
        stack.push_i32_u(53);
        stack.push_i32_u(59);
        stack.push_i32_u(61);

        // the stack data layout:
        //
        // ```diagram
        //
        //              |        |
        //              | 61     |
        //              | 59     |
        //              | 53     | <-- operands 3
        //              | 257    | <-- args 3, also local vars 3
        //              |--------|
        // FP-->        | info 3 |
        //              |--------|
        //              | 47     | <-- operands 2
        //              |--------|
        //       0d0136 | info 2 |
        //              |--------|
        //       0d0128 | 239    | <-- args 1, also local vars 1
        //              |--------|
        //              | info 1 |
        //              |--------|
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //       0d0072 | 251    |
        //       0d0064 | 241    | <-- local vars 0
        //       0d0056 | 233    |
        //       0d0048 | 229    | <-- args 0, also local vars 0
        //              |--------|
        //              | info 0 |
        // ```

        // tasks:
        //
        //   1. create function frame (f0) .. with 2 args + 2 local vars, 0 results.
        //   2. create block frame (f1) ..... with 1 args + 0 local vars, 2 results.
        //   3. create block frame (f2) ..... with 0 args + 0 local vars, 0 results.
        //   4. create function frame (f3) .. with 1 args + 0 local vars, 3 results.
        // > 5. remove `f3`.
        //   6. remove `f2` and `f1` at once.
        //   7. remove `f0`

        let opt_return_pc0 = stack.remove_frames(0);

        assert_eq!(
            opt_return_pc0,
            Some(ProgramCounter {
                module_index: 47,
                function_internal_index: 43,
                instruction_address: 53,
            })
        );

        assert_eq!(
            stack.get_frame_info_by_layers(0),
            expected_frame_info_2
        );
        assert_eq!(
            stack.get_frame_info_by_layers(1),
            expected_frame_info_1
        );
        assert_eq!(
            stack.get_frame_info_by_layers(2),
            expected_frame_info_0
        );

        // the stack data layout:
        //
        // ```diagram
        //
        //              | 61     |
        //              | 59     |
        //              | 53     | <-- results from operands 3
        //              | 47     | <-- operands 2
        //              |--------|
        //       0d0136 | info 2 |
        //              |--------|
        //       0d0128 | 239    | <-- args 1, also local vars 1
        //              |--------|
        //              | info 1 |
        //              |--------|
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //       0d0072 | 251    |
        //       0d0064 | 241    | <-- local vars 0
        //       0d0056 | 233    |
        //       0d0048 | 229    | <-- args 0, also local vars 0
        //              |--------|
        //              | info 0 |
        // ```

        // check operands
        assert_eq!(stack.read_primitive_i32_u(stack.sp - 8, 0), 61);
        assert_eq!(stack.read_primitive_i32_u(stack.sp - 16, 0), 59);
        assert_eq!(stack.read_primitive_i32_u(stack.sp - 24, 0), 53);
        assert_eq!(stack.read_primitive_i32_u(stack.sp - 32, 0), 47);

        // check local variables start address
        assert_eq!(
            (421, local_start_2),
            stack.get_local_variable_list_index_and_start_address_by_layers(0),
        );

        assert_eq!(
            (419, local_start_1),
            stack.get_local_variable_list_index_and_start_address_by_layers(1),
        );

        assert_eq!(
            (401, local_start_0),
            stack.get_local_variable_list_index_and_start_address_by_layers(2),
        );

        // check local variables, frame 2
        // frame 2 has no local vars

        // check local variables, frame 1
        assert_eq!(stack.read_primitive_i32_u(local_start_1, 0), 239);

        // check local variables, frame 0
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 0), 229);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 8), 233);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 2 * 8), 241);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 3 * 8), 251);

        // tasks:
        //
        //   1. create function frame (f0) .. with 2 args + 2 local vars, 0 results.
        //   2. create block frame (f1) ..... with 1 args + 0 local vars, 2 results.
        //   3. create block frame (f2) ..... with 0 args + 0 local vars, 0 results.
        //   4. create function frame (f3) .. with 1 args + 0 local vars, 3 results.
        //   5. remove `f3`.
        // > 6. remove `f2` and `f1` at once.
        //   7. remove `f0`

        // note:
        //
        // although the signature of "frame 2" has no results, but the signature
        // of "frame 1" has two i32 results, and "frame 1" is the
        // target frame of removing, so 2 operands will be
        // carried to the top of stack

        let opt_return_pc1 = stack.remove_frames(1);
        assert_eq!(opt_return_pc1, None);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0112 |        |
        //       0d0104 | 61     |
        //       0d0096 | 59     | <-- results from operands 3 (takes 2 operands from top)
        //              |--------|
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //              |--------|
        //       0d0072 | 251    |
        //       0d0064 | 241    | <-- local vars 0
        //              |--------|
        //       0d0056 | 233    |
        //       0d0048 | 229    | <-- args 0, also local vars 0
        //              |--------| <-- fp0
        //              |        | <-- operands
        //              \--------/
        // ```

        // check FP and SP
        assert_eq!(stack.fp, fp0);
        assert_eq!(stack.sp, 112);

        // check operands
        assert_eq!(stack.read_primitive_i32_u(stack.sp - 8, 0), 61);
        assert_eq!(stack.read_primitive_i32_u(stack.sp - 2 * 8, 0), 59);
        assert_eq!(stack.read_primitive_i32_u(stack.sp - 3 * 8, 0), 43);
        assert_eq!(stack.read_primitive_i32_u(stack.sp - 4 * 8, 0), 41);

        // check local variables
        assert_eq!(
            (401, local_start_0),
            stack.get_local_variable_list_index_and_start_address_by_layers(0),
        );

        assert_eq!(stack.read_primitive_i32_u(local_start_0, 0), 229);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 8), 233);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 2 * 8), 241);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 3 * 8), 251);

        // tasks:
        //
        //   1. create function frame (f0) .. with 2 args + 2 local vars, 0 results.
        //   2. create block frame (f1) ..... with 1 args + 0 local vars, 2 results.
        //   3. create block frame (f2) ..... with 0 args + 0 local vars, 0 results.
        //   4. create function frame (f3) .. with 1 args + 0 local vars, 3 results.
        //   5. remove `f3`.
        //   6. remove `f2` and `f1` at once.
        // > 7. remove `f0`

        let opt_return_pc2 = stack.remove_frames(0);

        assert_eq!(
            opt_return_pc2,
            Some(ProgramCounter {
                module_index: 503,
                function_internal_index: 509,
                instruction_address: 521,
            })
        );

        // the stack data layout:
        //
        // ```diagram
        //
        // SP    0d0016 |        |
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/
        // ```

        // check FP and SP

        assert_eq!(stack.sp, 16);
        assert_eq!(stack.fp, 0);
    }

    #[test]
    fn test_reset_frame() {
        // tasks:
        //
        //  1. create function frame (f0) ... with 2 args + 2 local vars, 0 results
        //  2. reset f0
        //  3. reset f0
        //  4. create block frame (f1) ...... with 1 args + 1 local vars, 2 results
        //  5. reset f1
        //  6. reset f1
        //  7. create block frame (f2) ...... with 0 args + 0 local vars, 0 results
        //  8. reset f2
        //  9. crossing reset, reset to f1
        // 10. crossing reset, reset to f0

        let mut stack = NostdStack::new();

        // the arguments for the first functon call
        stack.push_i32_u(23);
        stack.push_i32_u(29);
        stack.push_i32_u(31);
        stack.push_i32_u(37);

        // the stack data layout:
        //
        // ```diagram
        //
        //              |        |
        //       0d0024 | 37     |
        //       0d0016 | 31     |
        //       0d0008 | 29     |
        // FP,SP 0d0000 | 23     |
        //              \--------/
        // ```

        // tasks:
        //
        // > 1. create function frame (f0) ... with 2 args + 2 local vars, 0 results
        //   2. reset f0
        //   3. reset f0
        //   4. create block frame (f1) ...... with 1 args + 1 local vars, 2 results
        //   5. reset f1
        //   6. reset f1
        //   7. create block frame (f2) ...... with 0 args + 0 local vars, 0 results
        //   8. reset f2
        //   9. crossing reset, reset to f1
        //  10. crossing reset, reset to f0

        stack
            .create_frame(
                2, // params count
                0, // results count
                401,
                16 + 16, // local vars length: 2 args + 2 locals
                Some(ProgramCounter {
                    module_index: 503,            // return module idx
                    function_internal_index: 509, // return function internal idx
                    instruction_address: 521,     // return instruction address
                }),
            )
            .unwrap();

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0080 |        |
        //       0d0072 | 0      |
        //       0d0064 | 0      | <-- local vars 0
        //              |--------|
        //       0d0056 | 37     |
        //       0d0048 | 31     | <-- args 0, also local vars 0
        //              |--------|
        //       0d0044 | 521    | return instruction address
        //       0d0040 | 509    | return function internal idx
        //       0d0036 | 503    | return module idx
        //       0d0032 | 32     | local vars length
        //       0d0028 | 401    | local vars list idx
        //       0d0024 | 2/0    | params/results count
        //       0d0020 | 16     | func FP
        // FP--> 0d0016 | 0      | prev FP
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/
        // ```

        // check local variables
        let (_, local_start_0) =
            stack.get_local_variable_list_index_and_start_address_by_layers(0);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 0), 31);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 8), 37);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 2 * 8), 0);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 3 * 8), 0);

        // update local variables
        stack.write_primitive_i32_u(local_start_0, 2 * 8, 211);
        stack.write_primitive_i32_u(local_start_0, 3 * 8, 223);

        // check local variables again
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 2 * 8), 211);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 3 * 8), 223);

        // add some operands
        stack.push_i32_u(41);
        stack.push_i32_u(43);
        stack.push_i32_u(47);
        stack.push_i32_u(53);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0112 |        |
        //       0d0104 | 53     |
        //       0d0096 | 47     |
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //              |--------|
        //       0d0072 | 223    |
        //       0d0064 | 211    | <-- local vars 0
        //              |--------|
        //       0d0056 | 37     |
        //       0d0048 | 31     | <-- args 0, also local vars 0
        //              |--------|
        // ```

        // check FP and SP
        let fp0 = 16;
        assert_eq!(stack.sp, 112);
        assert_eq!(stack.fp, fp0);

        // tasks:
        //
        //   1. create function frame (f0) ... with 2 args + 2 local vars, 0 results
        // > 2. reset f0
        //   3. reset f0
        //   4. create block frame (f1) ...... with 1 args + 1 local vars, 2 results
        //   5. reset f1
        //   6. reset f1
        //   7. create block frame (f2) ...... with 0 args + 0 local vars, 0 results
        //   8. reset f2
        //   9. crossing reset, reset to f1
        //  10. crossing reset, reset to f0

        let frame_type0 = stack.reset_frames(0);
        assert_eq!(frame_type0, FrameType::Function);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0080 |        |
        //       0d0072 | 0      |
        //       0d0064 | 0      | <-- local vars 0, RESET
        //              |--------|
        //       0d0056 | 53     |
        //       0d0048 | 47     | <-- args 0, also local vars 0, UPDATED
        //              |--------|
        // FP--> 0d0016 | info 0 |
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/
        // ```

        // check frame
        let frame_info_0 = stack.get_frame_info_by_layers(0);
        let expected_frame_info_0 = FrameInfo {
            address: fp0,
            info_data: &FrameInfoData {
                previous_frame_address: 0,
                function_frame_address: 16,
                params_count: 2,
                results_count: 0,
                local_variable_list_index: 401,
                local_variables_with_arguments_allocated_bytes: 32,
                return_module_index: 503,
                return_function_internal_index: 509,
                return_instruction_address: 521,
            },
        };
        assert_eq!(frame_info_0, expected_frame_info_0);

        // check local variables
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 0), 47); // updated
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 8), 53); // updated
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 2 * 8), 0); // reset
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 3 * 8), 0); // reset

        // update local variables (but keeps args unchange)
        stack.write_primitive_i32_u(local_start_0, 2 * 8, 227);
        stack.write_primitive_i32_u(local_start_0, 3 * 8, 229);

        // check local variables
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 0), 47);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 8), 53);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 2 * 8), 227);
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 3 * 8), 229);

        // add some operands
        stack.push_i32_u(59);
        stack.push_i32_u(61);

        // tasks:
        //
        //   1. create function frame (f0) ... with 2 args + 2 local vars, 0 results
        //   2. reset f0
        // > 3. reset f0
        //   4. create block frame (f1) ...... with 1 args + 1 local vars, 2 results
        //   5. reset f1
        //   6. reset f1
        //   7. create block frame (f2) ...... with 0 args + 0 local vars, 0 results
        //   8. reset f2
        //   9. crossing reset, reset to f1
        //  10. crossing reset, reset to f0

        // because there is no extra operands (there are only local vars and args),
        // so this reset should be optimizied.
        stack.reset_frames(0);

        // check local variables
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 0), 59); // updated
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 8), 61); // updated
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 2 * 8), 0); // reset
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 3 * 8), 0); // reset

        // prepare for the next reset
        //
        // update local variables
        stack.write_primitive_i32_u(local_start_0, 2 * 8, 233);
        stack.write_primitive_i32_u(local_start_0, 3 * 8, 239);

        // add some operands
        stack.push_i32_u(67);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0088 |        |
        //       0d0080 | 67     | <-- operands 0
        //       0d0072 | 239    |
        //       0d0064 | 233    | <-- local vars 0
        //              |--------|
        //       0d0056 | 61     |
        //       0d0048 | 59     | <-- args 0
        //              |--------|
        // ```

        // tasks:
        //
        //   1. create function frame (f0) ... with 2 args + 2 local vars, 0 results
        //   2. reset f0
        //   3. reset f0
        // > 4. create block frame (f1) ...... with 1 args + 1 local vars, 2 results
        //   5. reset f1
        //   6. reset f1
        //   7. create block frame (f2) ...... with 0 args + 0 local vars, 0 results
        //   8. reset f2
        //   9. crossing reset, reset to f1
        //  10. crossing reset, reset to f0

        stack
            .create_frame(
                1,
                2,
                419,
                8 + 8, // local variables length, 1 arg + 1 local
                None,
            )
            .unwrap();

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0128 |        |
        //       0d0120 | 0      | <-- local vars 1
        //       0d0112 | 67     | <-- args 1, also local vars 1
        //              |--------|
        //       0d0108 | 0      | return instruction address
        //       0d0104 | 0      | return function internal idx
        //       0d0100 | 0      | return module idx
        //       0d0096 | 16     | local vars length
        //       0d0092 | 419    | local vars list idx
        //       0d0088 | 1/2    | params/results count
        //       0d0084 | 16     | func FP
        // FP--> 0d0080 | 16     | prev FP
        //              |========| <-- fp1
        //       0d0072 | 239    |
        //       0d0064 | 233    | <-- local vars 0
        //              |--------|
        //       0d0056 | 61     |
        //       0d0048 | 59     | <-- args 0
        //              |--------|
        //       0d0016 | info 0 |
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/
        // ```

        // check FP and SP
        let fp1 = 80;
        assert_eq!(stack.fp, fp1);
        assert_eq!(stack.sp, 128);

        // check frame
        let frame_info_1 = stack.get_frame_info_by_layers(0);
        let expected_frame_info_1 = FrameInfo {
            address: fp1,
            info_data: &FrameInfoData {
                previous_frame_address: 16,
                function_frame_address: 16,
                params_count: 1,
                results_count: 2,
                local_variable_list_index: 419,
                local_variables_with_arguments_allocated_bytes: 16,
                return_module_index: 0,
                return_function_internal_index: 0,
                return_instruction_address: 0,
            },
        };
        assert_eq!(frame_info_1, expected_frame_info_1);

        // check local variables
        let (_, local_start_1) =
            stack.get_local_variable_list_index_and_start_address_by_layers(0);

        assert_eq!(stack.read_primitive_i32_u(local_start_1, 0), 67); // argument
        assert_eq!(stack.read_primitive_i32_u(local_start_1, 8), 0); // local variable

        // update and check local variables
        stack.write_primitive_i32_u(local_start_1, 8, 241);
        assert_eq!(stack.read_primitive_i32_u(local_start_1, 8), 241);

        // add some operands
        stack.push_i32_u(73);
        stack.push_i32_u(79);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0144 |        |
        //       0d0136 | 79     |
        //       0d0128 | 73     | <-- operands 1
        //       0d0120 | 241    | <-- local vars 1
        //       0d0112 | 67     | <-- args 1, also local vars 1
        //              |--------|
        // ```

        // tasks:
        //
        //   1. create function frame (f0) ... with 2 args + 2 local vars, 0 results
        //   2. reset f0
        //   3. reset f0
        //   4. create block frame (f1) ...... with 1 args + 1 local vars, 2 results
        // > 5. reset f1
        //   6. reset f1
        //   7. create block frame (f2) ...... with 0 args + 0 local vars, 0 results
        //   8. reset f2
        //   9. crossing reset, reset to f1
        //  10. crossing reset, reset to f0

        let frame_type1 = stack.reset_frames(0);
        assert_eq!(frame_type1, FrameType::Block);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0128 |        |
        //       0d0120 | 0      | <-- local vars 1
        //       0d0112 | 79     | <-- args 1, also local vars 1
        //              |--------|
        // FP--> 0d0080 | info 1 |
        //              |========| <-- fp1
        // ```

        // check FP and SP
        assert_eq!(stack.fp, fp1);
        assert_eq!(stack.sp, 128);

        // check local variables
        assert_eq!(stack.read_primitive_i32_u(local_start_1, 0), 79);
        assert_eq!(stack.read_primitive_i32_u(local_start_1, 8), 0);

        // tasks:
        //
        //   1. create function frame (f0) ... with 2 args + 2 local vars, 0 results
        //   2. reset f0
        //   3. reset f0
        //   4. create block frame (f1) ...... with 1 args + 1 local vars, 2 results
        //   5. reset f1
        // > 6. reset f1
        //   7. create block frame (f2) ...... with 0 args + 0 local vars, 0 results
        //   8. reset f2
        //   9. crossing reset, reset to f1
        //  10. crossing reset, reset to f0

        // add some operands
        stack.push_i32_u(83);

        let frame_type2 = stack.reset_frames(0);
        assert_eq!(frame_type2, FrameType::Block);

        // check FP and SP
        assert_eq!(stack.fp, fp1);
        assert_eq!(stack.sp, 128);

        // check local variables
        assert_eq!(stack.read_primitive_i32_u(local_start_1, 0), 83); // updated
        assert_eq!(stack.read_primitive_i32_u(local_start_1, 8), 0); // reset

        // prepare for next reset
        //
        // update local variables
        stack.write_primitive_i32_u(local_start_1, 8, 251);

        // check local variables
        assert_eq!(stack.read_primitive_i32_u(local_start_1, 0), 83);
        assert_eq!(stack.read_primitive_i32_u(local_start_1, 8), 251);

        // add some operands
        stack.push_i32_u(89);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0136 |        |
        //       0d0128 | 89     | <-- operands 1
        //       0d0120 | 251    | <-- local vars 1
        //       0d0112 | 83     | <-- args 1, also local vars 1
        //              |--------|
        // FP--> 0d0080 | info 1 |
        //              |========| <-- fp1
        // ```

        // tasks:
        //
        //    1. create function frame (f0) ... with 2 args + 2 local vars, 0 results
        //    2. reset f0
        //    3. reset f0
        //    4. create block frame (f1) ...... with 1 args + 1 local vars, 2 results
        //    5. reset f1
        //    6. reset f1
        // >  7. create block frame (f2) ...... with 0 args + 0 local vars, 0 results
        //    8. reset f2
        //    9. crossing reset, reset to f1
        //   10. crossing reset, reset to f0

        stack.create_frame(0, 0, 421, 0, None).unwrap();

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0168 |        |
        //              |--------|
        //       0d0164 | 0      | return instruction address
        //       0d0160 | 0      | return function internal idx
        //       0d0156 | 0      | return module idx
        //       0d0152 | 0      | local vars length
        //       0d0148 | 421    | local vars list idx
        //       0d0144 | 0/0    | params/results count
        //       0d0140 | 16     | func FP
        // FP--> 0d0136 | 80     | prev FP
        //              |========| <-- fp2
        //       0d0128 | 89     | <-- operands 1
        //       0d0120 | 251    | <-- local vars 1
        //       0d0112 | 83     | <-- args 1, also local vars 1
        //              |--------|
        //       0d0080 | info 1 |
        //              |========| <-- fp1
        // ```

        let fp2 = 136;

        // check FP and SP
        assert_eq!(stack.fp, fp2);
        assert_eq!(stack.sp, 168);

        // add some operands
        stack.push_i32_u(97);
        stack.push_i32_u(101);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0184 |        |
        //       0d0176 | 101    |
        //       0d0168 | 97     |
        //              |--------|
        // FP--> 0d0136 | info 2 |
        //              |========| <-- fp2
        //       0d0128 | 89     | <-- operands 1
        //       0d0120 | 251    | <-- local vars 1
        //       0d0112 | 83     | <-- args 1, also local vars 1
        // ```

        assert_eq!(stack.sp, 184);

        // tasks:
        //
        //    1. create function frame (f0) ... with 2 args + 2 local vars, 0 results
        //    2. reset f0
        //    3. reset f0
        //    4. create block frame (f1) ...... with 1 args + 1 local vars, 2 results
        //    5. reset f1
        //    6. reset f1
        //    7. create block frame (f2) ...... with 0 args + 0 local vars, 0 results
        // >  8. reset f2
        //    9. crossing reset, reset to f1
        //   10. crossing reset, reset to f0

        // the current frame has no local vars, neither args
        stack.reset_frames(0);

        // check SP
        assert_eq!(stack.fp, fp2);
        assert_eq!(stack.sp, 168);

        // add some operands, again
        stack.push_i32_u(103);
        stack.push_i32_u(107);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0184 |        |
        //       0d0176 | 107    |
        //       0d0168 | 103    |
        //              |--------|
        // FP--> 0d0136 | info 2 |
        //              |========| <-- fp2
        //       0d0128 | 89     | <-- operands 1
        //       0d0120 | 251    | <-- local vars 1
        //       0d0112 | 83     | <-- args 1, also local vars 1
        // ```

        // tasks:
        //
        //    1. create function frame (f0) ... with 2 args + 2 local vars, 0 results
        //    2. reset f0
        //    3. reset f0
        //    4. create block frame (f1) ...... with 1 args + 1 local vars, 2 results
        //    5. reset f1
        //    6. reset f1
        //    7. create block frame (f2) ...... with 0 args + 0 local vars, 0 results
        //    8. reset f2
        // >  9. crossing reset, reset to f1
        //   10. crossing reset, reset to f0

        // the params count of target frame is 1
        let frame_type3 = stack.reset_frames(1);
        assert_eq!(frame_type3, FrameType::Block);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0128 |        |
        //       0d0120 | 0      | <-- local vars 1
        //       0d0112 | 107    | <-- args 1 (value from operands 2)
        //              |--------|
        // FP--> 0d0080 | info 1 |
        //              |========| <-- fp1
        // ```

        assert_eq!(stack.fp, fp1);
        assert_eq!(stack.sp, 128);

        // check local variables
        assert_eq!(stack.read_primitive_i32_u(local_start_1, 0), 107);
        assert_eq!(stack.read_primitive_i32_u(local_start_1, 8), 0);

        // tasks:
        //
        //    1. create function frame (f0) ... with 2 args + 2 local vars, 0 results
        //    2. reset f0
        //    3. reset f0
        //    4. create block frame (f1) ...... with 1 args + 1 local vars, 2 results
        //    5. reset f1
        //    6. reset f1
        //    7. create block frame (f2) ...... with 0 args + 0 local vars, 0 results
        //    8. reset f2
        //    9. crossing reset, reset to f1
        // > 10. crossing reset, reset to f0

        // add some operands
        stack.push_i32_u(109);
        stack.push_i32_u(113);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0144 |        |
        //       0d0136 | 113    |
        //       0d0128 | 109    |
        //       0d0120 | 0      | <-- local vars 1
        //       0d0112 | 509    | <-- args 1 (value from operands 2)
        //              |--------|
        // ```

        // the params count of target frame (frame 0) is 2
        let frame_type4 = stack.reset_frames(1);
        assert_eq!(frame_type4, FrameType::Function);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0080 |        |
        //       0d0072 | 0      |
        //       0d0064 | 0      | <-- local vars 0
        //              |--------|
        //       0d0056 | 113    |
        //       0d0048 | 109    | <-- args 0 (value from operands 1)
        //              |--------|
        // FP--> 0d0016 | info 0 |
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/
        // ```

        assert_eq!(stack.fp, fp0);
        assert_eq!(stack.sp, 80);

        // check local variables
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 0), 109); // updated
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 8), 113); // updated
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 2 * 8), 0); // reset
        assert_eq!(stack.read_primitive_i32_u(local_start_0, 3 * 8), 0); // reset
    }
}
