// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::mem::size_of;

use ancvm_types::OPERAND_SIZE_IN_BYTES;

use crate::{
    memory::Memory, resizeable_memory::ResizeableMemory, MEMORY_PAGE_SIZE_IN_BYTES,
    STACK_FRAME_SIZE_IN_PAGES,
};

pub struct Stack {
    data: Vec<u8>,

    // a temporary storage.
    // when creating a new stack frame, the arguments (operands) are moved
    // from stack to swap area first, then the frame information is created,
    // as well as allocating local variables slots,
    // finnaly the arguments are restored from the swap to stack.
    // when exiting a stack frame, the process is similar.
    swap: Vec<u8>,

    // the end position of the stack (a.k.a. SP)
    pub sp: usize,

    // the current frame position (a.k.a. FP)
    pub fp: usize,
}

// the calling frame and the frame information
//
// | ...                  |
// | ...                  |
// |======================|
// | operand N            |                                   | ...                  |
// | operand 1            |                                   | ...                  |
// | operand 0            | <-- operands                      |======================|
// |----------------------|                                   | operand N            |
// | arg 1                |                                   | operand 1            |
// | arg 0                | <-- args from caller              | operand 0            |
// |----------------------|                                   |----------------------|
// | local 1              |                                   | arg 1                |
// | local 0              | <-- local variable slots          | arg 0                |
// |----------------------| <-------------------------------> |----------------------|
// | return inst addr     |                                   | 0                    |
// | return module idx    |                                   | 0                    |
// | local vars len       |                                   | 0                    |
// | current func idx     |                                   | 0                    |
// | current module index |                                   | current module index |
// | current func type    |                                   | current block type   |
// | function FP          |     func           block          | function FP          |
// | previous FP          | <-- frame info --- frame info --> | previous FP          |
// |======================| <-- FP                     FP --> |======================|
// | ...                  |                                   | ...                  |
// | ...                  |                                   | ...                  |
// \----------------------/ <-- stack start                   \----------------------/
//
// the chain of calling frames
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

#[derive(Debug)]
#[repr(C)]
pub struct Frame {
    pub previous_fp: u32,       //--\
    pub func_fp: u32,           //--/-- 8 bytes <-- addr low
    pub func_type: u32,         //--\
    pub module_index: u32,      //--/-- 8 bytes
    pub func_index: u32,        //--\
    pub local_vars_len: u32,    //--/-- 8 bytes
    pub return_module_idx: u32, //--\
    pub return_inst_addr: u32,  //--/-- 8 bytes <-- addr high
}

#[derive(Debug)]
pub struct FrameItem<'a> {
    pub addr: usize,
    pub frame: &'a Frame,
}

impl<'a> FrameItem<'a> {
    pub fn new(addr: usize, frame: &'a Frame) -> Self {
        Self { addr, frame }
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
    fn get_ptr(&self, addr: usize) -> *const u8 {
        (&self.data[addr..]).as_ptr()
    }

    #[inline]
    fn get_mut_ptr(&mut self, addr: usize) -> *mut u8 {
        (&mut self.data[addr..]).as_mut_ptr()
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

    pub fn push_i64(&mut self, value: i64) {
        self.write_i64(self.sp, value);
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

    pub fn peek_i64(&self) -> i64 {
        self.read_i64(self.sp - OPERAND_SIZE_IN_BYTES)
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

    pub fn pop_i64(&mut self) -> i64 {
        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_i64(self.sp)
    }

    pub fn pop_f32(&mut self) -> f32 {
        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_f32(self.sp)
    }

    pub fn pop_f64(&mut self) -> f64 {
        self.sp -= OPERAND_SIZE_IN_BYTES;
        self.read_f64(self.sp)
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

        let mut remains = reversed_index;
        let mut fp = self.fp;
        let mut frame = self.read_frame(fp);

        while remains > 0 {
            fp = frame.previous_fp as usize;
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
    pub fn get_func_frame(&self) -> FrameItem {
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
        if frame.func_fp as usize == self.fp {
            FrameItem::new(self.fp, frame)
        } else {
            let func_fp = frame.func_fp as usize;
            let func_frame = self.read_frame(func_fp);
            FrameItem::new(func_fp, func_frame)
        }
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
    fn move_operands_to_swap(&mut self, operands_count: usize) {
        if operands_count == 0 {
            return;
        }

        let count_in_bytes = operands_count * OPERAND_SIZE_IN_BYTES;
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

    fn restore_operands_from_swap(&mut self, operands_count: usize) {
        if operands_count == 0 {
            return;
        }

        let count_in_bytes = operands_count * OPERAND_SIZE_IN_BYTES;

        // memory copy
        let src = self.swap.as_ptr();
        let dst = (&mut self.data[self.sp..]).as_mut_ptr();
        unsafe {
            std::ptr::copy(src, dst, count_in_bytes);
        }

        // update the SP
        self.sp += count_in_bytes;
    }

    pub fn create_block_frame(&mut self, params_count: usize, func_type: u32) {
        let previous_fp = self.fp;

        // the 'func_fp' and 'module_index' are inherited from the previous frame
        let (func_fp, module_index) = {
            let frame_item = self.get_frame(0);
            let frame = frame_item.frame;
            let func_fp = frame.func_fp;
            let module_index = frame.module_index;
            (func_fp, module_index)
        };

        // move the arguments to swap first
        self.move_operands_to_swap(params_count);

        // create new block frame at the current position (it's the value of SP)
        let fp = self.sp;
        let mut block_frame = self.write_frame(fp);

        // write values
        block_frame.previous_fp = previous_fp as u32; // singly linked list
        block_frame.func_fp = func_fp;
        block_frame.func_type = func_type;
        block_frame.module_index = module_index;

        block_frame.func_index = 0;
        block_frame.local_vars_len = 0;
        block_frame.return_module_idx = 0;
        block_frame.return_inst_addr = 0;

        // update sp and fp
        self.sp += size_of::<Frame>();
        self.fp = fp;

        // restore the arguments from swap
        self.restore_operands_from_swap(params_count);
    }

    pub fn create_func_frame(
        &mut self,
        local_variables_length_in_bytes: u32,
        params_count: usize,
        func_type: u32,
        module_index: u32,
        func_index: u32,
        return_module_idx: u32,
        return_inst_addr: u32,
    ) {
        // the function frame
        //
        // |                 |
        // |-----------------|
        // | arguments       |
        // |-----------------|
        // | local variables |
        // |-----------------|
        // | frame info      |
        // |=================| <-- FP, FFP
        // | ...             |
        // \-----------------/

        let previous_fp = self.fp;

        // move the arguments to swap first
        self.move_operands_to_swap(params_count);

        // ensure the free space
        self.ensure_stack_space();

        // create new block frame at the current position (it's the value of SP)
        let fp = self.sp;
        let mut func_frame = self.write_frame(fp);

        // write values
        func_frame.previous_fp = previous_fp as u32;
        func_frame.func_fp = fp as u32; // the function FP point to the current frame itself.
        func_frame.func_type = func_type;
        func_frame.module_index = module_index;

        func_frame.func_index = func_index;
        func_frame.local_vars_len = local_variables_length_in_bytes;
        func_frame.return_module_idx = return_module_idx;
        func_frame.return_inst_addr = return_inst_addr;

        // update sp and fp
        self.sp += size_of::<Frame>();
        self.fp = fp;

        // allocate local variable slots
        self.sp += local_variables_length_in_bytes as usize;

        // restore the arguments from swap
        self.restore_operands_from_swap(params_count);
    }

    /// remove the specified frame and all frames that follows this frame.
    pub fn remove_frames(&mut self, frame_item: &FrameItem) {
        let fp = frame_item.frame.previous_fp;
        self.sp = frame_item.addr;
        self.fp = fp as usize;
    }

    /// reset the specified frame:
    /// - initialize all local variable slots (if present) to value 0
    /// - remove all oprands which follow the local variable area
    /// - remove all frames which follow the current frame
    /// - moves the specified number of operands to the top of stack
    ///
    /// this function is commonly used for 'loop' structure or 'tail call' statement.
    pub fn reset_to_frame(&mut self, frame_item: &FrameItem, params_count: usize) {
        //

    }
}

#[cfg(test)]
mod tests {
    use ancvm_types::OPERAND_SIZE_IN_BYTES;

    use crate::{
        resizeable_memory::ResizeableMemory, stack::Stack, MEMORY_PAGE_SIZE_IN_BYTES,
        STACK_FRAME_SIZE_IN_PAGES,
    };

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

        // check sp, peek, pop
        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES);
        assert_eq!(stack.peek_i32(), 11);
        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES);
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

        // check data value
        stack.push_i32(13);
        stack.push_i64(17);
        stack.push_f32(3.14);
        stack.push_f64(2.9979e8);

        assert_eq!(stack.sp, OPERAND_SIZE_IN_BYTES * 4);
        assert_eq!(stack.pop_f64(), 2.9979e8);
        assert_eq!(stack.pop_f32(), 3.14);
        assert_eq!(stack.pop_i64(), 17);
        assert_eq!(stack.pop_i32(), 13);
        assert_eq!(stack.sp, 0);
    }
}
