// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::fmt::Display;

mod bridge_handler;
mod code_generator;
mod envcall_handler;
mod extcall_handler;
mod jit_context;
mod multithread_handler;
mod syscall_handler;

pub mod bridge_process;
pub mod envcall_num;
pub mod handler;

// https://doc.rust-lang.org/reference/conditional-compilation.html#debug_assertions
// https://doc.rust-lang.org/reference/conditional-compilation.html#test
#[cfg(debug_assertions)]
pub mod in_memory_program_source;

pub mod multithread_process;
pub mod process;

pub const TERMINATE_CODE_PANIC: i32 = 0x1000_0000;
pub const TERMINATE_CODE_UNREACHABLE: i32 = 0x1000_0001;
pub const TERMINATE_CODE_FAILED_TO_CREATE_EXTERNAL_FUNCTION: i32 = 0x1000_1000;
pub const TERMINATE_CODE_FAILED_TO_CREATE_BRIDGE_FUNCTION: i32 = 0x1000_1001;

#[derive(Debug)]
pub struct FunctionEntryError {
    todo!() // rename to error_type
    pub type_: FunctionEntryErrorType,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Clone)]
pub enum FunctionEntryErrorType {
    ParametersAmountMissmatch, // The number of arguments does not match the specified funcion.
    ResultsAmountMissmatch,    //
    DataTypeMissmatch,         // data type does not match
    // InvalidOperation, // such as invoke 'popx' instructions when there is no operands on the stack
    // IndexNotFound,    // the index of function (or data, local variables) does not found
    // OutOfBoundary,    // out of boundary
    ItemNotFound, // the specified item (module, function or data) does not found.
    EntryPointNotFound(String),
    Terminate(i32), //
}

impl FunctionEntryError {
    pub fn new(type_: FunctionEntryErrorType) -> Self {
        Self { type_ }
    }
}

impl Display for FunctionEntryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.type_ {
            FunctionEntryErrorType::ParametersAmountMissmatch => {
                f.write_str("The number of parameters doesn't match.")
            }
            FunctionEntryErrorType::ResultsAmountMissmatch => {
                f.write_str("The number of results doesn't match.")
            }
            FunctionEntryErrorType::DataTypeMissmatch => f.write_str("Data type missmatch."),
            // HandleErrorType::InvalidOperation => f.write_str("Invalid operation."),
            // HandleErrorType::IndexNotFound => f.write_str("Index not found."),
            // HandleErrorType::OutOfBoundary => f.write_str("Out of boundary."),
            FunctionEntryErrorType::ItemNotFound => f.write_str("Item not found."),
            FunctionEntryErrorType::EntryPointNotFound(entry_point_name) => {
                write!(f, "Entry point \"{entry_point_name}\" not found.")
            }
            FunctionEntryErrorType::Terminate(terminate_code) => {
                write!(f, "Program terminated, code: {}.", terminate_code)
            }
        }
    }
}

impl std::error::Error for FunctionEntryError {}

pub type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;
