// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

//                                      runtime (native)
//  XiaoXuan Core application        /------------------------\
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

use std::{ffi::c_void, sync::Once};

use ancvm_isa::OperandDataType;

// the signature of the wrapper function
pub type WrapperFunction = extern "C" fn(
    external_function_pointer: *const c_void,
    params_ptr: *const u8,
    results_ptr: *mut u8,
);

/// the external function pointer table
#[derive(Default)]
pub struct ExtenalFunctionTable {
    // "unified external library section"  1:1
    pub unified_external_library_pointer_list: Vec<Option<UnifiedExternalLibraryPointerItem>>,

    // "unified external functioa section"  1:1
    pub unified_external_function_pointer_list: Vec<Option<UnifiedExternalFunctionPointerItem>>,

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

static INIT: Once = Once::new();

impl ExtenalFunctionTable {
    pub fn init(
        &mut self,
        unified_external_library_count: usize,
        unified_external_function_count: usize,
    ) {
        // https://doc.rust-lang.org/reference/conditional-compilation.html#debug_assertions
        if cfg!(debug_assertions) {
            // for
            // - unit test
            // - profile dev
            self.unified_external_library_pointer_list
                .resize(unified_external_library_count, None);
            self.unified_external_function_pointer_list
                .resize(unified_external_function_count, None);
        } else {
            // for profile release
            INIT.call_once(|| {
                self.unified_external_library_pointer_list
                    .resize(unified_external_library_count, None);
                self.unified_external_function_pointer_list
                    .resize(unified_external_function_count, None);
            });
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
