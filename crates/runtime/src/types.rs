// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataType {
    I32 = 0x0,
    I64,
    F32,
    F64,
}

pub type Operand = [u8; 8];

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionType<'a> {
    pub params: &'a [DataType],
    pub results: &'a [DataType],
}
