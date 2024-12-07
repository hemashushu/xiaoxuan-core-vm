// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::sync::{Mutex, MutexGuard, Once};

use anc_isa::OperandDataType;
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::Type;
use cranelift_jit::JITModule;

use crate::code_generator::Generator;

static mut JIT_GENERATOR_WITHOUT_IMPORTED_SYMBOLS: Mutex<Option<Generator<JITModule>>> =
    Mutex::new(None);
static INIT: Once = Once::new();

pub fn convert_vm_operand_data_type_to_jit_type(dt: OperandDataType) -> Type {
    match dt {
        OperandDataType::I32 => types::I32,
        OperandDataType::I64 => types::I64,
        OperandDataType::F32 => types::F32,
        OperandDataType::F64 => types::F64,
    }
}

pub fn get_jit_generator_without_imported_symbols(
) -> MutexGuard<'static, Option<Generator<JITModule>>> {
    INIT.call_once(|| {
        unsafe {
            JIT_GENERATOR_WITHOUT_IMPORTED_SYMBOLS =
                Mutex::new(Some(Generator::<JITModule>::new(vec![])))
        };
    });

    unsafe {
        let a = JIT_GENERATOR_WITHOUT_IMPORTED_SYMBOLS.lock().unwrap();
        a
    }
}
