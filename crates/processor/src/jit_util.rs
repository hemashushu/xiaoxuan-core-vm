// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::sync::{Mutex, MutexGuard, Once};

use cranelift_codegen::ir::{
    types, AbiParam, Function, InstBuilder, MemFlags, StackSlotData, StackSlotKind, UserFuncName,
};
use cranelift_codegen::settings::Configurable;
use cranelift_codegen::{ir::Type, settings};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, Linkage, Module};
use rand::Rng;

use ancvm_isa::{OperandDataType, OPERAND_SIZE_IN_BYTES};

static mut JIT_UTIL_WITHOUT_IMPORTED_SYMBOLS: Mutex<Option<JITUtil>> = Mutex::new(None);
static INIT: Once = Once::new();

fn get_jit_util_without_imported_symbols() -> MutexGuard<'static, Option<JITUtil>> {
    INIT.call_once(|| {
        unsafe { JIT_UTIL_WITHOUT_IMPORTED_SYMBOLS = Mutex::new(Some(JITUtil::new(vec![]))) };
    });

    unsafe {
        let a = JIT_UTIL_WITHOUT_IMPORTED_SYMBOLS.lock().unwrap();
        a
    }
}

pub struct JITUtil {
    // function builder context, for reusing across multiple FunctionBuilder.
    pub function_builder_context: FunctionBuilderContext,

    // data description for functions.
    pub data_description: DataDescription,

    // JIT module, holds and manages the JIT functions.
    pub module: JITModule,
}

impl JITUtil {
    // ref:
    // - https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/jit/examples/jit-minimal.rs
    // - https://github.com/bytecodealliance/cranelift-jit-demo/blob/main/src/jit.rs
    pub fn new(symbols: Vec<(String, *const u8)>) -> Self {
        // the building flow:
        //
        // flag builder -> isa builder -> jit builder -> jit module

        let mut flag_builder = settings::builder();

        // Use colocated libcalls.
        // Generate code that assumes that libcalls can be declared “colocated”,
        // meaning they will be defined along with the current function,
        // such that they can use more efficient addressing.
        // ref:
        // https://docs.rs/cranelift-codegen/0.100.0/cranelift_codegen/settings/struct.Flags.html#method.use_colocated_libcalls
        flag_builder.set("use_colocated_libcalls", "false").unwrap();

        // Enable Position-Independent Code generation.
        // ref:
        // https://docs.rs/cranelift-codegen/0.100.0/cranelift_codegen/settings/struct.Flags.html#method.is_pic
        flag_builder.set("is_pic", "true").unwrap();

        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });

        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();

        let mut jit_builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

        // import external symbols
        //
        // to add single symbol:
        // `jit_builder.symbol(name:String, ptr:*const u8)`
        jit_builder.symbols(symbols);

        let jit_module = JITModule::new(jit_builder);

        Self {
            function_builder_context: FunctionBuilderContext::new(),
            // codegen_context: jit_module.make_context(),
            data_description: DataDescription::new(),
            module: jit_module,
        }
    }
}

fn convert_vm_data_type_to_jit_type(dt: OperandDataType) -> Type {
    match dt {
        OperandDataType::I32 => types::I32,
        OperandDataType::I64 => types::I64,
        OperandDataType::F32 => types::F32,
        OperandDataType::F64 => types::F64,
    }
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
pub fn build_host_to_vm_function(
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
pub fn build_vm_to_external_function(
    _wrapper_function_index: usize,
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
            .push(AbiParam::new(convert_vm_data_type_to_jit_type(*dt)));
    }
    if !results.is_empty() {
        func_external_sig
            .returns
            .push(AbiParam::new(convert_vm_data_type_to_jit_type(results[0])));
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

    // failed in unit test because the multiple Program(s) create duplicated id.
    /*
    let func_wrapper_name = format!("wrapper_{}", wrapper_function_index);
     */
    let func_wrapper_name = format!("wrapper_{}", rand::thread_rng().gen::<u32>());

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
                convert_vm_data_type_to_jit_type(params[idx]),
                mem_flags,
                value_params_ptr,
                (idx * OPERAND_SIZE_IN_BYTES) as i32,
            )
        })
        .collect::<Vec<_>>();

    // call external function
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

#[cfg(test)]
mod tests {
    use cranelift_codegen::ir::{
        types, AbiParam, Function, InstBuilder, StackSlotData, StackSlotKind, UserFuncName,
    };
    use cranelift_frontend::FunctionBuilder;
    use cranelift_module::{Linkage, Module};

    use crate::jit_util::JITUtil;

    #[test]
    fn test_jit_base() {
        let mut jit_helper = JITUtil::new(vec![]);

        // build func_a
        //
        // ```rust
        // fn func_a (a:i32) -> i32 {
        //    a+11
        // }
        // ```

        let mut func_a_sig = jit_helper.module.make_signature();
        func_a_sig.params.push(AbiParam::new(types::I32));
        func_a_sig.returns.push(AbiParam::new(types::I32));

        let func_a_id = jit_helper
            .module
            .declare_function("func_a", Linkage::Export, &func_a_sig)
            .unwrap();

        {
            // the following 'let mut func_a = ...' and 'let mut function_builder = ...' is equivalent to:
            //
            // jit.codegen_context.func.signature = sig_a;
            // jit.codegen_context.func.name = UserFuncName::user(0, func_id_a.as_u32());
            //
            // let mut function_builder = FunctionBuilder::new(
            //     &mut jit.codegen_context.func,
            //     &mut jit.function_builder_context,
            // );

            let mut func_a = Function::with_name_signature(
                UserFuncName::user(0, func_a_id.as_u32()),
                func_a_sig,
            );

            let mut function_builder =
                FunctionBuilder::new(&mut func_a, &mut jit_helper.function_builder_context);

            // about local variables:
            //
            // let x = Variable::new(0);
            // let y = Variable::new(1);
            // let z = Variable::new(2);
            // function_builder.declare_var(x, types::I32);
            // function_builder.declare_var(y, types::I32);
            // function_builder.declare_var(z, types::I32);
            // function_builder.def_var(x, tmp);        // set value
            // let .. = function_builder.use_var(x);    // get value
            //
            // ref:
            // - https://docs.rs/cranelift-frontend/latest/cranelift_frontend/
            //
            // about stack slots:
            //
            // a sequence memory area in the stack, it is equivalent to
            // the XiaoXuan VM function's local variables area).
            //
            // func.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8));
            // function_builder.ins().stack_load(Mem, SS, Offset);
            // function_builder.ins().stack_store(x, SS, Offset);
            // function_builder.ins().stack_addr(iAddr, SS, Offset);
            //
            // ref:
            // - https://docs.rs/cranelift-codegen/latest/cranelift_codegen/ir/trait.InstBuilder.html#method.stack_load

            let block = function_builder.create_block();
            function_builder.append_block_params_for_function_params(block);

            function_builder.switch_to_block(block);

            // about the instructions:
            // - https://docs.rs/cranelift-codegen/latest/cranelift_codegen/ir/trait.InstBuilder.html

            let value_0 = function_builder.ins().iconst(types::I32, 11);
            let value_1 = function_builder.block_params(block)[0];
            let value_2 = function_builder.ins().iadd(value_0, value_1);
            function_builder.ins().return_(&[value_2]);

            function_builder.seal_all_blocks();
            function_builder.finalize();

            // to display the text of IR
            // `println!("{}", func_a.display());`

            // generate func_a body's (machine/native) code

            // codegen context, a context per thread
            let mut codegen_context = jit_helper.module.make_context();
            codegen_context.func = func_a;

            jit_helper
                .module
                .define_function(func_a_id, &mut codegen_context)
                .unwrap();
            jit_helper.module.clear_context(&mut codegen_context);
        }

        // build func_b
        //
        // ```rust
        // fn func_b () -> i32 {
        //    func_a(13)
        // }
        // ```
        let mut func_b_sig = jit_helper.module.make_signature();
        func_b_sig.returns.push(AbiParam::new(types::I32));

        let func_b_id = jit_helper
            .module
            .declare_function("func_b", Linkage::Export, &func_b_sig)
            .unwrap();

        {
            let mut func_b = Function::with_name_signature(
                UserFuncName::user(0, func_b_id.as_u32()),
                func_b_sig,
            );

            let mut function_builder =
                FunctionBuilder::new(&mut func_b, &mut jit_helper.function_builder_context);

            let block = function_builder.create_block();
            function_builder.switch_to_block(block);

            let func_ref0 = jit_helper
                .module
                .declare_func_in_func(func_a_id, function_builder.func);
            let value0 = function_builder.ins().iconst(types::I32, 13);
            let call0 = function_builder.ins().call(func_ref0, &[value0]);
            let value1 = {
                let results = function_builder.inst_results(call0);
                assert_eq!(results.len(), 1);
                results[0]
            };
            function_builder.ins().return_(&[value1]);

            function_builder.seal_all_blocks();
            function_builder.finalize();

            // generate func_b body's (machine/native) code

            let mut codegen_context = jit_helper.module.make_context();
            codegen_context.func = func_b;

            jit_helper
                .module
                .define_function(func_b_id, &mut codegen_context)
                .unwrap();
            jit_helper.module.clear_context(&mut codegen_context);
        }

        // linking
        jit_helper.module.finalize_definitions().unwrap();

        // get function pointers
        let fn_a_ptr = jit_helper.module.get_finalized_function(func_a_id);
        let fn_b_ptr = jit_helper.module.get_finalized_function(func_b_id);

        // cast ptr to Rust function
        let fn_a: extern "C" fn(i32) -> i32 = unsafe { std::mem::transmute(fn_a_ptr) };
        let fn_b: extern "C" fn() -> i32 = unsafe { std::mem::transmute(fn_b_ptr) };

        assert_eq!(fn_a(0), 11);
        assert_eq!(fn_a(3), 14);
        assert_eq!(fn_a(13), 24);
        assert_eq!(fn_b(), 24);
    }

    // for test 'test_jit_call_external_func_by_import_symbol' and
    // 'test_jit_call_external_func_by_address'
    extern "C" fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    #[test]
    fn test_jit_call_external_func_by_import_symbol() {
        let fn_add_ptr = add as *const u8; // as *const extern "C" fn(i32,i32)->i32;

        let mut jit_helper = JITUtil::new(vec![("fn_add".to_string(), fn_add_ptr)]);

        let mut fn_add_sig = jit_helper.module.make_signature();
        fn_add_sig.params.push(AbiParam::new(types::I32));
        fn_add_sig.params.push(AbiParam::new(types::I32));
        fn_add_sig.returns.push(AbiParam::new(types::I32));

        let fn_add_id = jit_helper
            .module
            .declare_function("fn_add", Linkage::Import, &fn_add_sig)
            .unwrap();

        let mut func_main_sig = jit_helper.module.make_signature();
        func_main_sig.returns.push(AbiParam::new(types::I32));

        let func_main_id = jit_helper
            .module
            .declare_function("main", Linkage::Local, &func_main_sig)
            .unwrap();

        let mut func_main = Function::with_name_signature(
            UserFuncName::user(0, func_main_id.as_u32()),
            func_main_sig,
        );

        let mut function_builder =
            FunctionBuilder::new(&mut func_main, &mut jit_helper.function_builder_context);

        let func_ref0 = jit_helper
            .module
            .declare_func_in_func(fn_add_id, function_builder.func);

        let block_0 = function_builder.create_block();
        function_builder.switch_to_block(block_0);

        let value_0 = function_builder.ins().iconst(types::I32, 11);
        let value_1 = function_builder.ins().iconst(types::I32, 13);
        let call0 = function_builder.ins().call(func_ref0, &[value_0, value_1]);
        let value_2 = function_builder.inst_results(call0)[0];

        function_builder.ins().return_(&[value_2]);
        function_builder.seal_all_blocks();
        function_builder.finalize();

        // to display the text of IR
        // `println!("{}", func_main.display());`

        // generate the (machine/native) code of func_main
        let mut codegen_context = jit_helper.module.make_context();
        codegen_context.func = func_main;

        jit_helper
            .module
            .define_function(func_main_id, &mut codegen_context)
            .unwrap();
        jit_helper.module.clear_context(&mut codegen_context);

        // link
        jit_helper.module.finalize_definitions().unwrap();

        // get func_main ptr
        let fn_main_ptr = jit_helper.module.get_finalized_function(func_main_id);
        let fn_main: extern "C" fn() -> i32 = unsafe { std::mem::transmute(fn_main_ptr) };

        // call func_main
        assert_eq!(fn_main(), 24);
    }

    #[test]
    fn test_jit_call_external_func_by_address() {
        let mut jit_helper = JITUtil::new(vec![]);
        let pointer_type = jit_helper.module.isa().pointer_type();

        let mut fn_add_sig = jit_helper.module.make_signature();
        fn_add_sig.params.push(AbiParam::new(types::I32));
        fn_add_sig.params.push(AbiParam::new(types::I32));
        fn_add_sig.returns.push(AbiParam::new(types::I32));

        let mut func_main_sig = jit_helper.module.make_signature();
        func_main_sig.params.push(AbiParam::new(pointer_type));
        func_main_sig.returns.push(AbiParam::new(types::I32));

        let func_main_id = jit_helper
            .module
            .declare_function("main", Linkage::Local, &func_main_sig)
            .unwrap();

        let mut func_main = Function::with_name_signature(
            UserFuncName::user(0, func_main_id.as_u32()),
            func_main_sig,
        );

        let mut function_builder =
            FunctionBuilder::new(&mut func_main, &mut jit_helper.function_builder_context);

        let block_0 = function_builder.create_block();
        function_builder.append_block_params_for_function_params(block_0);
        function_builder.switch_to_block(block_0);

        let value_0 = function_builder.ins().iconst(types::I32, 11);
        let value_1 = function_builder.ins().iconst(types::I32, 13);
        let value_2 = function_builder.block_params(block_0)[0];

        let sig_ref0 = function_builder.import_signature(fn_add_sig);
        let call0 = function_builder
            .ins()
            .call_indirect(sig_ref0, value_2, &[value_0, value_1]);
        let value_2 = function_builder.inst_results(call0)[0];

        function_builder.ins().return_(&[value_2]);
        function_builder.seal_all_blocks();
        function_builder.finalize();

        // generate the (machine/native) code of func_main
        let mut codegen_context = jit_helper.module.make_context();
        codegen_context.func = func_main;

        jit_helper
            .module
            .define_function(func_main_id, &mut codegen_context)
            .unwrap();
        jit_helper.module.clear_context(&mut codegen_context);

        // link
        jit_helper.module.finalize_definitions().unwrap();

        // get func_main ptr
        let fn_main_ptr = jit_helper.module.get_finalized_function(func_main_id);
        let fn_main: extern "C" fn(usize) -> i32 = unsafe { std::mem::transmute(fn_main_ptr) };

        // call func_main
        let fn_add_addr = add as *const u8 as usize;
        assert_eq!(fn_main(fn_add_addr), 24);
    }

    extern "C" fn array(params: *const u8, results: *const u8) {
        // params:
        // | 8 bytes | 8 bytes | 8 bytes | 8 bytes | 8 bytes |
        // | i32     | i64     | f32     | f64     | iaddr   |
        //
        // results:
        // | 8 bytes |
        // | i32     |

        let i = unsafe { std::ptr::read(params.add(0) as *const i32) };
        let j = unsafe { std::ptr::read(params.add(8) as *const i64) };
        let m = unsafe { std::ptr::read(params.add(16) as *const f32) };
        let n = unsafe { std::ptr::read(params.add(24) as *const f64) };
        let p = unsafe { std::ptr::read(params.add(32) as *const i64) };

        // write '211' to the 'results' when values of all parameter are as expected,
        // otherwise write '199'

        let b = (i == 41) && (j == 43) && (m == 3.5) && (n == 7.5);
        let result = if b { 211 } else { 199 };
        unsafe { std::ptr::write(results as *mut i32, result) };

        // p:
        // | 4 bytes | 4 bytes |
        // | i32     | i32     |

        // write '109' and '113' to 'p'
        let s = unsafe { std::slice::from_raw_parts_mut(p as *mut i32, 2) };
        s[0] = 53;
        s[1] = 59;
    }

    #[test]
    fn test_jit_call_external_func_by_address_with_params_and_results() {
        let mut jit_helper = JITUtil::new(vec![]);
        let pointer_type = jit_helper.module.isa().pointer_type();

        let fn_array_addr = array as *const u8 as usize;

        let mut fn_array_sig = jit_helper.module.make_signature();
        fn_array_sig.params.push(AbiParam::new(pointer_type));
        fn_array_sig.params.push(AbiParam::new(pointer_type));

        let mut func_main_sig = jit_helper.module.make_signature();
        func_main_sig.params.push(AbiParam::new(types::I32));
        func_main_sig.params.push(AbiParam::new(types::I64));
        func_main_sig.params.push(AbiParam::new(types::F32));
        func_main_sig.params.push(AbiParam::new(types::F64));
        func_main_sig.params.push(AbiParam::new(pointer_type));
        func_main_sig.returns.push(AbiParam::new(types::I32));

        // the IR of func_main:
        //
        // ```ir
        // function u0:0(i32, i64, f32, f64, i64) -> i32 system_v {
        //     ss0 = explicit_slot 40
        //     ss1 = explicit_slot 8
        //     sig0 = (i64, i64) system_v
        //
        // block0(v0: i32, v1: i64, v2: f32, v3: f64, v4: i64):
        //     stack_store v0, ss0
        //     stack_store v1, ss0+8
        //     stack_store v2, ss0+16
        //     stack_store v3, ss0+24
        //     stack_store v4, ss0+32
        //     v5 = iconst.i64 0x559f_1144_8df0
        //     v6 = stack_addr.i64 ss0
        //     v7 = stack_addr.i64 ss1
        //     call_indirect sig0, v5(v6, v7)
        //     v8 = stack_load.i32 ss1
        //     return v8
        // }
        // ```

        let func_main_id = jit_helper
            .module
            .declare_function("main", Linkage::Local, &func_main_sig)
            .unwrap();

        let mut func_main = Function::with_name_signature(
            UserFuncName::user(0, func_main_id.as_u32()),
            func_main_sig,
        );

        // create two stack slots
        let ss0 = func_main.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            40,
            2,
        ));
        let ss1 = func_main.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            8,
            2,
        ));

        let mut function_builder =
            FunctionBuilder::new(&mut func_main, &mut jit_helper.function_builder_context);

        let block_0 = function_builder.create_block();
        function_builder.append_block_params_for_function_params(block_0);
        function_builder.switch_to_block(block_0);

        let value_0 = function_builder.block_params(block_0)[0];
        let value_1 = function_builder.block_params(block_0)[1];
        let value_2 = function_builder.block_params(block_0)[2];
        let value_3 = function_builder.block_params(block_0)[3];
        let value_4 = function_builder.block_params(block_0)[4];

        function_builder.ins().stack_store(value_0, ss0, 0);
        function_builder.ins().stack_store(value_1, ss0, 8);
        function_builder.ins().stack_store(value_2, ss0, 16);
        function_builder.ins().stack_store(value_3, ss0, 24);
        function_builder.ins().stack_store(value_4, ss0, 32);

        let addr_0 = function_builder
            .ins()
            .iconst(pointer_type, fn_array_addr as i64);
        let ptr_0 = function_builder.ins().stack_addr(pointer_type, ss0, 0);
        let ptr_1 = function_builder.ins().stack_addr(pointer_type, ss1, 0);

        let sig_ref0 = function_builder.import_signature(fn_array_sig);
        function_builder
            .ins()
            .call_indirect(sig_ref0, addr_0, &[ptr_0, ptr_1]);

        let value_ret = function_builder.ins().stack_load(types::I32, ss1, 0);

        function_builder.ins().return_(&[value_ret]);
        function_builder.seal_all_blocks();
        function_builder.finalize();

        // println!("{}", func_main.display());

        // generate the (machine/native) code of func_main
        let mut codegen_context = jit_helper.module.make_context();
        codegen_context.func = func_main;

        jit_helper
            .module
            .define_function(func_main_id, &mut codegen_context)
            .unwrap();
        jit_helper.module.clear_context(&mut codegen_context);

        // link
        jit_helper.module.finalize_definitions().unwrap();

        // get func_main ptr
        let fn_main_ptr = jit_helper.module.get_finalized_function(func_main_id);
        let fn_main: extern "C" fn(i32, i64, f32, f64, usize) -> i32 =
            unsafe { std::mem::transmute(fn_main_ptr) };

        // call func_main
        let buf: [u8; 8] = [31, 0, 0, 0, 37, 0, 0, 0];
        let buf_addr = buf.as_ptr() as usize;
        assert_eq!(fn_main(41, 43, 3.5, 7.5, buf_addr), 211);

        let buf_i32 = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const i32, 2) };
        assert_eq!(buf_i32[0], 53);
        assert_eq!(buf_i32[1], 59);
    }
}
