// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::mem::size_of;

use ancvm_types::OPERAND_SIZE_IN_BYTES;

use crate::{
    memory::Memory, resizeable_memory::ResizeableMemory, thread::ProgramCounter,
    type_memory::TypeMemory, MEMORY_PAGE_SIZE_IN_BYTES, STACK_FRAME_SIZE_IN_PAGES,
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

// the frame structure and the frame information
//
// | ...                   |
// | ...                   |
// |=======================|
// | operand N             |                                   | ...                    |
// | operand 1             |                                   | ...                    |
// | operand 0             | <-- operands                      |========================|
// |-----------------------|                                   | operand N              |
// | arg 1 (local 3)       |                                   | operand 1              |
// | arg 0 (local 2)       | <-- args from caller              | operand 0              |
// |-----------------------|                                   |------------------------|
// | local 1               |                                   | arg 1                  |
// | local 0               | <-- local variable area           | arg 0                  |
// |-----------------------| <-------------------------------> |------------------------|
// | return inst addr      |                                   | 0                      |
// | return func idx       |                                   | 0                      |
// | return module idx     |                                   | 0                      |
// | local var alloc bytes |                                   | 0                      |
// | params/results count  |                                   | params/results count   |
// | function FP           |     func                block     | function FP            |
// | previous FP           | <-- frame info     frame info --> | previous FP            |
// |=======================| <-- FP                     FP --> |========================|
// | ...                   |                                   | ...                    |
// | ...                   |                                   | ...                    |
// \-----------------------/ <-- stack start                   \------------------------/
//
// note:
// - function arguments can also be read/write as local variables.
// - block has arguments but has NO local variables.
// - block arguments can NOT be read/write as local variables.
//
// |                 |
// |-----------------| <------
// | arg 1 (local 5) |     ^
// | arg 0 (local 4) |     |
// |-----------------|     |
// | local 3         | local vars area
// | local 2         |     |
// | local 1         |     v
// |-----------------| <------
// |                 |
// \-----------------/ <-- stack start

// the chain of frames
//
//             |             |
//             | ...         |
//             | func FP     | --------\
//  func frame | previous FP | ---\    |
//             |-------------| <--|----/
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
//             | func FP     | ------------|----| the value of FFP in the function frame is the current FP itself
//  func frame | previous FP |             |    |
//             |-------------| <-----------/ <--/
//             | ...         |
//             \-------------/ <-- stack start

#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct FrameInfo {
    pub previous_frame_address: u32,         //--\  <-- addr low
    pub function_frame_address: u32,         //--/  8 bytes
    pub params_count: u16,                   //--\
    pub results_count: u16,                  //  |  8 bytes
    _padding0: u32,                          //--/
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

impl Stack {
    pub fn new(init_size_in_pages: usize) -> Self {
        let len = init_size_in_pages * MEMORY_PAGE_SIZE_IN_BYTES;
        let data: Vec<u8> = vec![0u8; len];
        let swap: Vec<u8> = vec![0u8; len];
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
}

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

impl ResizeableMemory for Stack {
    fn get_capacity_in_pages(&self) -> usize {
        self.data.len() / MEMORY_PAGE_SIZE_IN_BYTES
    }

    fn resize(&mut self, new_size_in_pages: usize) -> usize {
        let new_len = new_size_in_pages * MEMORY_PAGE_SIZE_IN_BYTES;
        self.data.resize(new_len, 0);
        new_size_in_pages
    }
}

impl TypeMemory for Stack {
    //
}

/// implements stack general functions
///
/// - push/pop/peek
/// - create frame
/// - remove frame
/// - reset frame
impl Stack {
    pub fn ensure_stack_space(&mut self) {
        // check the capacity of the stack to make sure
        // there is enough space for a call stack frame.
        // as well as increasing the capacity in the specified
        // increment (the default value is 32 KiB), that is,
        // the capacity of the stack can only be 32, 64, 96, 128 KiB and so on.

        let len = self.data.len();
        if len - self.sp < (STACK_FRAME_SIZE_IN_PAGES * MEMORY_PAGE_SIZE_IN_BYTES) {
            let new_size_in_pages = self.get_capacity_in_pages() + STACK_FRAME_SIZE_IN_PAGES;
            self.resize(new_size_in_pages);
        }
    }

    pub fn push_i32_s(&mut self, value: i32) {
        self.write_i32_s(self.sp, value);
        self.sp += OPERAND_SIZE_IN_BYTES;
    }

    pub fn push_i32_u(&mut self, value: u32) {
        self.write_i32_u(self.sp, value);
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
        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_i32_s(self.sp)
    }

    pub fn pop_i32_u(&mut self) -> u32 {
        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_i32_u(self.sp)
    }

    pub fn pop_i64_s(&mut self) -> i64 {
        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_i64_s(self.sp)
    }

    pub fn pop_i64_u(&mut self) -> u64 {
        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_i64_u(self.sp)
    }

    pub fn pop_f32(&mut self) -> f32 {
        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_f32(self.sp)
    }

    pub fn pop_f64(&mut self) -> f64 {
        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_f64(self.sp)
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

    // this is an unsafe function, the caller should write
    // data to stack immediately after calling this function.
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
    pub fn push_from_memory(&mut self) -> *mut u8 {
        let ptr = self.get_mut_ptr(self.sp);
        self.sp += OPERAND_SIZE_IN_BYTES;
        ptr
    }

    // this is an unsafe function, the caller should write
    // data to memory immediately after calling this function.
    //
    // e.g.
    //
    // ```rust
    // let ptr = stack.pop_to_memory();
    // memory.store_64(ptr, address);
    // ```
    pub fn pop_to_memory(&mut self) -> *const u8 {
        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.get_ptr(self.sp)
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
            // if fp == 0 {
            //     panic!("The index is out of bounds.")
            // }
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
        // the FFP pointer:
        //
        //       |         |           |         |
        //       |---------|           |---------|
        //       | ...     |           | ...     |
        //       | Func FP |----\      | Func FP |
        //       | prev FP |    |      | prev FP |
        // FP -> |---------|    \----> |---------|
        //       | ...     |           | ...     |
        //       \---------/           \---------/
        //
        // the function FP sometimes point to the frame itself when the frame
        // is "function stack frame".

        let frame_info = self.read_frame_info(self.fp);
        if frame_info.function_frame_address as usize == self.fp {
            // the current frame is function frame
            FramePack::new(self.fp, frame_info)
        } else {
            // the current frame is block frame
            let func_fp = frame_info.function_frame_address as usize;
            let func_frame_info = self.read_frame_info(func_fp);
            FramePack::new(func_fp, func_frame_info)
        }
    }

    /// get the local variables area start address
    ///
    /// note that the address is calculated by 'FP + the size of Frame', so
    /// even if there is no local variable area in the current function frame,
    /// this function always return the calculated address.
    pub fn get_local_variables_start_address(&self) -> usize {
        // |            |
        // | ...        | <-- args
        // |------------|
        // | ...        |
        // |------------| <-- local vars start
        // | frame info |
        // |------------| <-- FP
        // | ...        |
        // \------------/

        let frame_info = self.read_frame_info(self.fp);
        let func_fp = frame_info.function_frame_address;
        func_fp as usize + size_of::<FrameInfo>()
    }

    fn read_frame_info(&self, addr: usize) -> &FrameInfo {
        let ptr = self.data[addr..].as_ptr();
        unsafe { &*(ptr as *const FrameInfo) }
    }

    /// mapping a structure to the specified address.
    /// the caller must write value of each field through the return object.
    fn write_frame_info(&mut self, addr: usize) -> &mut FrameInfo {
        let ptr = self.data[addr..].as_mut_ptr();
        unsafe { &mut *(ptr as *mut FrameInfo) }
    }

    /// move the specified number of operands to the swap are
    fn move_operands_to_swap(&mut self, operands_count: u16) {
        if operands_count == 0 {
            return;
        }

        let count_in_bytes = operands_count as usize * OPERAND_SIZE_IN_BYTES;
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

    fn restore_operands_from_swap(&mut self, operands_count: u16) {
        if operands_count == 0 {
            return;
        }

        let count_in_bytes = operands_count as usize * OPERAND_SIZE_IN_BYTES;

        // memory copy
        let src = self.swap.as_ptr();
        let dst = self.data[self.sp..].as_mut_ptr();
        unsafe {
            std::ptr::copy(src, dst, count_in_bytes);
        }

        // update the SP
        self.sp += count_in_bytes;
    }

    pub fn create_function_frame(
        &mut self,
        params_count: u16,
        results_count: u16,
        local_variables_allocate_bytes: u32,
        return_pc: ProgramCounter,
    ) {
        let previous_fp = self.fp;

        // move the arguments to swap first
        self.move_operands_to_swap(params_count);

        // ensure the free space
        self.ensure_stack_space();

        // create new block frame at the current position (it's the value of SP)
        let fp = self.sp;
        let func_frame_info = self.write_frame_info(fp);

        // write values
        func_frame_info.previous_frame_address = previous_fp as u32;
        func_frame_info.function_frame_address = fp as u32; // the function FP point to the current frame itself.
        func_frame_info.params_count = params_count;
        func_frame_info.results_count = results_count;
        func_frame_info._padding0 = 0;

        func_frame_info.local_variables_allocate_bytes = local_variables_allocate_bytes;
        func_frame_info.return_module_index = return_pc.module_index as u32;
        func_frame_info.return_function_internal_index = return_pc.function_internal_index as u32;
        func_frame_info.return_instruction_address = return_pc.instruction_address as u32;

        // update sp and fp
        self.sp += size_of::<FrameInfo>();
        self.fp = fp;

        // allocate local variable area
        //
        // note that 'local_variables_allocate_bytes' includes the size of arguments
        //
        //  |                 |
        //  |-----------------| <-----
        //  | arg 1 (local 5) |    ^
        //  | arg 0 (local 4) |    |
        //  |-----------------|    | <----------
        //  | local 3         |  vars local  ^
        //  | local 2         |    | area    |  'local_variables_allocate_bytes -
        //  | local 1         |    v         v   params_count * OPERAND_SIZE_IN_BYTES'
        //  |-----------------| <---------------
        //  |                 |
        //  \-----------------/ <-- stack start

        let local_variables_allocate_bytes_without_args =
            local_variables_allocate_bytes as usize - params_count as usize * OPERAND_SIZE_IN_BYTES;
        // clear the local variables area
        self.data[self.sp..(self.sp + local_variables_allocate_bytes_without_args)].fill(0);
        self.sp += local_variables_allocate_bytes_without_args;

        // restore the arguments from swap
        self.restore_operands_from_swap(params_count);
    }

    pub fn create_block_frame(&mut self, params_count: u16, results_count: u16) {
        let previous_fp = self.fp;

        // the 'func_fp' is inherited from the previous frame
        let func_fp = {
            let frame_pack = self.get_frame_pack(0);
            frame_pack.frame_info.function_frame_address
        };

        // move the arguments to swap first
        self.move_operands_to_swap(params_count);

        // create new block frame at the current position (it's the value of SP)
        let fp = self.sp;
        let block_frame_info = self.write_frame_info(fp);

        // write values
        block_frame_info.previous_frame_address = previous_fp as u32; // singly linked list
        block_frame_info.function_frame_address = func_fp;
        block_frame_info.params_count = params_count;
        block_frame_info.results_count = results_count;
        block_frame_info._padding0 = 0;

        // block_frame_info.module_index = 0;
        block_frame_info.local_variables_allocate_bytes = 0;
        block_frame_info.return_module_index = 0;
        block_frame_info.return_function_internal_index = 0;
        block_frame_info.return_instruction_address = 0;

        // update sp and fp
        self.sp += size_of::<FrameInfo>();
        self.fp = fp;

        // restore the arguments from swap
        self.restore_operands_from_swap(params_count);
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
        self.move_operands_to_swap(results_count);

        self.sp = sp;
        self.fp = fp;

        // restore parameters from swap
        self.restore_operands_from_swap(results_count);

        if is_function_frame {
            Some(return_pc)
        } else {
            None
        }
    }

    /// reset the specified function frame or block frame:
    /// - initialize all local variables to value 0 (only if the target frame is function frame)
    /// - remove all oprands which follow the local variable area
    /// - remove all frames which follow the current frame
    /// - moves the specified number of operands to the top of stack
    ///
    /// this function is commonly used for 'loop' structure or 'tail call' statement.
    ///
    /// return TRUE if the target frame is function frame.
    pub fn reset_to_frame(&mut self, reversed_index: u16) -> bool{
        let (frame_addr, params_count, frame_func_fp, local_variables_allocate_bytes) = {
            let frame_pack = self.get_frame_pack(reversed_index);
            (
                frame_pack.address,
                frame_pack.frame_info.params_count,
                frame_pack.frame_info.function_frame_address as usize,
                frame_pack.frame_info.local_variables_allocate_bytes as usize,
            )
        };

        let is_function_frame: bool = frame_addr == frame_func_fp;

        // optimized for blocks like 'loop' structures.
        //
        // - block frame
        // - the specified frame is the current frame
        // - there is no other operands than parameters
        //
        // just do nothing when all conditions are met.
        if (!is_function_frame)
            && (frame_addr == self.fp)
            && (self.sp - params_count as usize * OPERAND_SIZE_IN_BYTES
                == self.fp + size_of::<FrameInfo>())
        {
            return false;
        }

        // move the specified number of operands to swap
        self.move_operands_to_swap(params_count);

        if is_function_frame {
            // the current frame is function frame

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

            let local_vars_addr_start = frame_addr + size_of::<FrameInfo>();

            // note that 'local_variables_allocate_bytes' includes the size of arguments
            //
            //  |                 |
            //  |-----------------| <-----
            //  | arg 1 (local 5) |    ^
            //  | arg 0 (local 4) |    |
            //  |-----------------|    | <----------
            //  | local 3         |  vars local  ^
            //  | local 2         |    | area    |  'local_variables_allocate_bytes -
            //  | local 1         |    v         v   params_count * OPERAND_SIZE_IN_BYTES'
            //  |-----------------| <---------------
            //  |                 |
            //  \-----------------/ <-- stack start

            let local_variables_allocate_bytes_without_args =
                local_variables_allocate_bytes - params_count as usize * OPERAND_SIZE_IN_BYTES;
            self.sp = local_vars_addr_start + local_variables_allocate_bytes_without_args;

            // re-initialize the local variable area
            let dst = self.data[local_vars_addr_start..].as_mut_ptr();
            unsafe {
                std::ptr::write_bytes(dst, 0, local_variables_allocate_bytes_without_args);
            }

            // restore parameters from swap
            self.restore_operands_from_swap(params_count);

            true
        } else {
            // the current frame is block frame

            // remove all operands and frames which follows the current frame
            //
            // |            |
            // | ...        |
            // |------------| <-- move SP to here
            // | frame info |
            // |------------| <-- frame addr, move FP to here
            // | ...        |
            // \------------/

            self.fp = frame_addr;
            self.sp = frame_addr + size_of::<FrameInfo>();

            // restore parameters from swap
            self.restore_operands_from_swap(params_count);

            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use ancvm_types::OPERAND_SIZE_IN_BYTES;

    use crate::{
        memory::Memory,
        resizeable_memory::ResizeableMemory,
        stack::{FrameInfo, Stack},
        thread::ProgramCounter,
        type_memory::TypeMemory,
        MEMORY_PAGE_SIZE_IN_BYTES, STACK_FRAME_SIZE_IN_PAGES,
    };

    /// private functions for helping unit test
    impl Stack {
        fn read_local_by_offset_i32(&self, offset: usize) -> i32 {
            self.read_i32_s(self.get_local_variables_start_address() + offset)
        }

        fn write_local_by_offset_i32(&mut self, offset: usize, value: i32) {
            self.write_i32_s(self.get_local_variables_start_address() + offset, value)
        }
    }

    #[test]
    fn test_stack_capacity() {
        let mut stack = Stack::new(STACK_FRAME_SIZE_IN_PAGES);
        assert_eq!(stack.sp, 0);
        assert_eq!(stack.fp, 0);

        // check the initial size
        assert_eq!(stack.get_capacity_in_pages(), STACK_FRAME_SIZE_IN_PAGES);
        stack.ensure_stack_space();
        assert_eq!(stack.get_capacity_in_pages(), STACK_FRAME_SIZE_IN_PAGES);

        // add one operand
        stack.push_i32_s(11);
        assert_eq!(stack.get_capacity_in_pages(), STACK_FRAME_SIZE_IN_PAGES);
        stack.ensure_stack_space();
        assert_eq!(stack.get_capacity_in_pages(), STACK_FRAME_SIZE_IN_PAGES * 2);

        // clear
        assert_eq!(stack.pop_i32_s(), 11);
        assert_eq!(stack.sp, 0);

        // fill up one frame
        for i in 0..(STACK_FRAME_SIZE_IN_PAGES * MEMORY_PAGE_SIZE_IN_BYTES / OPERAND_SIZE_IN_BYTES)
        {
            stack.push_i64_s(i as i64);
        }

        assert_eq!(
            stack.sp,
            STACK_FRAME_SIZE_IN_PAGES * MEMORY_PAGE_SIZE_IN_BYTES
        );

        // add one operand
        stack.push_i32_s(11);
        assert_eq!(stack.get_capacity_in_pages(), STACK_FRAME_SIZE_IN_PAGES * 2);
        stack.ensure_stack_space();
        assert_eq!(stack.get_capacity_in_pages(), STACK_FRAME_SIZE_IN_PAGES * 3);
    }

    #[test]
    fn test_push_and_pop_values() {
        let mut stack = Stack::new(STACK_FRAME_SIZE_IN_PAGES);

        // check push, peek and pop
        stack.push_i32_s(11);
        stack.push_i64_s(13);
        stack.push_f32(3.14);
        stack.push_f64(2.9979e8);

        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES * 4);
        assert_eq!(stack.peek_f64(), 2.9979e8);
        assert_eq!(stack.pop_f64(), 2.9979e8);
        assert_eq!(stack.pop_f32(), 3.14);

        assert_eq!(stack.peek_i64_s(), 13);
        assert_eq!(stack.pop_i64_s(), 13);
        assert_eq!(stack.pop_i32_s(), 11);
        assert_eq!(stack.sp, 0);

        // check duplicate
        stack.push_i32_s(17);
        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES);

        stack.duplicate();
        assert_eq!(stack.peek_i32_s(), 17);
        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES * 2);

        // check drop
        stack.push_i32_s(19);
        stack.push_i32_s(23);
        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES * 4);

        stack.drop_();
        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES * 3);

        assert_eq!(stack.peek_i32_s(), 19);
    }

    #[test]
    fn test_host_address() {
        let mut stack = Stack::new(STACK_FRAME_SIZE_IN_PAGES);

        stack.push_i32_s(11);
        stack.push_i64_s(13);
        stack.push_i32_s(17);
        stack.push_i64_s(19);

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
        let mut stack = Stack::new(STACK_FRAME_SIZE_IN_PAGES);

        stack.push_i32_s(23);
        stack.push_i32_s(29);
        stack.push_i32_s(31);
        stack.push_i32_s(37);

        // the current layout
        //
        //              |        |
        //       0d0024 | 37     |
        //       0d0016 | 31     |
        //       0d0008 | 29     |
        // FP,SP 0d0000 | 23     |
        //              \--------/

        stack.create_function_frame(
            2, // params count
            0, // results count
            // 73,      // mod idx
            16 + 16, // local vars len
            ProgramCounter {
                module_index: 83,            // ret mod idx
                function_internal_index: 79, // func idx
                instruction_address: 89,     //ret inst addr
            },
        );

        // the current layout
        //
        // SP--> 0d0080 |        |
        //       0d0072 | 37     |
        //       0d0064 | 31     | <-- args 0
        //              |--------|
        //       0d0056 | 0      |
        //       0d0048 | 0      | <-- local vars 0
        //              |--------|
        //       0d0044 | 89     | return inst addr
        //       0d0040 | 79     | return func idx
        //       0d0036 | 83     | return module idx
        //       0d0032 | 32     | local vars len
        //       0d0028 | 0      | padding
        //       0d0024 | 2/0    | params/results count
        //       0d0020 | 16     | func FP
        // FP--> 0d0016 | 0      | prev FP
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/

        // check raw data
        assert_eq!(stack.read_i64_s(0), 23);
        assert_eq!(stack.read_i64_s(8), 29);

        assert_eq!(stack.read_i32_s(16), 0);
        assert_eq!(stack.read_i32_s(20), 16);
        assert_eq!(stack.read_i32_s(24), 0 << 16 | 2); // results count << 16 | params count
        assert_eq!(stack.read_i32_s(28), 0);
        assert_eq!(stack.read_i32_s(32), 32);
        assert_eq!(stack.read_i32_s(36), 83);
        assert_eq!(stack.read_i32_s(40), 79);
        assert_eq!(stack.read_i32_s(44), 89);

        assert_eq!(stack.read_i64_s(48), 0);
        assert_eq!(stack.read_i64_s(56), 0);

        assert_eq!(stack.read_i64_s(64), 31);
        assert_eq!(stack.read_i64_s(72), 37);

        // check status
        assert_eq!(stack.sp, 80);
        assert_eq!(stack.fp, 16);

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
                _padding0: 0,
                local_variables_allocate_bytes: 32,
                return_module_index: 83,
                return_function_internal_index: 79,
                return_instruction_address: 89
            }
        );

        let f0b = stack.get_function_frame_pack();
        assert_eq!(f0, f0b);

        let fp0 = f0.address;

        // check local variables

        // the current layout (partial)
        //
        //       0d0072 | 37     |
        //       0d0064 | 31     | <-- args 0
        //              |--------|
        //       0d0056 | 0      |
        //       0d0048 | 0      | <-- local vars 0
        //              |--------|

        assert_eq!(
            stack.get_local_variables_start_address(),
            fp0 + size_of::<FrameInfo>()
        );
        assert_eq!(stack.read_local_by_offset_i32(0), 0);
        assert_eq!(stack.read_local_by_offset_i32(8), 0);
        assert_eq!(stack.read_local_by_offset_i32(16), 31);
        assert_eq!(stack.read_local_by_offset_i32(24), 37);

        // write local variables

        stack.write_local_by_offset_i32(0, 211);
        stack.write_local_by_offset_i32(8, 223);

        assert_eq!(stack.read_local_by_offset_i32(0), 211);
        assert_eq!(stack.read_local_by_offset_i32(8), 223);

        // add more operands

        stack.push_i32_s(41);
        stack.push_i32_s(43);
        stack.push_i32_s(47);

        // the current layout (partial)
        //
        // SP--> 0d0104 |        |
        //       0d0096 | 47     |
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //              |--------|
        //       0d0072 | 37     |
        //       0d0064 | 31     | <-- args 0
        //              |--------|
        //       0d0056 | 223    |
        //       0d0048 | 211    | <-- local vars 0
        //              |--------|
        //
        assert_eq!(stack.fp, 16);
        assert_eq!(stack.sp, 104);

        // create block frame
        stack.create_block_frame(1, 2);

        // the current layout
        //
        // SP--> 0d0136 |        |
        //       0d0128 | 47     | <-- args 1
        //              |--------|
        //       0d0124 | 0      | return inst addr
        //       0d0120 | 0      | return func idx
        //       0d0116 | 0      | return module idx
        //       0d0112 | 0      | local vars len
        //       0d0108 | 0      | padding
        //       0d0104 | 1/2    | params/results count
        //       0d0100 | 16     | func FP
        // FP--> 0d0096 | 16     | prev FP
        //              |--------| <-- fp1
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //              |--------|
        //       0d0072 | 37     |
        //       0d0064 | 31     | <-- args 0
        //              |--------|
        //       0d0056 | 223    |
        //       0d0048 | 211    | <-- local vars 0
        //              |--------|
        //
        assert_eq!(stack.fp, 96);
        assert_eq!(stack.sp, 136);
        assert_eq!(stack.read_i32_s(128), 47); // one operand has been moved
        assert_eq!(stack.read_i32_s(88), 43); // the old operand has no change

        let f1 = stack.get_frame_pack(0);
        assert_eq!(f1.address, 96);
        assert_eq!(
            f1.frame_info,
            &FrameInfo {
                previous_frame_address: 16,
                function_frame_address: 16,
                params_count: 1,
                results_count: 2,
                _padding0: 0,
                local_variables_allocate_bytes: 0,
                return_module_index: 0,
                return_function_internal_index: 0,
                return_instruction_address: 0
            }
        );

        assert_eq!(stack.get_frame_pack(1).address, fp0);
        assert_eq!(stack.get_function_frame_pack(), stack.get_frame_pack(1));

        let fp1 = f1.address;

        // check local variables

        // the values have no change
        assert_eq!(stack.read_local_by_offset_i32(0), 211);
        assert_eq!(stack.read_local_by_offset_i32(8), 223);
        assert_eq!(stack.read_local_by_offset_i32(16), 31);
        assert_eq!(stack.read_local_by_offset_i32(24), 37);

        // create block frame
        stack.create_block_frame(0, 0);

        let fp2 = fp1 + size_of::<FrameInfo>() + 8; // 1 args in the 1st block frame

        // the current layout
        //
        //              |            |
        //  block frame | frame 2    |
        //              |------------| <-- fp2
        //              | args 1     |
        //  block frame | frame 1    |
        //              |------------| <-- fp1
        //              | operands 0 |
        //              | args 0     |
        //              | local 0    |
        //   func frame | frame 0    |
        //              |------------| <-- fp0
        //              | operands   |
        //              \------------/

        assert_eq!(stack.fp, fp2);
        assert_eq!(stack.sp, fp2 + size_of::<FrameInfo>());
        assert_eq!(stack.read_i32_s(fp2 - 8), 47); // the old operand has no change

        let f2 = stack.get_frame_pack(0);
        assert_eq!(f2.address, fp2);
        assert_eq!(
            f2.frame_info,
            &FrameInfo {
                previous_frame_address: fp1 as u32,
                function_frame_address: fp0 as u32,
                params_count: 0,
                results_count: 0,
                _padding0: 0,
                local_variables_allocate_bytes: 0,
                return_module_index: 0,
                return_function_internal_index: 0,
                return_instruction_address: 0
            }
        );

        assert_eq!(stack.get_frame_pack(1).address, fp1);
        assert_eq!(stack.get_frame_pack(2).address, fp0);
        assert_eq!(stack.get_function_frame_pack(), stack.get_frame_pack(2));

        let fp2 = f2.address;

        // check local variables

        assert_eq!(stack.read_local_by_offset_i32(0), 211); // only function frame has local variables
        assert_eq!(stack.read_local_by_offset_i32(8), 223); // so the block frame does change the local variable address
        assert_eq!(stack.read_local_by_offset_i32(16), 31);
        assert_eq!(stack.read_local_by_offset_i32(24), 37);

        // add operands
        stack.push_i32_s(239);
        stack.push_i32_s(241);

        // create func frame
        stack.create_function_frame(
            1, // params count
            3, // results count
            // 107,   // mod idx
            0 + 8, // local vars len
            ProgramCounter {
                module_index: 113,            // ret mod idx
                function_internal_index: 109, // func idx
                instruction_address: 127,     // ret inst addr
            },
        );

        // the current layout
        //
        //              |            |
        //              | 241        | <-- args 3
        //   func frame | frame 3    |
        //              |------------| <-- fp3
        //              | 239        | <-- operands 2
        //  block frame | frame 2    |
        //              |------------| <-- fp2
        //              | args 1     |
        //  block frame | frame 1    |
        //              |------------| <-- fp1
        //              | operands 0 |
        //              | args 0     |
        //              | local 0    |
        //   func frame | frame 0    |
        //              |------------| <-- fp0
        //              | operands   |
        //              \------------/

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
                _padding0: 0,
                local_variables_allocate_bytes: 8,
                return_module_index: 113,
                return_function_internal_index: 109,
                return_instruction_address: 127
            }
        );

        let f3b = stack.get_function_frame_pack();
        assert_eq!(f3, f3b);

        assert_eq!(stack.get_frame_pack(1).address, fp2);
        assert_eq!(stack.get_frame_pack(2).address, fp1);
        assert_eq!(stack.get_frame_pack(3).address, fp0);

        // check local variables

        // because function frame created new local variable area, the address should be updated
        assert_eq!(
            stack.get_local_variables_start_address(),
            fp3 + size_of::<FrameInfo>()
        );
        assert_eq!(stack.read_local_by_offset_i32(0), 241);

        // remove the current frame

        // push some oparnds first
        stack.push_i32_s(251);
        stack.push_i32_s(257);

        // the current layout
        //
        //              |            |
        //              | 257        |
        //              | 251        |
        //              | 241        | <-- args 3
        //   func frame | frame 3    |
        //              |------------| <-- fp3

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

        // check operands

        // the current layout (partial)
        //
        //              |          |
        //              | 257      |
        //              | 251      | <-- from operands 3
        //              | 241      | <-- from args 3
        //              | 239      | <-- operands 2
        //  block frame | frame 2  |
        //              |----------| <-- fp2

        assert_eq!(stack.read_i32_s(stack.sp - 8), 257);
        assert_eq!(stack.read_i32_s(stack.sp - 16), 251);
        assert_eq!(stack.read_i32_s(stack.sp - 24), 241);
        assert_eq!(stack.read_i32_s(stack.sp - 32), 239);

        // check local variables

        // because the 2nd function frame has been removed, so the address should be restored to
        // the 1st function frame
        assert_eq!(
            stack.get_local_variables_start_address(),
            fp0 + size_of::<FrameInfo>()
        );

        assert_eq!(stack.read_local_by_offset_i32(0), 211);
        assert_eq!(stack.read_local_by_offset_i32(8), 223);
        assert_eq!(stack.read_local_by_offset_i32(16), 31);
        assert_eq!(stack.read_local_by_offset_i32(24), 37);

        // remove the parent frame
        let opt_return_pc1 = stack.remove_frames(1);
        assert_eq!(opt_return_pc1, None);

        // SP--> 0d0112 |        |
        //       0d0104 | 257    |
        //       0d0096 | 251    | <-- from operands 3
        //              |--------|
        //       0d0088 | 43     |
        //       0d0080 | 41     | <-- operands 0
        //              |--------|
        //       0d0072 | 37     |
        //       0d0064 | 31     | <-- args 0
        //              |--------|
        //       0d0056 | 223    |
        //       0d0048 | 211    | <-- local vars 0
        //              |--------| <-- fp0
        //              |        | <-- operands
        //              \--------/

        assert_eq!(stack.get_frame_pack(0).address, fp0);
        assert_eq!(stack.sp, 112);

        // check operands
        assert_eq!(stack.read_i32_s(104), 257);
        assert_eq!(stack.read_i32_s(96), 251);
        assert_eq!(stack.read_i32_s(88), 43);
        assert_eq!(stack.read_i32_s(80), 41);

        // remove the last frame
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
        let mut stack = Stack::new(STACK_FRAME_SIZE_IN_PAGES);

        stack.push_i32_s(23);
        stack.push_i32_s(29);
        stack.push_i32_s(31);
        stack.push_i32_s(37);

        // the current layout
        //
        //              |        |
        //       0d0024 | 37     |
        //       0d0016 | 31     |
        //       0d0008 | 29     |
        // FP,SP 0d0000 | 23     |
        //              \--------/

        stack.create_function_frame(
            2, // params count
            0, // results count
            // 73,      // mod idx
            16 + 16, // local vars len
            ProgramCounter {
                instruction_address: 89,     //ret inst addr
                function_internal_index: 79, // func idx
                module_index: 83,            // ret mod idx
            },
        );

        // the current layout
        //
        // SP--> 0d0080 |        |
        //       0d0072 | 37     |
        //       0d0064 | 31     | <-- args 0
        //              |--------|
        //       0d0056 | 0      |
        //       0d0048 | 0      | <-- local vars 0
        //              |--------|
        //       0d0044 | 89     | return inst addr
        //       0d0040 | 79     | return func idx
        //       0d0036 | 83     | return module idx
        //       0d0032 | 32     | local vars len
        //       0d0028 | 0      | _padding
        //       0d0024 | 2/0    | params/results count
        //       0d0020 | 16     | func FP
        // FP--> 0d0016 | 0      | prev FP
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/

        // update local variables
        stack.write_local_by_offset_i32(0, 101);
        stack.write_local_by_offset_i32(8, 103);

        // check local variables
        assert_eq!(stack.read_local_by_offset_i32(0), 101);
        assert_eq!(stack.read_local_by_offset_i32(8), 103);
        assert_eq!(stack.read_local_by_offset_i32(16), 31);
        assert_eq!(stack.read_local_by_offset_i32(24), 37);

        // push some operands
        stack.push_i32_s(107);
        stack.push_i32_s(109);
        stack.push_i32_s(113);
        stack.push_i32_s(127);

        // the current layout (partial)
        //
        // SP--> 0d0112 |        |
        //       0d0104 | 127    |
        //       0d0096 | 113    |
        //       0d0088 | 109    |
        //       0d0080 | 107    | <-- operands 0
        //              |--------|
        //       0d0072 | 37     |
        //       0d0064 | 31     | <-- args 0
        //              |--------|
        //       0d0056 | 103    |
        //       0d0048 | 101    | <-- local vars 0
        //              |--------|

        // check SP
        assert_eq!(stack.sp, 112);
        assert_eq!(stack.peek_i32_s(), 127);

        // reset the frame
        let isfunc0 = stack.reset_to_frame(0);
        assert!(isfunc0);

        // the current layout
        //
        // SP--> 0d0080 |        |
        //       0d0072 | 127    |
        //       0d0064 | 113    | <-- args 0 (updated)
        //              |--------|
        //       0d0056 | 0      |
        //       0d0048 | 0      | <-- local vars 0 (reset)
        //              |--------|
        //       0d0044 | 89     | return inst addr
        //       0d0040 | 79     | return func idx
        //       0d0036 | 83     | return module idx
        //       0d0032 | 32     | local vars len
        //       0d0028 | 0      | _padding
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
                _padding0: 0,
                // module_index: 73,
                local_variables_allocate_bytes: 32,
                return_module_index: 83,
                return_function_internal_index: 79,
                return_instruction_address: 89
            }
        );

        // check local variables
        assert_eq!(stack.read_local_by_offset_i32(0), 0); // reset
        assert_eq!(stack.read_local_by_offset_i32(8), 0); // reset
        assert_eq!(stack.read_local_by_offset_i32(16), 113); // updated
        assert_eq!(stack.read_local_by_offset_i32(24), 127); // updated

        // add some operands and change local variables to
        // prepare for the next reset

        stack.write_local_by_offset_i32(0, 131);
        stack.write_local_by_offset_i32(8, 137);
        stack.push_i32_s(139);

        // the current layout (partial)
        //
        // SP--> 0d0088 |        |
        //       0d0080 | 139    | <-- operands 0
        //       0d0072 | 127    |
        //       0d0064 | 113    | <-- args 0
        //              |--------|
        //       0d0056 | 137    |
        //       0d0048 | 131    | <-- local vars 0
        //              |--------|

        // create block frame
        stack.create_block_frame(1, 2);

        // the current layout
        //
        // SP--> 0d0120 |        |
        //       0d0112 | 139    | <-- args 1
        //              |--------|
        //       0d0108 | 0      |
        //       0d0104 | 0      |
        //       0d0100 | 0      |
        //       0d0096 | 0      |
        //       0d0092 | 0      |
        //       0d0088 | 1/2    | params/results count
        //       0d0084 | 16     | func FP
        // FP--> 0d0080 | 16     | prev FP
        //              |========| <-- fp1
        //       0d0072 | 127    |
        //       0d0064 | 113    | <-- args 0
        //              |--------|
        //       0d0056 | 137    |
        //       0d0048 | 131    | <-- local vars 0
        //              |--------|
        //       0d0044 | 89     | return inst addr
        //       0d0040 | 79     | return func idx
        //       0d0036 | 83     | return module idx
        //       0d0032 | 32     | local vars len
        //       0d0028 | 0      | _padding
        //       0d0024 | 2/0    | params/results count
        //       0d0020 | 16     | func FP
        //       0d0016 | 0      | prev FP
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/

        assert_eq!(stack.fp, 80);
        assert_eq!(stack.sp, 120);

        assert_eq!(stack.peek_i32_s(), 139);

        // add operands
        stack.push_i32_s(149);
        stack.push_i32_s(151);

        // the current layout (partial)
        //
        // SP--> 0d0136 |        |
        //       0d0128 | 151    |
        //       0d0120 | 149    | <-- operands 1
        //       0d0112 | 139    | <-- args 1
        //              |--------|

        // reset the frame
        let isfunc1 = stack.reset_to_frame(0);
        assert!(!isfunc1);

        // the current layout (partial)
        //
        // SP--> 0d0120 |        |
        //       0d0112 | 151    | <-- args 1
        //              |--------|
        //       0d0108 | 0      |
        //       0d0104 | 0      |
        //       0d0100 | 0      |
        //       0d0096 | 0      |
        //       0d0092 | 0      |
        //       0d0088 | 1/2    | params/results count
        //       0d0084 | 16     | func FP
        // FP--> 0d0080 | 16     | prev FP
        //              |========| <-- fp1

        assert_eq!(stack.fp, 80);
        assert_eq!(stack.sp, 120);
        assert_eq!(stack.peek_i32_s(), 151);

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
                _padding0: 0,
                // module_index: 0,
                local_variables_allocate_bytes: 0,
                return_module_index: 0,
                return_function_internal_index: 0,
                return_instruction_address: 0
            }
        );

        // reset block frame again
        let isfunc2 = stack.reset_to_frame(0);
        assert!(!isfunc2);

        // nothings has changed
        assert_eq!(stack.fp, 80);
        assert_eq!(stack.sp, 120);
        assert_eq!(stack.peek_i32_s(), 151);

        // create block frame

        // add some operands for preparing for the next reset
        stack.push_i32_s(157);

        stack.create_block_frame(0, 0);

        // the current layout (partial)
        //
        // SP--> 0d0160 |        |
        //              |--------|
        //       0d0156 | 0      |
        //       0d0152 | 0      |
        //       0d0148 | 0      |
        //       0d0144 | 0      |
        //       0d0140 | 0      |
        //       0d0136 | 0/0    | params/results count
        //       0d0132 | 16     | func FP
        // FP--> 0d0128 | 80     | prev FP
        //              |========| <-- fp2
        //       0d0120 | 157    | <-- operands 1
        //       0d0112 | 151    | <-- args 1
        //              |--------|
        //       0d0108 | 0      |
        //       0d0104 | 0      |
        //       0d0100 | 0      |
        //       0d0096 | 0      |
        //       0d0092 | 0      |
        //       0d0088 | 1/2    | params/results count
        //       0d0084 | 16     | func FP
        //       0d0080 | 16     | prev FP
        //              |========| <-- fp1

        assert_eq!(stack.fp, 128);
        assert_eq!(stack.sp, 160);

        // add two operands
        stack.push_i32_s(167);
        stack.push_i32_s(173);

        // the current layout (partial)
        //
        // SP--> 0d0176 |        |
        //       0d0168 | 173    |
        //       0d0160 | 167    |
        //              |--------|
        //       0d0156 | 0      |

        assert_eq!(stack.sp, 176);

        // crossing reset

        // the params count of target frame is 1
        let isfunc3 = stack.reset_to_frame(1);
        assert!(!isfunc3);

        // the current layout (partial)
        //
        // SP--> 0d0120 |        |
        //       0d0112 | 173    | <-- args 1 from operands 2
        //              |--------|
        //       0d0108 | 0      |
        //       0d0104 | 0      |
        //       0d0100 | 0      |
        //       0d0096 | 0      |
        //       0d0092 | 0      |
        //       0d0088 | 149    | func type
        //       0d0084 | 16     | func FP
        // FP--> 0d0080 | 16     | prev FP
        //              |========| <-- fp1

        assert_eq!(stack.fp, 80);
        assert_eq!(stack.sp, 120);

        // check args
        assert_eq!(stack.read_i32_s(stack.sp - 8), 173);

        // crossing reset

        // add two operands
        stack.push_i32_s(181);
        stack.push_i32_s(191);

        // the current layout (partial)
        //
        // SP--> 0d0136 |        |
        //       0d0128 | 191    |
        //       0d0120 | 181    |
        //       0d0112 | 173    | <-- args 1 from operands 2
        //              |--------|

        // the params count of target frame is 2
        let isfunc4 = stack.reset_to_frame(1);
        assert!(isfunc4);

        // the current layout
        //
        // SP--> 0d0080 |        |
        //       0d0072 | 191    |
        //       0d0064 | 181    | <-- args 0 from operands 1
        //              |--------|
        //       0d0056 | 0      |
        //       0d0048 | 0      | <-- local vars 0
        //              |--------|
        //       0d0044 | 89     | return inst addr
        //       0d0040 | 79     | return func idx
        //       0d0036 | 83     | return module idx
        //       0d0032 | 16     | local vars len
        //       0d0028 | 0      |
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
        assert_eq!(stack.read_local_by_offset_i32(0), 0); // reset
        assert_eq!(stack.read_local_by_offset_i32(8), 0); // reset
        assert_eq!(stack.read_local_by_offset_i32(16), 181); // updated
        assert_eq!(stack.read_local_by_offset_i32(24), 191); // updated
    }
}
