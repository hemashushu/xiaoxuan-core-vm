// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::types::Operand;

pub struct Proccess {
    pub heap: Vec<u8>,
    pub stack: Vec<Operand>,
    pub frames: Vec<FrameItem>,

    // the end position of the operand stack
    pub current_stack_position: usize,

    // the current frame position in the operand stack
    pub current_stack_frame_position: usize,

    // the position of the current executing instruction
    // note:
    // it's different from the RISC-V or ARM ISA, the value of SP
    // is the next instruction followed by the current executing instruction.
    // because the xiaoxuan vm's instructions are variable length.
    pub current_instruction_position: usize,
    pub current_module_index: usize,
    pub current_function_index: usize,

    pub callback_function_table: Vec<CallbackFunctionItem>,

    // for accelerating the obtaining of process information.
    pub read_only_data_start_addr: usize,
    pub read_write_data_start_addr: usize,
    pub uninitialized_start_addr: usize,
    pub heap_start_addr: usize,
    pub operand_stack_start_addr: usize,
}

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
