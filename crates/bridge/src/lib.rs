// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// there are 2 uses of bridge function
//
// 1. on XiaoXuan core script application, pass VM function as a callback function to external C function
//
//                                      runtime (native)
//                                   /------------------------\
//                                   |                        |
//                                   | external func list     |
//                                   | |--------------------| |
//  XiaoXuan core application        | | idx | lib  | name  | |
// /------------------------\   /--> | | 0   | ".." | ".."  | | --\
// |                        |   |    | |--------------------| |   |
// | fn $demo () -> ()      |   |    |                        |   |
// |   extcall do_something | --/    | bridge func table      |   |
// | end                    |        | |--------------------| |   |
// |                        |        | | mod idx | func idx | |   |      libxyz.so
// | fn $callback () -> ()  | <----- | | 0       | 0        | |   |    /----------------------\
// |   ...                  |        | | ...     | ...      | |   \--> | void do_something (  |
// | end                    |        | |--------------------| |        |     void* () cb) {   |
// |                        |        |                        |        |     ...              |
// \------------------------/        | bridge func code 0     | <----- |     (cb)(11, 13)     |
//                                   | 0x0000 0xb8, 0x34,     |        | }                    |
//                                   | 0x0000 0x12, 0x00...   |        |                      |
//                                   |                        |        \----------------------/
//                                   | bridge func code 1     |
//                                   | ...                    |
//                                   |                        |
//                                   \------------------------/
//
//
// 2. on C/Rust application, embed XiaoXuan core script as a library and call VM function as a native function.
//
//
//    C/Rust application                  runtime (native)
// /------------------------\          /------------------------\          XiaoXuan VM module
// |                        |          | bridge func table      |       /------------------------\
// | int (*add)(int,int)=.. |          | |--------------------| |       |                        |
// | int c = add(11,13);    | ---\     | | mod idx | func idx | | ----> | fn (i32, i32) -> (i32) |
// | printf("%d", c);       |    |     | | 0       | 0        | |       |     i32.add            |
// |                        |    |     | | ...     | ...      | |       | end                    |
// \------------------------/    |     | |--------------------| |       |                        |
//                               |     |                        |       \------------------------/
//                               \---> | bridge func code 0     |
//                                     | 0x0000 0xb8, 0x34,     |
//                                     | 0x0000 0x12, 0x00...   |
//                                     |                        |
//                                     | bridge func code 1     |
//                                     | ...                    |
//                                     |                        |
//                                     \------------------------/
//

// 'bridge function' is actually a native function which is created at runtime, it is similar to JIT,
// the principle of building native funtion at runtime is quite simple:
// 1. allocates a block of memeory
// 2. set the memory permission to READ+WRITE
// 3. copy the native code of function to the memory
// 4. set the memory permission to READ+EXEC
//
// the following is a snippet for creating a simple native function:
//
// use libc::{c_void, memset, perror, size_t, sysconf};
// use libc::{memalign, memcpy, mprotect};
//
// fn main() {
//     /** the function and its native x86_64 code:
//         fn f() -> i64 {
//             return 0x1234;
//         }
//         */
//     let code: [u8; 6] = [
//         0xb8, 0x34, 0x12, 0x00, 0x00, // mov $0x1234,%eax
//         0xc3, // ret
//     ];
//
//     let pagesize = sysconf(libc::_SC_PAGE_SIZE) as size_t;
//     let buffer_length = 4 * pagesize;
//
//     /** allocate memory block for executable code
//
//         `void *aligned_alloc(size_t alignment, size_t size);`
//         `int posix_memalign(void **memptr, size_t alignment, size_t size);`
//         `void *memalign(size_t alignment, size_t size);` (deprecated)
//
//         ref:
//         https://www.gnu.org/software/libc/manual/html_node/Memory_002dmapped-I_002fO.html
//         https://www.gnu.org/software/libc/manual/html_node/Aligned-Memory-Blocks.html
//         */
//     let buffer_ptr = memalign(pagesize, buffer_length);
//
//     /** change the permission for this memory block
//
//         `int mprotect(void *addr, size_t len, int prot);`
//
//         ref:
//         https://www.gnu.org/software/libc/manual/html_node/Memory-Protection.html
//         */
//     let mprotect_result = mprotect(
//         buffer_ptr,
//         buffer_length,
//         libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
//     );
//
//     if mprotect_result == -1 {
//         perror(b"mprotect\0".as_ptr() as *const i8);
//         return;
//     }
//
//     /* fill memory block with instruction 'ret' (optional) */
//     memset(buffer_ptr, 0xc3, buffer_length);
//
//     /* copy native code to the memory block */
//     let func_ptr = memcpy(buffer_ptr, code.as_ptr() as *const c_void, code.len());
//
//     /** flush the icache and dcache (only necessary on non-x86_64 arch)
//         e.g.
//         macos: sys_icache_invalidate
//         windows: FlushInstructionCache
//         linux on aarch64: dc civac, dsb ish, ic ivau, dsb ish, ish
//         */
//
//     /* convert function pointer into function */
//     let func: fn() -> i64 = std::mem::transmute(func_ptr);
//
//     let val = func();
//     println!("function return: 0x{:x}", val);
// }
//
// however, to build native functions on various arch and platforms is boring job,
// to make life easy, this module uses crate 'cranelift-jit' :D.

// use cranelift_jit::

use cranelift_codegen::settings::Configurable;
use cranelift_codegen::{settings, Context};
use cranelift_frontend::FunctionBuilderContext;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, Module};

pub struct JIT {
    // function builder context, for reusing across multiple FunctionBuilder.
    pub function_builder_context: FunctionBuilderContext,

    // codegen context
    pub codegen_context: Context,

    // data description for functions.
    pub data_description: DataDescription,

    // JIT module, holds and manages the JIT functions.
    pub module: JITModule,
}

impl JIT {
    // ref:
    // - https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/jit/examples/jit-minimal.rs
    // - https://github.com/bytecodealliance/cranelift-jit-demo/blob/main/src/jit.rs
    pub fn new() -> Self {
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

        let jit_builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let jit_module = JITModule::new(jit_builder);

        Self {
            function_builder_context: FunctionBuilderContext::new(),
            codegen_context: jit_module.make_context(),
            data_description: DataDescription::new(),
            module: jit_module,
        }
    }
}

#[cfg(test)]
mod tests {
    use cranelift_codegen::ir::{types, AbiParam, InstBuilder, UserFuncName};
    use cranelift_frontend::FunctionBuilder;
    use cranelift_module::{Linkage, Module};

    use crate::JIT;

    #[test]
    fn test_jit_base() {
        let mut jit = JIT::new();

        // build func_a
        //
        // ```rust
        // fn func_a (a:i32) -> i32 {
        //    a+11
        // }
        // ```

        let mut sig_a = jit.module.make_signature();
        sig_a.params.push(AbiParam::new(types::I32));
        sig_a.returns.push(AbiParam::new(types::I32));

        let func_id_a = jit
            .module
            .declare_function("func_a", Linkage::Export, &sig_a)
            .unwrap();

        jit.codegen_context.func.signature = sig_a;
        jit.codegen_context.func.name = UserFuncName::user(0, func_id_a.as_u32());

        {
            let mut function_builder = FunctionBuilder::new(
                &mut jit.codegen_context.func,
                &mut jit.function_builder_context,
            );

            let block = function_builder.create_block();

            function_builder.switch_to_block(block);
            function_builder.append_block_params_for_function_params(block);

            let value_0 = function_builder.ins().iconst(types::I32, 11);
            let value_1 = function_builder.block_params(block)[0];
            let value_2 = function_builder.ins().iadd(value_0, value_1);
            function_builder.ins().return_(&[value_2]);

            function_builder.seal_all_blocks();
            function_builder.finalize();
        }

        jit.module
            .define_function(func_id_a, &mut jit.codegen_context)
            .unwrap();
        jit.module.clear_context(&mut jit.codegen_context);

        // build func_b
        //
        // ```rust
        // fn func_b () -> i32 {
        //    func_a(13)
        // }
        // ```
        let mut sig_b = jit.module.make_signature();
        sig_b.returns.push(AbiParam::new(types::I32));

        let func_id_b = jit
            .module
            .declare_function("func_b", Linkage::Export, &sig_b)
            .unwrap();

        jit.codegen_context.func.signature = sig_b;
        jit.codegen_context.func.name = UserFuncName::user(0, func_id_b.as_u32());

        {
            let mut function_builder = FunctionBuilder::new(
                &mut jit.codegen_context.func,
                &mut jit.function_builder_context,
            );

            let block = function_builder.create_block();
            function_builder.switch_to_block(block);

            let func_ref0 = jit
                .module
                .declare_func_in_func(func_id_a, &mut function_builder.func);
            let value0 = function_builder.ins().iconst(types::I32, 13);
            let inst0 = function_builder.ins().call(func_ref0, &[value0]);
            let value1 = {
                let results = function_builder.inst_results(inst0);
                assert_eq!(results.len(), 1);
                results[0]
            };
            function_builder.ins().return_(&[value1]);
            function_builder.seal_all_blocks();
            function_builder.finalize();
        }

        jit.module
            .define_function(func_id_b, &mut jit.codegen_context)
            .unwrap();
        jit.module.clear_context(&mut jit.codegen_context);

        // linking
        jit.module.finalize_definitions().unwrap();

        // get function pointers
        let func_a_ptr = jit.module.get_finalized_function(func_id_a);
        let func_b_ptr = jit.module.get_finalized_function(func_id_b);

        // cast ptr to Rust function
        let func_a: extern "C" fn(i32) -> i32 = unsafe { std::mem::transmute(func_a_ptr) };
        let func_b: extern "C" fn() -> i32 = unsafe { std::mem::transmute(func_b_ptr) };

        assert_eq!(func_a(0), 11);
        assert_eq!(func_a(3), 14);
        assert_eq!(func_a(13), 24);
        assert_eq!(func_b(), 24);
    }
}
