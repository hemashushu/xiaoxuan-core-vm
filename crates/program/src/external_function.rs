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

    pub fn get_external_function_pointer_and_wrapper_function(
        &self,
        unified_external_function_index: usize,
    ) -> Option<(*mut c_void, WrapperFunction)> {
        let opt_pointer = &self.external_function_pointer_list[unified_external_function_index];

        match opt_pointer {
            Some(ext_func_ptr_item) => {
                let wrapper_func_item =
                    &self.wrapper_function_list[ext_func_ptr_item.wrapper_function_index];
                let wrapper_func = wrapper_func_item.wrapper_function;
                Some((ext_func_ptr_item.pointer, wrapper_func))
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
        let ext_library_pointer = if let Some(uni_ext_lib_ptr_item) =
            &self.external_library_pointer_list[unified_external_library_index]
        {
            uni_ext_lib_ptr_item.pointer
        } else {
            self.add_external_library(
                unified_external_library_index,
                external_library_file_path_or_name,
            )?
        };

        let ext_func_pointer = load_symbol(ext_library_pointer, external_function_name)?;

        // find wrapper function index
        let wrapper_function_index = if let Some(wrapper_function_index) = self
            .wrapper_function_list
            .iter()
            .position(|wrapper_func_item| {
                wrapper_func_item.param_types == param_datatypes
                    && wrapper_func_item.result_types == result_datatypes
            }) {
            wrapper_function_index
        } else {
            self.add_wrapper_function(param_datatypes, result_datatypes)
        };

        // update external_function_pointer_list
        let uni_ext_func_ptr_item = UnifiedExternalFunctionPointerItem {
            pointer: ext_func_pointer,
            wrapper_function_index,
        };

        self.external_function_pointer_list[unified_external_function_index] =
            Some(uni_ext_func_ptr_item);

        Ok((
            ext_func_pointer,
            self.wrapper_function_list[wrapper_function_index].wrapper_function,
        ))
    }

    fn add_external_library(
        &mut self,
        unified_external_library_index: usize,
        external_library_file_path_or_name: &str,
    ) -> Result<*mut c_void, &'static str> {
        let pointer = load_library(external_library_file_path_or_name)?;
        self.external_library_pointer_list[unified_external_library_index] =
            Some(UnifiedExternalLibraryPointerItem { pointer });
        Ok(pointer)
    }

    fn add_wrapper_function(
        &mut self,
        param_types: &[DataType],
        result_types: &[DataType],
    ) -> usize {
        // build wrapper function
        let wrapper_function_index = self.wrapper_function_list.len();

        let wrapper_function_ptr = build_vm_to_external_function(
            wrapper_function_index,
            param_types,
            result_types,
        );

        let item = WrapperFunctionItem {
            param_types: param_types.to_vec(),
            result_types: result_types.to_vec(),
            wrapper_function: transmute_symbol_to::<WrapperFunction>(
                wrapper_function_ptr as *mut c_void,
            ),
        };

        self.wrapper_function_list.push(item);

        wrapper_function_index
    }
}

#[derive(Debug, Clone)]
pub struct UnifiedExternalFunctionPointerItem {
    pub pointer: *mut c_void,
    pub wrapper_function_index: usize,
}

#[derive(Debug, Clone)]
pub struct UnifiedExternalLibraryPointerItem {
    pub pointer: *mut c_void,
}

pub struct WrapperFunctionItem {
    pub param_types: Vec<DataType>,
    pub result_types: Vec<DataType>,
    pub wrapper_function: WrapperFunction,
}
