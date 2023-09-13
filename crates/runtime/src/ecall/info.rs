// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::{
    thread::Thread, RUNTIME_CODE_NAME, RUNTIME_MAJOR_VERSION, RUNTIME_MINOR_VERSION,
    RUNTIME_PATCH_VERSION,
};

pub fn runtime_name(thread: &mut Thread) -> Result<(), usize> {
    // runtime_name(buf_ptr: u64) -> name_len:u32
    let buf_ptr_value = thread.stack.pop_u64();

    let name_len = RUNTIME_CODE_NAME.len();

    let src_ptr = RUNTIME_CODE_NAME.as_ptr() as *const u8;
    let dst_ptr = buf_ptr_value as *mut u8;
    unsafe {
        std::ptr::copy(src_ptr, dst_ptr, name_len);
    }

    thread.stack.push_u32(name_len as u32);
    Ok(())
}

pub fn runtime_version(thread: &mut Thread) -> Result<(), usize> {
    // runtime_version() -> version:u64
    // 0x0000_0000_0000_0000
    //        |    |    |
    //        |    |    |patch version
    //        |    |minor
    //        |major

    let version_number = RUNTIME_PATCH_VERSION as u64
        | (RUNTIME_MINOR_VERSION as u64) << 16
        | (RUNTIME_MAJOR_VERSION as u64) << 32;

    thread.stack.push_u64(version_number);
    Ok(())
}
