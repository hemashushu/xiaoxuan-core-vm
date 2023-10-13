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

use std::{ffi::c_void, sync::Once};

use ancvm_extfunc_util::{load_library, load_symbol, transmute_symbol_to};
use ancvm_types::DataType;

use crate::jit_util::build_vm_to_external_function;

// the signature of the wrapper function
pub type WrapperFunction = extern "C" fn(
    external_function_pointer: *const c_void,
    params_ptr: *const u8,
    results_ptr: *mut u8,
);

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
    pub param_datatypes: Vec<DataType>,
    pub result_datatypes: Vec<DataType>,
    pub wrapper_function: WrapperFunction,
}

static INIT: Once = Once::new();

impl ExtenalFunctionTable {
    // pub fn new(external_library_count: usize, external_function_count: usize) -> Self {
    pub fn new() -> Self {
        Self {
            unified_external_library_pointer_list: vec![],
            unified_external_function_pointer_list: vec![],
            wrapper_function_list: vec![],
        }
    }

    pub fn init(
        &mut self,
        unified_external_library_count: usize,
        unified_external_function_count: usize,
    ) {
        // https://doc.rust-lang.org/reference/conditional-compilation.html#debug_assertions
        if cfg!(debug_assertions) {
            // unit test or NOT release
            self.unified_external_library_pointer_list
                .resize(unified_external_library_count, None);
            self.unified_external_function_pointer_list
                .resize(unified_external_function_count, None);
        } else {
            // profile release
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

    pub fn add_external_function(
        &mut self,
        unified_external_function_index: usize,
        unified_external_library_index: usize,
        external_library_file_path_or_name: &str,
        external_function_name: &str,
        param_datatypes: &[DataType],
        result_datatypes: &[DataType],
    ) -> Result<(*mut c_void, WrapperFunction), &'static str> {
        if result_datatypes.len() > 1 {
            return Err("The specified function has more than 1 return value.");
        }

        // find library pointer
        let library_pointer = if let Some(unified_external_library_pointer_item) =
            &self.unified_external_library_pointer_list[unified_external_library_index]
        {
            unified_external_library_pointer_item.address as *mut c_void
        } else {
            self.add_external_library(
                unified_external_library_index,
                external_library_file_path_or_name,
            )?
        };

        let function_pointer = load_symbol(library_pointer, external_function_name)?;

        // find wrapper function index
        let wrapper_function_index = if let Some(wrapper_function_index) = self
            .wrapper_function_list
            .iter()
            .position(|wrapper_function_item| {
                wrapper_function_item.param_datatypes == param_datatypes
                    && wrapper_function_item.result_datatypes == result_datatypes
            }) {
            wrapper_function_index
        } else {
            self.add_wrapper_function(param_datatypes, result_datatypes)
        };

        // update external_function_pointer_list
        let unified_external_function_pointer_item = UnifiedExternalFunctionPointerItem {
            address: function_pointer as usize,
            wrapper_function_index,
        };

        self.unified_external_function_pointer_list[unified_external_function_index] =
            Some(unified_external_function_pointer_item);

        let wrapper_function = self.wrapper_function_list[wrapper_function_index].wrapper_function;

        Ok((function_pointer, wrapper_function))
    }

    fn add_external_library(
        &mut self,
        unified_external_library_index: usize,
        external_library_file_path_or_name: &str,
    ) -> Result<*mut c_void, &'static str> {
        let library_pointer = load_library(external_library_file_path_or_name)?;
        self.unified_external_library_pointer_list[unified_external_library_index] =
            Some(UnifiedExternalLibraryPointerItem {
                address: library_pointer as usize,
            });
        Ok(library_pointer)
    }

    fn add_wrapper_function(
        &mut self,
        param_types: &[DataType],
        result_types: &[DataType],
    ) -> usize {
        // build wrapper function
        let wrapper_function_index = self.wrapper_function_list.len();

        let wrapper_function_pointer =
            build_vm_to_external_function(wrapper_function_index, param_types, result_types);

        let wrapper_function_item = WrapperFunctionItem {
            param_datatypes: param_types.to_vec(),
            result_datatypes: result_types.to_vec(),
            wrapper_function: transmute_symbol_to::<WrapperFunction>(
                wrapper_function_pointer as *mut c_void,
            ),
        };

        self.wrapper_function_list.push(wrapper_function_item);

        wrapper_function_index
    }
}
