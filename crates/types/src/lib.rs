// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::{
    any::Any,
    fmt::{Debug, Display},
};

pub mod ecallcode;
pub mod opcode;

/// the raw data type of operands
pub type Operand = [u8; 8];
pub const OPERAND_SIZE_IN_BYTES: usize = 8;

/// the data type of
/// - function parameters and results
/// - the operand of instructions
///
/// note that 'i32' here means a 32-bit integer, which is equivalent to
/// the 'uint32_t' in C or 'u32' in Rust. do not confuse it with 'i32' in Rust.
/// the same applies to the i8, i16 and i64.
///
/// the function `std::mem::transmute` can be used for converting data type
/// between `enum` and `u8` date, e.g.
///
/// ```rust
/// use ancvm_types::DataType;
/// let a = unsafe { std::mem::transmute::<DataType, u8>(DataType::F32) };
/// assert_eq!(a, 2);
/// ```
///
/// it can be also done through 'union', e.g.
///
/// ```rust
/// use ancvm_types::DataType;
/// union S2U {
///     src: DataType,
///     dst: u8
/// }
///
/// let a = unsafe{ S2U { src: DataType::F32 }.dst };
/// assert_eq!(a, 2);
/// ```
///
/// or, add `#[repr(u8)]` notation for converting enum to u8.
///
/// ```rust
/// use ancvm_types::DataType;
/// let a = DataType::F32 as u8;
/// assert_eq!(a, 2);
/// ```
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataType {
    I32 = 0x0,
    I64,
    F32,
    F64,
}

/// the data type of
/// - local variables
/// - data in the DATA sections
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MemoryDataType {
    I32 = 0x0,
    I64,
    F32,
    F64,
    BYTES,
}

// for foreign function interface (FFI)
// that is, for calling function (in a module of the VM) from the outside,
// or returning values to the outside.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ForeignValue {
    UInt32(u32),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
}

pub trait RuntimeError: Debug + Display {
    fn get_message(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}
