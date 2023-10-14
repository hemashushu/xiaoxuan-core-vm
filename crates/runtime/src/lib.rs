// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::{
    any::Any,
    cell::RefCell,
    collections::BTreeMap,
    fmt::{Debug, Display},
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
};

use ancvm_types::{ForeignValue, RuntimeError};

pub mod bridge;
pub mod ecall;
pub mod in_memory_program_source;
pub mod interpreter;
pub mod multithread_program;

const RUNTIME_CODE_NAME: &[u8; 6] = b"Selina";

// Semantic Versioning
// - https://semver.org/
const RUNTIME_MAJOR_VERSION: u16 = 1;
const RUNTIME_MINOR_VERSION: u16 = 0;
const RUNTIME_PATCH_VERSION: u16 = 0;

// about the Tx and Rx:
//
// threads communicate through message pipe, the raw type of message is u8 array, so it can be:
// - primitive data
// - a struct
// - an array
// - (the address of) a function
// - (the address of) a closure function
thread_local! {
    pub static CHILD_THREADS:RefCell<BTreeMap<u32, ChildThread>> = RefCell::new(BTreeMap::new());
    pub static CHILD_THREAD_MAX_ID:RefCell<u32> = RefCell::new(0);
    pub static CURRENT_THREAD_ID:RefCell<u32> = RefCell::new(0);
    pub static RX:RefCell<Option<Receiver<Vec<u8>>>> = RefCell::new(None);
    pub static TX:RefCell<Option<Sender<Vec<u8>>>> = RefCell::new(None);
    pub static THREAD_START_DATA:RefCell<Vec<u8>> = RefCell::new(vec![]);
    pub static LAST_MESSAGE:RefCell<Vec<u8>> = RefCell::new(vec![]);
}

#[derive(Debug)]
pub struct InterpreterError {
    pub error_type: InterpreterErrorType,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InterpreterErrorType {
    InvalidFunctionCall, // The number of arguments does not match the specified funcion.
    DataTypeMissmatch,   // data type does not match
    InvalidOperation, // such as invoke 'popx' instructions when there is no operands on the stack
    IndexNotFound,    // the index of function (data, local variables) not found
    OutOfBoundary,    // out of boundary
}

impl InterpreterError {
    pub fn new(error_type: InterpreterErrorType) -> Self {
        Self {
            error_type
        }
    }
}

impl Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error_code = match self.error_type {
            InterpreterErrorType::InvalidFunctionCall => "invalid_function_call",
            InterpreterErrorType::DataTypeMissmatch => "data_type_missmatch",
            InterpreterErrorType::InvalidOperation => "invalid_operation",
            InterpreterErrorType::IndexNotFound => "index_not_found",
            InterpreterErrorType::OutOfBoundary => "out_of_boundary",
        };

        f.write_fmt(format_args!("interpreter error: {}", error_code))
    }
}

impl RuntimeError for InterpreterError {
    fn as_any(&self) -> &dyn Any {
        self
    }

    // fn get_message(&self) -> &str {
    //     &self.message
    // }
}

pub struct ChildThread {
    pub join_handle: JoinHandle<Result<Vec<ForeignValue>, Box<dyn RuntimeError + Send>>>,
    pub rx: RefCell<Option<Receiver<Vec<u8>>>>,
    pub tx: RefCell<Option<Sender<Vec<u8>>>>,
}
