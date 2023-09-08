// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::thread::Thread;

use super::InterpretResult;

pub fn nop(_: &mut Thread) -> InterpretResult {
    InterpretResult::MoveOn(2)
}

pub fn drop(thread: &mut Thread) -> InterpretResult {
    thread.stack.drop();
    InterpretResult::MoveOn(2)
}

pub fn duplicate(thread: &mut Thread) -> InterpretResult {
    thread.stack.duplicate();
    InterpretResult::MoveOn(2)
}
