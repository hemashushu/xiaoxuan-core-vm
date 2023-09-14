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
use ecall::init_ecall_handlers;
use interpreter::init_interpreters;

pub mod context;
pub mod datas;
pub mod ecall;
pub mod heap;
pub mod indexed_memory;
pub mod interpreter;
pub mod memory;
pub mod resizeable_memory;
pub mod stack;
pub mod thread;
pub mod type_memory;

const MEMORY_PAGE_SIZE_IN_BYTES: usize = 32 * 1024;
const STACK_FRAME_SIZE_IN_PAGES: usize = 1;
const INIT_STACK_SIZE_IN_PAGES: usize = STACK_FRAME_SIZE_IN_PAGES;
const INIT_HEAP_SIZE_IN_PAGES: usize = 0;

const RUNTIME_CODE_NAME: &[u8; 6] = b"Selina";

// Semantic Versioning
// - https://semver.org/
const RUNTIME_MAJOR_VERSION: u16 = 1;
const RUNTIME_MINOR_VERSION: u16 = 0;
const RUNTIME_PATCH_VERSION: u16 = 0;

pub fn init_runtime() {
    init_interpreters();
    init_ecall_handlers();
}

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
