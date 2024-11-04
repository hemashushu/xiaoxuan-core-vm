// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

pub fn build_bridge_function(
    thread_context: &mut ThreadContext,
    module_index: usize,
    function_public_index: usize,
) -> Result<*const (), InterpreterError> {
    let (target_module_index, function_internal_index) = thread_context
        .get_function_target_module_index_and_internal_index(module_index, function_public_index);

    // check if the specified (target_module_index, function_internal_index) already
    // exists in the bridge function table
    let opt_bridge_function_ptr =
        thread_context.find_bridge_function(target_module_index, function_internal_index);

    if let Some(bridge_function_ptr) = opt_bridge_function_ptr {
        return Ok(bridge_function_ptr as *const ());
    }

    let type_index = thread_context.module_common_instances[target_module_index]
        .function_section
        .items[function_internal_index]
        .type_index;
    let (params, results) = thread_context.module_common_instances[target_module_index]
        .type_section
        .get_item_params_and_results(type_index as usize);

    if results.len() > 1 {
        // The specified function has more than 1 return value.
        return Err(InterpreterError::new(
            InterpreterErrorType::ResultsAmountMissmatch,
        ));
    }

    let delegate_function_addr = process_bridge_function_call as *const u8 as usize;
    let thread_context_addr = thread_context as *const ThreadContext as *const u8 as usize;
    let bridge_function_ptr = do_build_bridge_function(
        delegate_function_addr,
        thread_context_addr,
        target_module_index,
        function_internal_index,
        params,
        results,
    );

    // store the function pointer into table
    thread_context.insert_bridge_function(
        target_module_index,
        function_internal_index,
        bridge_function_ptr,
    );

    Ok(bridge_function_ptr as *const ())
}

pub fn build_bridge_data(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_public_index: usize,
    offset_bytes: usize,
    data_length_in_bytes: usize,
) -> Result<*const (), InterpreterError> {
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_data_target_module_index_and_internal_index_and_data_object_with_bounds_check(
            module_index,
            data_public_index,
            offset_bytes,
            data_length_in_bytes,
        );

    let total_offset =
        data_object.get_data_address_by_index_and_offset(data_internal_index, offset_bytes);
    let ptr = data_object.get_ptr(total_offset);
    Ok(ptr as *const ())
}

pub fn build_bridge_callback_function(
    thread_context: &mut ThreadContext,
    module_index: usize,
    function_public_index: usize,
) -> Result<*const u8, InterpreterError> {
    // get the internal index of function
    let (target_module_index, function_internal_index) = thread_context
        .get_function_target_module_index_and_internal_index(module_index, function_public_index);

    // check if the specified (target_module_index, function_internal_index) already
    // exists in the callback function table
    let opt_callback_function_ptr =
        thread_context.find_callback_function(target_module_index, function_internal_index);

    if let Some(callback_function_ptr) = opt_callback_function_ptr {
        return Ok(callback_function_ptr);
    }

    let type_index = thread_context.module_common_instances[target_module_index]
        .function_section
        .items[function_internal_index]
        .type_index;
    let (params, results) = thread_context.module_common_instances[target_module_index]
        .type_section
        .get_item_params_and_results(type_index as usize);

    if results.len() > 1 {
        // The specified function has more than 1 return value.
        return Err(InterpreterError::new(
            InterpreterErrorType::ResultsAmountMissmatch,
        ));
    }

    let delegate_function_addr = process_callback_function_call as *const u8 as usize;
    let thread_context_addr = thread_context as *const ThreadContext as *const u8 as usize;
    let callback_function_ptr = do_build_bridge_function(
        delegate_function_addr,
        thread_context_addr,
        target_module_index,
        function_internal_index,
        params,
        results,
    );

    // store the function pointer into table
    thread_context.insert_callback_function(
        target_module_index,
        function_internal_index,
        callback_function_ptr,
    );

    Ok(callback_function_ptr)
}


//     Rust/C application
// /-----------------------------------------------\
// | fn exported_0_0 (a:i32, b:i32) -> i32 {...}   |
// |    |                    (generated by JIT)    |
// |    |                                          |
// \-----------------------------------------------/
//      |
//      v  VM Runtime
// /--------------------------------\
// |    |                           |
// | delegate function (Rust)       |
// |    |                           |
// |    V      VM module            |
// | /----------------------------\ |
// | | module function (bytecode) | |
// | \----------------------------/ |
// \--------------------------------/
//
// about the exported function:
//
// extern "C" fn exported_function (
//     param0,
//     param1,
//     paramN) -> ? {
//     1. create stack slots
//     2. put parameters to stack slots
//     3. call delegate_function (...stack slots...)
//     4. return value
// }
fn do_build_bridge_function(
    delegate_function_addr: usize,
    thread_context_addr: usize,
    target_module_index: usize,
    function_internal_index: usize,
    params: &[OperandDataType],
    results: &[OperandDataType],
) -> *const u8 {
    let mut mutex_jit_helper = get_jit_util_without_imported_symbols();
    let jit_helper = mutex_jit_helper.as_mut().unwrap();

    let pointer_type = jit_helper.module.isa().pointer_type();

    // the signature of the delegate function:
    //
    // extern "C" fn delegate_function (
    //     thread_context_ptr: *mut u8,
    //     target_module_index: usize,
    //     function_internal_index: usize,
    //     params_ptr: *const u8,
    //     results_ptr: *mut u8);

    let mut func_delegate_sig = jit_helper.module.make_signature();
    func_delegate_sig.params.push(AbiParam::new(pointer_type)); // thread_context_ptr
    func_delegate_sig.params.push(AbiParam::new(types::I32)); // target_module_index
    func_delegate_sig.params.push(AbiParam::new(types::I32)); // function_internal_index
    func_delegate_sig.params.push(AbiParam::new(pointer_type)); // params_ptr
    func_delegate_sig.params.push(AbiParam::new(pointer_type)); // results_ptr

    // the signature of the exported function:
    //
    // extern "C" fn exported_function (
    //     param0,
    //     param1,
    //     paramN) -> ?;

    let mut func_exported_sig = jit_helper.module.make_signature();
    for dt in params {
        func_exported_sig
            .params
            .push(AbiParam::new(convert_vm_data_type_to_jit_type(*dt)));
    }
    if !results.is_empty() {
        func_exported_sig
            .returns
            .push(AbiParam::new(convert_vm_data_type_to_jit_type(results[0])));
    }

    // failed in unit test because the multiple Program(s) create duplicated id.
    /*
    let func_exported_name = format!(
        "exported_{}_{}",
        target_module_index, function_internal_index
    );
     */

    let func_exported_name = format!("exported_{}", rand::thread_rng().gen::<u32>());

    let func_exported_id = jit_helper
        .module
        .declare_function(&func_exported_name, Linkage::Local, &func_exported_sig)
        .unwrap();

    let mut func_exported = Function::with_name_signature(
        UserFuncName::user(0, func_exported_id.as_u32()),
        func_exported_sig,
    );

    // create two stack slots, one for parameters, one for results.
    let ss0 = func_exported.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        (OPERAND_SIZE_IN_BYTES * params.len()) as u32,
        2,
    ));
    let ss1 = func_exported.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        OPERAND_SIZE_IN_BYTES as u32,
        2,
    ));

    let mut function_builder =
        FunctionBuilder::new(&mut func_exported, &mut jit_helper.function_builder_context);

    let block_0 = function_builder.create_block();
    function_builder.append_block_params_for_function_params(block_0);
    function_builder.switch_to_block(block_0);

    for idx in 0..params.len() {
        let value_param = function_builder.block_params(block_0)[idx];
        function_builder
            .ins()
            .stack_store(value_param, ss0, (idx * OPERAND_SIZE_IN_BYTES) as i32);
    }

    // build params for calling delegate function
    let callee_0 = function_builder
        .ins()
        .iconst(pointer_type, delegate_function_addr as i64);
    let param_0 = function_builder
        .ins()
        .iconst(pointer_type, thread_context_addr as i64);
    let param_1 = function_builder
        .ins()
        .iconst(types::I32, target_module_index as i64);
    let param_2 = function_builder
        .ins()
        .iconst(types::I32, function_internal_index as i64);
    let param_3 = function_builder.ins().stack_addr(pointer_type, ss0, 0);
    let param_4 = function_builder.ins().stack_addr(pointer_type, ss1, 0);

    // call delegate function
    let sig_ref0 = function_builder.import_signature(func_delegate_sig);
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

    // println!("{}", func_exported.display());

    // generate the (machine/native) code of func_bridge
    let mut codegen_context = jit_helper.module.make_context();
    codegen_context.func = func_exported;

    jit_helper
        .module
        .define_function(func_exported_id, &mut codegen_context)
        .unwrap();

    jit_helper.module.clear_context(&mut codegen_context);

    // link
    jit_helper.module.finalize_definitions().unwrap();

    // get func_bridge ptr
    jit_helper.module.get_finalized_function(func_exported_id)
}

/*
#[cfg(test)]
mod tests {
    #[test]
    fn test_get_function() {
        // bytecode
        //
        // 0x0000 local_load32         0 0 0
        // 0x0008 local_load32         0 0 1
        // 0x0010 add_i32
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::add_i32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![OperandDataType::I32, OperandDataType::I32], // params
            vec![OperandDataType::I32],                       // results
            vec![],                                           // local variables
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let fn_add_ptr = build_bridge_function(&mut thread_context0, 0, 0).unwrap();

        let fn_add: extern "C" fn(i32, i32) -> i32 =
            unsafe { std::mem::transmute_copy(&fn_add_ptr) };

        assert_eq!(fn_add(11, 13), 24);
        assert_eq!(fn_add(23, 29), 52);
    }
}

 */