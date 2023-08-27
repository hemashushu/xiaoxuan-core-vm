// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

const STACK_FRAME_SIZE_IN_BYTES: usize = 32 * 1024;
const INIT_STACK_SIZE_IN_BYTES: usize = STACK_FRAME_SIZE_IN_BYTES;
const INIT_HEAP_SIZE_IN_BYTES: usize = 0;

/// a thread per VM instance
pub struct VM {
    // operand stack
    // also includes the function/block frame info when call a function or enter a block,
    //
    // the default stack capacity is 32 KiB, when a new stack frame is created, the
    // VM will check the capacity of the stack and ensure there is at lease 32 KiB
    // for the current frame.
    // the capacity of stack will be incremented in 32 KiB, i.e. the capacity will be
    // 32, 64, 96, 128 KiB and so on.
    //
    // the following diagram demostrates the stack changing when entering or leaving a function/block.
    //
    // 1. function 1 is going to call function 2,
    //    the arguments were ready.
    // |         |
    // |         |
    // |  arg 1  | <-- operands that will be used as arguments
    // |  arg 0  |
    // |---------|
    // |   ###   | <-- other operands of function 1
    // |---------| <-- current stack frame pointer (FP)
    // |   ...   |
    // \---------/ <-- stack start
    //
    // 2. called function 2.
    //
    // |         |
    // | local 1 |
    // | local 0 | <-- allocates the local variable slots
    // |---------|
    // |  arg 1  | <-- arguments will be copied to the top of stack, follows the frame infos.
    // |  arg 0  |
    // |---------|
    // |   $$$   | <-- the frame infos, includes the previous FP, return address (func and module) etc.
    // |   $$$   |     also includes the current function infos, such as func type, index, module, type etc.
    // |   $$$   |     note that the arguments will be moved out first, and then the frame infos were created.
    // |---------| <-- new stack frame pointer (FP of function 2)
    // |   ###   | <-- other operands of function 1
    // |---------| <-- function 1 stack frame pointer (FP of function 1)
    // |   ...   |
    // \---------/
    //
    // 3. function 2 is going to return function 1 with two results
    //
    // |         |
    // | resul 1 |
    // | resul 0 | <-- results
    // |---------|
    // |   ###   | <-- other operands of function 2
    // |---------|
    // | local 1 |
    // | local 0 |
    // |---------|
    // |  arg 1  |
    // |  arg 0  |
    // |---------|
    // |   $$$   |
    // |   $$$   |
    // |   $$$   |
    // |---------| <-- FP of function 2
    // |   ###   | <-- other operands of function 1
    // |---------| <-- FP of function 1
    // |   ...   |
    // \---------/
    //
    // 4. returned
    //
    // |         |
    // | resul 1 | <-- the results were copied to the position immediately following the
    // | resul 0 |     function 1 operands, all data between the results and FP 2 will be removed.
    // |---------|
    // |   ###   | <-- other operands of function 1
    // |---------| <-- FP of function 1
    // |   ...   |
    // \---------/
    //
    stack: Vec<u8>,

    // for copying operands
    swap: Vec<u8>,

    // in XiaoXuan VM, the data sections (read-only, read-write, uninit) are all thread-local,
    // and the heap is thread-local also.
    // threads/processes can communicated through the MessageBox/MessagePipe or the SharedMemory
    //
    // note that the initial capacity of heap is 0 byte
    heap: Vec<u8>,

    // the end position of the operand stack (a.k.a. SP)
    pub sp: usize,

    // the current frame position in the operand stack (a.k.a. FP)
    pub fp: usize,

    // the position of the next executing instruction (a.k.a. IP/PC)
    // the XiaoXuan VM load multiple modules for a application, thus the
    // "complete IP" consists of the module index and the instruction position.
    pub pc: ProgramCounter,
}

#[derive(Debug, PartialEq)]
pub struct ProgramCounter {
    pub addr: usize,
    pub module_index: u16,
}

// the calling frame and the frame information
//
// | ...                  |
// | ...                  |
// |======================|
// | operand N            |
// | operand 1            |
// | operand 0            | <-- operands
// |----------------------|
// | local 1              |
// | local 0              |
// |----------------------|
// | arg 1                |
// | arg 0                | <-- args from caller
// |----------------------|
// | return inst addr     |
// | return module idx    |
// | current func type    |
// | current func idx     |
// | current module index |
// | frame type           |
// | previous FP          | <-- frame information
// |======================| <-- FP
// | ...                  |
// | ...                  |
// \----------------------/ <-- stack start
//
// the chain of calling frames
//
// |             |
// | ...         |
// | ...         |
// | previous FP | ---\
// |-------------|    |
// | ...         |    |
// | ...         |    |
// | previous FP | ---|--\
// |-------------| <--/   |
// | ...         |        |
// | ...         |        |
// | previous FP | -------|--\
// |-------------| <-----/   |
// | ...         |           |
// | ...         |           |
// | previous FP |           |
// |-------------| <---------/
// | ...         |
// \-------------/ <-- stack start

#[derive(Debug)]
#[repr(u16)]
pub enum FrameType {
    Function = 0x0,
    Block,
}

#[derive(Debug)]
#[repr(C)]
pub struct FrameInfo {
    pub previous_fp: u64, // 8 bytes

    pub module_index: u16, //---\
    pub func_type: u16,    //   | 8 bytes
    pub func_index: u32,   //---/

    pub frame_type: FrameType,  //--\
    pub return_module_idx: u16, //  | 8 bytes
    pub return_inst_addr: u32,  //--/
}

impl VM {
    pub fn new() -> Self {
        let pc = ProgramCounter {
            addr: 0,
            module_index: 0,
        };

        let stack: Vec<u8> = vec![0u8; INIT_STACK_SIZE_IN_BYTES];
        let heap: Vec<u8> = vec![0u8; INIT_HEAP_SIZE_IN_BYTES];
        let swap: Vec<u8> = vec![0u8; INIT_STACK_SIZE_IN_BYTES];

        Self {
            stack,
            swap,
            heap,
            sp: 0,
            fp: 0,
            pc,
        }
    }
}

// implement the stack

impl VM {
    pub fn get_stack_capacity(&self) -> usize {
        self.stack.len()
    }

    pub fn ensure_stack_capacity(&mut self) -> usize {
        // check the capacity of the stack to make sure
        // there is enough space for a call stack frame.
        // as well as increasing the capacity in the specified
        // increment (the default value is 32 KiB), that is,
        // the capacity of the stack can only be 32, 64, 96, 128 KiB and so on.

        let len = self.stack.len();
        if len - self.sp < STACK_FRAME_SIZE_IN_BYTES {
            let new_len = len + STACK_FRAME_SIZE_IN_BYTES;
            self.stack.resize(new_len, 0);
            new_len
        } else {
            len
        }
    }

    pub fn push_i32(&mut self, value: i32) {
        let data = &mut self.stack[self.sp..];
        let ptr = data as *mut [u8] as *mut i32;
        unsafe { std::ptr::write(ptr, value) };
        self.sp += 8;
    }

    pub fn stack_read_i32(&self, addr: usize) -> i32 {
        let data = &self.stack[addr..];
        let ptr = data as *const [u8] as *const i32;
        unsafe { std::ptr::read(ptr) }
    }

    pub fn peek_i32(&self) -> i32 {
        self.stack_read_i32(self.sp - 8)
    }

    pub fn pop_i32(&mut self) -> i32 {
        let value = self.peek_i32();
        self.sp -= 8;
        value
    }
}

#[cfg(test)]
mod tests {
    use crate::vm::{STACK_FRAME_SIZE_IN_BYTES, VM};

    #[test]
    fn test_stack_capacity() {
        let mut vm = VM::new();
        assert_eq!(vm.sp, 0);

        // check the initial size
        assert_eq!(vm.get_stack_capacity(), STACK_FRAME_SIZE_IN_BYTES);
        assert_eq!(vm.ensure_stack_capacity(), STACK_FRAME_SIZE_IN_BYTES);
        assert_eq!(vm.get_stack_capacity(), STACK_FRAME_SIZE_IN_BYTES);

        // add some bytes
        vm.push_i32(11);
        assert_eq!(vm.get_stack_capacity(), STACK_FRAME_SIZE_IN_BYTES);
        assert_eq!(vm.ensure_stack_capacity(), STACK_FRAME_SIZE_IN_BYTES * 2);
        assert_eq!(vm.get_stack_capacity(), STACK_FRAME_SIZE_IN_BYTES * 2);

        assert_eq!(vm.peek_i32(), 11);
    }
}
