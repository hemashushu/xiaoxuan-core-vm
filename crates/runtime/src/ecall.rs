// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

#[repr(u16)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum ECall {
    write_char = 0x0,           // (param fd:u32 char:u32)
    write_bytes,                // (param fd:u32 addr:u64 length:u64)
    write_i32,                  // (param fd:u32 value:i32)
    write_i64,                  // (param fd:u32 value:i64)
}