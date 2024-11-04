// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::fmt::Display;

pub mod envcall_num;
pub mod handler;
pub mod in_memory_resource;
pub mod jit_util;
pub mod multithread_handler;
pub mod multithread_process;
pub mod process;

mod envcall_handler;
mod syscall_handler;

#[derive(Debug)]
pub struct HandlerError {
    pub error_type: HandleErrorType,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum HandleErrorType {
    ParametersAmountMissmatch, // The number of arguments does not match the specified funcion.
    ResultsAmountMissmatch,    //
    DataTypeMissmatch,         // data type does not match
    InvalidOperation, // such as invoke 'popx' instructions when there is no operands on the stack
    IndexNotFound,    // the index of function (or data, local variables) does not found
    OutOfBoundary,    // out of boundary
    ItemNotFound,     // the specified item (module, function or data) does not found.
    Panic(u32),
    // Debug(u32),
    // Unreachable(u32),
}

impl HandlerError {
    pub fn new(error_type: HandleErrorType) -> Self {
        Self { error_type }
    }
}

impl Display for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.error_type {
            HandleErrorType::ParametersAmountMissmatch => write!(
                f,
                "Handler error: {}",
                "The number of parameters doesn't match"
            ),
            HandleErrorType::ResultsAmountMissmatch => write!(
                f,
                "Handler error: {}",
                "The number of results doesn't match"
            ),
            HandleErrorType::DataTypeMissmatch => {
                write!(f, "Handler error: {}", "Data type missmatch")
            }
            HandleErrorType::InvalidOperation => {
                write!(f, "Handler error: {}", "Invalid operation")
            }
            HandleErrorType::IndexNotFound => {
                write!(f, "Handler error: {}", "Index not found")
            }
            HandleErrorType::OutOfBoundary => {
                write!(f, "Handler error: {}", "Out of boundary")
            }
            HandleErrorType::ItemNotFound => f.write_str("Item not found."),
            HandleErrorType::Panic(code) => write!(
                f,
                "VM was terminated by instruction \"panic\", code: {}.",
                code
            ),
        }
    }
}

impl std::error::Error for HandlerError {}
