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

impl From<u8> for DataSectionType {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute::<u8, DataSectionType>(value) }
    }
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

impl From<u8> for ModuleShareType {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute::<u8, ModuleShareType>(value) }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ModuleIndexEntry<'a> {
    pub module_share_type: ModuleShareType,
    pub name: &'a str,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DataEntry {
    pub data_type: DataType,
    pub data: Vec<u8>,
}

impl DataEntry {
    pub fn from_i32(value: i32) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(8);
        data.extend(value.to_le_bytes().iter());
        data.extend([0u8; 4].iter());

        Self {
            data_type: DataType::I32,
            data,
        }
    }

    pub fn from_i64(value: i64) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(8);
        data.extend(value.to_le_bytes().iter());

        Self {
            data_type: DataType::I64,
            data,
        }
    }

    pub fn from_f32(value: f32) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(8);
        data.extend(value.to_le_bytes().iter());
        data.extend([0u8; 4].iter());

        Self {
            data_type: DataType::F32,
            data,
        }
    }

    pub fn from_f64(value: f64) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(8);
        data.extend(value.to_le_bytes().iter());

        Self {
            data_type: DataType::F64,
            data,
        }
    }

    pub fn from_bytes(value: &[u8]) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(value.len());
        data.extend_from_slice(value);

        Self {
            data_type: DataType::BYTE,
            data,
        }
    }
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

impl From<u16> for SectionId {
    fn from(value: u16) -> Self {
        unsafe { std::mem::transmute::<u16, SectionId>(value) }
    }
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
