// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::{
    any::Any,
    fmt::{Debug, Display},
};

use ancvm_types::RuntimeError;

pub mod context;
pub mod datas;
pub mod heap;
pub mod host_accessable_memory;
pub mod indexed_memory;
pub mod memory;
pub mod processor;
pub mod resizeable_memory;
pub mod stack;
pub mod thread;
pub mod type_memory;

pub const MEMORY_PAGE_SIZE_IN_BYTES: usize = 32 * 1024;
pub const STACK_FRAME_SIZE_IN_PAGES: usize = 1;
pub const INIT_STACK_SIZE_IN_PAGES: usize = STACK_FRAME_SIZE_IN_PAGES;
pub const INIT_HEAP_SIZE_IN_PAGES: usize = 0;

#[derive(Debug)]
pub struct VMError {
    message: String,
}

impl VMError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_owned(),
        }
    }
}

impl Display for VMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("vm error: {}", self.message))
    }
}

impl RuntimeError for VMError {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_message(&self) -> &str {
        &self.message
    }
}
