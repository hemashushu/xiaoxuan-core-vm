// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_types::{opcode::Opcode, DataType, MemoryDataType};

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
    TODONode,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FuncNode {
    pub name: Option<String>,
    pub params: Vec<ParamNode>,
    pub results: Vec<DataType>,
    pub locals: Vec<LocalNode>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParamNode {
    pub tag: String,
    pub data_type: DataType,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LocalNode {
    pub tag: String,
    pub memory_data_type: MemoryDataType,
    pub data_length: u32,

    // if the data is a byte array (includes string), the value should be 1,
    // if the data is a struct, the value should be the max one of the length of its fields.
    // currently the MAX value of align is 8, MIN value is 1.
    pub align: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    NoParams(Opcode),

    ImmI32(u32),
    ImmI64(u64),
    ImmF32(ImmF32),
    ImmF64(ImmF64),

    LocalAccess(Opcode, /* tag */ String, /* offset */ u16),
    LocalLongAccess(Opcode, /* tag */ String),

    DataAccess(Opcode, /* tag */ String, /* offset */ u16),
    DataLongAccess(Opcode, /* tag */ String),

    HeapAccess(Opcode, u16 /* offset */),

    UnaryOp(Opcode),
    UnaryOpParamI16(Opcode, u16),
    BinaryOp(Opcode),

    When(When),
    If(If),
    Branch(Branch),
    For(For),

    Break,
    Recur,
    Return,
    TailCall,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ImmF32 {
    Float(f32),
    Hex(u32),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ImmF64 {
    Float(f64),
    Hex(u64),
}

#[derive(Debug, PartialEq, Clone)]
pub struct When {
    // structure 'when' has NO params and NO results, however,
    // can contains local variables.
    locals: Vec<LocalNode>,
    test: Vec<Instruction>,
    consequent: Vec<Instruction>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct If {
    params: Vec<ParamNode>,
    results: Vec<DataType>,
    locals: Vec<LocalNode>,
    test: Vec<Instruction>,
    consequent: Vec<Instruction>,
    alternate: Vec<Instruction>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Branch {
    params: Vec<ParamNode>,
    results: Vec<DataType>,
    locals: Vec<LocalNode>,
    cases: Vec<BranchCase>,
    default: Vec<Instruction>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BranchCase {
    test: Vec<Instruction>,
    consequent: Vec<Instruction>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct For {
    params: Vec<ParamNode>,
    results: Vec<DataType>,
    locals: Vec<LocalNode>,
    instructions: Vec<Instruction>,
}
