// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// the external function
// ---------------------
//
//      VM Runtime
// /------------------------------------\
// |                    VM module       |
// | /----------------------------\     |
// | | module function (bytecode) |     |
// | \----------------------------/     |
// |    |                               |
// | fn wrapper_0 (...)                 |
// |    |         (generate by JIT)     |
// \------------------------------------/
//      |
//      V                   Rust/C application
// /----------------------------------------------\
// | fn external_func (a:i32, b:i32) -> i32 {...} |
// |                                              |
// \----------------------------------------------/
//

use std::{ffi::c_void, sync::Mutex};

use ancvm_context::external_function_table::{
    ExtenalFunctionTable, UnifiedExternalFunctionPointerItem, UnifiedExternalLibraryPointerItem,
    WrapperFunction, WrapperFunctionItem,
};
use ancvm_isa::{OperandDataType, OPERAND_SIZE_IN_BYTES};
use cranelift_codegen::ir::{types, AbiParam, Function, InstBuilder, MemFlags, Type, UserFuncName};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::{Linkage, Module};
use dyncall_util::{load_library, load_symbol, transmute_symbol_to};

use crate::jit_util::get_jit_util_without_imported_symbols;

static mut LAST_WRAPPER_FUNCTION_ID: Mutex<usize> = Mutex::new(0);

pub fn add_external_function(
    external_function_table: &mut ExtenalFunctionTable,
    unified_external_function_index: usize,
    unified_external_library_index: usize,
    external_library_file_path_or_name: &str, // `/path/to/library/libabc.so` or `libc.so`
    external_function_name: &str,
    param_datatypes: &[OperandDataType],
    result_datatypes: &[OperandDataType],
) -> Result<(*mut c_void, WrapperFunction), &'static str> {
    if result_datatypes.len() > 1 {
        return Err("The specified function has more than 1 return value.");
    }

    // find external library pointer
    let library_pointer = if let Some(unified_external_library_pointer_item) =
        &external_function_table.unified_external_library_pointer_list
            [unified_external_library_index]
    {
        unified_external_library_pointer_item.address as *mut c_void
    } else {
        add_external_library(
            external_function_table,
            unified_external_library_index,
            external_library_file_path_or_name,
        )?
    };

    // find external function pointer
    let function_pointer = load_symbol(library_pointer, external_function_name)?;

    // find wrapper function index
    //
    // note that not every external function has a corresponding wrapper function,
    // in fact, as long as the functions are of the same type (i.e., have the same
    // parameters and return values), they will share a wrapper function.
    let wrapper_function_index = if let Some(wrapper_function_index) = external_function_table
        .wrapper_function_list
        .iter()
        .position(|wrapper_function_item| {
            wrapper_function_item.param_datatypes == param_datatypes
                && wrapper_function_item.result_datatypes == result_datatypes
        }) {
        wrapper_function_index
    } else {
        add_wrapper_function(external_function_table, param_datatypes, result_datatypes)
    };

    // update external_function_pointer_list
    let unified_external_function_pointer_item = UnifiedExternalFunctionPointerItem {
        address: function_pointer as usize,
        wrapper_function_index,
    };
    external_function_table.unified_external_function_pointer_list
        [unified_external_function_index] = Some(unified_external_function_pointer_item);

    let wrapper_function =
        external_function_table.wrapper_function_list[wrapper_function_index].wrapper_function;

    Ok((function_pointer, wrapper_function))
}

fn add_external_library(
    external_function_table: &mut ExtenalFunctionTable,
    unified_external_library_index: usize,
    external_library_file_path_or_name: &str,
) -> Result<*mut c_void, &'static str> {
    let library_pointer = load_library(external_library_file_path_or_name)?;
    external_function_table.unified_external_library_pointer_list[unified_external_library_index] =
        Some(UnifiedExternalLibraryPointerItem {
            address: library_pointer as usize,
        });
    Ok(library_pointer)
}

fn add_wrapper_function(
    external_function_table: &mut ExtenalFunctionTable,
    param_types: &[OperandDataType],
    result_types: &[OperandDataType],
) -> usize {
    // build wrapper function
    let wrapper_function_index = external_function_table.wrapper_function_list.len();

    let wrapper_function_pointer =
        build_wrapper_function(/* wrapper_function_index, */ param_types, result_types);

    let wrapper_function_item = WrapperFunctionItem {
        param_datatypes: param_types.to_vec(),
        result_datatypes: result_types.to_vec(),
        wrapper_function: transmute_symbol_to::<WrapperFunction>(
            wrapper_function_pointer as *mut c_void,
        ),
    };

    external_function_table
        .wrapper_function_list
        .push(wrapper_function_item);

    wrapper_function_index
}

// the wrapper function:
//
// extern "C" fn wrapper_function (
//     external_function_pointer: *const c_void,
//     params_ptr: *const u8,
//     results_ptr: *mut u8) {
//     1. read params from memory of 'params'
//     2. call external function
//     3. write return value to memory of 'results'
// }
pub fn build_wrapper_function(
    /* wrapper_function_index: usize, */
    params: &[OperandDataType],
    results: &[OperandDataType],
) -> *const u8 {
    let mut mutex_jit_helper = get_jit_util_without_imported_symbols();
    let jit_helper = mutex_jit_helper.as_mut().unwrap();

    let pointer_type = jit_helper.module.isa().pointer_type();
    let mem_flags = MemFlags::new();

    // the signature of the external function:
    //
    // extern "C" fn external_function (
    //     param0,
    //     param1,
    //     paramN) -> ?;
    let mut func_external_sig = jit_helper.module.make_signature();
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

    // the signature of the wrapper function:
    //
    // extern "C" fn wrapper_function (
    //     external_function_pointer: *const c_void,
    //     params_ptr: *const u8,
    //     results_ptr: *mut u8);

    let mut func_wrapper_sig = jit_helper.module.make_signature();
    func_wrapper_sig.params.push(AbiParam::new(pointer_type)); // external_function_pointer
    func_wrapper_sig.params.push(AbiParam::new(pointer_type)); // params_ptr
    func_wrapper_sig.params.push(AbiParam::new(pointer_type)); // results_ptr

    // the name of wrapper function needs to be constructed using a
    // "process global unique" id, otherwise duplicate ids will be generated
    // in unit tests due to parallet running, which will eventually cause
    // the wrapper function construction to fail.
    let next_id = unsafe {
        let mut last_id = LAST_WRAPPER_FUNCTION_ID.lock().unwrap();
        let next_id: usize = *last_id;
        *last_id = next_id + 1;
        next_id
    };

    let func_wrapper_name = format!("wrapper_{}", next_id);

    let func_wrapper_id = jit_helper
        .module
        .declare_function(&func_wrapper_name, Linkage::Local, &func_wrapper_sig)
        .unwrap();

    let mut func_wrapper = Function::with_name_signature(
        UserFuncName::user(0, func_wrapper_id.as_u32()),
        func_wrapper_sig,
    );

    let mut function_builder =
        FunctionBuilder::new(&mut func_wrapper, &mut jit_helper.function_builder_context);

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

    // println!("{}", func_wrapper.display());

    // generate the (machine/native) code of func_bridge
    let mut codegen_context = jit_helper.module.make_context();
    codegen_context.func = func_wrapper;

    jit_helper
        .module
        .define_function(func_wrapper_id, &mut codegen_context)
        .unwrap();

    jit_helper.module.clear_context(&mut codegen_context);

    // link
    jit_helper.module.finalize_definitions().unwrap();

    // get func_bridge ptr
    jit_helper.module.get_finalized_function(func_wrapper_id)
}

fn convert_vm_operand_data_type_to_jit_type(dt: OperandDataType) -> Type {
    match dt {
        OperandDataType::I32 => types::I32,
        OperandDataType::I64 => types::I64,
        OperandDataType::F32 => types::F32,
        OperandDataType::F64 => types::F64,
    }
}
