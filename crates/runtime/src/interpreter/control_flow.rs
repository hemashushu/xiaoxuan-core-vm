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
            InterpretResult::End
        } else {
            InterpretResult::Jump(pc)
        }
    } else {
        InterpretResult::MoveOn(2)
    }
}
