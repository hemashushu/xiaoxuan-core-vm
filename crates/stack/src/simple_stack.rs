// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

// The stack consists of contiguous stack frames. Each frame contains
// frame information and operands (local variables and arguments).
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
// |-------------------------|                           |-------------------------|
// | arg 1 (local var idx 1) |                           | arg 1 (local var idx 1) |
// | arg 0 (local var idx 0) | <-- args from caller      | arg 0 (local var idx 0) |
// |-------------------------|                           |-------------------------|
// | return inst addr        |                           | 0                       | <-- 0
// | return func idx         |                           | 0                       | <-- 0
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
//
// Block frames are similar to function frames. They also have arguments and local variables,
// but block frames do not have a return program counter (PC).

// Notes on arguments and local variables:
//
// Arguments of functions and blocks are also part of the local variables.
//
// |                         |
// |-------------------------| <------
// | local var 2 (idx 4)     |     ^
// | local var 1 (idx 3)     |     |
// | local var 0 (idx 2)     |     local vars area
// |-------------------------|     |
// | arg 1 (local var idx 1) |     |
// | arg 0 (local var idx 0) |     v
// |-------------------------| <------
// | frame info              |
// \-------------------------/ <-- frame start
//
// The value of `local_variables_with_arguments_allocated_bytes` in the `FrameInfo` includes the length of
// the arguments from functions and blocks. For example, a function with two `i32` arguments
// and four `i32` local variables will have a `local_variables_with_arguments_allocated_bytes` value of
// (4 * 8 bytes) + (2 * 8 bytes) = 48 bytes. (Each `i32` occurs 8 bytes in the stack.)
//
// In some stack-based VMs, the arguments of a function or block are placed at the top
// of the stack, allowing direct access using instructions with implicit `pop` behavior
// (e.g., `eq_i32` or `add_i32`). However, the XiaoXuan Core VM does not guarantee this behavior.
// Local variables may be placed in a separate memory region. Always use "local_load/store_xxx"
// instructions to read/write arguments.

// Notes on `return_module_index` in `FrameInfo`:
//
// If the most significant bit (MSB) of `return_module_index` in `FrameInfo` is 1, it indicates
// that this is the first frame in the function call path. A stack may have multiple function
// call paths, as each callback creates a new function call path.
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

// Notes on `previous_frame_address` in `FrameInfo`:
//
// Since frame lengths are variable, each stack frame contains a `previous_frame_address` pointer.
// These pointers form a singly linked list of frames.
//
// Frame chain:
//
//             |             |
//             |-------------| <----------------------- stack.sp
//             | ...         |
//             | func FP     | -----------------\
//  func frame | previous FP | ---\             |
//             |-------------|    |     <-------/ <---- stack.fp
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

// Comprehensive stack approach:
//
// A better stack design consists of two separate stacks:
// - The frame information stack.
// - The operand stack (for both local variables and operands).
//
// ```diagram
//           |                 |             |                 |
//           |                 |             |                 |
//           |-----------------|             |                 |
//           |                 |             | operands 2      |
//           |                 |   locate    | local vars 2    |     frame 2
//           |                 | ----------> |-----------------| <-- start
//           | info 2          |             |                 |
//           |                 |             |                 |
//           |                 |             |                 |
// frame 2   |                 |             |                 |
// start --> |-----------------|             |                 |
//           |                 |             | operands 1      |
//           |                 |   locate    | local vars 1    |     frame 1
//           |                 | ----------> |-----------------| <-- start
//           | info 1          |             |                 |
//           |                 |             |                 |
//           |                 |             |                 |
// frame 1   |                 |             |                 |
// start --> |-----------------|             |                 |
//           |                 |             |                 |
//           |                 |             |                 |
//           | info 0          |             |                 |
//           |                 |   locate    | operands 0      |     frame 0
// stack     |                 | ----------> | local vars 0    | <-- start
// start --> \=================/             \=================/
//              info stack                      operand stack
// ```
//
// Separating the stack into two parts has advantages:
// An incorrect operation on the operand stack (e.g., incorrect local variable writes or operand pops)
// will not corrupt the information stack. This ensures the function can still return to the correct
// call path.
//
// The operand stack can be further divided into "local variables stack" and "operand stack",
// but for simplicity, the two-stack approach is sufficient.

use std::mem::size_of;

use anc_isa::OPERAND_SIZE_IN_BYTES;
use anc_memory::{
    memory_access::MemoryAccess, primitive_memory_access::PrimitiveMemoryAccess, MemoryError,
};

use crate::{
    stack::Stack, FrameType, ProgramCounter, StackError, StackErrorType, INIT_STACK_SIZE_IN_BYTES,
    MAX_STACK_SIZE_IN_BYTES,
};

pub struct SimpleStack {
    // The stack data is stored in a contiguous memory area.
    // The stack pointer (SP) points to the end of the stack,
    // while the frame pointer (FP) points to the start of the current frame.
    data: Vec<u8>,

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
    swap: Vec<u8>,
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

// for local variables load/store
impl MemoryAccess for SimpleStack {
    #[inline]
    fn get_ptr(&self, address: usize, offset_in_bytes: usize) -> *const u8 {
        unsafe { self.data[address..].as_ptr().add(offset_in_bytes) }
    }

    #[inline]
    fn get_mut_ptr(&mut self, address: usize, offset_in_bytes: usize) -> *mut u8 {
        unsafe { self.data[address..].as_mut_ptr().add(offset_in_bytes) }
    }
}

// for operands push/pop
impl PrimitiveMemoryAccess for SimpleStack {
    //
}

const SWAP_SIZE_IN_BYTES: usize = INIT_STACK_SIZE_IN_BYTES; // 64 KiB

impl Default for SimpleStack {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleStack {
    /// Creates a new `SimpleStack` instance with initialized stack and swap areas.
    pub fn new() -> Self {
        let data: Vec<u8> = vec![0u8; INIT_STACK_SIZE_IN_BYTES];
        let swap: Vec<u8> = vec![0u8; SWAP_SIZE_IN_BYTES];
        Self {
            data,
            swap,
            sp: 0,
            fp: 0,
        }
    }

    // /// Resets the stack to its initial state by clearing all data and resetting pointers.
    // pub fn reset(&mut self) {
    //     self.data = vec![0u8; INIT_STACK_SIZE_IN_BYTES];
    //     self.swap = vec![0u8; SWAP_SIZE_IN_BYTES];
    //     self.fp = 0;
    //     self.sp = 0;
    // }

    /// Returns the current capacity of the stack in bytes.
    fn get_stack_capacity_in_bytes(&self) -> usize {
        self.data.len()
    }

    /// Doubles the stack capacity if it does not exceed the maximum allowed size.
    /// Returns the new capacity or an error if the maximum size is exceeded.
    fn increase_stack_capacity(&mut self) -> Result<usize, StackError> {
        let new_size_in_bytes = self.get_stack_capacity_in_bytes() * 2;
        if new_size_in_bytes > MAX_STACK_SIZE_IN_BYTES {
            return Err(StackError::new(StackErrorType::StackOverflow));
        }

        self.data.resize(new_size_in_bytes, 0);
        Ok(new_size_in_bytes)
    }

    /// Ensures there is enough space for a new stack frame.
    /// If the stack pointer exceeds half the current capacity, the stack is resized.
    fn check_and_increase_stack_capacity(&mut self) -> Result<usize, StackError> {
        let stack_size_in_bytes = self.get_stack_capacity_in_bytes();
        let new_size_in_bytes = if self.sp > stack_size_in_bytes / 2 {
            self.increase_stack_capacity()?
        } else {
            stack_size_in_bytes
        };
        Ok(new_size_in_bytes)
    }

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

    /// Retrieves `FrameInfo` by the given reversed index.
    ///
    /// The reversed index specifies the depth of the frame relative to the current frame.
    /// For example:
    /// - `0` retrieves the current frame.
    /// - `1` retrieves the parent frame.
    /// - `n` retrieves the nth parent frame.
    ///
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
    ///         ;; - to get frame 3: reversed index = 0
    ///         ;; - to get frame 2: reversed index = 1
    ///         ;; - to get frame 0: reversed index = 3
    ///       end
    ///     end
    ///   end
    /// }
    /// ```
    ///
    /// Panics if the reversed index exceeds the available frames or crosses function boundaries.
    fn get_frame_info_by_reversed_index(&self, reversed_index: u16) -> FrameInfo {
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
        //          reversed idx 0        reversed idx 1        reversed idx 2
        // ```

        let mut remains = reversed_index;
        let mut fp = self.fp;
        let mut frame_info_data = self.get_frame_info_data(fp);
        let mut is_function_frame = fp == frame_info_data.function_frame_address as usize;

        while remains > 0 {
            if is_function_frame {
                // crossing function is not allowed
                panic!(
                    "The reversed index is out of bounds when retrieving stack frame information.
FP: {}, SP: {}, reversed index: {}.",
                    self.fp, self.sp, reversed_index
                )
            }

            fp = frame_info_data.previous_frame_address as usize;
            frame_info_data = self.get_frame_info_data(fp);
            is_function_frame = fp == frame_info_data.function_frame_address as usize;
            remains -= 1;
        }

        FrameInfo::new(fp, frame_info_data)
    }

    /// Retrieves `FrameInfo` for the current function frame.
    ///
    /// Function frames and block frames are distinguished by the `function_frame_address` field
    /// in `FrameInfoData`. This method identifies the function frame associated with the current frame.
    fn get_function_frame_info(&self) -> FrameInfo {
        // Function frames and block frames
        // --------------------------------
        // There are two types of stack frames: block frames and function frames.
        // Both types include the `FunctionFramePointer` field in the `FrameInfoData` structure.
        // This field makes it straightforward to identify the function frame associated with a given frame.
        //
        // Example 1: A block frame followed by a function frame
        //
        // ```diagram
        //               |         |
        //               |---------|
        //               | ...     |
        //   block frame | Func FP | ---\
        //               | prev FP |    |
        // current FP -> |---------|    |
        //               | ...     |    |
        //               | Func FP |    |
        //    func frame | prev FP |    |
        //               |---------| <--/
        //               | ...     |
        //               \---------/
        // ```
        //
        // Example 2: A standalone function frame
        //
        // ```diagram
        //               |         |
        //               |---------|
        //               | ...     |
        //    func frame | Func FP | ---\
        //               | prev FP |    |
        // current FP -> |---------| <--/
        //               | ...     |
        //               \---------/
        // ```
        //
        // Explanation:
        // - The `Func FP` field in the block frame points to the start of the function frame.
        // - The `prev FP` field in the block frame points to the previous frame (if any).
        // - The function frame contains its own `Func FP` field, which points to itself.

        let frame_info_data = self.get_frame_info_data(self.fp);
        if frame_info_data.function_frame_address as usize == self.fp {
            // the current frame itself is function frame.
            FrameInfo::new(self.fp, frame_info_data)
        } else {
            // the current frame is a block frame, get the function frame with the
            // `FunctionFramePointer` field in the `FrameInfoData`
            let function_fp = frame_info_data.function_frame_address as usize;
            let function_frame_info_data = self.get_frame_info_data(function_fp);
            FrameInfo::new(function_fp, function_frame_info_data)
        }
    }

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
Expected: SP > FP + frame info length + local vars length + popping length.
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

impl Stack for SimpleStack {
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
        self.check_and_increase_stack_capacity()?;

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
    fn remove_frames(&mut self, reversed_index: u16) -> Option<ProgramCounter> {
        let (sp, fp, is_function_frame, results_count, return_pc) = {
            let frame_info = self.get_frame_info_by_reversed_index(reversed_index);
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
    fn reset_frames(&mut self, reversed_index: u16) -> FrameType {
        let (
            is_function_frame,
            frame_addr,
            params_count,
            local_variables_with_arguments_allocated_bytes,
        ) = {
            let frame_info = self.get_frame_info_by_reversed_index(reversed_index);
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
        if (reversed_index == 0)
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
    /// identified by the given reversed index.
    ///
    /// The address is computed as `frame pointer + size of FrameInfoData`.
    /// This method always returns the calculated address, even if no local variables exist.
    fn get_frame_local_variable_list_index_and_start_address_by_reversed_index(
        &self,
        reversed_index: u16,
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

        let frame_info = self.get_frame_info_by_reversed_index(reversed_index);

        (
            frame_info.info_data.local_variable_list_index as usize,
            frame_info.address + size_of::<FrameInfoData>(),
        )
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

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use anc_isa::OPERAND_SIZE_IN_BYTES;
    use anc_memory::{memory_access::MemoryAccess, primitive_memory_access::PrimitiveMemoryAccess};

    use crate::{stack::Stack, FrameType, ProgramCounter, INIT_STACK_SIZE_IN_BYTES};

    use super::{FrameInfoData, SimpleStack};

    // helper functions for unit test
    impl SimpleStack {
        fn read_local_by_offset_i32(&self, reversed_index: u16, offset: usize) -> i32 {
            let (_, address) = self
                .get_frame_local_variable_list_index_and_start_address_by_reversed_index(
                    reversed_index,
                );

            self.read_primitive_i32_s(address, offset)
        }

        fn write_local_by_offset_i32(&mut self, reversed_index: u16, offset: usize, value: i32) {
            let (_, address) = self
                .get_frame_local_variable_list_index_and_start_address_by_reversed_index(
                    reversed_index,
                );

            self.write_primitive_i32_s(address, offset, value)
        }

        fn create_empty_frame(&mut self) {
            self.create_frame(0, 0, 0, 0, None).unwrap();
        }
    }

    #[test]
    fn test_stack_capacity() {
        let mut stack = SimpleStack::new();
        assert_eq!(stack.fp, 0);
        assert_eq!(stack.sp, 0);

        const FRAME_INFO_DATA_SIZE_IN_BYTES: usize = size_of::<FrameInfoData>();

        stack.create_empty_frame();
        assert_eq!(stack.sp, FRAME_INFO_DATA_SIZE_IN_BYTES);
        assert_eq!(stack.fp, 0);

        let repeat = INIT_STACK_SIZE_IN_BYTES / FRAME_INFO_DATA_SIZE_IN_BYTES;
        for _ in 0..repeat {
            stack.create_empty_frame();
        }

        assert_eq!(stack.fp, repeat * FRAME_INFO_DATA_SIZE_IN_BYTES);
        assert_eq!(stack.sp, (repeat + 1) * FRAME_INFO_DATA_SIZE_IN_BYTES);
    }

    #[test]
    fn test_push_pop_and_peek() {
        let mut stack = SimpleStack::new();

        // `pop_xxx` functions require a stack frame to work.
        stack.create_empty_frame();

        const FRAME_INFO_DATA_SIZE_IN_BYTES: usize = size_of::<FrameInfoData>();
        const INITIAL_SP: usize = FRAME_INFO_DATA_SIZE_IN_BYTES;

        // check push, peek and pop
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
        let mut stack = SimpleStack::new();

        // `pop_xxx` functions require a stack frame to work.
        stack.create_empty_frame();

        // check signed-extend
        stack.push_i32_s(0x8000_0000_u32 as i32);
        stack.push_i32_u(0x8000_0000_u32);
        assert_eq!(stack.peek_i64_s(), 0x0000_0000_8000_0000_u64 as i64);
        assert_eq!(stack.pop_i64_u(), 0x0000_0000_8000_0000_u64);
        assert_eq!(stack.peek_i64_s(), 0xffff_ffff_8000_0000_u64 as i64);
        assert_eq!(stack.pop_i64_u(), 0xffff_ffff_8000_0000_u64);
    }

    #[test]
    // #[should_panic]
    fn test_operand_stack_boundary_check() {
        let mut stack = SimpleStack::new();

        // `pop_xxx` functions require a stack frame to work.
        stack.create_empty_frame();

        stack.push_i32_u(11);
        stack.push_i32_u(13);

        assert_eq!(stack.pop_i32_u(), 13);
        assert_eq!(stack.pop_i32_u(), 11);

        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move ||
            // should panic
            stack.pop_i32_u());

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_frame_layer_boundary_check() {
        let mut stack = SimpleStack::new();

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

        // function frame
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

        // block frame
        stack.create_frame(0, 0, 2, 16, None).unwrap();
        stack.create_frame(0, 0, 3, 24, None).unwrap();
        stack.create_frame(0, 0, 4, 32, None).unwrap();

        let frame_info0 = stack.get_frame_info_by_reversed_index(0);
        assert_eq!(frame_info0.info_data.local_variable_list_index, 4);

        let frame_info1 = stack.get_frame_info_by_reversed_index(1);
        assert_eq!(frame_info1.info_data.local_variable_list_index, 3);

        let frame_info2 = stack.get_frame_info_by_reversed_index(2);
        assert_eq!(frame_info2.info_data.local_variable_list_index, 2);

        let frame_info3 = stack.get_frame_info_by_reversed_index(3);
        assert_eq!(frame_info3.info_data.local_variable_list_index, 1);

        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            // should panic
            stack.get_frame_info_by_reversed_index(4);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_floating_point_variants_check() {
        let mut stack = SimpleStack::new();

        // `pop_xxx` functions require a stack frame to work.
        stack.create_empty_frame();

        // check floating point variants
        stack.push_f32(std::f32::consts::PI);
        stack.push_f64(std::f64::consts::E);
        assert_eq!(stack.pop_f64().unwrap(), std::f64::consts::E);
        assert_eq!(stack.pop_f32().unwrap(), std::f32::consts::PI);

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
    fn test_host_address() {
        let mut stack = SimpleStack::new();

        stack.push_i32_u(11);
        stack.push_i64_u(13);
        stack.push_i32_u(17);
        stack.push_i64_u(19);

        let ptr0 = stack.get_ptr(0, 0) as u64;
        let ptr1 = stack.get_ptr(8, 0) as u64;
        let ptr2 = stack.get_ptr(16, 0) as u64;
        let ptr3 = stack.get_ptr(24, 0) as u64;

        let read_i64 = |addr: u64| -> u64 {
            let ptr = addr as *const u64;
            unsafe { std::ptr::read(ptr) }
        };

        let read_i32 = |addr: u64| -> u32 {
            let ptr = addr as *const u32;
            unsafe { std::ptr::read(ptr) }
        };

        // each operand occurs 8 bytes.
        assert_eq!(read_i32(ptr0), 11);
        assert_eq!(read_i64(ptr1), 13);
        assert_eq!(read_i32(ptr2), 17);
        assert_eq!(read_i64(ptr3), 19);
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

        let mut stack = SimpleStack::new();

        // simulate the arguments of first functon call
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
                73,
                16 + 16, // local vars len
                Some(ProgramCounter {
                    module_index: 83,            // return mod idx
                    function_internal_index: 79, // func idx
                    instruction_address: 89,     //return inst addr
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
        //       0d0044 | 89     | return inst addr
        //       0d0040 | 79     | return func idx
        //       0d0036 | 83     | return module idx
        //       0d0032 | 32     | local vars len
        //       0d0028 | 73     | local vars list idx
        //       0d0024 | 2/0    | params/results count
        //       0d0020 | 16     | func FP
        // FP--> 0d0016 | 0      | prev FP
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/
        // ```

        // check raw data
        assert_eq!(stack.read_primitive_i64_u(0, 0), 23);
        assert_eq!(stack.read_primitive_i64_u(8, 0), 29);
        // frame info data
        assert_eq!(stack.read_primitive_i32_u(16, 0), 0);
        assert_eq!(stack.read_primitive_i32_u(20, 0), 16);
        assert_eq!(stack.read_primitive_i32_u(24, 0), 2); // `(results count << 16) | (params count)`
        assert_eq!(stack.read_primitive_i32_u(28, 0), 73);
        assert_eq!(stack.read_primitive_i32_u(32, 0), 32);
        assert_eq!(stack.read_primitive_i32_u(36, 0), 83);
        assert_eq!(stack.read_primitive_i32_u(40, 0), 79);
        assert_eq!(stack.read_primitive_i32_u(44, 0), 89);
        // args
        assert_eq!(stack.read_primitive_i64_u(48, 0), 31);
        assert_eq!(stack.read_primitive_i64_u(56, 0), 37);
        // local vars
        assert_eq!(stack.read_primitive_i64_u(64, 0), 0);
        assert_eq!(stack.read_primitive_i64_u(72, 0), 0);

        // check status
        let fp0 = 16;

        assert_eq!(stack.sp, 80);
        assert_eq!(stack.fp, fp0);

        // check frame
        let f0_a = stack.get_frame_info_by_reversed_index(0);
        assert_eq!(f0_a.address, fp0);
        assert_eq!(
            f0_a.info_data,
            &FrameInfoData {
                previous_frame_address: 0,
                function_frame_address: fp0 as u32,
                params_count: 2,
                results_count: 0,
                local_variable_list_index: 73,
                local_variables_with_arguments_allocated_bytes: 32,
                return_module_index: 83,
                return_function_internal_index: 79,
                return_instruction_address: 89
            }
        );

        // `get_function_frame_info()` should points to the current frame
        let f0_b = stack.get_function_frame_info();
        assert_eq!(f0_a, f0_b);

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
        //              | info0  |
        // ```

        assert_eq!(
            (73, fp0 + size_of::<FrameInfoData>()),
            stack.get_frame_local_variable_list_index_and_start_address_by_reversed_index(0)
        );

        // local vars 0
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 31);
        assert_eq!(stack.read_local_by_offset_i32(0, 8), 37);
        assert_eq!(stack.read_local_by_offset_i32(0, 16), 0);
        assert_eq!(stack.read_local_by_offset_i32(0, 24), 0);

        // update local variables
        stack.write_local_by_offset_i32(0, 16, 211);
        stack.write_local_by_offset_i32(0, 24, 223);

        // local vars0
        assert_eq!(stack.read_local_by_offset_i32(0, 16), 211);
        assert_eq!(stack.read_local_by_offset_i32(0, 24), 223);

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
        //       0d0048 | 31     | <-- args 0 (local vars 0)
        //              |--------|
        //              | info0  |
        // ```

        // check status again
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

        stack.create_frame(1, 2, 97, 8, None).unwrap();

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0136 |        |
        //       0d0128 | 47     | <-- args 1 (local vars 1)
        //              |--------|
        //       0d0124 | 0      | return inst addr
        //       0d0120 | 0      | return func idx
        //       0d0116 | 0      | return module idx
        //       0d0112 | 8      | local vars len
        //       0d0108 | 97     | local vars list idx
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
        //       0d0048 | 31     | <-- args 0 (local vars 0)
        //              |--------|
        //              | info0  |
        // ```

        let fp1 = 96;

        assert_eq!(stack.fp, fp1);
        assert_eq!(stack.sp, 136);
        assert_eq!(stack.read_primitive_i32_u(128, 0), 47); // one operand has been moved to the top of stack
        assert_eq!(stack.read_primitive_i32_u(88, 0), 43); // the operands of the previous frame should has no change
        assert_eq!(stack.read_primitive_i32_u(80, 0), 41); // the operands of the previous frame should has no change

        let f1 = stack.get_frame_info_by_reversed_index(0);
        assert_eq!(f1.address, fp1);
        assert_eq!(
            f1.info_data,
            &FrameInfoData {
                previous_frame_address: 16,
                function_frame_address: fp0 as u32,
                params_count: 1,
                results_count: 2,
                local_variable_list_index: 97,
                local_variables_with_arguments_allocated_bytes: 8,
                return_module_index: 0,
                return_function_internal_index: 0,
                return_instruction_address: 0
            }
        );

        assert_eq!(stack.get_frame_info_by_reversed_index(1).address, fp0);

        // `get_function_frame_info()` should points to fp0.
        assert_eq!(
            stack.get_function_frame_info(),
            stack.get_frame_info_by_reversed_index(1)
        );

        // check local variables
        assert_eq!(
            (97, fp1 + size_of::<FrameInfoData>()),
            stack.get_frame_local_variable_list_index_and_start_address_by_reversed_index(0),
        );

        assert_eq!(
            (73, fp0 + size_of::<FrameInfoData>()),
            stack.get_frame_local_variable_list_index_and_start_address_by_reversed_index(1),
        );

        // local vars 1
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 47);
        // local vars 0
        assert_eq!(stack.read_local_by_offset_i32(1, 0), 31);
        assert_eq!(stack.read_local_by_offset_i32(1, 8), 37);
        assert_eq!(stack.read_local_by_offset_i32(1, 16), 211);
        assert_eq!(stack.read_local_by_offset_i32(1, 24), 223);

        // update current frame local vars
        stack.write_local_by_offset_i32(0, 0, 307);
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 307);

        // update previous frame local vars
        stack.write_local_by_offset_i32(1, 0, 311);
        stack.write_local_by_offset_i32(1, 8, 313);
        assert_eq!(stack.read_local_by_offset_i32(1, 0), 311);
        assert_eq!(stack.read_local_by_offset_i32(1, 8), 313);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0136 |        |
        //       0d0128 | 307    | <-- args 1 (local vars 1)
        //              |--------|
        //       0d0124 | 0      | return inst addr
        //       0d0120 | 0      | return func idx
        //       0d0116 | 0      | return module idx
        //       0d0112 | 8      | local vars len
        //       0d0108 | 97     | local vars list idx
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
        //       0d0056 | 313    |
        //       0d0048 | 311    | <-- args 0 (local vars 0)
        //              |--------|
        //              | info0  |
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

        stack.create_frame(0, 0, 701, 0, None).unwrap();

        let fp2 = fp1 + size_of::<FrameInfoData>() + 8; // 1 args in the 1st block frame

        // the stack data layout:
        //
        // ```diagram
        //
        // FP--> 0d0136 | info2  |
        //              |--------|
        //       0d0128 | 307    | <-- args 1 (local vars 1)
        //              |--------|
        //              | info1  |
        //              |--------| <-- fp1
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //       0d0072 | 223    |
        //       0d0064 | 211    | <-- local vars 0
        //       0d0056 | 313    |
        //       0d0048 | 311    | <-- args 0 (local vars 0)
        //              |--------|
        //              | info0  |
        // ```

        assert_eq!(stack.fp, fp2);
        assert_eq!(stack.sp, fp2 + size_of::<FrameInfoData>());
        assert_eq!(stack.read_primitive_i32_u(fp2 - 8, 0), 307); // the operands of the previous frame should has no change

        let f2 = stack.get_frame_info_by_reversed_index(0);
        assert_eq!(f2.address, fp2);
        assert_eq!(
            f2.info_data,
            &FrameInfoData {
                previous_frame_address: fp1 as u32,
                function_frame_address: fp0 as u32,
                params_count: 0,
                results_count: 0,
                local_variable_list_index: 701,
                local_variables_with_arguments_allocated_bytes: 0,
                return_module_index: 0,
                return_function_internal_index: 0,
                return_instruction_address: 0
            }
        );

        assert_eq!(stack.get_frame_info_by_reversed_index(1).address, fp1);
        assert_eq!(stack.get_frame_info_by_reversed_index(2).address, fp0);

        // `get_function_frame_info()` should points to fp0.
        assert_eq!(
            stack.get_function_frame_info(),
            stack.get_frame_info_by_reversed_index(2)
        );

        // check local variables
        assert_eq!(
            (701, fp2 + size_of::<FrameInfoData>()),
            stack.get_frame_local_variable_list_index_and_start_address_by_reversed_index(0),
        );

        assert_eq!(
            (97, fp1 + size_of::<FrameInfoData>()),
            stack.get_frame_local_variable_list_index_and_start_address_by_reversed_index(1),
        );

        assert_eq!(
            (73, fp0 + size_of::<FrameInfoData>()),
            stack.get_frame_local_variable_list_index_and_start_address_by_reversed_index(2),
        );

        // current frame (frame 2) has no local vars

        // local vars 1
        assert_eq!(stack.read_local_by_offset_i32(1, 0), 307);

        // local vars 0
        assert_eq!(stack.read_local_by_offset_i32(2, 0), 311);
        assert_eq!(stack.read_local_by_offset_i32(2, 8), 313);
        assert_eq!(stack.read_local_by_offset_i32(2, 16), 211);
        assert_eq!(stack.read_local_by_offset_i32(2, 24), 223);

        // update local vars
        stack.write_local_by_offset_i32(1, 0, 317);
        stack.write_local_by_offset_i32(2, 16, 331);
        stack.write_local_by_offset_i32(2, 24, 337);

        assert_eq!(stack.read_local_by_offset_i32(1, 0), 317);
        assert_eq!(stack.read_local_by_offset_i32(2, 16), 331);
        assert_eq!(stack.read_local_by_offset_i32(2, 24), 337);

        // add operands
        stack.push_i32_u(239);
        stack.push_i32_u(241);

        // the stack data layout:
        //
        // ```diagram
        //              | 241    |
        //              | 239    |
        //              |--------|
        // FP--> 0d0136 | info2  |
        //              |--------|
        //       0d0128 | 317    | <-- args 1 (local vars 1)
        //              |--------|
        //              | info1  |
        //              |--------| <-- fp1
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //       0d0072 | 337    |
        //       0d0064 | 331    | <-- local vars 0
        //       0d0056 | 313    |
        //       0d0048 | 311    | <-- args 0 (local vars 0)
        //              |--------|
        //              | info0  |
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
                709,
                8, // local vars len
                Some(ProgramCounter {
                    module_index: 113,            // ret mod idx
                    function_internal_index: 109, // func idx
                    instruction_address: 127,     // ret inst addr
                }),
            )
            .unwrap();

        // the stack data layout:
        //
        // ```diagram
        //
        //              | 241    | <-- args 3 (local vars 3)
        //              |--------|
        // FP-->        | info3  |
        //              |--------|
        //              | 239    | <-- operands 2
        //              |--------|
        //       0d0136 | info2  |
        //              |--------|
        //       0d0128 | 317    | <-- args 1 (local vars 1)
        //              |--------|
        //              | info1  |
        //              |--------|
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //       0d0072 | 337    |
        //       0d0064 | 331    | <-- local vars 0
        //       0d0056 | 313    |
        //       0d0048 | 311    | <-- args 0 (local vars 0)
        //              |--------|
        //              | info0  |
        // ```

        let fp3 = fp2 + size_of::<FrameInfoData>() + 8; // 1 operand in the 2nd block frame
        assert_eq!(stack.fp, fp3);
        assert_eq!(stack.sp, fp3 + size_of::<FrameInfoData>() + 8); // 1 args in the current frame

        let f3 = stack.get_frame_info_by_reversed_index(0);
        assert_eq!(f3.address, fp3);
        assert_eq!(
            f3.info_data,
            &FrameInfoData {
                previous_frame_address: fp2 as u32,
                function_frame_address: fp3 as u32,
                params_count: 1,
                results_count: 3,
                local_variable_list_index: 709,
                local_variables_with_arguments_allocated_bytes: 8,
                return_module_index: 113,
                return_function_internal_index: 109,
                return_instruction_address: 127
            }
        );

        // `get_function_frame_info()` should be updated,
        // it is now points to the current frame.
        let f3b = stack.get_function_frame_info();
        assert_eq!(f3, f3b);

        // check local variables
        assert_eq!(
            (709, fp3 + size_of::<FrameInfoData>()),
            stack.get_frame_local_variable_list_index_and_start_address_by_reversed_index(0),
        );

        // local vars 3
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 241);

        // update local vars
        stack.write_local_by_offset_i32(0, 0, 401);
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 401);

        // push some oparnds first
        stack.push_i32_u(251);
        stack.push_i32_u(257);
        stack.push_i32_u(263);

        // the stack data layout:
        //
        // ```diagram
        //
        //              |        |
        //              | 263    |
        //              | 257    |
        //              | 251    | <-- operands 3
        //              | 401    | <-- args 3 (local vars 3)
        //              |--------|
        // FP-->        | info3  |
        //              |--------|
        //              | 239    | <-- operands 2
        //              |--------|
        //       0d0136 | info2  |
        //              |--------|
        //       0d0128 | 317    | <-- args 1 (local vars 1)
        //              |--------|
        //              | info1  |
        //              |--------|
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //       0d0072 | 337    |
        //       0d0064 | 331    | <-- local vars 0
        //       0d0056 | 313    |
        //       0d0048 | 311    | <-- args 0 (local vars 0)
        //              |--------|
        //              | info0  |
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
                module_index: 113,
                function_internal_index: 109,
                instruction_address: 127,
            })
        );

        assert_eq!(stack.get_frame_info_by_reversed_index(0).address, fp2);
        assert_eq!(stack.get_frame_info_by_reversed_index(1).address, fp1);
        assert_eq!(stack.get_frame_info_by_reversed_index(2).address, fp0);

        // `get_function_frame_info()` should points to fp0
        assert_eq!(
            stack.get_function_frame_info(),
            stack.get_frame_info_by_reversed_index(2)
        );

        // check operands

        // the stack data layout:
        //
        // ```diagram
        //
        //              | 263    |
        //              | 257    |
        //              | 251    | <-- results from operands 3
        //              | 239    | <-- operands 2
        //              |--------|
        //       0d0136 | info2  |
        //              |--------|
        //       0d0128 | 317    | <-- args 1 (local vars 1)
        //              |--------|
        //              | info1  |
        //              |--------|
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //       0d0072 | 337    |
        //       0d0064 | 331    | <-- local vars 0
        //       0d0056 | 313    |
        //       0d0048 | 311    | <-- args 0 (local vars 0)
        //              |--------|
        //              | info0  |
        // ```

        assert_eq!(stack.read_primitive_i32_u(stack.sp - 8, 0), 263);
        assert_eq!(stack.read_primitive_i32_u(stack.sp - 16, 0), 257);
        assert_eq!(stack.read_primitive_i32_u(stack.sp - 24, 0), 251);
        assert_eq!(stack.read_primitive_i32_u(stack.sp - 32, 0), 239);

        // check local variables start address
        assert_eq!(
            (701, fp2 + size_of::<FrameInfoData>()),
            stack.get_frame_local_variable_list_index_and_start_address_by_reversed_index(0),
        );

        assert_eq!(
            (97, fp1 + size_of::<FrameInfoData>()),
            stack.get_frame_local_variable_list_index_and_start_address_by_reversed_index(1),
        );

        assert_eq!(
            (73, fp0 + size_of::<FrameInfoData>()),
            stack.get_frame_local_variable_list_index_and_start_address_by_reversed_index(2),
        );

        // frame 2 has no local vars

        // local vars 1
        assert_eq!(stack.read_local_by_offset_i32(1, 0), 317);
        // local vars 0
        assert_eq!(stack.read_local_by_offset_i32(2, 0), 311);
        assert_eq!(stack.read_local_by_offset_i32(2, 8), 313);
        assert_eq!(stack.read_local_by_offset_i32(2, 16), 331);
        assert_eq!(stack.read_local_by_offset_i32(2, 24), 337);

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
        //       0d0104 | 263    |
        //       0d0096 | 257    | <-- results from operands 3 (takes top 2, drops bottom 2)
        //              |--------|
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //              |--------|
        //       0d0072 | 337    |
        //       0d0064 | 331    | <-- local vars 0
        //              |--------|
        //       0d0056 | 313    |
        //       0d0048 | 311    | <-- args 0 (local vars 0)
        //              |--------| <-- fp0
        //              |        | <-- operands
        //              \--------/
        // ```

        assert_eq!(stack.get_frame_info_by_reversed_index(0).address, fp0);
        assert_eq!(stack.sp, 112);

        // check operands
        assert_eq!(stack.read_primitive_i32_u(104, 0), 263);
        assert_eq!(stack.read_primitive_i32_u(96, 0), 257);
        assert_eq!(stack.read_primitive_i32_u(88, 0), 43);
        assert_eq!(stack.read_primitive_i32_u(80, 0), 41);

        assert_eq!(
            (73, fp0 + size_of::<FrameInfoData>()),
            stack.get_frame_local_variable_list_index_and_start_address_by_reversed_index(0),
        );

        // local vars 0
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 311);
        assert_eq!(stack.read_local_by_offset_i32(0, 8), 313);
        assert_eq!(stack.read_local_by_offset_i32(0, 16), 331);
        assert_eq!(stack.read_local_by_offset_i32(0, 24), 337);

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
                module_index: 83,
                function_internal_index: 79,
                instruction_address: 89,
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

        let mut stack = SimpleStack::new();

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
                73,
                16 + 16, // local vars len
                Some(ProgramCounter {
                    instruction_address: 89,     //return inst addr
                    function_internal_index: 79, // func idx
                    module_index: 83,            // return mod idx
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
        //       0d0048 | 31     | <-- args 0 (local vars 0)
        //              |--------|
        //       0d0044 | 89     | return inst addr
        //       0d0040 | 79     | return func idx
        //       0d0036 | 83     | return module idx
        //       0d0032 | 32     | local vars len
        //       0d0028 | 73     | local vars list idx
        //       0d0024 | 2/0    | params/results count
        //       0d0020 | 16     | func FP
        // FP--> 0d0016 | 0      | prev FP
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/
        // ```

        // check local variables
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 31);
        assert_eq!(stack.read_local_by_offset_i32(0, 8), 37);
        assert_eq!(stack.read_local_by_offset_i32(0, 16), 0);
        assert_eq!(stack.read_local_by_offset_i32(0, 24), 0);

        // update local variables
        stack.write_local_by_offset_i32(0, 16, 101);
        stack.write_local_by_offset_i32(0, 24, 103);

        // check local variables
        assert_eq!(stack.read_local_by_offset_i32(0, 16), 101);
        assert_eq!(stack.read_local_by_offset_i32(0, 24), 103);

        // push some operands
        stack.push_i32_u(107);
        stack.push_i32_u(109);
        stack.push_i32_u(113);
        stack.push_i32_u(127);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0112 |        |
        //       0d0104 | 127    |
        //       0d0096 | 113    |
        //       0d0088 | 109    |
        //       0d0080 | 107    | <-- operands 0
        //              |--------|
        //       0d0072 | 103    |
        //       0d0064 | 101    | <-- local vars 0
        //              |--------|
        //       0d0056 | 37     |
        //       0d0048 | 31     | <-- args 0 (local vars 0)
        //              |--------|
        // ```

        // check SP
        assert_eq!(stack.sp, 112);

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
        //       0d0056 | 127    |
        //       0d0048 | 113    | <-- args 0 (local vars 0), UPDATED
        //              |--------|
        //       0d0044 | 89     | return inst addr
        //       0d0040 | 79     | return func idx
        //       0d0036 | 83     | return module idx
        //       0d0032 | 32     | local vars len
        //       0d0028 | 73     | local vars list idx
        //       0d0024 | 2/0    | params/results count
        //       0d0020 | 16     | func FP
        // FP--> 0d0016 | 0      | prev FP
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/
        // ```

        // check frame
        let f0 = stack.get_frame_info_by_reversed_index(0);
        assert_eq!(f0.address, 16);
        assert_eq!(
            f0.info_data,
            &FrameInfoData {
                previous_frame_address: 0,
                function_frame_address: 16,
                params_count: 2,
                results_count: 0,
                local_variable_list_index: 73,
                local_variables_with_arguments_allocated_bytes: 32,
                return_module_index: 83,
                return_function_internal_index: 79,
                return_instruction_address: 89
            }
        );

        // check local variables
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 113); // updated
        assert_eq!(stack.read_local_by_offset_i32(0, 8), 127); // updated
        assert_eq!(stack.read_local_by_offset_i32(0, 16), 0); // reset
        assert_eq!(stack.read_local_by_offset_i32(0, 24), 0); // reset

        // update local variables (keeps args unchange)
        stack.write_local_by_offset_i32(0, 16, 307);
        stack.write_local_by_offset_i32(0, 24, 311);

        // check local variables
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 113);
        assert_eq!(stack.read_local_by_offset_i32(0, 8), 127);
        assert_eq!(stack.read_local_by_offset_i32(0, 16), 307);
        assert_eq!(stack.read_local_by_offset_i32(0, 24), 311);

        stack.push_i32_u(114);
        stack.push_i32_u(128);

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

        assert_eq!(stack.read_local_by_offset_i32(0, 0), 114); // updated
        assert_eq!(stack.read_local_by_offset_i32(0, 8), 128); // updated
        assert_eq!(stack.read_local_by_offset_i32(0, 16), 0); // reset
        assert_eq!(stack.read_local_by_offset_i32(0, 24), 0); // reset

        //
        // prepare for the next reset
        //

        // add some operands and change local variables to

        stack.write_local_by_offset_i32(0, 16, 131);
        stack.write_local_by_offset_i32(0, 24, 137);
        stack.push_i32_u(139);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0088 |        |
        //       0d0080 | 139    | <-- operands 0
        //       0d0072 | 137    |
        //       0d0064 | 131    | <-- local vars 0
        //              |--------|
        //       0d0056 | 128    |
        //       0d0048 | 114    | <-- args 0
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

        stack.create_frame(1, 2, 97, 8 + 8, None).unwrap();

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0128 |        |
        //       0d0120 | 0      | <-- local vars 1
        //       0d0112 | 139    | <-- args 1 (local vars 1)
        //              |--------|
        //       0d0108 | 0      |
        //       0d0104 | 0      |
        //       0d0100 | 0      |
        //       0d0096 | 16     | local vars len
        //       0d0092 | 97     | local vars list idx
        //       0d0088 | 1/2    | params/results count
        //       0d0084 | 16     | func FP
        // FP--> 0d0080 | 16     | prev FP
        //              |========| <-- fp1
        //       0d0072 | 137    |
        //       0d0064 | 131    | <-- local vars 0
        //              |--------|
        //       0d0056 | 128    |
        //       0d0048 | 114    | <-- args 0
        //              |--------|
        //       0d0044 | 89     | return inst addr
        //       0d0040 | 79     | return func idx
        //       0d0036 | 83     | return module idx
        //       0d0032 | 32     | local vars len
        //       0d0028 | 73     | local vars list idx
        //       0d0024 | 2/0    | params/results count
        //       0d0020 | 16     | func FP
        //       0d0016 | 0      | prev FP
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/
        // ```

        assert_eq!(stack.fp, 80);
        assert_eq!(stack.sp, 128);

        // check local vars
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 139); // arg
        assert_eq!(stack.read_local_by_offset_i32(0, 8), 0); // local var

        // update local vars
        stack.write_local_by_offset_i32(0, 8, 401);
        assert_eq!(stack.read_local_by_offset_i32(0, 8), 401);

        // add operands
        stack.push_i32_u(149);
        stack.push_i32_u(151);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0144 |        |
        //       0d0136 | 151    |
        //       0d0128 | 149    | <-- operands 1
        //       0d0120 | 401    | <-- local vars 1
        //       0d0112 | 139    | <-- args 1 (local vars 1)
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
        //       0d0112 | 151    | <-- args 1 (local vars 1)
        //              |--------|
        //       0d0108 | 0      |
        //       0d0104 | 0      |
        //       0d0100 | 0      |
        //       0d0096 | 16     | local vars len
        //       0d0092 | 97     | local vars list idx
        //       0d0088 | 1/2    | params/results count
        //       0d0084 | 16     | func FP
        // FP--> 0d0080 | 16     | prev FP
        //              |========| <-- fp1
        // ```

        assert_eq!(stack.fp, 80);
        assert_eq!(stack.sp, 128);

        // check frame
        let f1 = stack.get_frame_info_by_reversed_index(0);
        assert_eq!(f1.address, 80);
        assert_eq!(
            f1.info_data,
            &FrameInfoData {
                previous_frame_address: 16,
                function_frame_address: 16,
                params_count: 1,
                results_count: 2,
                local_variable_list_index: 97,
                local_variables_with_arguments_allocated_bytes: 16,
                return_module_index: 0,
                return_function_internal_index: 0,
                return_instruction_address: 0
            }
        );

        // check local vars
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 151);
        assert_eq!(stack.read_local_by_offset_i32(0, 8), 0);

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

        stack.push_i32_u(152);

        let frame_type2 = stack.reset_frames(0);
        assert_eq!(frame_type2, FrameType::Block);

        // nothings changes
        assert_eq!(stack.fp, 80);
        assert_eq!(stack.sp, 128);

        // check local vars
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 152);
        assert_eq!(stack.read_local_by_offset_i32(0, 8), 0);

        // prepare for next reset
        // update local vars
        stack.write_local_by_offset_i32(0, 8, 601);

        // check local vars
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 152);
        assert_eq!(stack.read_local_by_offset_i32(0, 8), 601);

        // add some operands for preparing for the next reset
        stack.push_i32_u(157);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0136 |        |
        //       0d0128 | 157    | <-- operands 1
        //       0d0120 | 601    | <-- local vars 1
        //       0d0112 | 152    | <-- args 1 (local vars 1)
        //              |--------|
        //       0d0108 | 0      |
        //       0d0104 | 0      |
        //       0d0100 | 0      |
        //       0d0096 | 16     | local vars len
        //       0d0092 | 97     | local vars list idx
        //       0d0088 | 1/2    | params/results count
        //       0d0084 | 16     | func FP
        // FP--> 0d0080 | 16     | prev FP
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

        stack.create_frame(0, 0, 701, 0, None).unwrap();

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0168 |        |
        //              |--------|
        //       0d0164 | 0      |
        //       0d0160 | 0      |
        //       0d0156 | 0      |
        //       0d0152 | 0      | local vars len
        //       0d0148 | 701    | local vars list idx
        //       0d0144 | 0/0    | params/results count
        //       0d0140 | 16     | func FP
        // FP--> 0d0136 | 80     | prev FP
        //              |========| <-- fp2
        //       0d0128 | 157    | <-- operands 1
        //       0d0120 | 601    | <-- local vars 1
        //       0d0112 | 152    | <-- args 1 local vars 1
        //              |--------|
        //       0d0108 | 0      |
        //       0d0104 | 0      |
        //       0d0100 | 0      |
        //       0d0096 | 16     | local vars len
        //       0d0092 | 97     | local vars list idx
        //       0d0088 | 1/2    | params/results count
        //       0d0084 | 16     | func FP
        //       0d0080 | 16     | prev FP
        //              |========| <-- fp1
        // ```

        assert_eq!(stack.fp, 136);
        assert_eq!(stack.sp, 168);

        // add two operands
        stack.push_i32_u(167);
        stack.push_i32_u(173);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0184 |        |
        //       0d0176 | 173    |
        //       0d0168 | 167    |
        //              |--------|
        //       0d0164 | 0      |
        //       0d0160 | 0      |
        //       0d0156 | 0      |
        //       0d0152 | 0      | local vars len
        //       0d0148 | 701    | local vars list idx
        //       0d0144 | 0/0    | params/results count
        //       0d0140 | 16     | func FP
        // FP--> 0d0136 | 80     | prev FP
        //              |========| <-- fp2
        //       0d0128 | 157    | <-- operands 1
        //       0d0120 | 601    | <-- local vars 1
        //       0d0112 | 152    | <-- args 1 local vars 1
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

        // note:
        // the current frame has no local vars, neither args

        stack.reset_frames(0);

        // check SP
        assert_eq!(stack.fp, 136);
        assert_eq!(stack.sp, 168);

        // add two operands again
        stack.push_i32_u(503);
        stack.push_i32_u(509);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0184 |        |
        //       0d0176 | 509    |
        //       0d0168 | 503    |
        //              |--------|
        //       0d0164 | 0      |
        //       0d0160 | 0      |
        //       0d0156 | 0      |
        //       0d0152 | 0      | local vars len
        //       0d0148 | 701    | local vars list idx
        //       0d0144 | 0/0    | params/results count
        //       0d0140 | 16     | func FP
        // FP--> 0d0136 | 80     | prev FP
        //              |========| <-- fp2
        //       0d0128 | 157    | <-- operands 1
        //       0d0120 | 601    | <-- local vars 1
        //       0d0112 | 152    | <-- args 1 local vars 1
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
        //       0d0112 | 509    | <-- args 1 from operands 2
        //              |--------|
        //       0d0108 | 0      |
        //       0d0104 | 0      |
        //       0d0100 | 0      |
        //       0d0096 | 16     | local vars len
        //       0d0092 | 97     | local vars list idx
        //       0d0088 | 149    | func type
        //       0d0084 | 16     | func FP
        // FP--> 0d0080 | 16     | prev FP
        //              |========| <-- fp1
        // ```

        assert_eq!(stack.fp, 80);
        assert_eq!(stack.sp, 128);

        // check local vars
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 509);
        assert_eq!(stack.read_local_by_offset_i32(0, 8), 0);

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

        // add two operands
        stack.push_i32_u(181);
        stack.push_i32_u(191);

        // the stack data layout:
        //
        // ```diagram
        //
        // SP--> 0d0144 |        |
        //       0d0136 | 191    |
        //       0d0128 | 181    |
        //       0d0120 | 0      | <-- local vars 1
        //       0d0112 | 509    | <-- args 1 from operands 2
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
        //       0d0056 | 191    |
        //       0d0048 | 181    | <-- args 0 from operands 1
        //              |--------|
        //       0d0044 | 89     | return inst addr
        //       0d0040 | 79     | return func idx
        //       0d0036 | 83     | return module idx
        //       0d0032 | 32     | local vars len
        //       0d0028 | 73     | local vars list idx
        //       0d0024 | 2/0    | params/results count
        //       0d0020 | 16     | func FP
        // FP--> 0d0016 | 0      | prev FP
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/
        // ```

        assert_eq!(stack.fp, 16);
        assert_eq!(stack.sp, 80);

        // check local variables
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 181); // updated
        assert_eq!(stack.read_local_by_offset_i32(0, 8), 191); // updated
        assert_eq!(stack.read_local_by_offset_i32(0, 16), 0); // reset
        assert_eq!(stack.read_local_by_offset_i32(0, 24), 0); // reset
    }
}
