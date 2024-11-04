// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

pub mod datas;
pub mod external_function_table;
pub mod heap;
pub mod indexed_memory;
pub mod memory;
pub mod module_common_instance;
pub mod process_context;
pub mod module_index_instance;
pub mod environment;
pub mod resource;
pub mod resizeable_memory;
pub mod stack;
pub mod thread_context;
pub mod typed_memory;

pub const STACK_FRAME_INCREMENT_SIZE_IN_BYTES: usize = 64 * 1024;
// the stack will be enlarge when the free size of stack is less than this value
pub const STACK_FRAME_ENSURE_FREE_SIZE_IN_BYTES: usize = STACK_FRAME_INCREMENT_SIZE_IN_BYTES / 2;
pub const INIT_STACK_SIZE_IN_BYTES: usize = STACK_FRAME_INCREMENT_SIZE_IN_BYTES;

pub const MEMORY_PAGE_SIZE_IN_BYTES: usize = 64 * 1024;
pub const INIT_HEAP_SIZE_IN_PAGES: usize = 0;

// #[repr(u8)]
// #[derive(Debug, PartialEq, Clone, Copy)]
// pub enum ResourceType {
//     InMemory = 0x0,
//     File,
// }
