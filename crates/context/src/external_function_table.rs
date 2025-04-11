// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

// This module manages the external function table.
//
// It provides a 1:1 mapping to the "unified external function table" in the XiaoXuan Core application.
// The external function table stores pointers to external functions and their corresponding wrapper functions.
//
// ```diagram
//                                      XiaoXuan Core Runtime
//  XiaoXuan Core application        /--------------------------\
// /------------------------\        |                          |
// |                        |        | Wrapper Function Table   |
// | fn demo () -> ()       |        | |----------------------| |          libxyz.so
// |   extcall do_something | -----> | | mod idx  | func idx  | |        /----------------------\
// | end                    |        | | 0        | 0         | |   /--> | void do_something {  |
// |                        |        | | ...      | ...       | |   |    |     ...              |
// \------------------------/        | |----------------------| |   |    |     ...              |
//                                   |                          |   |    |     ...              |
//                                   | Wrapper Function Code 0  |   |    | }                    |
//                                   | |----------------------| |   |    |                      |
//                                   | | 0x0000 0xb8, 0x34,   | | --/    |                      |
//                                   | | 0x000a 0x12, 0x00... | |        \----------------------/
//                                   | |----------------------| |
//                                   |                          |
//                                   \--------------------------/
// ```
//
// Wrapper functions are shared among external functions with the same signature
// (i.e., identical parameter and return types).
//
// These wrapper functions are generated dynamically at runtime using JIT compilation
// when an external function is called for the first time.

use std::ffi::c_void;

use anc_isa::OperandDataType;

/// Represents the external function table.
pub struct ExternalFunctionTable {
    // A list of pointers to unified external libraries.
    //
    // This list corresponds 1:1 to the "unified external library section."
    pub unified_external_library_pointer_list: Vec<Option<UnifiedExternalLibraryPointerItem>>,

    // A list of pointers to unified external functions.
    //
    // This list corresponds 1:1 to the "unified external function section."
    // Each external function is loaded only once per process, even if referenced by multiple modules.
    pub unified_external_function_pointer_list: Vec<Option<UnifiedExternalFunctionPointerItem>>,

    // A list of wrapper functions.
    //
    // Not every external function has a corresponding wrapper function.
    // Wrapper functions are shared among external functions with the same signature
    // (i.e., identical parameter and return types).
    pub wrapper_function_list: Vec<WrapperFunctionItem>,
}

#[derive(Debug, Clone)]
pub struct UnifiedExternalLibraryPointerItem {
    // The memory address of the external library.
    pub address: usize,
}

#[derive(Debug, Clone)]
pub struct UnifiedExternalFunctionPointerItem {
    // The memory address of the external function.
    pub address: usize,
    // The index of the corresponding wrapper function in the wrapper function list.
    pub wrapper_function_index: usize,
}

pub struct WrapperFunctionItem {
    // The parameter data types of the wrapper function.
    pub param_datatypes: Vec<OperandDataType>,
    // The result data types of the wrapper function.
    pub result_datatypes: Vec<OperandDataType>,
    // The actual wrapper function.
    pub wrapper_function: WrapperFunction,
}

// The signature of a wrapper function.
pub type WrapperFunction = extern "C" fn(
    external_function_pointer: *const c_void, // Pointer to the external function.
    params_ptr: *const u8,                    // Pointer to the input parameters.
    results_ptr: *mut u8,                     // Pointer to the output results.
);

impl ExternalFunctionTable {
    /// Creates a new `ExternalFunctionTable` with the specified number of unified external libraries
    /// and unified external functions.
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

    /// Retrieves the external function pointer and its corresponding wrapper function
    /// for the given unified external function index.
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
