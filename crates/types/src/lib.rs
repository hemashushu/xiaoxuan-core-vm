// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::{
    any::Any,
    fmt::{Debug, Display},
};

pub mod opcode;

pub type Operand = [u8; 8];

pub const OPERAND_SIZE_IN_BYTES: usize = 8;

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataType {
    I32 = 0x0,
    I64,
    F32,
    F64,
    BYTE, // only available for data section
}

// for foreign function interface (FFI)
// that is, for calling function (in a module of the VM) from the outside,
// or returning values to the outside.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ForeignValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

pub trait RuntimeError: Debug + Display {
    fn get_message(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}
