// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

// This module is responsible for managing the external function table.
//
// It is a 1:1 mapping to the "unified external function table" in the XiaoXuan Core application.
// The external function table is used to store the pointers to the external functions
// and the corresponding wrapper functions.
//
// ```diagram
//                                      XiaoXuan Core Runtime
//  XiaoXuan Core application        /--------------------------\
// /------------------------\        |                          |
// |                        |        | wrapper func table       |
// | fn demo () -> ()       |        | |----------------------| |          libxyz.so
// |   extcall do_something | -----> | | mod idx  | func idx  | |        /----------------------\
// | end                    |        | | 0        | 0         | |   /--> | void do_something {  |
// |                        |        | | ...      | ...       | |   |    |     ...              |
// \------------------------/        | |----------------------| |   |    |     ...              |
//                                   |                          |   |    |     ...              |
//                                   | wrapper func code 0      |   |    | }                    |
//                                   | |----------------------| |   |    |                      |
//                                   | | 0x0000 0xb8, 0x34,   | | --/    |                      |
//                                   | | 0x000a 0x12, 0x00... | |        \----------------------/
//                                   | |----------------------| |
//                                   |                          |
//                                   \--------------------------/
// ```
//
// The wrapper functions are shared between the external functions which have the same
// signature (i.e., the same parameters and return values).
//
// Wrapper functions are generated dynamically at runtime using JIT compilation
// when a external function is called for the first time.

use std::ffi::c_void;

use anc_isa::OperandDataType;

/// the external function pointer table
// #[derive(Default)]
pub struct ExternalFunctionTable {
    // the unified external library pointer list.
    //
    // it is 1:1 to the "unified external library section".
    pub unified_external_library_pointer_list: Vec<Option<UnifiedExternalLibraryPointerItem>>,

    // the unified external function pointer list.
    //
    // it is 1:1 to the "unified external functioa section".
    // An external function only load once in a process even if it is
    // referenced by multiple modules.
    pub unified_external_function_pointer_list: Vec<Option<UnifiedExternalFunctionPointerItem>>,

    // wrapper function list.
    //
    // not every external function has a corresponding wrapper function.
    // The wrapper functions are shared between the external functions which have the same
    // signature (i.e., the same parameters and return values).
    pub wrapper_function_list: Vec<WrapperFunctionItem>,
}

#[derive(Debug, Clone)]
pub struct UnifiedExternalLibraryPointerItem {
    pub address: usize,
}

#[derive(Debug, Clone)]
pub struct UnifiedExternalFunctionPointerItem {
    pub address: usize,
    pub wrapper_function_index: usize,
}

pub struct WrapperFunctionItem {
    pub param_datatypes: Vec<OperandDataType>,
    pub result_datatypes: Vec<OperandDataType>,
    pub wrapper_function: WrapperFunction,
}

// the signature of the wrapper function
pub type WrapperFunction = extern "C" fn(
    external_function_pointer: *const c_void,
    params_ptr: *const u8, // pointer to a range of bytes
    results_ptr: *mut u8,  // pointer to a range of bytes
);

impl ExternalFunctionTable {
    pub fn new(
        unified_external_library_count: usize,
        unified_external_function_count: usize,
    ) -> Self {
        let unified_external_library_pointer_list = vec![None; unified_external_library_count];
        let unified_external_function_pointer_list = vec![None; unified_external_function_count];
        Self {
            unified_external_library_pointer_list,
            unified_external_function_pointer_list,
            wrapper_function_list: Vec::new(),
        }
    }

    pub fn get_external_function_pointer_and_wrapper_function(
        &self,
        unified_external_function_index: usize,
    ) -> Option<(*mut c_void, WrapperFunction)> {
        let opt_unified_external_function_pointer_item =
            &self.unified_external_function_pointer_list[unified_external_function_index];

        match opt_unified_external_function_pointer_item {
            Some(unified_external_function_pointer_item) => {
                let wrapper_function_item = &self.wrapper_function_list
                    [unified_external_function_pointer_item.wrapper_function_index];
                let wrapper_function = wrapper_function_item.wrapper_function;
                Some((
                    unified_external_function_pointer_item.address as *mut c_void,
                    wrapper_function,
                ))
            }
            _ => None,
        }
    }
}
