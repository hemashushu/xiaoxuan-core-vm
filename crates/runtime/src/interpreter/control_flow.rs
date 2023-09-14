// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::thread::Thread;

use super::InterpretResult;

pub fn end(thread: &mut Thread) -> InterpretResult {
    let (is_function_frame, return_module_index, return_instruction_address) =
        thread.stack.exit_frames(0);

    if is_function_frame && return_instruction_address == 0 {
        InterpretResult::End
    } else {
        InterpretResult::Jump(
            return_module_index as usize,
            return_instruction_address as usize,
        )
    }
}
