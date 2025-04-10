// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::fmt::Display;

pub mod indexed_memory_access;
pub mod memory_access;
pub mod primitive_memory_access;

#[derive(Debug)]
pub enum MemoryErrorType {
    UnsupportedFloatingPointVariants,
}

#[derive(Debug)]
pub struct MemoryError {
    pub error_type: MemoryErrorType,
}

impl MemoryError {
    pub fn new(error_type: MemoryErrorType) -> Self {
        MemoryError { error_type }
    }
}

impl Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.error_type {
            MemoryErrorType::UnsupportedFloatingPointVariants => write!(f, "Unsupported floating point variants: NaN, +Inf, and -Inf."),
        }
    }
}

impl std::error::Error for MemoryError {}
