// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;

use super::{HandleResult, Handler};

pub fn memory_allocate(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand align_in_bytes:i16 size_in_bytes:i64) -> i32

    HandleResult::Move(2)
}

pub fn memory_resize(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand data_public_index:i32 new_size_in_bytes:i64) -> i32

    HandleResult::Move(2)
}

pub fn memory_free(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand data_public_index:i32) -> ()

    HandleResult::Move(2)
}

pub fn memory_fill(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand
    // data_module_index:i32 data_public_index:i32 offset_in_bytes:i64
    // size_in_bytes:i64 value:i8) -> ()

    HandleResult::Move(2)
}

pub fn memory_copy(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand
    // source_data_module_index:i32 source_data_public_index:i32 source_offset_in_bytes:i64
    // dest_data_module_index:i32 dest_data_public_index:i32 dest_offset_in_bytes:i64
    // size_in_bytes:i64 value:i8) -> ()

    HandleResult::Move(2)
}

#[cfg(test)]
mod tests {
    //

}
