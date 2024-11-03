// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

pub mod envcall_num;
pub mod jit_util;
// pub mod delegate;
pub mod handler;
pub mod in_memory_resource;
// pub mod multithread_program;
pub mod process;

mod syscall_handler;
mod envcall_handler;

use std::{
    cell::RefCell,
    collections::BTreeMap,
    fmt::Display,
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
};

use ancvm_isa::GenericError;

// about the Tx and Rx:
//
// threads communicate through message pipe, the raw type of message is u8 array, so it can be:
// - primitive data
// - a struct
// - an array
// - (the address of) a function
// - (the address of) a closure function
thread_local! {
    // the collection of child threads
    pub static CHILD_THREADS:RefCell<BTreeMap<u32, ChildThread>> = RefCell::new(BTreeMap::new());

    // an incremented only integer that is used to generate the child thread id.
    pub static CHILD_THREAD_NEXT_ID:RefCell<u32> = RefCell::new(0);

    pub static CURRENT_THREAD_ID:RefCell<u32> = RefCell::new(0);

    // the receiver to the parent thread
    pub static RX:RefCell<Option<Receiver<Vec<u8>>>> = RefCell::new(None);

    // the sender to the parent thread
    pub static TX:RefCell<Option<Sender<Vec<u8>>>> = RefCell::new(None);

    // the data (an u8 array) that comes from the parent thread (i.e. the creator of the current thread)
    pub static THREAD_START_DATA:RefCell<Vec<u8>> = RefCell::new(vec![]);

    // the message that comes from the parent thread (i.e. the creator of the current thread)
    //
    // the data comes from other thread (includes the parent thread and child thread) is
    // temporary stored in LAST_MESSAGE each time the function 'thread_receive_msg' or
    // 'thread_receive_msg_from_parent' is called.
    pub static LAST_MESSAGE:RefCell<Vec<u8>> = RefCell::new(vec![]);
}

#[derive(Debug)]
pub struct InterpreterError {
    pub error_type: InterpreterErrorType,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InterpreterErrorType {
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

impl InterpreterError {
    pub fn new(error_type: InterpreterErrorType) -> Self {
        Self { error_type }
    }
}

impl Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.error_type {
            InterpreterErrorType::ParametersAmountMissmatch => write!(
                f,
                "Interpreter error: {}",
                "The number of parameters doesn't match"
            ),
            InterpreterErrorType::ResultsAmountMissmatch => write!(
                f,
                "Interpreter error: {}",
                "The number of results doesn't match"
            ),
            InterpreterErrorType::DataTypeMissmatch => {
                write!(f, "Interpreter error: {}", "Data type missmatch")
            }
            InterpreterErrorType::InvalidOperation => {
                write!(f, "Interpreter error: {}", "Invalid operation")
            }
            InterpreterErrorType::IndexNotFound => {
                write!(f, "Interpreter error: {}", "Index not found")
            }
            InterpreterErrorType::OutOfBoundary => {
                write!(f, "Interpreter error: {}", "Out of boundary")
            }
            InterpreterErrorType::ItemNotFound => f.write_str("Item not found."),
            InterpreterErrorType::Panic(code) => write!(
                f,
                "VM was terminated by instruction \"panic\", code: {}.",
                code
            ),
        }
    }
}

impl std::error::Error for InterpreterError {}

pub struct ChildThread {
    // the child thread on host will return the 'thread_exit_code'
    pub join_handle: JoinHandle<Result<u64, GenericError>>,

    // the receiver to the child thread
    pub rx: Receiver<Vec<u8>>,

    // the sender to the child thread
    pub tx: Sender<Vec<u8>>,
}
