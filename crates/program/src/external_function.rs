// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

//                                      runtime (native)
//  XiaoXuan core application        /------------------------\
// /------------------------\        |                        |
// |                        |        | wrapper func table     |
// | fn $demo () -> ()      |        | |--------------------| |          libxyz.so
// |   extcall do_something | -----> | | mod idx | func idx | |        /----------------------\
// | end                    |        | | 0       | 0        | |   /--> | void do_something {  |
// |                        |        | | ...     | ...      | |   |    |     ...              |
// \------------------------/        | |--------------------| |   |    |     ...              |
//                                   |                        |   |    |     ...              |
//                                   | wrapper func code 0    | --/    | }                    |
//                                   | 0x0000 0xb8, 0x34,     |        |                      |
//                                   | 0x000a 0x12, 0x00...   |        \----------------------/
//                                   |                        |
//                                   \------------------------/

use std::ffi::c_void;

use ancvm_types::DataType;

pub type WrapperFunction = extern "C" fn(
    external_function_pointer: *const c_void,
    params_ptr: *const u8,
    results_ptr: *mut u8,
);

pub struct ExtenalFunctionTable {
    // "unified external library section"  1:1
    pub external_library_pointer_list: Vec<Option<UnifiedExternalLibraryPointerItem>>,

    // "unified external functioa section"  1:1
    pub external_function_pointer_list: Vec<Option<UnifiedExternalFunctionPointerItem>>,

    pub wrapper_function_list: Vec<WrapperFunctionItem>,
}

impl ExtenalFunctionTable {
    pub fn new(external_library_count: usize, external_function_count: usize) -> Self {
        Self {
            external_library_pointer_list: vec![None; external_library_count],
            external_function_pointer_list: vec![None; external_function_count],
            wrapper_function_list: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnifiedExternalFunctionPointerItem {
    pub pointer: *const c_void,
    pub wrapper_function_index: usize,
}

#[derive(Debug, Clone)]
pub struct UnifiedExternalLibraryPointerItem {
    pub pointer: *const c_void,
}

pub struct WrapperFunctionItem {
    pub param_types: Vec<DataType>,
    pub result_types: Vec<DataType>,
    pub wrapper_function: WrapperFunction,
}
