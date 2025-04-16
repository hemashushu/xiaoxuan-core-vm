// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use crate::handler::Handler;
use anc_context::thread_context::ThreadContext;
use anc_isa::RUNTIME_EDITION;

pub fn host_arch(_handler: &Handler, thread_context: &mut ThreadContext) {
    // `fn (data_: u64) -> len:u32`

    let buf_ptr_value = thread_context.stack.pop_i64_u();

    let len = if let Some(len) = RUNTIME_EDITION.iter().position(|c| *c == 0) {
        len
    } else {
        RUNTIME_EDITION.len()
    };

    let src_ptr = RUNTIME_EDITION.as_ptr();
    let dst_ptr = buf_ptr_value as *mut u8;
    unsafe {
        std::ptr::copy(src_ptr, dst_ptr, len);
    }

    thread_context.stack.push_i32_u(len as u32);
}
