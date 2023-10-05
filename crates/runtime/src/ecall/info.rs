// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_thread::thread::Thread;

use crate::{
    RUNTIME_CODE_NAME, RUNTIME_MAJOR_VERSION, RUNTIME_MINOR_VERSION, RUNTIME_PATCH_VERSION,
};

pub fn runtime_name(thread: &mut Thread) {
    // `fn (buf_ptr: u64) -> name_len:u32`

    let buf_ptr_value = thread.stack.pop_i64_u();

    let name_len = RUNTIME_CODE_NAME.len();

    let src_ptr = RUNTIME_CODE_NAME.as_ptr();
    let dst_ptr = buf_ptr_value as *mut u8;
    unsafe {
        std::ptr::copy(src_ptr, dst_ptr, name_len);
    }

    thread.stack.push_i32_u(name_len as u32);
}

pub fn runtime_version(thread: &mut Thread) {
    // `fn () -> version:u64`
    //
    // 0x0000_0000_0000_0000
    //        |    |    |
    //        |    |    |patch version
    //        |    |minor
    //        |major

    let version_number = RUNTIME_PATCH_VERSION as u64
        | (RUNTIME_MINOR_VERSION as u64) << 16
        | (RUNTIME_MAJOR_VERSION as u64) << 32;

    thread.stack.push_i64_u(version_number);
}

pub fn features(_thread: &mut Thread) {
    // `fn (buf_ptr: i64) -> feature_list_len:i32`
    unimplemented!()
}
