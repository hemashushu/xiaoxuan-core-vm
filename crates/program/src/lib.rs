// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

pub mod datas;
pub mod external_function;
pub mod heap;
pub mod indexed_memory;
pub mod jit_util;
pub mod memory;
pub mod program;
pub mod program_module;
pub mod program_context;
pub mod program_settings;
pub mod program_source;
pub mod resizeable_memory;
pub mod stack;
pub mod thread_context;
pub mod type_memory;

pub const MEMORY_PAGE_SIZE_IN_BYTES: usize = 32 * 1024;
pub const STACK_FRAME_SIZE_IN_PAGES: usize = 1;
pub const INIT_STACK_SIZE_IN_PAGES: usize = STACK_FRAME_SIZE_IN_PAGES;
pub const INIT_HEAP_SIZE_IN_PAGES: usize = 0;

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ProgramSourceType {
    InMemory = 0x0,
    File
}