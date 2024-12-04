// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::fmt::Display;

mod bridge_handler;
mod envcall_handler;
mod extcall_handler;
mod jit_util;
mod multithread_handler;
mod syscall_handler;

pub mod in_memory_resource;
pub mod handler;
pub mod bridge_process;
pub mod envcall_num;
pub mod multithread_process;
pub mod process;

pub const PANIC_CODE_EXTERNAL_FUNCTION_CREATE_FAILURE: u32 = 0x1000_0001;
pub const PANIC_CODE_BRIDGE_FUNCTION_CREATE_FAILURE: u32 = 0x1000_0002;

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
}

impl HandlerError {
    pub fn new(error_type: HandleErrorType) -> Self {
        Self { error_type }
    }
}

impl Display for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.error_type {
            HandleErrorType::ParametersAmountMissmatch => {
                f.write_str("Handler error: the number of parameters doesn't match")
            }
            HandleErrorType::ResultsAmountMissmatch => {
                f.write_str("Handler error: the number of results doesn't match")
            }
            HandleErrorType::DataTypeMissmatch => f.write_str("Handler error: data type missmatch"),
            HandleErrorType::InvalidOperation => f.write_str("Handler error: invalid operation"),
            HandleErrorType::IndexNotFound => f.write_str("Handler error: index not found"),
            HandleErrorType::OutOfBoundary => f.write_str("Handler error: out of boundary"),
            HandleErrorType::ItemNotFound => f.write_str("Item not found."),
            HandleErrorType::Panic(code) => write!(
                f,
                "XiaoXuan Core VM was terminated by instruction \"panic\", code: {}.",
                code
            ),
        }
    }
}

impl std::error::Error for HandlerError {}

pub type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;
