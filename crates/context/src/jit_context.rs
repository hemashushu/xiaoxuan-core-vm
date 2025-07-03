// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::sync::Mutex;

use anc_isa::OperandDataType;
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::Type;
use cranelift_jit::JITModule;

use crate::code_generator::Generator;

pub fn convert_vm_operand_data_type_to_jit_type(dt: OperandDataType) -> Type {
    match dt {
        OperandDataType::I32 => types::I32,
        OperandDataType::I64 => types::I64,
        OperandDataType::F32 => types::F32,
        OperandDataType::F64 => types::F64,
    }
}
