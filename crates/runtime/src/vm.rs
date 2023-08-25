// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

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
    pub stack: Vec<u8>,

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
    // \---------/
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

    // the end position of the operand stack (a.k.a. SP)
    pub current_stack_position: usize,

    // the current frame position in the operand stack (a.k.a. FP)
    pub current_stack_frame_position: usize,

    // the calling frame
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
    // \----------------------/

    // the calling frames
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
    // \-------------/

    // the position of the next executing instruction (a.k.a. IP/PC)
    // the XiaoXuan VM load multiple modules for a application, thus the
    // "complete IP" consists of the module index and the instruction position.
    pub next_instruction_position: ProgramCounter,
    // pub current_module_index: usize,

    // pub current_function_index: usize,
    // pub callback_function_table: Vec<CallbackFunctionItem>,
    // for accelerating the obtaining of process information.
    // pub read_only_data_start_addr: usize,
    // pub read_write_data_start_addr: usize,
    // pub uninitialized_start_addr: usize,
    // pub heap_start_addr: usize,
    // pub operand_stack_start_addr: usize,
}

pub struct ProgramCounter {
    pub addr: usize,
    pub module_index: u16
}

/*
pub enum FrameClass {
    Function = 0x0,
    Block,
}

// only avaiable for 'call', 'cfcall' and 'block*' instructions
// the 'syscall', 'envcall' and 'cextcall' do not create stack frame
pub struct FrameItem {
    pub stack_start_position: usize,
    pub frame_func_type: u16,    // function or block signature
    pub frame_class: FrameClass, // function frame or block frame

    pub previous_stack_frame_position: usize,

    // this is the position of instruction 'call', 'cfcall' and 'block*'
    pub previous_instruction_position: usize,
    pub previous_module_index: usize,
    pub previous_function_index: usize,
}

pub struct CallbackFunctionItem {
    pub module_idx: usize,
    pub function_idx: usize,

    // the memory address of the executable code in the host machine
    pub host_function_addr: usize,
}
 */
