// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

// This module manages pointers and wrapper functions for external functions.
//
// The following diagram illustrates how a XiaoXuan Core Application invokes external functions:
//
// ```diagram
//                                      XiaoXuan Core Runtime
//  XiaoXuan Core Application        /--------------------------\
// /------------------------\        |                          |
// |                        |        | Wrapper Function Table   |
// | fn demo () -> ()       |        | |----------------------| |               libxyz.so
// |   extcall do_something | -----> | | mod idx  | func idx  | |   invoke    /----------------------\
// | end                    | <----- | | 0        | 0         | |   /-------> | void do_something {  |
// |                        |        | | ...      | ...       | |   |         |     ...              |
// \------------------------/        | |----------------------| |   |         |     ...              |
//                                   |                          |   |    /--- |     ...              |
//                                   | Wrapper Function Code 0  |   |    |    | }                    |
//                                   | |----------------------| |   |    |    |                      |
//                                   | | 0x0000 0xb8, 0x34,   | | --/    |    |                      |
//                                   | | 0x000a 0x12, 0x00... | | <------/    \----------------------/
//                                   | |----------------------| |   return
//                                   |                          |
//                                   \--------------------------/
// ```
//
// Note: A wrapper function can be shared by multiple external functions with the same signature
// (i.e., identical parameter and return types).
//
// These wrapper functions are generated dynamically at runtime using JIT compilation
// when an external function is called for the first time.
//
// How the `extcall` instruction is executed
// -----------------------------------------
//
// ```diagram
// | module M, function A |             | external func |
// |----------------------|             |---------------|
// |                      |    wrapper  |               |
// | 0x0000 ...           |    function |               |
// | 0x0004 push args     |    |        |               |
// | 0x0008 extcall idx -------O---------> 0x0000       |
// |                    /-------------\ |  0x0004       |
// |                    | |           | |  0x0008       |
// |                    | |           \--- 0x000c       |
// |                    | |             |               |
// | 0x0010 ...      <--/ |             |---------------|
// |                      |
// ```
use std::ffi::c_void;

use anc_isa::OperandDataType;

/// Represents the external function table.
///
/// This table stores pointers to external functions and their corresponding wrapper functions.
pub struct ExternalFunctionTable {
    // A list of pointers to unified external libraries.
    //
    // This list corresponds 1:1 to the "unified_external_library_section" section in the binary image.
    // Each external library is loaded only once per process, even if referenced by multiple modules.
    pub unified_external_library_pointer_list: Vec<Option<UnifiedExternalLibraryPointerItem>>,

    // A list of pointers to unified external functions.
    //
    // This list corresponds 1:1 to the "unified_external_function_section" section in the binary image.
    // Each external function is loaded only once per process, even if referenced by multiple modules.
    pub unified_external_function_pointer_list: Vec<Option<UnifiedExternalFunctionPointerItem>>,

    // A list of wrapper functions.
    //
    // Note: A wrapper function can be shared among multiple external functions with the same signature
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
        // pre-allocate the lists to make sure the indices
        // are 1:1 mapping to the `unified_external_library_section` and
        // `unified_external_function_section` in the binary image.
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
        self.unified_external_function_pointer_list[unified_external_function_index]
            .as_ref()
            .map(|unified_external_function_pointer_item| {
                let wrapper_function_item = &self.wrapper_function_list
                    [unified_external_function_pointer_item.wrapper_function_index];
                let wrapper_function = wrapper_function_item.wrapper_function;
                (
                    unified_external_function_pointer_item.address as *mut c_void,
                    wrapper_function,
                )
            })
    }
}
