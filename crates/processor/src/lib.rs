// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::fmt::Display;

mod envcall_handler;
mod extcall_handler;
mod multithread_handler;
mod syscall_handler;

pub mod envcall_num;
pub mod in_memory_program_source;
pub mod instruction_handler;
pub mod process;
pub mod program;

pub const TERMINATE_CODE_PANIC: i32 = 0x1000_0000;
pub const TERMINATE_CODE_UNREACHABLE: i32 = 0x1000_0001;
pub const TERMINATE_CODE_STACK_OVERFLOW: i32 = 0x1000_0002;
pub const TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS: i32 = 0x1000_0003;
pub const TERMINATE_CODE_FAILED_TO_LOAD_EXTERNAL_FUNCTION: i32 = 0x1000_0010;
pub const TERMINATE_CODE_FAILED_TO_CREATE_DELEGATE_FUNCTION: i32 = 0x1000_0011;

#[derive(Debug)]
pub struct ProcessorError {
    pub error_type: ProcessorErrorType,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Clone)]
pub enum ProcessorErrorType {
    ParametersAmountMissmatch, // The number of arguments does not match the specified funcion.
    ResultsAmountMissmatch,    // The number of return values does not match the specified funcion.
    DataTypeMissmatch,         // data type does not match
    ItemNotFound, // the specified item (such as module, function, local vaiarble, or data) does not found.
    StackOverflow, // stack overflow
    UnsupportedFloatingPointVariants, // Unsupported floating point variants: NaN, +Inf, and -Inf.
    ExternalFunctionMoreThanOneResult, // The external function has more than one return value.
    EntryPointNotFound(String),
    Terminate(i32),
}

impl ProcessorError {
    pub fn new(type_: ProcessorErrorType) -> Self {
        Self { error_type: type_ }
    }
}

impl Display for ProcessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.error_type {
            ProcessorErrorType::ParametersAmountMissmatch => {
                f.write_str("The number of parameters doesn't match.")
            }
            ProcessorErrorType::ResultsAmountMissmatch => {
                f.write_str("The number of results doesn't match.")
            }
            ProcessorErrorType::DataTypeMissmatch => f.write_str("Data type missmatch."),
            ProcessorErrorType::ItemNotFound => f.write_str("Item not found."),
            ProcessorErrorType::StackOverflow => f.write_str("Stack overflow."),
            ProcessorErrorType::UnsupportedFloatingPointVariants => {
                f.write_str("Unsupported floating point variants: NaN, +Inf, and -Inf.")
            }
            ProcessorErrorType::ExternalFunctionMoreThanOneResult => {
                f.write_str("The external function has more than one return value.")
            }
            ProcessorErrorType::EntryPointNotFound(entry_point_name) => {
                write!(f, "Entry point \"{entry_point_name}\" not found.")
            }
            ProcessorErrorType::Terminate(terminate_code) => {
                write!(f, "Program terminated, code: {}.", terminate_code)
            }
        }
    }
}

impl std::error::Error for ProcessorError {}

pub type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;
