// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

#[derive(Debug, Clone, Default)]
pub struct Capability {
    pub syscall: bool,
    pub extcall: bool,
    pub shell_execute: bool,
    pub capable_shell_execute_specify: Vec<String>,
    pub file_execute: bool,
    pub file_execute_specified: Vec<String>,
    pub dir_access: Vec<FileAccess>,
    pub file_access: Vec<FileAccess>,
}

#[derive(Debug, Clone, Default)]
pub struct FileAccess {
    pub path: String,
    pub type_: FileAccessType,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FileAccessType {
    #[default]
    Read,
    Write,
    ReadWrite,
}
