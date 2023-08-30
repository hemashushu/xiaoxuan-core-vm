// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::{
    any::Any,
    fmt::{Debug, Display},
};

pub mod context;
pub mod heap;
pub mod indexed_memory;
pub mod memory;
pub mod resizeable_memory;
pub mod stack;
pub mod thread;
pub mod vm;

pub const MEMORY_PAGE_SIZE_IN_BYTES: usize = 32 * 1024;
pub const STACK_FRAME_SIZE_IN_PAGES: usize = 1;
pub const INIT_STACK_SIZE_IN_PAGES: usize = STACK_FRAME_SIZE_IN_PAGES;
pub const INIT_HEAP_SIZE_IN_PAGES: usize = 0;

pub trait VMError: Debug + Display {
    fn as_any(&self) -> &dyn Any;
}
