// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::{
    any::Any,
    fmt::{Debug, Display},
};

// Semantic Versioning
// - https://semver.org/
//
// a module will only run if its required major and minor
// versions match the current runtime version 100%.
pub const RUNTIME_MAJOR_VERSION: u16 = 1;
pub const RUNTIME_MINOR_VERSION: u16 = 0;
pub const RUNTIME_PATCH_VERSION: u16 = 0;
pub const RUNTIME_CODE_NAME: &[u8; 6] = b"Selina"; // is also my lovely daughter's name (XiaoXuan for zh-Hans) :D

// the relationship between the version of programs, shared modules and runtime
// ----------------------------------------------------------------------------
//
// for programs:
//
// every program (source code) declares a desired runtime version, which can only be run
// if the major and minor versions are identical. in short:
//
// - app major == runtime major
// - app minor == runtime minor
// - app patch == any
//
// for shared module:
//
// every shared module (source code) also declares a desired runtime version, since it is
// not a standalone executable module, when it is referenced (as dependency) by other
// programs, it will be compiled to the same runtime version as the main module requires.
// however, if the major version required by the shared module does not match that of
// the main module, compilation will be rejected. in short:
//
// - shared module major == runtime major
// - shared module minor == any
// - shared module patch == any
//
// for dependencies:
//
// a program may depend on one or more shared modules, when the program references a
// shared module, it is also necessary to declare the major and minor version.
// unlike many other language, 'XiaoXuan Core Script' requires the version of the dependencies
// (shared modules) must be strictly consistent with the declaration, that is to say:
//
// - dependency declare major == shared module major
// - dependency declare minor == shared module minor
// - dependency declare patch, shared module patch == any

// the max version number the current runtime supported
pub const IMAGE_MAJOR_VERSION: u16 = 1;
pub const IMAGE_MINOR_VERSION: u16 = 0;
pub const IMAGE_MAGIC_NUMBER: &[u8; 8] = b"ancsmod\0"; // the abbr of "ANCS module"

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
// https://doc.rust-lang.org/nomicon/other-reprs.html
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
pub enum ModuleShareType {
    User = 0x0,
    Shared,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExternalLibraryType {
    User = 0x0,
    Shared,
    System,
}

/// sometimes you may want to get a specified type from 'dyn RuntimeError',
/// there is a approach to downcast the 'dyn RuntimeError' object to a specified type, e.g.
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
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub struct CompileError {
    pub message: String,
}

impl CompileError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_owned(),
        }
    }
}
