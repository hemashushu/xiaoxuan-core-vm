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

pub mod ecall;
pub mod bridge;
pub mod in_memory_program;
pub mod interpreter;

const RUNTIME_CODE_NAME: &[u8; 6] = b"Selina";

// Semantic Versioning
// - https://semver.org/
const RUNTIME_MAJOR_VERSION: u16 = 1;
const RUNTIME_MINOR_VERSION: u16 = 0;
const RUNTIME_PATCH_VERSION: u16 = 0;

#[derive(Debug)]
pub struct InterpreterError {
    message: String,
}

impl InterpreterError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_owned(),
        }
    }
}

impl Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("vm error: {}", self.message))
    }
}

impl RuntimeError for InterpreterError {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_message(&self) -> &str {
        &self.message
    }
}
