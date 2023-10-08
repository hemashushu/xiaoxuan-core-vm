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

use ancvm_types::{DataType, ExternalLibraryType};

pub type WrapperFunction = extern "C" fn(
    external_function_pointer: *const c_void,
    params_ptr: *const u8,
    results_ptr: *mut u8,
);

pub struct ExtenalFunctionTable {
    external_function_map: Vec<ExternalFunctionMappingItem>,
    wrapper_function_list: Vec<WrapperFunctionItem>,
    library_pointer_list: Vec<LibraryPointerItem>,
}

impl ExtenalFunctionTable {
    pub fn new() -> Self {
        Self {
            external_function_map: vec![],
            wrapper_function_list: vec![],
            library_pointer_list: vec![],
        }
    }
}

// /----------------------------------\    /---------------------------\
// | external functioa internal index | -> | external function pointer |
// \----------------------------------/    \---------------------------/
pub struct ExternalFunctionMappingItem {
    pub external_function_internal_index: usize,
    pub external_function_pointer: *const c_void,
    pub wrapper_function_index: usize,
}

pub struct LibraryPointerItem {
    pub library_type: ExternalLibraryType,
    pub file_path: String,
    pub pointer: *const c_void,
}

pub struct WrapperFunctionItem {
    pub param_types: Vec<DataType>,
    pub result_types: Vec<DataType>,
    pub wrapper_function: WrapperFunction,
}
