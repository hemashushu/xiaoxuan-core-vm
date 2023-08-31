// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::mem::size_of;

use ancvm_types::OPERAND_SIZE_IN_BYTES;

use crate::{
    host_accessable_memory::HostAccessableMemory, memory::Memory,
    resizeable_memory::ResizeableMemory, MEMORY_PAGE_SIZE_IN_BYTES, STACK_FRAME_SIZE_IN_PAGES,
};

pub struct Stack {
    data: Vec<u8>,

    // a temporary storage.
    //
    // when creating a new stack frame:
    //
    // 1. the arguments (i.e. the operands on the top of stack) are moved from stack to swap first,
    // 2. then the frame information is created,
    // 3. as well as allocating local variables slots,
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
// | function FP          |     func                block     | function FP          |
// | previous FP          | <-- frame info     frame info --> | previous FP          |
// |======================| <-- FP                     FP --> |======================|
// | ...                  |                                   | ...                  |
// | ...                  |                                   | ...                  |
// \----------------------/ <-- stack start                   \----------------------/
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
    pub previous_fp: u32,       //--\
    pub func_fp: u32,           //--/-- 8 bytes <-- addr low
    pub func_type: u32,         //--\
    pub module_index: u32,      //--/-- 8 bytes
    pub func_index: u32,        //--\
    pub local_vars_len: u32,    //--/-- 8 bytes
    pub return_module_idx: u32, //--\
    pub return_inst_addr: u32,  //--/-- 8 bytes <-- addr high
}

#[derive(Debug, PartialEq)]
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

impl HostAccessableMemory for Stack {
    #[inline]
    fn get_host_address(&self, offset: usize) -> usize {
        (&self.data[offset..]).as_ptr() as usize
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
            // the current frame is function frame
            FrameItem::new(self.fp, frame)
        } else {
            // the current frame is block frame
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
    pub fn remove_frames(&mut self, reversed_index: usize, results_count: usize) {
        // move the specified number of operands to swap
        self.move_operands_to_swap(results_count);

        let (sp, fp) = {
            let frame_item = self.get_frame(reversed_index);
            (frame_item.addr, frame_item.frame.previous_fp as usize)
        };
        self.sp = sp;
        self.fp = fp;

        // restore parameters from swap
        self.restore_operands_from_swap(results_count);
    }

    /// reset the specified frame:
    /// - initialize all local variable slots (if present) to value 0
    /// - remove all oprands which follow the local variable slots
    /// - remove all frames which follow the current frame
    /// - moves the specified number of operands to the top of stack
    ///
    /// this function is commonly used for 'loop' structure or 'tail call' statement.
    pub fn reset_to_frame(&mut self, reversed_index: usize, params_count: usize) {
        let (frame_addr, frame_func_fp, frame_local_vars_len_in_bytes) = {
            let frame_item = self.get_frame(reversed_index);
            (
                frame_item.addr,
                frame_item.frame.func_fp as usize,
                frame_item.frame.local_vars_len as usize,
            )
        };

        let is_func_frame: bool = frame_addr == frame_func_fp;

        // optimized for blocks like 'loop' structures.
        //
        // - block frame
        // - the specified frame is the current frame
        // - there is no other operands than parameters
        //
        // just do nothing when all conditions are met.
        if (!is_func_frame)
            && (frame_addr == self.fp)
            && (self.sp - params_count == self.fp + size_of::<Frame>())
        {
            return;
        }

        // move the specified number of operands to swap
        self.move_operands_to_swap(params_count);

        if is_func_frame {
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

            // re-initialize the local variable slots
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

/// local variables access
///
/// note:
/// - function arguments can also be read/write as local variables.
/// - block has arguments but has no local variables.
/// - block arguments can NOT be read/write as local variables.
impl Stack {
    /// get the local variables start address
    ///
    /// note that the address is calculated by 'FP + the size of Frame', so
    /// even if there is no local variable slots in the current function frame,
    /// this function always return the calculated address.
    pub fn get_local_address(&self) -> usize {
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
        let func_fp = frame.func_fp;
        func_fp as usize + size_of::<Frame>()
    }

    pub fn get_local_host_address(&self) -> usize {
        let addr = self.get_local_address();
        (&self.data[addr..]).as_ptr() as usize
    }

    pub fn read_local_i32(&self, offset: usize) -> i32 {
        self.read_i32(self.get_local_address() + offset)
    }

    pub fn read_local_i64(&self, offset: usize) -> i64 {
        self.read_i64(self.get_local_address() + offset)
    }

    pub fn read_local_f32(&self, offset: usize) -> f32 {
        self.read_f32(self.get_local_address() + offset)
    }

    pub fn read_local_f64(&self, offset: usize) -> f64 {
        self.read_f64(self.get_local_address() + offset)
    }

    pub fn write_local_i32(&mut self, offset: usize, value: i32) {
        self.write_i32(self.get_local_address() + offset, value)
    }

    pub fn write_local_i64(&mut self, offset: usize, value: i64) {
        self.write_i64(self.get_local_address() + offset, value)
    }

    pub fn write_local_f32(&mut self, offset: usize, value: f32) {
        self.write_f32(self.get_local_address() + offset, value)
    }

    pub fn write_local_f64(&mut self, offset: usize, value: f64) {
        self.write_f64(self.get_local_address() + offset, value)
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
        MEMORY_PAGE_SIZE_IN_BYTES, STACK_FRAME_SIZE_IN_PAGES,
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

        stack.create_func_frame(
            16, // local vars len
            2,  // params count
            71, // func type
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
        //       0d0024 | 71     | func type
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
        assert_eq!(stack.read_i32(24), 71);
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
        assert_eq!(f0.addr, 16);
        assert_eq!(
            f0.frame,
            &Frame {
                previous_fp: 0,
                func_fp: 16,
                func_type: 71,
                module_index: 73,
                func_index: 79,
                local_vars_len: 16,
                return_module_idx: 83,
                return_inst_addr: 89
            }
        );

        let f0b = stack.get_func_frame();
        assert_eq!(f0, f0b);

        let fp0 = f0.addr;

        // check local variables

        // the current layout (partial)
        //
        //       0d0072 | 37     |
        //       0d0064 | 31     | <-- args 0
        //              |--------|
        //       0d0056 | 0      |
        //       0d0048 | 0      | <-- local vars 0
        //              |--------|

        assert_eq!(stack.get_local_address(), fp0 + size_of::<Frame>());
        assert_eq!(stack.read_local_i32(0), 0);
        assert_eq!(stack.read_local_i32(8), 0);
        assert_eq!(stack.read_local_i32(16), 31);
        assert_eq!(stack.read_local_i32(24), 37);

        stack.write_local_i32(0, 211);
        stack.write_local_i32(8, 223);

        assert_eq!(stack.read_local_i32(0), 211);
        assert_eq!(stack.read_local_i32(8), 223);

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
        stack.create_block_frame(1, 97);

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
        //       0d0104 | 97     | func type
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
        assert_eq!(f1.addr, 96);
        assert_eq!(
            f1.frame,
            &Frame {
                previous_fp: 16,
                func_fp: 16,
                func_type: 97,
                module_index: 73,
                func_index: 0,
                local_vars_len: 0,
                return_module_idx: 0,
                return_inst_addr: 0
            }
        );

        assert_eq!(stack.get_frame(1).addr, fp0);
        assert_eq!(stack.get_func_frame(), stack.get_frame(1));

        let fp1 = f1.addr;

        // check local variables

        assert_eq!(stack.read_local_i32(0), 211);
        assert_eq!(stack.read_local_i32(8), 223);
        assert_eq!(stack.read_local_i32(16), 31);
        assert_eq!(stack.read_local_i32(24), 37);

        // create block frame
        stack.create_block_frame(0, 101);

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
        assert_eq!(f2.addr, fp2);
        assert_eq!(
            f2.frame,
            &Frame {
                previous_fp: fp1 as u32,
                func_fp: fp0 as u32,
                func_type: 101,
                module_index: 73,
                func_index: 0,
                local_vars_len: 0,
                return_module_idx: 0,
                return_inst_addr: 0
            }
        );

        assert_eq!(stack.get_frame(1).addr, fp1);
        assert_eq!(stack.get_frame(2).addr, fp0);
        assert_eq!(stack.get_func_frame(), stack.get_frame(2));

        let fp2 = f2.addr;

        // check local variables

        assert_eq!(stack.read_local_i32(0), 211); // only function frame has local variables
        assert_eq!(stack.read_local_i32(8), 223); // so the block frame does change the local variable address
        assert_eq!(stack.read_local_i32(16), 31);
        assert_eq!(stack.read_local_i32(24), 37);

        // add operands
        stack.push_i32(239);
        stack.push_i32(241);

        // create func frame
        stack.create_func_frame(
            0,   // local vars len
            1,   // params count
            103, // func type
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
        assert_eq!(f3.addr, fp3);
        assert_eq!(
            f3.frame,
            &Frame {
                previous_fp: fp2 as u32,
                func_fp: fp3 as u32,
                func_type: 103,
                module_index: 107,
                func_index: 109,
                local_vars_len: 0,
                return_module_idx: 113,
                return_inst_addr: 127
            }
        );

        let f3b = stack.get_func_frame();
        assert_eq!(f3, f3b);

        assert_eq!(stack.get_frame(1).addr, fp2);
        assert_eq!(stack.get_frame(2).addr, fp1);
        assert_eq!(stack.get_frame(3).addr, fp0);

        // check local variables
        assert_eq!(stack.get_local_address(), fp3 + size_of::<Frame>()); // because function frame created new local variable slots, the address should be updated
        assert_eq!(stack.read_local_i32(0), 241);

        // remove the current frame

        // push some oparnds first
        stack.push_i32(251);
        stack.push_i32(257);

        stack.remove_frames(0, 3);

        assert_eq!(stack.get_frame(0).addr, fp2);
        assert_eq!(stack.get_frame(1).addr, fp1);
        assert_eq!(stack.get_frame(2).addr, fp0);

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
        assert_eq!(stack.get_local_address(), fp0 + size_of::<Frame>());

        assert_eq!(stack.read_local_i32(0), 211);
        assert_eq!(stack.read_local_i32(8), 223);
        assert_eq!(stack.read_local_i32(16), 31);
        assert_eq!(stack.read_local_i32(24), 37);

        // remove the parent frame
        stack.remove_frames(1, 2);

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

        assert_eq!(stack.get_frame(0).addr, fp0);
        assert_eq!(stack.sp, 112);

        // check operands
        assert_eq!(stack.read_i32(104), 257);
        assert_eq!(stack.read_i32(96), 251);
        assert_eq!(stack.read_i32(88), 43);
        assert_eq!(stack.read_i32(80), 41);
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

        stack.create_func_frame(
            16, // local vars len
            2,  // params count
            71, // func type
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
        //       0d0024 | 71     | func type
        //       0d0020 | 16     | func FP
        // FP--> 0d0016 | 0      | prev FP
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/

        // update local variables
        stack.write_local_i32(0, 101);
        stack.write_local_i32(8, 103);

        // check local variables
        assert_eq!(stack.read_local_i32(0), 101);
        assert_eq!(stack.read_local_i32(8), 103);
        assert_eq!(stack.read_local_i32(16), 31);
        assert_eq!(stack.read_local_i32(24), 37);

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
        stack.reset_to_frame(0, 2);

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
        //       0d0024 | 71     | func type
        //       0d0020 | 16     | func FP
        // FP--> 0d0016 | 0      | prev FP
        //              |========| <-- fp0
        //       0d0008 | 29     |
        //       0d0000 | 23     |
        //              \--------/

        // check local variables
        assert_eq!(stack.read_local_i32(0), 0); // reset
        assert_eq!(stack.read_local_i32(8), 0); // reset
        assert_eq!(stack.read_local_i32(16), 113); // updated
        assert_eq!(stack.read_local_i32(24), 127); // updated

        // TODO::
    }

    #[test]
    fn test_reset_frame_depth() {
        // TODO::
    }
}
