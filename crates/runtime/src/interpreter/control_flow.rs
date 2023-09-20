// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::thread::Thread;

use super::InterpretResult;

pub fn end(thread: &mut Thread) -> InterpretResult {
    let opt_return_pc = thread.stack.exit_frames(0);

    if let Some(pc) = opt_return_pc {
        if pc.instruction_address == 0 {
            // the PC reaches the first function end, it means
            // the program reaches the ending.
            InterpretResult::End
        } else {
            // call another function or come back from another function
            InterpretResult::Jump(pc)
        }
    } else {
        // just move on
        InterpretResult::Move(2)
    }
}

pub fn block(thread: &mut Thread) -> InterpretResult {
    todo!()
}

pub fn return_(thread: &mut Thread) -> InterpretResult {
    todo!()
}

pub fn recur(thread: &mut Thread) -> InterpretResult {
    todo!()
}

pub fn block_nez(thread: &mut Thread) -> InterpretResult {
    todo!()
}

pub fn return_nez(thread: &mut Thread) -> InterpretResult {
    todo!()
}

pub fn recur_nez(thread: &mut Thread) -> InterpretResult {
    todo!()
}
