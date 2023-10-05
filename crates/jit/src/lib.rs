// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use cranelift_codegen::settings;
use cranelift_codegen::settings::Configurable;
use cranelift_frontend::FunctionBuilderContext;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::DataDescription;

pub struct JIT {
    // function builder context, for reusing across multiple FunctionBuilder.
    pub function_builder_context: FunctionBuilderContext,

    // data description for functions.
    pub data_description: DataDescription,

    // JIT module, holds and manages the JIT functions.
    pub module: JITModule,
}

impl JIT {
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

#[cfg(test)]
mod tests {
    use cranelift_codegen::ir::{types, AbiParam, Function, InstBuilder, UserFuncName};
    use cranelift_frontend::FunctionBuilder;
    use cranelift_module::{Linkage, Module};

    use crate::JIT;

    #[test]
    fn test_jit_base() {
        let mut jit = JIT::new(vec![]);

        // build func_a
        //
        // ```rust
        // fn func_a (a:i32) -> i32 {
        //    a+11
        // }
        // ```

        let mut func_a_sig = jit.module.make_signature();
        func_a_sig.params.push(AbiParam::new(types::I32));
        func_a_sig.returns.push(AbiParam::new(types::I32));

        let func_a_id = jit
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
                FunctionBuilder::new(&mut func_a, &mut jit.function_builder_context);

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
            // function_builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8));
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
            let mut codegen_context = jit.module.make_context();
            codegen_context.func = func_a;

            jit.module
                .define_function(func_a_id, &mut codegen_context)
                .unwrap();
            jit.module.clear_context(&mut codegen_context);
        }

        // build func_b
        //
        // ```rust
        // fn func_b () -> i32 {
        //    func_a(13)
        // }
        // ```
        let mut func_b_sig = jit.module.make_signature();
        func_b_sig.returns.push(AbiParam::new(types::I32));

        let func_b_id = jit
            .module
            .declare_function("func_b", Linkage::Export, &func_b_sig)
            .unwrap();

        {
            let mut func_b = Function::with_name_signature(
                UserFuncName::user(0, func_b_id.as_u32()),
                func_b_sig,
            );

            let mut function_builder =
                FunctionBuilder::new(&mut func_b, &mut jit.function_builder_context);

            let block = function_builder.create_block();
            function_builder.switch_to_block(block);

            let func_ref0 = jit
                .module
                .declare_func_in_func(func_a_id, &mut function_builder.func);
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

            let mut codegen_context = jit.module.make_context();
            codegen_context.func = func_b;

            jit.module
                .define_function(func_b_id, &mut codegen_context)
                .unwrap();
            jit.module.clear_context(&mut codegen_context);
        }

        // linking
        jit.module.finalize_definitions().unwrap();

        // get function pointers
        let fn_a_ptr = jit.module.get_finalized_function(func_a_id);
        let fn_b_ptr = jit.module.get_finalized_function(func_b_id);

        // cast ptr to Rust function
        let fn_a: extern "C" fn(i32) -> i32 = unsafe { std::mem::transmute(fn_a_ptr) };
        let fn_b: extern "C" fn() -> i32 = unsafe { std::mem::transmute(fn_b_ptr) };

        assert_eq!(fn_a(0), 11);
        assert_eq!(fn_a(3), 14);
        assert_eq!(fn_a(13), 24);
        assert_eq!(fn_b(), 24);
    }

    // for test
    extern "C" fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    #[test]
    fn test_jit_call_external_func_by_import_symbol() {
        let fn_add_ptr = add as *const u8; // as *const extern "C" fn(i32,i32)->i32;

        let mut jit = JIT::new(vec![("fn_add".to_string(), fn_add_ptr)]);

        let mut fn_add_sig = jit.module.make_signature();
        fn_add_sig.params.push(AbiParam::new(types::I32));
        fn_add_sig.params.push(AbiParam::new(types::I32));
        fn_add_sig.returns.push(AbiParam::new(types::I32));

        let fn_add_id = jit
            .module
            .declare_function("fn_add", Linkage::Import, &fn_add_sig)
            .unwrap();

        let mut func_main_sig = jit.module.make_signature();
        func_main_sig.returns.push(AbiParam::new(types::I32));

        let func_main_id = jit
            .module
            .declare_function("main", Linkage::Local, &func_main_sig)
            .unwrap();

        let mut func_main = Function::with_name_signature(
            UserFuncName::user(0, func_main_id.as_u32()),
            func_main_sig,
        );

        let mut function_builder =
            FunctionBuilder::new(&mut func_main, &mut jit.function_builder_context);

        let func_ref0 = jit
            .module
            .declare_func_in_func(fn_add_id, &mut function_builder.func);

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
        let mut codegen_context = jit.module.make_context();
        codegen_context.func = func_main;

        jit.module
            .define_function(func_main_id, &mut codegen_context)
            .unwrap();
        jit.module.clear_context(&mut codegen_context);

        // link
        jit.module.finalize_definitions().unwrap();

        // get func_main ptr
        let fn_main_ptr = jit.module.get_finalized_function(func_main_id);
        let fn_main: extern "C" fn() -> i32 = unsafe { std::mem::transmute(fn_main_ptr) };

        // call func_main
        assert_eq!(fn_main(), 24);
    }

    #[test]
    fn test_jit_call_external_func_by_address() {
        let mut jit = JIT::new(vec![]);
        let pointer_type = jit.module.isa().pointer_type();

        let mut fn_add_sig = jit.module.make_signature();
        fn_add_sig.params.push(AbiParam::new(types::I32));
        fn_add_sig.params.push(AbiParam::new(types::I32));
        fn_add_sig.returns.push(AbiParam::new(types::I32));

        let mut func_main_sig = jit.module.make_signature();
        func_main_sig.params.push(AbiParam::new(pointer_type));
        func_main_sig.returns.push(AbiParam::new(types::I32));

        let func_main_id = jit
            .module
            .declare_function("main", Linkage::Local, &func_main_sig)
            .unwrap();

        let mut func_main = Function::with_name_signature(
            UserFuncName::user(0, func_main_id.as_u32()),
            func_main_sig,
        );

        let mut function_builder =
            FunctionBuilder::new(&mut func_main, &mut jit.function_builder_context);

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
        let mut codegen_context = jit.module.make_context();
        codegen_context.func = func_main;

        jit.module
            .define_function(func_main_id, &mut codegen_context)
            .unwrap();
        jit.module.clear_context(&mut codegen_context);

        // link
        jit.module.finalize_definitions().unwrap();

        // get func_main ptr
        let fn_main_ptr = jit.module.get_finalized_function(func_main_id);
        let fn_main: extern "C" fn(usize) -> i32 = unsafe { std::mem::transmute(fn_main_ptr) };

        // call func_main
        let fn_add_addr = add as *const u8 as usize;
        assert_eq!(fn_main(fn_add_addr), 24);
    }
}
