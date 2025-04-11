// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

pub mod datas;
pub mod external_function_table;
pub mod module_common_instance;
pub mod module_linking_instance;
pub mod delegate_function_table;
// pub mod process_context;
pub mod program_property;
// pub mod program_source;
pub mod thread_context;

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
