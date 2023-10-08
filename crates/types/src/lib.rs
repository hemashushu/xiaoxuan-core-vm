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
pub mod utils;

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

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExternalLibraryType {
    User = 0x0,
    Shared,
    System,
}

/// sometimes you may want to get a specified type from 'dyn RuntimeError',
/// there is a way to downcast the 'dyn RuntimeError' object to a specified type, e.g.
///
/// let some_error:T = unsafe {
///     &*(runtime_error as *const dyn RuntimeError as *const T)
/// };
///
/// the 'runtime_error' is a 'fat' pointer, it consists of a pointer to the data and
/// a another pointer to the 'vtable'. the slice object is also a 'fat' pointer, e.g.
///
/// let v:Vec<u8> = vec![1,2,3];
/// let p_fat = &v[..] as *const _;     // this is a fat pointer
/// let p_thin = p_fat as *const ();    // obtains the first pointer and discard the second pointer
/// let addr = p_thin as usize;         // check the address in memory
///
/// for simplicity, 'RuntimeError' provides function 'as_any' for downcasing, e.g.
///
/// let some_error = runtime_error
///     .as_any
///     .downcast_ref::<T>()
///     .expect("...");
///
/// ref:
/// - https://alschwalm.com/blog/static/2017/03/07/exploring-dynamic-dispatch-in-rust/
/// - https://geo-ant.github.io/blog/2023/rust-dyn-trait-objects-fat-pointers/
/// - https://doc.rust-lang.org/std/any/
/// - https://bennett.dev/rust/downcast-trait-object/
pub trait RuntimeError: Debug + Display {
    fn get_message(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}
