// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

pub mod datas;
pub mod process_property;
pub mod external_function_table;
pub mod indexed_memory_access;
pub mod memory;
pub mod memory_access;
pub mod module_common_instance;
pub mod module_index_instance;
pub mod process_context;
pub mod resizeable_memory;
pub mod process_resource;
pub mod stack;
pub mod thread_context;
pub mod typed_memory_access;

pub const MEMORY_PAGE_SIZE_IN_BYTES: usize = 64 * 1024;
pub const INIT_MEMORY_SIZE_IN_PAGES: usize = 0;

// use std::fmt::Display;
//
// #[derive(Debug)]
// pub struct ContextError {
//     // message: String,
// }
//
// impl ContextError {
//     pub fn new(message: &str) -> Self {
//         Self {
//             message: message.to_owned(),
//         }
//     }
// }
//
// impl Display for ContextError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         todo!()
//     }
// }
//
// impl std::error::Error for ContextError {}
