// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

pub mod datas;
pub mod environment;
pub mod external_function_table;
pub mod heap;
pub mod indexed_memory;
pub mod memory;
pub mod module_common_instance;
pub mod module_index_instance;
pub mod process_context;
pub mod resizeable_memory;
pub mod resource;
pub mod stack;
pub mod thread_context;
pub mod typed_memory;
pub mod loader;

// the stack will be enlarge when the free size of stack is less than this value
pub const STACK_FRAME_ENSURE_FREE_SIZE_IN_BYTES: usize = STACK_FRAME_INCREMENT_SIZE_IN_BYTES / 2;
pub const STACK_FRAME_INCREMENT_SIZE_IN_BYTES: usize = 64 * 1024;

pub const INIT_STACK_SIZE_IN_BYTES: usize = STACK_FRAME_INCREMENT_SIZE_IN_BYTES;

pub const MEMORY_PAGE_SIZE_IN_BYTES: usize = 64 * 1024;
pub const INIT_HEAP_SIZE_IN_PAGES: usize = 0;
pub const LOCAL_LIST_INDEX_NOT_EXIST:u32 = 0x8000_0000;


use std::fmt::Display;

#[derive(Debug)]
pub struct ContextError {
    message: String,
}

impl ContextError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_owned(),
        }
    }
}

impl Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Context error: {}", self.message)
    }
}

impl std::error::Error for ContextError {}
