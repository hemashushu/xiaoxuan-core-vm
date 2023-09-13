// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::mem::size_of;

use ancvm_types::OPERAND_SIZE_IN_BYTES;

use crate::{
    memory::Memory, resizeable_memory::ResizeableMemory, type_memory::TypeMemory,
    MEMORY_PAGE_SIZE_IN_BYTES, STACK_FRAME_SIZE_IN_PAGES,
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
// | arg 1                 |                                   | operand 1              |
// | arg 0                 | <-- args from caller              | operand 0              |
// |-----------------------|                                   |------------------------|
// | local 1               |                                   | arg 1                  |
// | local 0               | <-- local variable area           | arg 0                  |
// |-----------------------| <-------------------------------> |------------------------|
// | return inst addr      |                                   | 0                      |
// | return module idx     |                                   | 0                      |
// | local var alloc bytes |                                   | 0                      |
// | internal func idx     |                                   | 0                      |
// | current module index  |                                   | current module index   |
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
pub struct Frame {
    pub previous_frame_address: u32, //--\
    pub function_frame_address: u32, //--/-- 8 bytes <-- addr low

    pub params_count: u16,  //--\
    pub results_count: u16, //--|
    pub module_index: u32,  //--/-- 8 bytes

    pub internal_function_index: u32,        //--\
    pub local_variables_allocate_bytes: u32, //--/-- 8 bytes

    pub return_module_index: u32,        //--\
    pub return_instruction_address: u32, //--/-- 8 bytes <-- addr high
}

#[derive(Debug, PartialEq)]
pub struct FrameItem<'a> {
    pub address: usize,
    pub frame: &'a Frame,
}

impl<'a> FrameItem<'a> {
    pub fn new(address: usize, frame: &'a Frame) -> Self {
        Self { address, frame }
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
}

impl Memory for Stack {
    #[inline]
    fn get_ptr(&self, address: usize) -> *const u8 {
        (&self.data[address..]).as_ptr()
    }

    #[inline]
    fn get_mut_ptr(&mut self, address: usize) -> *mut u8 {
        (&mut self.data[address..]).as_mut_ptr()
    }
}

impl ResizeableMemory for Stack {
    fn get_capacity_in_pages(&self) -> usize {
        self.data.len() / MEMORY_PAGE_SIZE_IN_BYTES
    }

    fn resize(&mut self, new_size_in_pages: usize) {
        let new_len = new_size_in_pages * MEMORY_PAGE_SIZE_IN_BYTES;
        self.data.resize(new_len, 0);
    }
}

// impl HostAccessableMemory for Stack {
//     #[inline]
//     fn get_host_address(&self, offset: usize) -> usize {
//         (&self.data[offset..]).as_ptr() as usize
//     }
// }

impl TypeMemory for Stack {
    //
}

/// implements stack general functions
///
/// - push/pop/peek
/// - create frame
/// - exit frame
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

    pub fn push_i32(&mut self, value: i32) {
        self.write_i32(self.sp, value);
        self.sp += OPERAND_SIZE_IN_BYTES;
    }

    pub fn push_u32(&mut self, value: u32) {
        self.write_u32(self.sp, value);
        self.sp += OPERAND_SIZE_IN_BYTES;
    }

    pub fn push_i64(&mut self, value: i64) {
        self.write_i64(self.sp, value);
        self.sp += OPERAND_SIZE_IN_BYTES;
    }

    pub fn push_u64(&mut self, value: u64) {
        self.write_u64(self.sp, value);
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

    pub fn peek_i32(&self) -> i32 {
        self.read_i32(self.sp - OPERAND_SIZE_IN_BYTES)
    }

    pub fn peek_u32(&self) -> u32 {
        self.read_u32(self.sp - OPERAND_SIZE_IN_BYTES)
    }

    pub fn peek_i64(&self) -> i64 {
        self.read_i64(self.sp - OPERAND_SIZE_IN_BYTES)
    }

    pub fn peek_u64(&self) -> u64 {
        self.read_u64(self.sp - OPERAND_SIZE_IN_BYTES)
    }

    pub fn peek_f32(&self) -> f32 {
        self.read_f32(self.sp - OPERAND_SIZE_IN_BYTES)
    }

    pub fn peek_f64(&self) -> f64 {
        self.read_f64(self.sp - OPERAND_SIZE_IN_BYTES)
    }

    pub fn pop_i32(&mut self) -> i32 {
        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_i32(self.sp)
    }

    pub fn pop_u32(&mut self) -> u32 {
        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_u32(self.sp)
    }

    pub fn pop_i64(&mut self) -> i64 {
        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_i64(self.sp)
    }

    pub fn pop_u64(&mut self) -> u64 {
        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_u64(self.sp)
    }

    pub fn pop_f32(&mut self) -> f32 {
        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_f32(self.sp)
    }

    pub fn pop_f64(&mut self) -> f64 {
        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_f64(self.sp)
    }

    pub fn drop(&mut self) {
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
    pub fn get_frame(&self, reversed_index: usize) -> FrameItem {
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
        let mut frame = self.read_frame(fp);

        while remains > 0 {
            fp = frame.previous_frame_address as usize;
            if fp == 0 {
                panic!("The index is out of bounds.")
            }
            frame = self.read_frame(fp);
            remains -= 1;
        }

        FrameItem::new(fp, frame)
    }

    /**
     * get the current function stack frame
     *
     * return the FFP (FP of function) and FuncFrame.
     */
    pub fn get_function_frame(&self) -> FrameItem {
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

        let frame = self.read_frame(self.fp);
        if frame.function_frame_address as usize == self.fp {
            // the current frame is function frame
            FrameItem::new(self.fp, frame)
        } else {
            // the current frame is block frame
            let func_fp = frame.function_frame_address as usize;
            let func_frame = self.read_frame(func_fp);
            FrameItem::new(func_fp, func_frame)
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

        let frame = self.read_frame(self.fp);
        let func_fp = frame.function_frame_address;
        func_fp as usize + size_of::<Frame>()
    }

    fn read_frame(&self, addr: usize) -> &Frame {
        let ptr = (&self.data[addr..]).as_ptr();
        let obj = unsafe { &*(ptr as *const Frame) };
        obj
    }

    /// mapping a structure to the specified address.
    /// the caller must write value of each field through the return object.
    fn write_frame(&mut self, addr: usize) -> &mut Frame {
        let ptr = (&mut self.data[addr..]).as_mut_ptr();
        let obj = unsafe { &mut *(ptr as *mut Frame) };
        obj
    }

    /// move the specified number of operands to the swap are
    fn move_operands_to_swap(&mut self, operands_count: u16) {
        if operands_count == 0 {
            return;
        }

        let count_in_bytes = operands_count as usize * OPERAND_SIZE_IN_BYTES;
        let offset = self.sp - count_in_bytes;

        // memory copy
        let src = (&self.data[offset..]).as_ptr();
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
        let dst = (&mut self.data[self.sp..]).as_mut_ptr();
        unsafe {
            std::ptr::copy(src, dst, count_in_bytes);
        }

        // update the SP
        self.sp += count_in_bytes;
    }

    pub fn create_function_frame(
        &mut self,
        local_variables_allocate_bytes: u32,
        params_count: u16,
        results_count: u16,
        module_index: u32,
        internal_function_index: u32,
        return_module_index: u32,
        return_instruction_address: u32,
    ) {
        let previous_fp = self.fp;

        // move the arguments to swap first
        self.move_operands_to_swap(params_count);

        // ensure the free space
        self.ensure_stack_space();

        // create new block frame at the current position (it's the value of SP)
        let fp = self.sp;
        let mut func_frame = self.write_frame(fp);

        // write values
        func_frame.previous_frame_address = previous_fp as u32;
        func_frame.function_frame_address = fp as u32; // the function FP point to the current frame itself.
        func_frame.params_count = params_count;
        func_frame.results_count = results_count;
        func_frame.module_index = module_index;

        func_frame.internal_function_index = internal_function_index;
        func_frame.local_variables_allocate_bytes = local_variables_allocate_bytes;
        func_frame.return_module_index = return_module_index;
        func_frame.return_instruction_address = return_instruction_address;

        // update sp and fp
        self.sp += size_of::<Frame>();
        self.fp = fp;

        // allocate local variable area
        self.sp += local_variables_allocate_bytes as usize;

        // restore the arguments from swap
        self.restore_operands_from_swap(params_count);
    }

    pub fn create_block_frame(&mut self, params_count: u16, results_count: u16) {
        let previous_fp = self.fp;

        // the 'func_fp' and 'module_index' are inherited from the previous frame
        let (func_fp, module_index) = {
            let frame_item = self.get_frame(0);
            let frame = frame_item.frame;
            let func_fp = frame.function_frame_address;
            let module_index = frame.module_index;
            (func_fp, module_index)
        };

        // move the arguments to swap first
        self.move_operands_to_swap(params_count);

        // create new block frame at the current position (it's the value of SP)
        let fp = self.sp;
        let mut block_frame = self.write_frame(fp);

        // write values
        block_frame.previous_frame_address = previous_fp as u32; // singly linked list
        block_frame.function_frame_address = func_fp;
        block_frame.params_count = params_count;
        block_frame.results_count = results_count;
        block_frame.module_index = module_index;

        block_frame.internal_function_index = 0;
        block_frame.local_variables_allocate_bytes = 0;
        block_frame.return_module_index = 0;
        block_frame.return_instruction_address = 0;

        // update sp and fp
        self.sp += size_of::<Frame>();
        self.fp = fp;

        // restore the arguments from swap
        self.restore_operands_from_swap(params_count);
    }

    /// remove the specified frame and all frames that follows this frame.
    ///
    /// return (is_function_frame, return_module_index, return_instruction_address)
    pub fn exit_frames(&mut self, reversed_index: usize) -> (bool, u32, u32) {
        let (
            sp,
            fp,
            is_function_frame,
            results_count,
            return_module_index,
            return_instruction_address,
        ) = {
            let frame_item = self.get_frame(reversed_index);
            let is_function_frame =
                frame_item.frame.function_frame_address as usize == frame_item.address;
            (
                frame_item.address,                               // current frame start address
                frame_item.frame.previous_frame_address as usize, // previous FP
                is_function_frame,
                frame_item.frame.results_count,
                frame_item.frame.return_module_index,
                frame_item.frame.return_instruction_address,
            )
        };

        // move the specified number of operands to swap
        self.move_operands_to_swap(results_count);

        self.sp = sp;
        self.fp = fp;

        // restore parameters from swap
        self.restore_operands_from_swap(results_count);

        (
            is_function_frame,
            return_module_index,
            return_instruction_address,
        )
    }

    /// reset the specified frame:
    /// - initialize all local variable area (if present) to value 0
    /// - remove all oprands which follow the local variable area
    /// - remove all frames which follow the current frame
    /// - moves the specified number of operands to the top of stack
    ///
    /// this function is commonly used for 'loop' structure or 'tail call' statement.
    pub fn reset_to_frame(&mut self, reversed_index: usize) {
        let (frame_addr, params_count, frame_func_fp, frame_local_vars_len_in_bytes) = {
            let frame_item = self.get_frame(reversed_index);
            (
                frame_item.address,
                frame_item.frame.params_count,
                frame_item.frame.function_frame_address as usize,
                frame_item.frame.local_variables_allocate_bytes as usize,
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
                == self.fp + size_of::<Frame>())
        {
            return;
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

            let local_vars_addr_start = frame_addr + size_of::<Frame>();
            self.sp = local_vars_addr_start + frame_local_vars_len_in_bytes;

            // re-initialize the local variable area
            let dst = self.data[local_vars_addr_start..].as_mut_ptr();
            unsafe {
                std::ptr::write_bytes(dst, 0, frame_local_vars_len_in_bytes);
            }

            // restore parameters from swap
            self.restore_operands_from_swap(params_count);
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
            self.sp = frame_addr + size_of::<Frame>();

            // restore parameters from swap
            self.restore_operands_from_swap(params_count);
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
        stack::{Frame, Stack},
        type_memory::TypeMemory,
        MEMORY_PAGE_SIZE_IN_BYTES, STACK_FRAME_SIZE_IN_PAGES,
    };

    impl Stack {
        fn read_local_by_offset_i32(&self, offset: usize) -> i32 {
            self.read_i32(self.get_local_variables_start_address() + offset)
        }

        fn write_local_by_offset_i32(&mut self, offset: usize, value: i32) {
            self.write_i32(self.get_local_variables_start_address() + offset, value)
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
        stack.push_i32(11);
        assert_eq!(stack.get_capacity_in_pages(), STACK_FRAME_SIZE_IN_PAGES);
        stack.ensure_stack_space();
        assert_eq!(stack.get_capacity_in_pages(), STACK_FRAME_SIZE_IN_PAGES * 2);

        // clear
        assert_eq!(stack.pop_i32(), 11);
        assert_eq!(stack.sp, 0);

        // fill up one frame
        for i in 0..(STACK_FRAME_SIZE_IN_PAGES * MEMORY_PAGE_SIZE_IN_BYTES / OPERAND_SIZE_IN_BYTES)
        {
            stack.push_i64(i as i64);
        }

        assert_eq!(
            stack.sp,
            STACK_FRAME_SIZE_IN_PAGES * MEMORY_PAGE_SIZE_IN_BYTES
        );

        // add one operand
        stack.push_i32(11);
        assert_eq!(stack.get_capacity_in_pages(), STACK_FRAME_SIZE_IN_PAGES * 2);
        stack.ensure_stack_space();
        assert_eq!(stack.get_capacity_in_pages(), STACK_FRAME_SIZE_IN_PAGES * 3);
    }

    #[test]
    fn test_push_and_pop_values() {
        let mut stack = Stack::new(STACK_FRAME_SIZE_IN_PAGES);

        // check push, peek and pop
        stack.push_i32(11);
        stack.push_i64(13);
        stack.push_f32(3.14);
        stack.push_f64(2.9979e8);

        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES * 4);
        assert_eq!(stack.peek_f64(), 2.9979e8);
        assert_eq!(stack.pop_f64(), 2.9979e8);
        assert_eq!(stack.pop_f32(), 3.14);

        assert_eq!(stack.peek_i64(), 13);
        assert_eq!(stack.pop_i64(), 13);
        assert_eq!(stack.pop_i32(), 11);
        assert_eq!(stack.sp, 0);

        // check duplicate
        stack.push_i32(17);
        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES);

        stack.duplicate();
        assert_eq!(stack.peek_i32(), 17);
        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES * 2);

        // check drop
        stack.push_i32(19);
        stack.push_i32(23);
        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES * 4);

        stack.drop();
        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES * 3);

        assert_eq!(stack.peek_i32(), 19);
    }

    #[test]
    fn test_host_address() {
        let mut stack = Stack::new(STACK_FRAME_SIZE_IN_PAGES);

        stack.push_i32(11);
        stack.push_i64(13);
        stack.push_i32(17);
        stack.push_i64(19);

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

        stack.push_i32(23);
        stack.push_i32(29);
        stack.push_i32(31);
        stack.push_i32(37);

        // the current layout
        //
        //              |        |
        //       0d0024 | 37     |
        //       0d0016 | 31     |
        //       0d0008 | 29     |
        // FP,SP 0d0000 | 23     |
        //              \--------/

        stack.create_function_frame(
            16, // local vars len
            2,  // params count
            0,  // results count
            73, // mod idx
            79, // func idx
            83, // ret mod idx
            89, //ret inst addr
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
        //       0d0040 | 83     | return module idx
        //       0d0036 | 16     | local vars len
        //       0d0032 | 79     | func idx
        //       0d0028 | 73     | module idx
        //       0d0024 | 2/0    | params/results count
        //       0d0020 | 16     | func FP
        // FP--> 0d0016 | 0      | prev FP
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/

        // check raw data
        assert_eq!(stack.read_i64(0), 23);
        assert_eq!(stack.read_i64(8), 29);

        assert_eq!(stack.read_i32(16), 0);
        assert_eq!(stack.read_i32(20), 16);
        assert_eq!(stack.read_i32(24), 0 << 16 | 2); // results count << 16 | params count
        assert_eq!(stack.read_i32(28), 73);
        assert_eq!(stack.read_i32(32), 79);
        assert_eq!(stack.read_i32(36), 16);
        assert_eq!(stack.read_i32(40), 83);
        assert_eq!(stack.read_i32(44), 89);

        assert_eq!(stack.read_i64(48), 0);
        assert_eq!(stack.read_i64(56), 0);

        assert_eq!(stack.read_i64(64), 31);
        assert_eq!(stack.read_i64(72), 37);

        // check status
        assert_eq!(stack.sp, 80);
        assert_eq!(stack.fp, 16);

        // check frame
        let f0 = stack.get_frame(0);
        assert_eq!(f0.address, 16);
        assert_eq!(
            f0.frame,
            &Frame {
                previous_frame_address: 0,
                function_frame_address: 16,
                params_count: 2,
                results_count: 0,
                // type_index: 71,
                module_index: 73,
                internal_function_index: 79,
                local_variables_allocate_bytes: 16,
                return_module_index: 83,
                return_instruction_address: 89
            }
        );

        let f0b = stack.get_function_frame();
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
            fp0 + size_of::<Frame>()
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

        stack.push_i32(41);
        stack.push_i32(43);
        stack.push_i32(47);

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
        //       0d0120 | 0      | return module idx
        //       0d0116 | 0      | local vars len
        //       0d0112 | 0      | func idx
        //       0d0108 | 73     | module idx
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
        assert_eq!(stack.read_i32(128), 47); // one operand has been moved
        assert_eq!(stack.read_i32(88), 43); // the old operand has no change

        let f1 = stack.get_frame(0);
        assert_eq!(f1.address, 96);
        assert_eq!(
            f1.frame,
            &Frame {
                previous_frame_address: 16,
                function_frame_address: 16,
                // type_index: 97,
                params_count: 1,
                results_count: 2,
                module_index: 73,
                internal_function_index: 0,
                local_variables_allocate_bytes: 0,
                return_module_index: 0,
                return_instruction_address: 0
            }
        );

        assert_eq!(stack.get_frame(1).address, fp0);
        assert_eq!(stack.get_function_frame(), stack.get_frame(1));

        let fp1 = f1.address;

        // check local variables

        // the values have no change
        assert_eq!(stack.read_local_by_offset_i32(0), 211);
        assert_eq!(stack.read_local_by_offset_i32(8), 223);
        assert_eq!(stack.read_local_by_offset_i32(16), 31);
        assert_eq!(stack.read_local_by_offset_i32(24), 37);

        // create block frame
        stack.create_block_frame(0, 0);

        let fp2 = fp1 + size_of::<Frame>() + 8; // 1 args in the 1st block frame

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
        assert_eq!(stack.sp, fp2 + size_of::<Frame>());
        assert_eq!(stack.read_i32(fp2 - 8), 47); // the old operand has no change

        let f2 = stack.get_frame(0);
        assert_eq!(f2.address, fp2);
        assert_eq!(
            f2.frame,
            &Frame {
                previous_frame_address: fp1 as u32,
                function_frame_address: fp0 as u32,
                // type_index: 101,
                params_count: 0,
                results_count: 0,
                module_index: 73,
                internal_function_index: 0,
                local_variables_allocate_bytes: 0,
                return_module_index: 0,
                return_instruction_address: 0
            }
        );

        assert_eq!(stack.get_frame(1).address, fp1);
        assert_eq!(stack.get_frame(2).address, fp0);
        assert_eq!(stack.get_function_frame(), stack.get_frame(2));

        let fp2 = f2.address;

        // check local variables

        assert_eq!(stack.read_local_by_offset_i32(0), 211); // only function frame has local variables
        assert_eq!(stack.read_local_by_offset_i32(8), 223); // so the block frame does change the local variable address
        assert_eq!(stack.read_local_by_offset_i32(16), 31);
        assert_eq!(stack.read_local_by_offset_i32(24), 37);

        // add operands
        stack.push_i32(239);
        stack.push_i32(241);

        // create func frame
        stack.create_function_frame(
            0, // local vars len
            1, // params count
            3, // results count
            // 103, // func type
            107, // mod idx
            109, // func idx
            113, // ret mod idx
            127, // ret inst addr
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

        let fp3 = fp2 + size_of::<Frame>() + 8; // 1 operand in the 2nd block frame
        assert_eq!(stack.fp, fp3);
        assert_eq!(stack.sp, fp3 + size_of::<Frame>() + 8); // 1 args in the current frame

        let f3 = stack.get_frame(0);
        assert_eq!(f3.address, fp3);
        assert_eq!(
            f3.frame,
            &Frame {
                previous_frame_address: fp2 as u32,
                function_frame_address: fp3 as u32,
                // type_index: 103,
                params_count: 1,
                results_count: 3,
                module_index: 107,
                internal_function_index: 109,
                local_variables_allocate_bytes: 0,
                return_module_index: 113,
                return_instruction_address: 127
            }
        );

        let f3b = stack.get_function_frame();
        assert_eq!(f3, f3b);

        assert_eq!(stack.get_frame(1).address, fp2);
        assert_eq!(stack.get_frame(2).address, fp1);
        assert_eq!(stack.get_frame(3).address, fp0);

        // check local variables

        // because function frame created new local variable area, the address should be updated
        assert_eq!(
            stack.get_local_variables_start_address(),
            fp3 + size_of::<Frame>()
        );
        assert_eq!(stack.read_local_by_offset_i32(0), 241);

        // remove the current frame

        // push some oparnds first
        stack.push_i32(251);
        stack.push_i32(257);

        // the current layout
        //
        //              |            |
        //              | 257        |
        //              | 251        |
        //              | 241        | <-- args 3
        //   func frame | frame 3    |
        //              |------------| <-- fp3

        let (is_function_frame0, ret_module_idx0, ret_inst_addr0) = stack.exit_frames(0);

        assert_eq!(is_function_frame0, true);
        assert_eq!(ret_module_idx0, 113);
        assert_eq!(ret_inst_addr0, 127);

        assert_eq!(stack.get_frame(0).address, fp2);
        assert_eq!(stack.get_frame(1).address, fp1);
        assert_eq!(stack.get_frame(2).address, fp0);

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

        assert_eq!(stack.read_i32(stack.sp - 8), 257);
        assert_eq!(stack.read_i32(stack.sp - 16), 251);
        assert_eq!(stack.read_i32(stack.sp - 24), 241);
        assert_eq!(stack.read_i32(stack.sp - 32), 239);

        // check local variables

        // because the 2nd function frame has been removed, so the address should be restored to
        // the 1st function frame
        assert_eq!(
            stack.get_local_variables_start_address(),
            fp0 + size_of::<Frame>()
        );

        assert_eq!(stack.read_local_by_offset_i32(0), 211);
        assert_eq!(stack.read_local_by_offset_i32(8), 223);
        assert_eq!(stack.read_local_by_offset_i32(16), 31);
        assert_eq!(stack.read_local_by_offset_i32(24), 37);

        // remove the parent frame
        let (is_function_frame1, ret_module_idx1, ret_inst_addr1) = stack.exit_frames(1);

        assert_eq!(is_function_frame1, false);
        assert_eq!(ret_module_idx1, 0);
        assert_eq!(ret_inst_addr1, 0);

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

        assert_eq!(stack.get_frame(0).address, fp0);
        assert_eq!(stack.sp, 112);

        // check operands
        assert_eq!(stack.read_i32(104), 257);
        assert_eq!(stack.read_i32(96), 251);
        assert_eq!(stack.read_i32(88), 43);
        assert_eq!(stack.read_i32(80), 41);

        // remove the last frame
        let (is_function_frame2, ret_module_idx2, ret_inst_addr2) = stack.exit_frames(0);

        assert_eq!(is_function_frame2, true);
        assert_eq!(ret_module_idx2, 83);
        assert_eq!(ret_inst_addr2, 89);

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

        stack.push_i32(23);
        stack.push_i32(29);
        stack.push_i32(31);
        stack.push_i32(37);

        // the current layout
        //
        //              |        |
        //       0d0024 | 37     |
        //       0d0016 | 31     |
        //       0d0008 | 29     |
        // FP,SP 0d0000 | 23     |
        //              \--------/

        stack.create_function_frame(
            16, // local vars len
            2,  // params count
            // 71, // func type
            0,  // results count
            73, // mod idx
            79, // func idx
            83, // ret mod idx
            89, //ret inst addr
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
        //       0d0040 | 83     | return module idx
        //       0d0036 | 16     | local vars len
        //       0d0032 | 79     | func idx
        //       0d0028 | 73     | module idx
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
        stack.push_i32(107);
        stack.push_i32(109);
        stack.push_i32(113);
        stack.push_i32(127);

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
        assert_eq!(stack.peek_i32(), 127);

        // reset the frame
        stack.reset_to_frame(0);

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
        //       0d0040 | 83     | return module idx
        //       0d0036 | 16     | local vars len
        //       0d0032 | 79     | func idx
        //       0d0028 | 73     | module idx
        //       0d0024 | 2/0    | params/results count
        //       0d0020 | 16     | func FP
        // FP--> 0d0016 | 0      | prev FP
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/

        // check frame
        let f0 = stack.get_frame(0);
        assert_eq!(f0.address, 16);
        assert_eq!(
            f0.frame,
            &Frame {
                previous_frame_address: 0,
                function_frame_address: 16,
                // type_index: 71,
                params_count: 2,
                results_count: 0,
                module_index: 73,
                internal_function_index: 79,
                local_variables_allocate_bytes: 16,
                return_module_index: 83,
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
        stack.push_i32(139);

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
        //       0d0092 | 73     | module idx
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
        //       0d0040 | 83     | return module idx
        //       0d0036 | 16     | local vars len
        //       0d0032 | 79     | func idx
        //       0d0028 | 73     | module idx
        //       0d0024 | 2/0    | params/results count
        //       0d0020 | 16     | func FP
        //       0d0016 | 0      | prev FP
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/

        assert_eq!(stack.fp, 80);
        assert_eq!(stack.sp, 120);

        assert_eq!(stack.peek_i32(), 139);

        // add operands
        stack.push_i32(149);
        stack.push_i32(151);

        // the current layout (partial)
        //
        // SP--> 0d0136 |        |
        //       0d0128 | 151    |
        //       0d0120 | 149    | <-- operands 1
        //       0d0112 | 139    | <-- args 1
        //              |--------|

        // reset the frame
        stack.reset_to_frame(0);

        // the current layout (partial)
        //
        // SP--> 0d0120 |        |
        //       0d0112 | 151    | <-- args 1
        //              |--------|
        //       0d0108 | 0      |
        //       0d0104 | 0      |
        //       0d0100 | 0      |
        //       0d0096 | 0      |
        //       0d0092 | 73     | module idx
        //       0d0088 | 1/2    | params/results count
        //       0d0084 | 16     | func FP
        // FP--> 0d0080 | 16     | prev FP
        //              |========| <-- fp1

        assert_eq!(stack.fp, 80);
        assert_eq!(stack.sp, 120);
        assert_eq!(stack.peek_i32(), 151);

        // check frame
        let f1 = stack.get_frame(0);
        assert_eq!(f1.address, 80);
        assert_eq!(
            f1.frame,
            &Frame {
                previous_frame_address: 16,
                function_frame_address: 16,
                // type_index: 149,
                params_count: 1,
                results_count: 2,
                module_index: 73,
                internal_function_index: 0,
                local_variables_allocate_bytes: 0,
                return_module_index: 0,
                return_instruction_address: 0
            }
        );

        // reset block frame again
        stack.reset_to_frame(0);

        // nothings has changed
        assert_eq!(stack.fp, 80);
        assert_eq!(stack.sp, 120);
        assert_eq!(stack.peek_i32(), 151);

        // create block frame

        // add some operands for preparing for the next reset
        stack.push_i32(157);

        stack.create_block_frame(0, 0);

        // the current layout (partial)
        //
        // SP--> 0d0160 |        |
        //              |--------|
        //       0d0156 | 0      |
        //       0d0152 | 0      |
        //       0d0148 | 0      |
        //       0d0144 | 0      |
        //       0d0140 | 73     | module idx
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
        //       0d0092 | 73     | module idx
        //       0d0088 | 1/2    | params/results count
        //       0d0084 | 16     | func FP
        //       0d0080 | 16     | prev FP
        //              |========| <-- fp1

        assert_eq!(stack.fp, 128);
        assert_eq!(stack.sp, 160);

        // add two operands
        stack.push_i32(167);
        stack.push_i32(173);

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
        stack.reset_to_frame(1);

        // the current layout (partial)
        //
        // SP--> 0d0120 |        |
        //       0d0112 | 173    | <-- args 1 from operands 2
        //              |--------|
        //       0d0108 | 0      |
        //       0d0104 | 0      |
        //       0d0100 | 0      |
        //       0d0096 | 0      |
        //       0d0092 | 73     | module idx
        //       0d0088 | 149    | func type
        //       0d0084 | 16     | func FP
        // FP--> 0d0080 | 16     | prev FP
        //              |========| <-- fp1

        assert_eq!(stack.fp, 80);
        assert_eq!(stack.sp, 120);

        // check args
        assert_eq!(stack.read_i32(stack.sp - 8), 173);

        // crossing reset

        // add two operands
        stack.push_i32(181);
        stack.push_i32(191);

        // the current layout (partial)
        //
        // SP--> 0d0136 |        |
        //       0d0128 | 191    |
        //       0d0120 | 181    |
        //       0d0112 | 173    | <-- args 1 from operands 2
        //              |--------|

        // the params count of target frame is 2
        stack.reset_to_frame(1);

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
        //       0d0040 | 83     | return module idx
        //       0d0036 | 16     | local vars len
        //       0d0032 | 79     | func idx
        //       0d0028 | 73     | module idx
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
