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

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataType {
    I32 = 0x0,
    I64,
    F32,
    F64,
    BYTE, // only available for data section
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataSectionType {
    ReadOnly = 0x0,
    ReadWrite,
    Uninit,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeEntry<'a> {
    pub params: &'a [DataType],
    pub results: &'a [DataType],
}

#[derive(Debug, PartialEq, Clone)]
pub struct FuncEntry<'a> {
    pub func_type: u16,
    pub code: &'a [u8],
}

// specify the data type of enum
// see also:
// https://doc.rust-lang.org/nomicon/other-reprs.html
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ModuleShareType {
    Local = 0x0,
    Shared,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ModuleIndexEntry<'a> {
    pub module_share_type: ModuleShareType,
    pub name: &'a str,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SectionId {
    Type = 0x0,    // 0
    ImportData,    // 1
    ReadOnlyData,  // 2
    ReadWriteData, // 3
    UninitData,    // 4
    ImportFunc,    // 5
    Func,          // 6
    ExportData,    // 7
    ExportFunc,    // 8
    ExternalFunc,  // 9
    AutoFunc,      // 10

    ModuleIndex, // 11
    DataIndex,   // 12
    FuncIndex,   // 13
}

pub trait SectionEntry<'a> {
    fn id(&'a self) -> SectionId;
    fn load(section_data: &'a [u8]) -> Self
    where
        Self: Sized;
    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()>;
}

pub trait VMErr: Debug + Display {
    fn as_any(&self) -> &dyn Any;
}
