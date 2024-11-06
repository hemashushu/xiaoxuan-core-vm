// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::mem::size_of;

use ancvm_isa::OPERAND_SIZE_IN_BYTES;

use crate::{
    memory::Memory, thread_context::ProgramCounter, typed_memory::TypedMemory,
    STACK_FRAME_ENSURE_FREE_SIZE_IN_BYTES, STACK_FRAME_INCREMENT_SIZE_IN_BYTES,
};

pub struct Stack {
    data: Vec<u8>,

    // a temporary storage.
    //
    // when creating a new stack frame:
    //
    // 1. the arguments (i.e. the operands on the top of stack) are moved from stack to swap first,
    // 2. then the frame information is created,
    // 3. as well as allocating local variables area,
    // 4. finnaly the arguments are restored from the swap to stack.
    //
    // when exiting a stack frame:
    //
    // 1. the results (i.e. the operands on the top of stack) are also moved to swap,
    // 2. then remove frame and all operands that follows this frame
    // 3. the results are restored from the swap.
    swap: Vec<u8>,

    // the end position of the stack (a.k.a. SP)
    pub sp: usize,

    // the current frame position (a.k.a. FP)
    pub fp: usize,
}

// the complete VM stack consists of 3 separated stacks:
// - info stack
// - local variables stack
// - operand stack
//
//           |                 |                 |                 |                  |                 |
//           |                 |                 |                 |                  |                 |
//           |-----------------|                 |-----------------|                  |-----------------|
//           | info 3          |                 | local vars 3    |                  | operands 3      |
//           |                 |      locate     |                 |     frame 3      |                 |     frame 3
//           |                 | --------------> |-----------------| <-- start   /--> |-----------------| <-- start
//           |                 | ---\            |                 |             |    |                 |
//           |                 |    |            |                 |             |    |                 |
//           |                 |    \-----------=====================------------/    |                 |
// frame 3   |                 |                 |                 |    locate        |                 |
// start --> |-----------------|                 |                 |                  |                 |
//           | info 2          |                 | local vars 2    |                  | operands 2      |
//           |                 |      locate     |                 |     frame 2      |                 |     frame 2
//           |                 | --------------> |-----------------| <-- start   /--> |-----------------| <-- start
//           |                 | ---\            |                 |             |    |                 |
//           |                 |    |            |                 |             |    |                 |
//           |                 |    \-----------=====================------------/    |                 |
// frame 2   |                 |                 |                 |    locate        |                 |
// start --> |-----------------|                 |                 |                  |                 |
//           | info 1          |                 | local vars 1    |                  | operands 1      |
//           |-----------------|                 |-----------------|                  |-----------------|
// stack     | info 0          |                 | local vars 0    |                  | operands 0      |
// start --> \=================/                 \=================/                  \=================/
//
//                info stack                     local variables stack                   operands stack
//                -------------------------------------------------------------------------------------
//                                                  ternary stack
//
//
// dividing the stack into 3 separate parts has serveral advantages:
//
// 1. an incorrect local variable write (even if accessed by the external library/function),
//    or an incorrectly operands popped, will not destroy the 'info stack', the function still
//    returns to the correct calling path and can find out where the error occurred instantly.
//
// 2. when a function is called, its return values are placed just on top of the 'operand stack',
//    which helps avoiding the need to copy or move operands, and improving the runtime efficiency.
//    (note that the arguments are moved to the local vars stack when calling a function, instead of
//     staying in the operands stack)

// note:
// for simplicity, the current implementation of the VM stack is to
// combine 3 sub-stacks into 1 stack.
//
// the frame structure and the frame information:
//
// | ...                    |                                   | ...                    |
// | ...                    |                                   | ...                    |
// |========================|                                   |========================|
// | operand N              |                                   | operand N              |
// | operand 1              |                                   | operand 1              |
// | operand 0              | <-- operands                      | operand 0              |
// |------------------------|                                   |------------------------|
// | local 3                |                                   | local 3                |
// | local 2                | <-- local variable area           | local 2                |
// |------------------------|                                   |------------------------|
// | arg 1 (local 1)        |                                   | arg 1 (local 1)        |
// | arg 0 (local 0)        | <-- args from caller              | arg 0 (local 0)        |
// |------------------------|                                   |------------------------|
// | return inst addr       |                                   | 0                      | <-- 0
// | return func idx        |                                   | 0                      | <-- 0
// | return module idx      |                                   | 0                      | <-- 0
// | local vars alloc bytes |                                   | local vars alloc bytes |
// | local vars idx         |                                   | local vars index       |
// | params/results count   |                                   | params/results count   |
// | function FP            |                                   | function FP            |
// | previous FP            | <-- frame info                    | previous FP            |
// |========================| <-- FP                            |========================|
// | ...                    |                                   | ...                    |
// | ...                    |                                   | ...                    |
// \------------------------/ <-- stack start                   \------------------------/
//      function frame                                                block frame
//
// please note that:
//
// - function arguments are part of local variables.
//
//   |                 |
//   |-----------------| <------
//   | local 4         |     ^
//   | local 3         |     |
//   | local 2         |     local vars area
//   |-----------------|     |
//   | arg 1 (local 1) |     |
//   | arg 0 (local 0) |     v
//   |-----------------| <------
//   | frame info      |
//   \-----------------/ <-- frame start
//
// - block frame also has local variables (and arguments).
// - block frame has NO return PC.
// - the value of 'local_variables_allocate_bytes' includes the length of
//   the arguments of function and block. e.g.
//   a function with two i32 arguments and four i32 local variables, the
//   value of 'local_variables_allocate_bytes' is (4 * 4 bytes) + (2 * 4 bytes) = 24 bytes
// - if the MSB of 'return module index' is '1', it indicates that it's the starting frame of the
//   current function call.

// because the length of frame is vaiable, so there are a pointer 'previous frame pointer' in each
// stack frame, all these pointers form a single linked list.
//
// the chain of frames
//                                                      Stack.sp    Stack.bp
//             |             |                          --------    --------
//             |-------------| <-----------------------/           /
//             | ...         |                                    /
//             | func FP     | -----------------\                /
//  func frame | previous FP | ---\             | FFP           /
//             |-------------| <--|-------------/ <------------/
//             |             |    |
//             | ...         |    |
//             | func FP     | ---|-------------\
// block frame | previous FP | ---|--\          | all "FFP" are refered to the current function
//             |-------------| <--/  |          | frame start position (FP)
//             | ...         |       |          |
//             | func FP     | ------|----------|
// block frame | previous FP | ------|--\       |
//             |-------------| <-----/  |       |
//             | ...         |          |       |
//             | func FP     | ---------|-------|
// block frame | previous FP | ---------|--\    |
//             |-------------| <-------/   |    |
//             | ...         |             |    |
//             | func FP     | ------------|----| the value of FFP in the current function frame is the frame FP itself
//  func frame | previous FP | ---\        |    |
//             |-------------| <--|--------/ <--/
//             | ...         |   ...
//             |             |
//             \-------------/ <-- stack start

#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct FrameInfo {
    pub previous_frame_address: u32,         //--\  <-- addr low
    pub function_frame_address: u32,         //--/  8 bytes
    pub params_count: u16,                   //--\
    pub results_count: u16,                  //  |  8 bytes
    pub local_list_index: u32,               //--/
    pub local_variables_allocate_bytes: u32, //--\
    pub return_module_index: u32,            //--/  8 bytes
    pub return_function_internal_index: u32, //--\  8 bytes
    pub return_instruction_address: u32,     //--/  <-- addr high
}

#[derive(Debug, PartialEq)]
pub struct FramePack<'a> {
    pub address: usize,
    pub frame_info: &'a FrameInfo,
}

impl<'a> FramePack<'a> {
    pub fn new(address: usize, frame_info: &'a FrameInfo) -> Self {
        Self {
            address,
            frame_info,
        }
    }
}

// for local variables load/store
impl Memory for Stack {
    #[inline]
    fn get_ptr(&self, address: usize) -> *const u8 {
        self.data[address..].as_ptr()
    }

    #[inline]
    fn get_mut_ptr(&mut self, address: usize) -> *mut u8 {
        self.data[address..].as_mut_ptr()
    }
}

impl TypedMemory for Stack {
    //
}

/// implements stack general functions
///
/// - push/pop/peek
/// - create frame
/// - remove frame
/// - reset frame
impl Stack {
    pub fn new(init_size_in_bytes: usize) -> Self {
        let data: Vec<u8> = vec![0u8; init_size_in_bytes];
        let swap: Vec<u8> = vec![0u8; init_size_in_bytes];
        Self {
            data,
            swap,
            sp: 0,
            fp: 0,
        }
    }

    pub fn reset(&mut self) {
        self.fp = 0;
        self.sp = 0;
    }

    fn get_capacity_in_bytes(&self) -> usize {
        self.data.len()
    }

    fn resize(&mut self, new_size_in_bytes: usize) -> usize {
        self.data.resize(new_size_in_bytes, 0);
        new_size_in_bytes
    }

    pub fn ensure_stack_free_space(&mut self) {
        // check the capacity of the stack to make sure
        // there is enough space for a stack frame.
        // as well as increasing the capacity in the specified
        // increment (the default value is 64 KiB).

        let len = self.get_capacity_in_bytes();
        if len - self.sp < STACK_FRAME_ENSURE_FREE_SIZE_IN_BYTES {
            let new_size_in_bytes = len + STACK_FRAME_INCREMENT_SIZE_IN_BYTES;
            self.resize(new_size_in_bytes);
        }
    }

    pub fn drop_(&mut self) {
        self.sp -= OPERAND_SIZE_IN_BYTES;
    }

    pub fn duplicate(&mut self) {
        let src = self.data[self.sp - OPERAND_SIZE_IN_BYTES..].as_ptr();
        let dst = self.data[self.sp..].as_mut_ptr();
        unsafe {
            std::ptr::copy(src, dst, OPERAND_SIZE_IN_BYTES);
        }
        self.sp += OPERAND_SIZE_IN_BYTES;
    }

    pub fn push_i32_s(&mut self, value: i32) {
        // note:
        // signed-extends i32 to i64
        self.write_i64_s(self.sp, value as i64);
        self.sp += OPERAND_SIZE_IN_BYTES;
    }

    pub fn push_i32_u(&mut self, value: u32) {
        // note:
        // unsigned-extends u32 to u64
        self.write_i64_u(self.sp, value as u64);
        self.sp += OPERAND_SIZE_IN_BYTES;
    }

    pub fn push_i64_s(&mut self, value: i64) {
        self.write_i64_s(self.sp, value);
        self.sp += OPERAND_SIZE_IN_BYTES;
    }

    pub fn push_i64_u(&mut self, value: u64) {
        self.write_i64_u(self.sp, value);
        self.sp += OPERAND_SIZE_IN_BYTES;
    }

    pub fn push_f32(&mut self, value: f32) {
        self.write_f32(self.sp, value);
        self.sp += OPERAND_SIZE_IN_BYTES;
    }

    pub fn push_f64(&mut self, value: f64) {
        self.write_f64(self.sp, value);
        self.sp += OPERAND_SIZE_IN_BYTES;
    }

    // note:
    // this is unsafe function.
    // the caller should write data to stack immediately after calling this function.
    //
    // e.g.
    //
    // ```rust
    // let ptr = stack.push_from_memory();
    // memory.load_64(address, ptr);
    // ```
    //
    // this function does not interpret the value of the data, so
    // it's supposed to fast than reading the value of data from
    // memory and push the value onto stack. the same purpose for
    // the function 'pop_to_memory'.
    pub fn push_operand_from_memory(&mut self) -> *mut u8 {
        let ptr = self.get_mut_ptr(self.sp);
        self.sp += OPERAND_SIZE_IN_BYTES;
        ptr
    }

    pub fn push_operands_from_memory(&mut self, count: usize) -> *mut u8 {
        let ptr = self.get_mut_ptr(self.sp);
        self.sp += OPERAND_SIZE_IN_BYTES * count;
        ptr
    }

    pub fn peek_i32_s(&self) -> i32 {
        self.read_i32_s(self.sp - OPERAND_SIZE_IN_BYTES)
    }

    pub fn peek_i32_u(&self) -> u32 {
        self.read_i32_u(self.sp - OPERAND_SIZE_IN_BYTES)
    }

    pub fn peek_i64_s(&self) -> i64 {
        self.read_i64_s(self.sp - OPERAND_SIZE_IN_BYTES)
    }

    pub fn peek_i64_u(&self) -> u64 {
        self.read_i64_u(self.sp - OPERAND_SIZE_IN_BYTES)
    }

    pub fn peek_f32(&self) -> f32 {
        self.read_f32(self.sp - OPERAND_SIZE_IN_BYTES)
    }

    pub fn peek_f64(&self) -> f64 {
        self.read_f64(self.sp - OPERAND_SIZE_IN_BYTES)
    }

    pub fn pop_i32_s(&mut self) -> i32 {
        self.operands_stack_bounds_check(1);

        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_i32_s(self.sp)
    }

    pub fn pop_i32_u(&mut self) -> u32 {
        self.operands_stack_bounds_check(1);

        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_i32_u(self.sp)
    }

    pub fn pop_i64_s(&mut self) -> i64 {
        self.operands_stack_bounds_check(1);

        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_i64_s(self.sp)
    }

    pub fn pop_i64_u(&mut self) -> u64 {
        self.operands_stack_bounds_check(1);

        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_i64_u(self.sp)
    }

    pub fn pop_f32(&mut self) -> f32 {
        self.operands_stack_bounds_check(1);

        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_f32(self.sp)
    }

    pub fn pop_f64(&mut self) -> f64 {
        self.operands_stack_bounds_check(1);

        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_f64(self.sp)
    }

    // note:
    // this is an unsafe function.
    // the caller should write data to memory immediately after calling this function.
    //
    // e.g.
    //
    // ```rust
    // let ptr = stack.pop_to_memory();
    // memory.store_64(ptr, address);
    // ```
    pub fn pop_operand_to_memory(&mut self) -> *const u8 {
        self.operands_stack_bounds_check(1);

        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.get_ptr(self.sp)
    }

    pub fn pop_operands(&mut self, count: usize) -> &[u8] {
        self.operands_stack_bounds_check(count);

        let length = count * OPERAND_SIZE_IN_BYTES;
        self.sp -= length;
        &self.data[self.sp..]
    }

    // because the info stack, local variable stack and operands stack are combined as
    // one stack, so there is a 'bound check' when perform a regular 'pop' operation,
    // but when the entry function is finish, there is no 'frame' on the stack, so the
    // 'bound check' operation will be failed.
    pub fn pop_operands_without_bound_check(&mut self, count: usize) -> &[u8] {
        let length = count * OPERAND_SIZE_IN_BYTES;
        self.sp -= length;
        &self.data[self.sp..]
    }

    // bounds check
    #[inline]
    fn operands_stack_bounds_check(&self, remove_operands_count: usize) {
        #[cfg(feature = "bounds_check")]
        {
            let frame_info = self.read_frame_info(self.fp);
            let local_variables_allocate_bytes = frame_info.local_variables_allocate_bytes as usize;

            if self.sp - (remove_operands_count * OPERAND_SIZE_IN_BYTES)
                < self.fp + size_of::<FrameInfo>() + local_variables_allocate_bytes
            {
                panic!(
                    "Out of the bound of the current operand stack frame.
FP: {}, frame info length in bytes: {}, local variables area length in bytes: {},
SP: {}, expect popping length in bytes: {}",
                    self.fp,
                    size_of::<FrameInfo>(),
                    local_variables_allocate_bytes,
                    self.sp,
                    remove_operands_count * OPERAND_SIZE_IN_BYTES
                )
            }
        }
    }

    /**
     * block frames are nested, the parameter 'reversed_index' is
     * the depth of the frame what you want to get, it is relate to the current frame.
     * e.g., if you want to get the stack frame which at the end of the stack
     * (i.e. the latest frame, the current frame), the index should be 0.
     * If the blocks has 3 nested level, and you want to get the outside most one, the index should be 2.
     *
     * ```bytecode
     * block        ;; reversed index = 2
     *   block      ;; reversed index = 1
     *     block    ;; reversed index = 0
     *       ;; the current stack frame
     *     end
     *   end
     * end
     * ```
     *
     * note:
     * - if the index value exceeds the number of stack frames, an uncatchable
     *   exception is raised.
     * - block frames include function frame, that is, a function stack frame
     *   is also a block stack frame.
     *
     * return the FP and BlockFrame.
     */
    pub fn get_frame_pack(&self, reversed_index: u16) -> FramePack {
        // the FP chain:
        //
        //       |         |           |         |           |         |
        //       |---------|           |---------|           |---------|
        //       | ...     |           | ...     |           | ...     |
        //       | prev FP |----\      | prev FP |----\      | prev FP |----> ...
        // FP -> |---------|    \----> |---------|    \----> |---------|
        //       | ...     |           | ...     |           | ...     |
        //       \---------/           \---------/           \---------/
        //
        //       reversed idx 0        reversed idx 1        reversed idx 2

        let mut remains = reversed_index;
        let mut fp = self.fp;
        let mut frame_info = self.read_frame_info(fp);

        while remains > 0 {
            fp = frame_info.previous_frame_address as usize;
            frame_info = self.read_frame_info(fp);
            remains -= 1;
        }

        FramePack::new(fp, frame_info)
    }

    /**
     * get the current function stack frame
     *
     * return the FFP (FP of function) and FuncFrame.
     */
    pub fn get_function_frame_pack(&self) -> FramePack {
        // the FFP pointer
        //
        // case 1:
        //
        //             |         |
        //             |---------|
        //             | ...     |
        // block frame | Func FP | ---\
        //             | prev FP |    |
        //       FP -> |---------|    |
        //             | ...     |    |
        //             | Func FP |    |
        //  func frame | prev FP |    |
        //             |---------| <--/
        //             | ...     |
        //             \---------/
        //
        // case 2:
        //
        //             |         |
        //             |---------|
        //             | ...     |
        //  func frame | Func FP | ---\
        //             | prev FP |    |
        //       FP -> |---------| <--/
        //             | ...     |
        //             \---------/
        //
        // in the case 2, the function FP point to the frame itself when the frame
        // is "function stack frame".

        let frame_info = self.read_frame_info(self.fp);
        if frame_info.function_frame_address as usize == self.fp {
            // the current frame is function frame
            FramePack::new(self.fp, frame_info)
        } else {
            // the current frame is block frame
            let function_fp = frame_info.function_frame_address as usize;
            let function_frame_info = self.read_frame_info(function_fp);
            FramePack::new(function_fp, function_frame_info)
        }
    }

    /// get the local variables area start address
    ///
    /// note that the address is simply calculated by 'FP + the size of Frame', so
    /// even if there is no local variable area in the current frame,
    /// this function always return the calculated address.
    pub fn get_local_variables_start_address(&self, reversed_index: u16) -> usize {
        // |            |
        // | ...        | <-- pure local vars
        // |------------|
        // | ...        | <-- args
        // |------------| <-- local vars start
        // | frame info |
        // |------------| <-- FP
        // | ...        |
        // \------------/

        // let frame_info = self.read_frame_info(self.fp);
        // let function_fp = frame_info.function_frame_address;
        let FramePack {
            address: fp,
            frame_info: _,
        } = self.get_frame_pack(reversed_index);

        self.get_frame_local_variables_start_address(fp)
    }

    pub fn get_frame_local_variables_start_address(&self, fp: usize) -> usize {
        fp + size_of::<FrameInfo>()
    }

    fn read_frame_info(&self, addr: usize) -> &FrameInfo {
        let ptr = self.data[addr..].as_ptr();
        unsafe { &*(ptr as *const FrameInfo) }
    }

    /// mapping a structure to the specified address.
    /// the caller must write value of each field through the return object.
    ///
    /// e.g.
    /// let frame_info = get_writable_frame_info(0xff);
    /// frame_info.x = ...;
    /// frame_info.y = ...;
    fn get_writable_frame_info(&mut self, addr: usize) -> &mut FrameInfo {
        let ptr = self.data[addr..].as_mut_ptr();
        unsafe { &mut *(ptr as *mut FrameInfo) }
    }

    /// move the specified number of operands to the swap are
    fn move_operands_to_swap(&mut self, operands_count: usize) {
        if operands_count == 0 {
            return;
        }

        // PATCH:
        // since there is no stack frame before the first function running, so
        // can not check the operands stack bounds
        if self.fp != 0 {
            self.operands_stack_bounds_check(operands_count);
        }

        let count_in_bytes = operands_count * OPERAND_SIZE_IN_BYTES;

        #[cfg(feature = "bounds_check")]
        {
            if self.sp < self.fp + count_in_bytes {
                panic!(
                    "Not enough operands for returning values.
FP: {}, SP: {}, expect returning operands: {} (in bytes: {})",
                    self.fp, self.sp, operands_count, count_in_bytes
                )
            }
        }

        let offset = self.sp - count_in_bytes;

        // memory copy
        let src = self.data[offset..].as_ptr();
        let dst = self.swap.as_mut_ptr();
        unsafe {
            std::ptr::copy(src, dst, count_in_bytes);
        }

        // update the SP
        self.sp = offset;
    }

    fn restore_operands_from_swap(&mut self, operands_count: usize) {
        if operands_count == 0 {
            return;
        }

        let count_in_bytes = operands_count * OPERAND_SIZE_IN_BYTES;

        // memory copy
        let src = self.swap.as_ptr();
        let dst = self.data[self.sp..].as_mut_ptr();
        unsafe {
            std::ptr::copy(src, dst, count_in_bytes);
        }

        // update the SP
        self.sp += count_in_bytes;
    }

    /// when creating a 'block frame', the value of parameter 'opt_return_pc' should be NONE.
    pub fn create_frame(
        &mut self,
        params_count: u16,
        results_count: u16,
        local_list_index: u32,
        local_variables_allocate_bytes: u32,
        opt_return_pc: Option<ProgramCounter>,
    ) {
        // move the arguments to swap first
        self.move_operands_to_swap(params_count as usize);

        let previous_fp = self.fp;
        let new_fp = self.sp;

        let function_fp = if opt_return_pc.is_some() {
            // ensure the free space
            self.ensure_stack_free_space();

            // in the function frame, the 'Function FP' point to the current frame FP itself.
            new_fp as u32
        } else {
            // in the block frame, the 'Function FP' is inherited from the previous frame.
            let frame_pack = self.get_frame_pack(0);
            frame_pack.frame_info.function_frame_address
        };

        // create new frame at the NEW_FP (i.e., the previous SP)
        let frame_info = self.get_writable_frame_info(new_fp);

        // write values
        frame_info.previous_frame_address = previous_fp as u32;
        frame_info.function_frame_address = function_fp;
        frame_info.params_count = params_count;
        frame_info.results_count = results_count;
        frame_info.local_list_index = local_list_index;

        frame_info.local_variables_allocate_bytes = local_variables_allocate_bytes;

        if let Some(return_pc) = opt_return_pc {
            frame_info.return_module_index = return_pc.module_index as u32;
            frame_info.return_function_internal_index = return_pc.function_internal_index as u32;
            frame_info.return_instruction_address = return_pc.instruction_address as u32;
        } else {
            frame_info.return_module_index = 0;
            frame_info.return_function_internal_index = 0;
            frame_info.return_instruction_address = 0;
        }

        // update sp and fp
        self.sp += size_of::<FrameInfo>();
        self.fp = new_fp;

        // allocate and clear the local variables area
        //
        // note that the value of 'local_variables_allocate_bytes' includes the length of arguments
        //
        //  |                 |
        //  |-----------------| <---------------
        //  |local 4          |    ^         ^  'local_variables_allocate_bytes -
        //  |local 3          |    | local   |   params_count * OPERAND_SIZE_IN_BYTES'
        //  |local 2          |    | vars    v
        //  |-----------------|----|-----------
        //  |arg 1 (local 1)  |    |         ^   params_count * OPERAND_SIZE_IN_BYTES'
        //  |arg 0 (local 0)  |    v         v
        //  |-----------------| <---------------
        //  |                 |
        //  \-----------------/ <-- stack start

        // restore the arguments from swap
        self.restore_operands_from_swap(params_count as usize);

        // clear the local variables area
        let local_variables_allocate_bytes_without_args =
            local_variables_allocate_bytes as usize - params_count as usize * OPERAND_SIZE_IN_BYTES;

        self.data[self.sp..(self.sp + local_variables_allocate_bytes_without_args)].fill(0);
        self.sp += local_variables_allocate_bytes_without_args;
    }

    /// remove the specified frame and all frames that follows this frame.
    ///
    /// return:
    /// - None, when the target frame is block frame
    /// - Some(ProgramCounter), when the target frame is function frame
    pub fn remove_frames(&mut self, reversed_index: u16) -> Option<ProgramCounter> {
        let (sp, fp, is_function_frame, results_count, return_pc) = {
            let frame_pack = self.get_frame_pack(reversed_index);
            let is_function_frame =
                frame_pack.frame_info.function_frame_address as usize == frame_pack.address;
            (
                frame_pack.address, // current frame start address
                frame_pack.frame_info.previous_frame_address as usize, // previous FP
                is_function_frame,
                frame_pack.frame_info.results_count,
                ProgramCounter {
                    instruction_address: frame_pack.frame_info.return_instruction_address as usize,
                    function_internal_index: frame_pack.frame_info.return_function_internal_index
                        as usize,
                    module_index: frame_pack.frame_info.return_module_index as usize,
                },
            )
        };

        // move the specified number of operands to swap
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

    /// reset the specified function frame or block frame:
    /// - initialize all local variables (if exists) to value 0
    /// - remove all oprands which follow the local variable area
    /// - remove all frames which follow the current frame
    /// - moves the specified number of operands to the top of stack
    ///
    /// return TRUE if the target frame is function frame.
    pub fn reset_frames(&mut self, reversed_index: u16) -> bool {
        let (is_function_frame, frame_addr, params_count, local_variables_allocate_bytes) = {
            let frame_pack = self.get_frame_pack(reversed_index);
            (
                (frame_pack.address == frame_pack.frame_info.function_frame_address as usize),
                frame_pack.address,
                frame_pack.frame_info.params_count,
                frame_pack.frame_info.local_variables_allocate_bytes as usize,
            )
        };

        // optimized for the looping in the current frame, when:
        // - the specified frame is the current frame (the top most frame)
        // - there is no other operands than the local vars and parameters on the top of stack
        //
        // diagram:
        //
        //                 operands that are about to
        //                 become arguments in recur                 move to args
        //                                   |                     /----------------\
        //                                   |                     |                |
        //        |            |       |     v      | <-- SP |     |      |         |
        //        |            |       | results    |        | x x x      |         |
        // SP --> |------------|       |------------|        |------------| <-- SP  |
        //        | local vars |       | local vars |        | local vars |         |
        //        |------------|  ==>  |------------|  ==>   |------------|         |
        //        | arg 1      |       | arg 1      |        | arg 1      |         |
        //        | arg 0      |       | arg 0      |        | arg 0      | <-------/
        //        |------------|       |------------|        |------------|
        //        | info       |       | info       |        | info       |
        // FP --> |============|       |============|        |============| <-- FP
        //        |            |       |            |        |            |
        //        \------------/       \------------/        \------------/
        //
        // when the conditions above are met, then there is no need to move the
        // argurments to the 'swap' and back again, but simply reset the local
        // variables to '0' and move the results to args.

        let params_bytes = params_count as usize * OPERAND_SIZE_IN_BYTES;
        if (reversed_index == 0)
            && (self.sp
                == self.fp + size_of::<FrameInfo>() + local_variables_allocate_bytes + params_bytes)
        {
            // move the results to params
            unsafe {
                std::ptr::copy(
                    self.data[(self.sp - params_bytes)..].as_ptr(),
                    self.data[self.fp + size_of::<FrameInfo>()..].as_mut_ptr(),
                    params_bytes,
                );
            }

            self.sp -= params_bytes;

            // reset the local vars
            let local_variables_addr_start = self.fp + size_of::<FrameInfo>() + params_bytes;

            let local_variables_allocate_bytes_without_args =
                local_variables_allocate_bytes - params_bytes;

            // let dst = self.data[local_variables_addr_start..].as_mut_ptr();
            // unsafe {
            //     std::ptr::write_bytes(dst, 0, local_variables_allocate_bytes_without_args);
            // }

            self.data[local_variables_addr_start
                ..(local_variables_addr_start + local_variables_allocate_bytes_without_args)]
                .fill(0);

            return is_function_frame;
        }

        // move the specified number of operands to swap
        self.move_operands_to_swap(params_count as usize);

        // remove all operands and frames which follows the current frame
        //
        // |            |
        // | ...        |
        // |------------| <-- move SP to here
        // | local vars |
        // |------------|
        // | frame info |
        // |------------| <-- frame addr, move FP to here
        // | ...        |
        // \------------/

        self.fp = frame_addr;
        self.sp = frame_addr + size_of::<FrameInfo>();

        // note that 'local_variables_allocate_bytes' includes the size of arguments
        //
        //  |                 |
        //  |-----------------| <---------------
        //  |local 4          |    ^         ^  'local_variables_allocate_bytes -
        //  |local 3          |    | local   |   params_count * OPERAND_SIZE_IN_BYTES'
        //  |local 2          |    | vars    v
        //  |-----------------|----|-----------
        //  |arg 1 (local 1)  |    |         ^   params_count * OPERAND_SIZE_IN_BYTES'
        //  |arg 0 (local 0)  |    v         v
        //  |-----------------| <---------------
        //  |                 |
        //  \-----------------/ <-- stack start

        // restore parameters from swap
        self.restore_operands_from_swap(params_count as usize);

        let local_variables_allocate_bytes_without_args =
            local_variables_allocate_bytes - params_count as usize * OPERAND_SIZE_IN_BYTES;

        // re-initialize the local variable area
        //
        // ```rust
        // let dst = self.data[self.sp..].as_mut_ptr();
        // unsafe {
        //     std::ptr::write_bytes(dst, 0, local_variables_allocate_bytes_without_args);
        // }
        // ```

        self.data[self.sp..(self.sp + local_variables_allocate_bytes_without_args)].fill(0);

        self.sp += local_variables_allocate_bytes_without_args;

        is_function_frame
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use ancvm_isa::OPERAND_SIZE_IN_BYTES;

    use crate::{
        memory::Memory,
        stack::{FrameInfo, Stack},
        thread_context::ProgramCounter,
        typed_memory::TypedMemory,
        INIT_STACK_SIZE_IN_BYTES, STACK_FRAME_ENSURE_FREE_SIZE_IN_BYTES,
        STACK_FRAME_INCREMENT_SIZE_IN_BYTES,
    };

    // private functions for helping unit test
    impl Stack {
        fn read_local_by_offset_i32(&self, reversed_index: u16, offset: usize) -> i32 {
            self.read_i32_s(self.get_local_variables_start_address(reversed_index) + offset)
        }

        fn write_local_by_offset_i32(&mut self, reversed_index: u16, offset: usize, value: i32) {
            self.write_i32_s(
                self.get_local_variables_start_address(reversed_index) + offset,
                value,
            )
        }

        // because there is bounds check in the 'pop..' operations, the 'pop..' functions
        // will panic without a frame.
        fn create_empty_frame(&mut self) {
            let frame_size_in_bytes = size_of::<FrameInfo>();
            // do not write any data, just move the FP and SP

            // let dst = self.data.as_mut_ptr();
            // unsafe {
            //     std::ptr::write_bytes(dst, 0, frame_size_in_bytes);
            // }

            self.fp = 0;
            self.sp = frame_size_in_bytes;
        }
    }

    #[test]
    fn test_stack_capacity() {
        let mut stack = Stack::new(INIT_STACK_SIZE_IN_BYTES);

        // check the initial size
        assert_eq!(stack.get_capacity_in_bytes(), INIT_STACK_SIZE_IN_BYTES);
        assert_eq!(stack.sp, 0);
        assert_eq!(stack.fp, 0);

        // because there is bounds check in the 'pop..' operations, the 'pop..' functions
        // will panic without a frame.
        stack.create_empty_frame();

        const FRAME_SIZE_IN_BYTES: usize = size_of::<FrameInfo>();
        assert_eq!(stack.sp, FRAME_SIZE_IN_BYTES);
        assert_eq!(stack.fp, 0);

        stack.ensure_stack_free_space();
        assert_eq!(stack.get_capacity_in_bytes(), INIT_STACK_SIZE_IN_BYTES);

        // fill operands
        for i in 0..(STACK_FRAME_ENSURE_FREE_SIZE_IN_BYTES / OPERAND_SIZE_IN_BYTES) {
            stack.push_i64_u(i as u64);
        }

        assert_eq!(
            stack.sp,
            STACK_FRAME_ENSURE_FREE_SIZE_IN_BYTES + FRAME_SIZE_IN_BYTES
        );

        assert_eq!(stack.get_capacity_in_bytes(), INIT_STACK_SIZE_IN_BYTES);
        stack.ensure_stack_free_space();
        assert_eq!(
            stack.get_capacity_in_bytes(),
            STACK_FRAME_INCREMENT_SIZE_IN_BYTES * 2
        );
    }

    #[test]
    fn test_push_pop_and_peek() {
        let mut stack = Stack::new(INIT_STACK_SIZE_IN_BYTES);

        // because there is bounds check in the 'pop..' operations, the 'pop..' functions
        // will panic without a frame.
        stack.create_empty_frame();

        const FRAME_SIZE_IN_BYTES: usize = size_of::<FrameInfo>();
        const INIT_SP: usize = FRAME_SIZE_IN_BYTES;

        // check push, peek and pop
        stack.push_i32_u(11);
        stack.push_i64_u(13);
        stack.push_f32(std::f32::consts::PI);
        stack.push_f64(std::f64::consts::E);

        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES * 4 + INIT_SP);
        assert_eq!(stack.peek_f64(), std::f64::consts::E);
        assert_eq!(stack.pop_f64(), std::f64::consts::E);
        assert_eq!(stack.pop_f32(), std::f32::consts::PI);

        assert_eq!(stack.peek_i64_u(), 13);
        assert_eq!(stack.pop_i64_u(), 13);
        assert_eq!(stack.pop_i32_u(), 11);
        assert_eq!(stack.sp, INIT_SP);

        // check duplicate
        stack.push_i32_u(17);
        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES + INIT_SP);
        stack.duplicate();
        assert_eq!(stack.peek_i32_u(), 17);
        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES * 2 + INIT_SP);

        // check drop
        stack.push_i32_u(19);
        stack.push_i32_u(23);
        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES * 4 + INIT_SP);
        stack.drop_();
        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES * 3 + INIT_SP);
        assert_eq!(stack.peek_i32_u(), 19);
    }

    #[test]
    fn test_operand_signed_extend() {
        let mut stack = Stack::new(INIT_STACK_SIZE_IN_BYTES);

        // because there is bounds check in the 'pop..' operations, the 'pop..' functions
        // will panic without a frame.
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
    fn test_operand_stack_bounds() {
        let mut stack = Stack::new(INIT_STACK_SIZE_IN_BYTES);
        stack.create_empty_frame();

        stack.push_i32_u(11);
        stack.push_i32_u(13);

        assert_eq!(stack.pop_i32_u(), 13);
        assert_eq!(stack.pop_i32_u(), 11);

        // stack.pop_i32_u(); // will panic

        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || stack.pop_i32_u());

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_host_address() {
        let mut stack = Stack::new(INIT_STACK_SIZE_IN_BYTES);

        stack.push_i32_u(11);
        stack.push_i64_u(13);
        stack.push_i32_u(17);
        stack.push_i64_u(19);

        let ptr0 = stack.get_ptr(0) as u64;
        let ptr1 = stack.get_ptr(8) as u64;
        let ptr2 = stack.get_ptr(16) as u64;
        let ptr3 = stack.get_ptr(24) as u64;

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
        // frames:
        //
        // 1. create function frame (f0),   with 2 args + 2 local vars, 0 results
        // 2. create block frame (f1),      with 1 args + 0 local vars, 2 results
        // 3. create block frame (f2),      with 0 args + 0 local vars, 0 results
        // 4. create function frame (f3),   with 1 args + 0 local vars, 3 results
        //
        // 5. remove f3
        // 6. remove f2+f1
        // 7. remove f0

        let mut stack = Stack::new(INIT_STACK_SIZE_IN_BYTES);

        stack.push_i32_u(23);
        stack.push_i32_u(29);
        stack.push_i32_u(31);
        stack.push_i32_u(37);

        // the current layout
        //
        //              |        |
        //       0d0024 | 37     |
        //       0d0016 | 31     |
        //       0d0008 | 29     |
        // FP,SP 0d0000 | 23     |
        //              \--------/

        //
        // create function frame (frame 0)
        //

        stack.create_frame(
            2, // params count
            0, // results count
            73,
            16 + 16, // local vars len
            Some(ProgramCounter {
                module_index: 83,            // ret mod idx
                function_internal_index: 79, // func idx
                instruction_address: 89,     //ret inst addr
            }),
        );

        // the current layout
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

        // check raw data
        assert_eq!(stack.read_i64_u(0), 23);
        assert_eq!(stack.read_i64_u(8), 29);
        // frame infos
        assert_eq!(stack.read_i32_u(16), 0);
        assert_eq!(stack.read_i32_u(20), 16);
        assert_eq!(stack.read_i32_u(24), 2); // = (results count << 16) | (params count)
        assert_eq!(stack.read_i32_u(28), 73);
        assert_eq!(stack.read_i32_u(32), 32);
        assert_eq!(stack.read_i32_u(36), 83);
        assert_eq!(stack.read_i32_u(40), 79);
        assert_eq!(stack.read_i32_u(44), 89);
        // args
        assert_eq!(stack.read_i64_u(48), 31);
        assert_eq!(stack.read_i64_u(56), 37);
        // local vars
        assert_eq!(stack.read_i64_u(64), 0);
        assert_eq!(stack.read_i64_u(72), 0);

        // check status
        let fp0 = 16;

        assert_eq!(stack.sp, 80);
        assert_eq!(stack.fp, fp0);

        // check frame
        let f0 = stack.get_frame_pack(0);
        assert_eq!(f0.address, fp0);
        assert_eq!(
            f0.frame_info,
            &FrameInfo {
                previous_frame_address: 0,
                function_frame_address: fp0 as u32,
                params_count: 2,
                results_count: 0,
                local_list_index: 73,
                local_variables_allocate_bytes: 32,
                return_module_index: 83,
                return_function_internal_index: 79,
                return_instruction_address: 89
            }
        );

        // the value of 'get_function_frame_pack()' should points to the current frame
        let f0b = stack.get_function_frame_pack();
        assert_eq!(f0, f0b);

        // check local variables

        // the current layout (partial)
        //
        //       0d0072 | 0      |
        //       0d0064 | 0      | <-- local vars 0
        //              |--------|
        //       0d0056 | 37     |
        //       0d0048 | 31     | <-- args 0
        //              |--------|
        //              | info0  |

        assert_eq!(
            stack.get_local_variables_start_address(0),
            fp0 + size_of::<FrameInfo>()
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

        // the current layout (partial)
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

        // check status again
        assert_eq!(stack.fp, fp0);
        assert_eq!(stack.sp, 104);

        //
        // create block frame (frame 1)
        //

        stack.create_frame(1, 2, 97, 8, None);

        // the current layout
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

        let fp1 = 96;

        assert_eq!(stack.fp, fp1);
        assert_eq!(stack.sp, 136);
        assert_eq!(stack.read_i32_u(128), 47); // one operand has been moved to the top of stack
        assert_eq!(stack.read_i32_u(88), 43); // the operands of the previous frame should has no change
        assert_eq!(stack.read_i32_u(80), 41); // the operands of the previous frame should has no change

        let f1 = stack.get_frame_pack(0);
        assert_eq!(f1.address, fp1);
        assert_eq!(
            f1.frame_info,
            &FrameInfo {
                previous_frame_address: 16,
                function_frame_address: fp0 as u32,
                params_count: 1,
                results_count: 2,
                local_list_index: 97,
                local_variables_allocate_bytes: 8,
                return_module_index: 0,
                return_function_internal_index: 0,
                return_instruction_address: 0
            }
        );

        assert_eq!(stack.get_frame_pack(1).address, fp0);

        // the value of 'get_function_frame_pack()' should points to fp0
        assert_eq!(stack.get_function_frame_pack(), stack.get_frame_pack(1));

        // check local variables
        assert_eq!(
            stack.get_local_variables_start_address(0),
            fp1 + size_of::<FrameInfo>()
        );

        assert_eq!(
            stack.get_local_variables_start_address(1),
            fp0 + size_of::<FrameInfo>()
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

        // the current layout
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

        //
        // create block frame (frame 2)
        //

        stack.create_frame(0, 0, 701, 0, None);

        let fp2 = fp1 + size_of::<FrameInfo>() + 8; // 1 args in the 1st block frame

        // the current layout
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

        assert_eq!(stack.fp, fp2);
        assert_eq!(stack.sp, fp2 + size_of::<FrameInfo>());
        assert_eq!(stack.read_i32_u(fp2 - 8), 307); // the operands of the previous frame should has no change

        let f2 = stack.get_frame_pack(0);
        assert_eq!(f2.address, fp2);
        assert_eq!(
            f2.frame_info,
            &FrameInfo {
                previous_frame_address: fp1 as u32,
                function_frame_address: fp0 as u32,
                params_count: 0,
                results_count: 0,
                local_list_index: 701,
                local_variables_allocate_bytes: 0,
                return_module_index: 0,
                return_function_internal_index: 0,
                return_instruction_address: 0
            }
        );

        assert_eq!(stack.get_frame_pack(1).address, fp1);
        assert_eq!(stack.get_frame_pack(2).address, fp0);

        // the value of 'get_function_frame_pack()' should points to fp0
        assert_eq!(stack.get_function_frame_pack(), stack.get_frame_pack(2));

        // check local variables
        assert_eq!(
            stack.get_local_variables_start_address(0),
            fp2 + size_of::<FrameInfo>()
        );

        assert_eq!(
            stack.get_local_variables_start_address(1),
            fp1 + size_of::<FrameInfo>()
        );

        assert_eq!(
            stack.get_local_variables_start_address(2),
            fp0 + size_of::<FrameInfo>()
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

        //
        // create func frame (frame 3)
        //

        stack.create_frame(
            1, // params count
            3, // results count
            709,
            8, // local vars len
            Some(ProgramCounter {
                module_index: 113,            // ret mod idx
                function_internal_index: 109, // func idx
                instruction_address: 127,     // ret inst addr
            }),
        );

        // the current layout
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

        let fp3 = fp2 + size_of::<FrameInfo>() + 8; // 1 operand in the 2nd block frame
        assert_eq!(stack.fp, fp3);
        assert_eq!(stack.sp, fp3 + size_of::<FrameInfo>() + 8); // 1 args in the current frame

        let f3 = stack.get_frame_pack(0);
        assert_eq!(f3.address, fp3);
        assert_eq!(
            f3.frame_info,
            &FrameInfo {
                previous_frame_address: fp2 as u32,
                function_frame_address: fp3 as u32,
                params_count: 1,
                results_count: 3,
                local_list_index: 709,
                local_variables_allocate_bytes: 8,
                return_module_index: 113,
                return_function_internal_index: 109,
                return_instruction_address: 127
            }
        );

        // the value of 'get_function_frame_pack()' should be updated, it should points to the current frame
        let f3b = stack.get_function_frame_pack();
        assert_eq!(f3, f3b);

        assert_eq!(stack.get_frame_pack(1).address, fp2);
        assert_eq!(stack.get_frame_pack(2).address, fp1);
        assert_eq!(stack.get_frame_pack(3).address, fp0);

        // check local variables
        assert_eq!(
            stack.get_local_variables_start_address(0),
            fp3 + size_of::<FrameInfo>()
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

        // the current layout
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

        //
        // remove the current frame (frame 3)
        //

        let opt_return_pc0 = stack.remove_frames(0);

        assert_eq!(
            opt_return_pc0,
            Some(ProgramCounter {
                module_index: 113,
                function_internal_index: 109,
                instruction_address: 127,
            })
        );

        assert_eq!(stack.get_frame_pack(0).address, fp2);
        assert_eq!(stack.get_frame_pack(1).address, fp1);
        assert_eq!(stack.get_frame_pack(2).address, fp0);

        // the value of 'get_function_frame_pack()' should points to fp0
        assert_eq!(stack.get_function_frame_pack(), stack.get_frame_pack(2));

        // check operands

        // the current layout (partial)
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

        assert_eq!(stack.read_i32_u(stack.sp - 8), 263);
        assert_eq!(stack.read_i32_u(stack.sp - 16), 257);
        assert_eq!(stack.read_i32_u(stack.sp - 24), 251);
        assert_eq!(stack.read_i32_u(stack.sp - 32), 239);

        // check local variables start address
        assert_eq!(
            stack.get_local_variables_start_address(0),
            fp2 + size_of::<FrameInfo>()
        );

        assert_eq!(
            stack.get_local_variables_start_address(1),
            fp1 + size_of::<FrameInfo>()
        );

        assert_eq!(
            stack.get_local_variables_start_address(2),
            fp0 + size_of::<FrameInfo>()
        );

        // frame 2 has no local vars

        // local vars 1
        assert_eq!(stack.read_local_by_offset_i32(1, 0), 317);
        // local vars 0
        assert_eq!(stack.read_local_by_offset_i32(2, 0), 311);
        assert_eq!(stack.read_local_by_offset_i32(2, 8), 313);
        assert_eq!(stack.read_local_by_offset_i32(2, 16), 331);
        assert_eq!(stack.read_local_by_offset_i32(2, 24), 337);

        //
        // remove the parent frame (frame2 + frame 1)
        //

        // note:
        //
        // although the type of 'frame 2' has no results, but the type
        // of 'frame 1' has two 'int' results, and 'frame 1' is the
        // target frame of removing, so there are 2 operands will be
        // carried to the top of stack
        //

        let opt_return_pc1 = stack.remove_frames(1);
        assert_eq!(opt_return_pc1, None);

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

        assert_eq!(stack.get_frame_pack(0).address, fp0);
        assert_eq!(stack.sp, 112);

        // check operands
        assert_eq!(stack.read_i32_u(104), 263);
        assert_eq!(stack.read_i32_u(96), 257);
        assert_eq!(stack.read_i32_u(88), 43);
        assert_eq!(stack.read_i32_u(80), 41);

        assert_eq!(
            stack.get_local_variables_start_address(0),
            fp0 + size_of::<FrameInfo>()
        );

        // local vars 0
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 311);
        assert_eq!(stack.read_local_by_offset_i32(0, 8), 313);
        assert_eq!(stack.read_local_by_offset_i32(0, 16), 331);
        assert_eq!(stack.read_local_by_offset_i32(0, 24), 337);

        //
        // remove the last frame
        //

        let opt_return_pc2 = stack.remove_frames(0);

        assert_eq!(
            opt_return_pc2,
            Some(ProgramCounter {
                module_index: 83,
                function_internal_index: 79,
                instruction_address: 89,
            })
        );

        // SP    0d0016 |        |
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/

        assert_eq!(stack.sp, 16);
        assert_eq!(stack.fp, 0);
    }

    #[test]
    fn test_reset_frame() {
        // frames:
        //
        // 1. create function frame (f0),   with 2 args + 2 local vars, 0 results
        // 2. reset f0
        // 3. reset f0
        // 4. create block frame (f1)       with 1 args + 1 local vars, 2 results
        // 5. reset f1
        // 6. reset f1
        // 7. create block frame (f2)       with 0 args + 0 local vars, 0 results
        // 8. reset f2
        // 9. reset to f1
        // 10. reset to f0

        let mut stack = Stack::new(INIT_STACK_SIZE_IN_BYTES);

        stack.push_i32_u(23);
        stack.push_i32_u(29);
        stack.push_i32_u(31);
        stack.push_i32_u(37);

        // the current layout
        //
        //              |        |
        //       0d0024 | 37     |
        //       0d0016 | 31     |
        //       0d0008 | 29     |
        // FP,SP 0d0000 | 23     |
        //              \--------/

        stack.create_frame(
            2, // params count
            0, // results count
            73,
            16 + 16, // local vars len
            Some(ProgramCounter {
                instruction_address: 89,     //ret inst addr
                function_internal_index: 79, // func idx
                module_index: 83,            // ret mod idx
            }),
        );

        // the current layout
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

        // the current layout (partial)
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

        // check SP
        assert_eq!(stack.sp, 112);

        // reset the frame
        let isfunc0 = stack.reset_frames(0);
        assert!(isfunc0);

        // the current layout
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

        // check frame
        let f0 = stack.get_frame_pack(0);
        assert_eq!(f0.address, 16);
        assert_eq!(
            f0.frame_info,
            &FrameInfo {
                previous_frame_address: 0,
                function_frame_address: 16,
                params_count: 2,
                results_count: 0,
                local_list_index: 73,
                local_variables_allocate_bytes: 32,
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

        // reset in the current frame
        // because there is no extra operands, there are only local vars (and args),
        // so the reseting this time should be optimizied.
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

        // the current layout (partial)
        //
        // SP--> 0d0088 |        |
        //       0d0080 | 139    | <-- operands 0
        //       0d0072 | 137    |
        //       0d0064 | 131    | <-- local vars 0
        //              |--------|
        //       0d0056 | 128    |
        //       0d0048 | 114    | <-- args 0
        //              |--------|

        // create block frame
        stack.create_frame(1, 2, 97, 8 + 8, None);

        // the current layout
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

        // the current layout (partial)
        //
        // SP--> 0d0144 |        |
        //       0d0136 | 151    |
        //       0d0128 | 149    | <-- operands 1
        //       0d0120 | 401    | <-- local vars 1
        //       0d0112 | 139    | <-- args 1 (local vars 1)
        //              |--------|

        // reset the frame
        let isfunc1 = stack.reset_frames(0);
        assert!(!isfunc1);

        // the current layout (partial)
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

        assert_eq!(stack.fp, 80);
        assert_eq!(stack.sp, 128);

        // check frame
        let f1 = stack.get_frame_pack(0);
        assert_eq!(f1.address, 80);
        assert_eq!(
            f1.frame_info,
            &FrameInfo {
                previous_frame_address: 16,
                function_frame_address: 16,
                params_count: 1,
                results_count: 2,
                local_list_index: 97,
                local_variables_allocate_bytes: 16,
                return_module_index: 0,
                return_function_internal_index: 0,
                return_instruction_address: 0
            }
        );

        // check local vars
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 151);
        assert_eq!(stack.read_local_by_offset_i32(0, 8), 0);

        // reset the current block frame again
        stack.push_i32_u(152);
        let isfunc2 = stack.reset_frames(0);
        assert!(!isfunc2);

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

        // the current layout (partial)
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

        //
        // create block frame
        //

        stack.create_frame(0, 0, 701, 0, None);

        // the current layout (partial)
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

        assert_eq!(stack.fp, 136);
        assert_eq!(stack.sp, 168);

        // add two operands
        stack.push_i32_u(167);
        stack.push_i32_u(173);

        // the current layout (partial)
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

        assert_eq!(stack.sp, 184);

        //
        // reset the current frame
        //

        // note:
        // the current frame has no local vars, neither args

        stack.reset_frames(0);

        // check SP
        assert_eq!(stack.fp, 136);
        assert_eq!(stack.sp, 168);

        // add two operands again
        stack.push_i32_u(503);
        stack.push_i32_u(509);

        // the current layout (partial)
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

        //
        // crossing reset, reset to frame 1
        //

        // the params count of target frame is 1
        let isfunc3 = stack.reset_frames(1);
        assert!(!isfunc3);

        // the current layout (partial)
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

        assert_eq!(stack.fp, 80);
        assert_eq!(stack.sp, 128);

        // check local vars
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 509);
        assert_eq!(stack.read_local_by_offset_i32(0, 8), 0);

        //
        // crossing reset, reset to frame 0
        //

        // add two operands
        stack.push_i32_u(181);
        stack.push_i32_u(191);

        // the current layout (partial)
        //
        // SP--> 0d0144 |        |
        //       0d0136 | 191    |
        //       0d0128 | 181    |
        //       0d0120 | 0      | <-- local vars 1
        //       0d0112 | 509    | <-- args 1 from operands 2
        //              |--------|

        // the params count of target frame (frame 0) is 2
        let isfunc4 = stack.reset_frames(1);
        assert!(isfunc4);

        // the current layout
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

        assert_eq!(stack.fp, 16);
        assert_eq!(stack.sp, 80);

        // check local variables
        assert_eq!(stack.read_local_by_offset_i32(0, 0), 181); // updated
        assert_eq!(stack.read_local_by_offset_i32(0, 8), 191); // updated
        assert_eq!(stack.read_local_by_offset_i32(0, 16), 0); // reset
        assert_eq!(stack.read_local_by_offset_i32(0, 24), 0); // reset
    }
}
