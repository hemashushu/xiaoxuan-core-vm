// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use core::str;
use std::{ffi::c_void, path::PathBuf, sync::Mutex};

use anc_context::{
    code_generator::Generator,
    external_function_table::{
        ExternalFunctionTable, UnifiedExternalFunctionPointerItem,
        UnifiedExternalLibraryPointerItem, WrapperFunction, WrapperFunctionItem,
    },
    jit_context::convert_vm_operand_data_type_to_jit_type,
    process_property::ProgramSourceType,
    thread_context::ThreadContext,
};
use anc_isa::{ExternalLibraryDependency, OperandDataType, OPERAND_SIZE_IN_BYTES};
use cranelift_codegen::ir::{AbiParam, Function, InstBuilder, MemFlags, UserFuncName};
use cranelift_frontend::FunctionBuilder;
use cranelift_jit::JITModule;
use cranelift_module::{Linkage, Module};
use dyncall_util::{load_library, load_symbol, transmute_symbol_to};
use resolve_path::PathResolveExt;

use crate::{ProcessorError, ProcessorErrorType};

static LAST_WRAPPER_FUNCTION_ID: Mutex<usize> = Mutex::new(0);

pub fn get_or_create_external_function_wrapper_function(
    thread_context: &mut ThreadContext,
    module_index: usize,
    external_function_index: usize,
) -> Result<
    (
        /* external_function_pointer */ *mut c_void,
        /* wrapper_function */ WrapperFunction,
        /* param_count */ usize,
        /* contains_return_value */ bool,
    ),
    ProcessorError,
> {
    // external function index bounds check.
    #[cfg(debug_assertions)]
    {
        let count = thread_context
            .module_linking_instance
            .external_function_index_section
            .get_items_count(module_index);

        if external_function_index > count as usize {
            panic!("Out of bounds of the external function index, module index:{}, total external functions: {}, request external function index: {}",
                module_index,
                count,
                external_function_index);
        }
    }

    // get the unified external function index
    let unified_external_function_index = thread_context
        .module_linking_instance
        .external_function_index_section
        .get_item_unified_external_function_index(module_index, external_function_index);

    let (external_function_name, unified_external_library_index, type_index) = thread_context
        .module_linking_instance
        .unified_external_function_section
        .get_item_name_and_external_library_index_and_type_index(unified_external_function_index);

    // get the data types of "params and results" of the external function
    let (param_datatypes, result_datatypes) = thread_context
        .module_linking_instance
        .unified_external_type_section
        .get_item_params_and_results(type_index);

    let param_count = param_datatypes.len();

    // for the compatible with C function callinf convention, only
    // one or zero result is allowed.
    if result_datatypes.len() > 1 {
        return Err(ProcessorError {
            error_type: ProcessorErrorType::ExternalFunctionMoreThanOneResult,
        });
    }

    let contains_return_value = !result_datatypes.is_empty();

    let opt_external_function_pointer_and_wrapper_function = {
        let table = thread_context.external_function_table.lock().unwrap();
        table.get_external_function_pointer_and_wrapper_function(unified_external_function_index)
    };

    if let Some((external_function_pointer, wrapper_function)) =
        opt_external_function_pointer_and_wrapper_function
    {
        return Ok((
            external_function_pointer,
            wrapper_function,
            param_count,
            contains_return_value,
        ));
    }

    // get the dependent info of the external library
    let (_, _, external_library_value_data) = thread_context
        .module_linking_instance
        .unified_external_library_section
        .get_item_name_and_external_library_dependent_type_and_value(
            unified_external_library_index,
        );

    let value_str = unsafe { str::from_utf8_unchecked(external_library_value_data) };
    let value: ExternalLibraryDependency = ason::from_str(value_str).unwrap();

    let external_library_file_path_or_system_library_name = match value {
        ExternalLibraryDependency::Local(_dependency_local) => todo!(),
        ExternalLibraryDependency::Remote(_dependency_remote) => todo!(),
        ExternalLibraryDependency::Share(_dependency_share) => todo!(),
        ExternalLibraryDependency::Runtime => todo!(),
        ExternalLibraryDependency::File(library_path) => {
            if library_path.starts_with("system:") {
                // system library, e.g., `libc.so.6`
                library_path.trim_start_matches("system:").to_string()
            } else {
                let mut module_path_buf =
                    PathBuf::from(&thread_context.process_property.program_path);

                if thread_context.process_property.program_source_type
                    == ProgramSourceType::ScriptFile
                    || thread_context.process_property.program_source_type
                        == ProgramSourceType::PackageImage
                {
                    // if the program is a script file or a package image,
                    // we need to pop the last path component, which is the script or image file name.
                    // this is because the external library path is relative to the module directory.

                    module_path_buf.pop();
                }

                let full_path = library_path
                    .try_resolve_in(&module_path_buf)
                    .unwrap_or_else(|_| {
                        panic!("Failed to resolve external library path: {}", library_path);
                    });

                full_path.to_str().unwrap().to_owned()
            }
        }
    };

    let mut external_function_table = thread_context.external_function_table.lock().unwrap();
    let mut jit_generator = thread_context.jit_generator.lock().unwrap();

    let (external_function_pointer, wrapper_function_index) = create_wrapper_function_item(
        &mut jit_generator,
        &mut external_function_table,
        unified_external_library_index,
        &external_library_file_path_or_system_library_name,
        external_function_name,
        param_datatypes,
        result_datatypes,
    )?;

    // add external function item

    let unified_external_function_pointer_item = UnifiedExternalFunctionPointerItem {
        address: external_function_pointer as usize,
        wrapper_function_index,
    };

    external_function_table.unified_external_function_pointer_list
        [unified_external_function_index] = Some(unified_external_function_pointer_item);

    // returns the external function pointer, wrapper function, and other information.

    let wrapper_function =
        external_function_table.wrapper_function_list[wrapper_function_index].wrapper_function;

    Ok((
        external_function_pointer,
        wrapper_function,
        param_count,
        contains_return_value,
    ))
}

// #[allow(clippy::too_many_arguments)]
fn create_wrapper_function_item(
    jit_generator: &mut Generator<JITModule>,
    external_function_table: &mut ExternalFunctionTable,
    unified_external_library_index: usize,
    // library file path (e.g. `/path/to/library/libabc.so.1`) or
    // system shared library name (e.g. `libc.so.1`)
    external_library_file_path_or_system_library_name: &str,
    external_function_name: &str,
    param_datatypes: &[OperandDataType],
    result_datatypes: &[OperandDataType],
) -> Result<(*mut c_void, usize), ProcessorError> {
    // find external library pointer
    let library_pointer = if let Some(unified_external_library_pointer_item) =
        &external_function_table.unified_external_library_pointer_list
            [unified_external_library_index]
    {
        unified_external_library_pointer_item.address as *mut c_void
    } else {
        // create external library item
        let library_pointer =
            if let Ok(p) = load_library(external_library_file_path_or_system_library_name) {
                p
            } else {
                return Err(ProcessorError {
                    error_type: ProcessorErrorType::ItemNotFound,
                });
            };

        external_function_table.unified_external_library_pointer_list
            [unified_external_library_index] = Some(UnifiedExternalLibraryPointerItem {
            address: library_pointer as usize,
        });

        library_pointer
    };

    // find external function pointer
    let external_function_pointer =
        if let Ok(p) = load_symbol(library_pointer, external_function_name) {
            p
        } else {
            return Err(ProcessorError {
                error_type: ProcessorErrorType::ItemNotFound,
            });
        };

    // find the wrapper function.
    //
    // note that not every external function has a corresponding wrapper function,
    // since all external functions with the same signature (i.e., the same
    // parameters and return values) share one wrapper function.
    let wrapper_function_index = if let Some(wrapper_function_index) = external_function_table
        .wrapper_function_list
        .iter()
        .position(|wrapper_function_item| {
            wrapper_function_item.param_datatypes == param_datatypes
                && wrapper_function_item.result_datatypes == result_datatypes
        }) {
        wrapper_function_index
    } else {
        // create wrapper function item
        let wrapper_function_index = external_function_table.wrapper_function_list.len();

        let wrapper_function_pointer =
            generate_wrapper_function(jit_generator, param_datatypes, result_datatypes);

        let wrapper_function_item = WrapperFunctionItem {
            param_datatypes: param_datatypes.to_vec(),
            result_datatypes: result_datatypes.to_vec(),
            wrapper_function: transmute_symbol_to::<WrapperFunction>(
                wrapper_function_pointer as *mut c_void,
            ),
        };

        external_function_table
            .wrapper_function_list
            .push(wrapper_function_item);

        wrapper_function_index
    };

    Ok((external_function_pointer, wrapper_function_index))
}

// the signature and body of wrapper function:
//
// ```rust
// extern "C" fn wrapper_function (
//     external_function_pointer: *const c_void,
//     params_ptr: *const u8,
//     results_ptr: *mut u8) {
//
//     /* 1. read params from memory pointer `params_ptr` */
//     /* 2. call external function */
//     /* 3. write return value to memory pointer `results_ptr` */
//
// }
// ```
pub fn generate_wrapper_function(
    jit_generator: &mut Generator<JITModule>,
    params: &[OperandDataType],
    results: &[OperandDataType],
) -> *const u8 {
    let pointer_type = jit_generator.module.isa().pointer_type();
    let mem_flags = MemFlags::new();

    // build the signature of the external function:
    //
    // ```rust
    // extern "C" fn external_function (
    //     param0,
    //     param1,
    //     paramN) -> (zero_or_one_result);
    // ```
    let mut func_external_sig = jit_generator.module.make_signature();
    for dt in params {
        func_external_sig
            .params
            .push(AbiParam::new(convert_vm_operand_data_type_to_jit_type(*dt)));
    }
    if !results.is_empty() {
        func_external_sig
            .returns
            .push(AbiParam::new(convert_vm_operand_data_type_to_jit_type(
                results[0],
            )));
    }

    // build the signature of the wrapper function:
    //
    // ```rust
    // extern "C" fn wrapper_function (
    //     external_function_pointer: *const c_void,
    //     params_ptr: *const u8,
    //     results_ptr: *mut u8);
    // ```

    let mut func_wrapper_sig = jit_generator.module.make_signature();
    func_wrapper_sig.params.push(AbiParam::new(pointer_type)); // external_function_pointer
    func_wrapper_sig.params.push(AbiParam::new(pointer_type)); // params_ptr
    func_wrapper_sig.params.push(AbiParam::new(pointer_type)); // results_ptr

    // the name of wrapper function needs to be constructed using a
    // "process global unique id", otherwise, duplicate ids will be generated
    // in unit tests due to parallet running, which will eventually cause
    // the wrapper function construction to fail.

    let mut last_id = LAST_WRAPPER_FUNCTION_ID.lock().unwrap();
    let next_id: usize = *last_id;
    *last_id = next_id + 1;

    let func_wrapper_name = format!("wrapper_{}", next_id);

    let func_wrapper_declare = jit_generator
        .module
        .declare_function(&func_wrapper_name, Linkage::Local, &func_wrapper_sig)
        .unwrap();

    {
        let mut func_wrapper = Function::with_name_signature(
            UserFuncName::user(0, func_wrapper_declare.as_u32()),
            func_wrapper_sig,
        );

        let mut function_builder = FunctionBuilder::new(
            &mut func_wrapper,
            &mut jit_generator.function_builder_context,
        );

        let block_0 = function_builder.create_block();
        function_builder.append_block_params_for_function_params(block_0);
        function_builder.switch_to_block(block_0);

        // build the params for calling external function
        let value_params_ptr = function_builder.block_params(block_0)[1];
        let value_params = (0..params.len())
            .map(|idx| {
                function_builder.ins().load(
                    convert_vm_operand_data_type_to_jit_type(params[idx]),
                    mem_flags,
                    value_params_ptr,
                    (idx * OPERAND_SIZE_IN_BYTES) as i32,
                )
            })
            .collect::<Vec<_>>();

        // the body of wrapper function
        //
        // building external function calling inst
        let callee_0 = function_builder.block_params(block_0)[0];
        let sig_ref0 = function_builder.import_signature(func_external_sig);
        let call0 = function_builder
            .ins()
            .call_indirect(sig_ref0, callee_0, &value_params);

        if !results.is_empty() {
            let value_ret = function_builder.inst_results(call0)[0];

            let value_results_ptr = function_builder.block_params(block_0)[2];
            function_builder
                .ins()
                .store(mem_flags, value_ret, value_results_ptr, 0);
        }

        function_builder.ins().return_(&[]);
        function_builder.seal_all_blocks();
        function_builder.finalize();

        // generate the (machine/native) code of wrapper function
        jit_generator.context.func = func_wrapper;

        jit_generator
            .module
            .define_function(func_wrapper_declare, &mut jit_generator.context)
            .unwrap();
    }

    jit_generator
        .module
        .clear_context(&mut jit_generator.context);

    // link
    jit_generator.module.finalize_definitions().unwrap();

    // get wrapper function pointer
    jit_generator
        .module
        .get_finalized_function(func_wrapper_declare)
}
