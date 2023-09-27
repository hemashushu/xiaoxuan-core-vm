// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

pub mod context;
pub mod datas;

pub mod heap;
pub mod indexed_memory;
pub mod memory;
pub mod resizeable_memory;
pub mod stack;
pub mod thread;
pub mod type_memory;

const MEMORY_PAGE_SIZE_IN_BYTES: usize = 32 * 1024;
const STACK_FRAME_SIZE_IN_PAGES: usize = 1;
const INIT_STACK_SIZE_IN_PAGES: usize = STACK_FRAME_SIZE_IN_PAGES;
const INIT_HEAP_SIZE_IN_PAGES: usize = 0;
