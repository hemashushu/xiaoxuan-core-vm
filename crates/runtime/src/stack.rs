// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::vm::VM;

impl VM {
    pub fn read_stack_i32(addr:usize ) -> i32 {
        todo!()
    }
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