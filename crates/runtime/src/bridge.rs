// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::sync::Once;

use ancvm_thread::thread_context::ThreadContext;
use ancvm_types::{DataType, OPERAND_SIZE_IN_BYTES};
use cranelift_codegen::ir::{
    types, AbiParam, Function, InstBuilder, StackSlotData, StackSlotKind, Type, UserFuncName,
};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::{Linkage, Module};

use crate::{interpreter::process_function_internal, jit_helper::JITHelper};

static mut JIT_HELPER: Option<JITHelper> = None;
static INIT: Once = Once::new();

fn get_jit_helper() -> &'static mut JITHelper {
    INIT.call_once(|| {
        unsafe { JIT_HELPER = Some(JITHelper::new(vec![])) };
    });

    unsafe { JIT_HELPER.as_mut().unwrap() }
}

pub fn get_function<T>(
    thread_context: &mut ThreadContext,
    _module_name: &str,
    _exported_function_name: &str,
) -> Result<T, &'static str> {
    //
    //
    //
    // todo find the specified module and the exported function by name
    //
    //
    //
    //

    let (target_module_index, function_internal_index) =
        thread_context.get_function_internal_index_and_module_index(0, 0);

    // check if the specified (target_module_index, function_internal_index) already
    // exists in the bridge function table
    let opt_bridge_function_ptr =
        thread_context.get_bridge_function(target_module_index, function_internal_index);

    if let Some(bridge_function_ptr) = opt_bridge_function_ptr {
        return Ok(unsafe { std::mem::transmute_copy::<*const u8, T>(&bridge_function_ptr) });
    }

    let type_index = thread_context.program_ref.modules[target_module_index]
        .func_section
        .items[function_internal_index]
        .type_index;
    let (params, results) = thread_context.program_ref.modules[target_module_index]
        .type_section
        .get_params_and_results_list(type_index as usize);

    if results.len() > 1 {
        return Err("The number of return values of the specified function is more than 1.");
    }

    let thread_context_addr = thread_context as *const ThreadContext as *const u8 as usize;
    let bridge_function_ptr = build_bridge_function(
        thread_context_addr,
        target_module_index,
        function_internal_index,
        params,
        results,
    );

    // store the function pointer into table
    thread_context.add_bridge_function(
        target_module_index,
        function_internal_index,
        bridge_function_ptr,
    );

    Ok(unsafe { std::mem::transmute_copy::<*const u8, T>(&bridge_function_ptr) })
}

fn convert_vm_data_type_to_jit_type(dt: DataType) -> Type {
    match dt {
        DataType::I32 => types::I32,
        DataType::I64 => types::I64,
        DataType::F32 => types::F32,
        DataType::F64 => types::F64,
    }
}

fn build_bridge_function(
    thread_context_addr: usize,
    target_module_index: usize,
    function_internal_index: usize,
    params: &[DataType],
    results: &[DataType],
) -> *const u8 {
    let jit_helper = get_jit_helper();
    let pointer_type = jit_helper.module.isa().pointer_type();
    let fn_internal_addr = process_function_internal as *const u8 as usize;

    // the interpreter internal call function:
    //
    // extern "C" fn process_function_internal(
    //     thread_context_ptr: *mut u8,
    //     target_module_index: usize,
    //     function_internal_index: usize,
    //     params_ptr: *const u8,
    //     results_ptr: *mut u8) {...}

    let mut fn_internal_sig = jit_helper.module.make_signature();
    fn_internal_sig.params.push(AbiParam::new(pointer_type)); // thread_context_ptr
    fn_internal_sig.params.push(AbiParam::new(types::I64)); // target_module_index
    fn_internal_sig.params.push(AbiParam::new(types::I64)); // function_internal_index
    fn_internal_sig.params.push(AbiParam::new(pointer_type)); // params_ptr
    fn_internal_sig.params.push(AbiParam::new(pointer_type)); // results_ptr

    let mut func_bridge_sig = jit_helper.module.make_signature();
    for dt in params {
        func_bridge_sig
            .params
            .push(AbiParam::new(convert_vm_data_type_to_jit_type(*dt)));
    }
    if !results.is_empty() {
        func_bridge_sig
            .returns
            .push(AbiParam::new(convert_vm_data_type_to_jit_type(results[0])));
    }

    let func_bridge_name = format!("f{}_{}", target_module_index, function_internal_index);
    let func_bridge_id = jit_helper
        .module
        .declare_function(&func_bridge_name, Linkage::Local, &func_bridge_sig)
        .unwrap();

    let mut func_bridge = Function::with_name_signature(
        UserFuncName::user(0, func_bridge_id.as_u32()),
        func_bridge_sig,
    );

    // create two stack slots, one for parameters, one for results.
    let ss0 = func_bridge.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        (OPERAND_SIZE_IN_BYTES * params.len()) as u32,
    ));
    let ss1 = func_bridge.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        OPERAND_SIZE_IN_BYTES as u32,
    ));

    let mut function_builder =
        FunctionBuilder::new(&mut func_bridge, &mut jit_helper.function_builder_context);

    let block_0 = function_builder.create_block();
    function_builder.append_block_params_for_function_params(block_0);
    function_builder.switch_to_block(block_0);

    for idx in 0..params.len() {
        let value_param = function_builder.block_params(block_0)[idx];
        function_builder
            .ins()
            .stack_store(value_param, ss0, (idx * OPERAND_SIZE_IN_BYTES) as i32);
    }

    // internal call
    let callee_0 = function_builder
        .ins()
        .iconst(pointer_type, fn_internal_addr as i64);
    let param_0 = function_builder
        .ins()
        .iconst(types::I64, thread_context_addr as i64);
    let param_1 = function_builder
        .ins()
        .iconst(types::I64, target_module_index as i64);
    let param_2 = function_builder
        .ins()
        .iconst(types::I64, function_internal_index as i64);
    let param_3 = function_builder.ins().stack_addr(pointer_type, ss0, 0);
    let param_4 = function_builder.ins().stack_addr(pointer_type, ss1, 0);

    let sig_ref0 = function_builder.import_signature(fn_internal_sig);
    function_builder.ins().call_indirect(
        sig_ref0,
        callee_0,
        &[param_0, param_1, param_2, param_3, param_4],
    );

    if !results.is_empty() {
        let value_ret =
            function_builder
                .ins()
                .stack_load(convert_vm_data_type_to_jit_type(results[0]), ss1, 0);
        function_builder.ins().return_(&[value_ret]);
    } else {
        function_builder.ins().return_(&[]);
    }

    function_builder.seal_all_blocks();
    function_builder.finalize();

    // println!("{}", func_bridge.display());

    // generate the (machine/native) code of func_bridge
    let mut codegen_context = jit_helper.module.make_context();
    codegen_context.func = func_bridge;

    jit_helper
        .module
        .define_function(func_bridge_id, &mut codegen_context)
        .unwrap();
    jit_helper.module.clear_context(&mut codegen_context);

    // link
    jit_helper.module.finalize_definitions().unwrap();

    // get func_bridge ptr
    jit_helper.module.get_finalized_function(func_bridge_id)
}

pub fn get_data<T>(
    _thread_context: &mut ThreadContext,
    _module_name: &str,
    _exported_function_name: &str,
) -> Result<T, &'static str> {
    // todo find the specified module and the exported function
    todo!()
}

#[cfg(test)]
mod tests {
    use ancvm_binary::utils::{build_module_binary_with_single_function, BytecodeWriter};
    use ancvm_types::{opcode::Opcode, DataType};

    use crate::{in_memory_program::InMemoryProgram, program::Program};

    use super::get_function;

    #[test]
    fn test_get_function() {
        // bytecode
        //
        // 0x0000 local_load32         0 0 0
        // 0x0008 local_load32         0 0 1
        // 0x0010 i32_add
        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_add)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32], // params
            vec![DataType::I32],                // results
            vec![],                             // local vars
            code0,
        );

        let program0 = InMemoryProgram::new(vec![binary0]);
        let program_context0 = program0.build_program_context().unwrap();
        let mut thread_context0 = program_context0.new_thread_context();

        let fn_add: extern "C" fn(i32, i32) -> i32 =
            get_function(&mut thread_context0, "main", "add").unwrap();

        assert_eq!(fn_add(11, 13), 24);
        assert_eq!(fn_add(23, 29), 52);
    }
}
