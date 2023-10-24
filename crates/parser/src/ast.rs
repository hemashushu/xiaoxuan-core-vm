// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_types::{DataType, MemoryDataType};

#[derive(Debug, PartialEq, Clone)]
pub struct ModuleNode {
    pub name: String,

    pub runtime_version_major: u16,
    pub runtime_version_minor: u16,

    pub shared_packages: Vec<String>,
    pub element_nodes: Vec<ModuleElementNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ModuleElementNode {
    FuncNode(FuncNode),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FuncNode {
    pub name: Option<String>,
    pub params: Vec<ParamNode>,
    pub results: Vec<DataType>,
    pub locals: Vec<LocalNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParamNode {
    pub name: Option<String>,
    pub data_type: DataType,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LocalNode {
    pub name: Option<String>,
    pub memory_data_type: MemoryDataType,
    pub data_length: u32,

    // if the data is a byte array (includes string), the value should be 1,
    // if the data is a struct, the value should be the max one of the length of its fields.
    // currently the MAX value of align is 8, MIN value is 1.
    pub align: u16,
}
