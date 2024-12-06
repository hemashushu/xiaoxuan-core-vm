// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// the bridge function
// -------------------
//
// the bridge function allows a C/Rust application to embed the XiaoXuan Core VM and
// image as a library and call the VM function as if it were a native C/Rust function.
//
//
//    C/Rust application                  runtime (native)
// /------------------------\          /------------------------\          XiaoXuan Core VM module
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

// 'bridge function' is actually a native function that is built at runtime (it is similar to JIT),
// the principle of building native function at runtime is quite simple:
//
// 1. allocate a block/region of memory (posix_memalign/mmap, VirtualAlloc(windows))
// 2. set the memory permissions to READ+WRITE (optional, vecause this is the default permissions)
// 3. copy the native code of the function to the memory
// 4. set the memory permission to READ+EXEC (using the function `mprotect` or `VirtualProtect(windows)`)
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
//         `mmap with MAP_ANONYMOUT option`
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
//     /** flush the i-cache and d-cache (only necessary on non-x86_64 arch)
//         e.g.
//         macos: sys_icache_invalidate
//         windows: FlushInstructionCache
//         linux on aarch64: dc civac, dsb ish, ic ivau, dsb ish, ish
//         ref:
//         - https://community.arm.com/arm-community-blogs/b/architectures-and-processors-blog/posts/caches-and-self-modifying-code
//         */
//
//     /* convert function pointer into function */
//     let func: fn() -> i64 = std::mem::transmute(func_ptr);
//
//     let val = func();
//     println!("function return: 0x{:x}", val);
// }
//
// however, to build native functions on different platforms is a boring job,
// to make life easy, this module uses crate 'cranelift-jit' to build the bridge functions :D.

// the bridge callback function
// ----------------------------
//
// it's used to pass XiaoXuan Core VM functions as a callback function to the external C library
// on the 'extcall'.
//
//                                      runtime (native)
//                                   /------------------------\
//                                   |                        |
//                                   | external func list     |
//                                   | |--------------------| |
//                                   | | idx | lib  | name  | |
//                              /--> | | 0   | ".." | ".."  | |
//                              |    | |--------------------| |
//                              |    |                        |
//                              |    | wrapper func code 0    |
//  XiaoXuan Core application   |    | 0x0000 0xb8, 0x34,     |
// /------------------------\   |    | 0x000a 0x12, 0x00...   | --\
// |                        |   |    |                        |   |
// | fn $demo () -> ()      |   |    |                        |   |
// |   extcall do_something | --/    | callback func table    |   |
// | end                    |        | |--------------------| |   |
// |                        |        | | mod idx | func idx | |   |      libxyz.so
// | fn $callback () -> ()  | <----- | | 0       | 0        | |   |    /----------------------\
// |   ...                  |        | | ...     | ...      | |   \--> | void do_something (  |
// | end                    |        | |--------------------| |        |     void* () cb) {   |
// |                        |        |                        |        |     ...              |
// \------------------------/        | bridge func code 0     | <----- |     (cb)(11, 13)     |
//                                   | 0x0000 0xb8, 0x34,     |        | }                    |
//                                   | 0x000a 0x12, 0x00...   |        |                      |
//                                   |                        |        \----------------------/
//                                   | bridge func code 1     |
//                                   | ...                    |
//                                   |                        |
//                                   \------------------------/
//

use std::sync::Mutex;

use anc_context::thread_context::{ProgramCounter, ThreadContext};
use anc_isa::{OperandDataType, OPERAND_SIZE_IN_BYTES};
use cranelift_codegen::ir::{
    types, AbiParam, Function, InstBuilder, StackSlotData, StackSlotKind, UserFuncName,
};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::{Linkage, Module};

use crate::{
    handler::Handler,
    jit_generator::{
        convert_vm_operand_data_type_to_jit_type, get_jit_generator_without_imported_symbols,
    },
    process::{process_continuous_instructions, EXIT_CURRENT_HANDLER_LOOP_BIT},
    HandleErrorType, HandlerError,
};

static mut LAST_BRIDGE_FUNCTION_ID: Mutex<usize> = Mutex::new(0);

pub fn get_or_create_bridge_function(
    handler: &Handler,
    thread_context: &mut ThreadContext,
    module_index: usize,
    function_public_index: usize,
) -> Result<*const u8, HandlerError> {
    let (target_module_index, function_internal_index) = thread_context
        .get_function_target_module_index_and_internal_index(module_index, function_public_index);

    // check if the function specified (target_module_index, function_internal_index)
    // already exists in the "bridge function table"
    let opt_bridge_function_ptr =
        thread_context.find_bridge_function(target_module_index, function_internal_index);

    // found it
    if let Some(bridge_function_ptr) = opt_bridge_function_ptr {
        return Ok(bridge_function_ptr /* as *const () */);
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
        return Err(HandlerError::new(HandleErrorType::ResultsAmountMissmatch));
    }

    let delegate_function_addr = delegate_bridge_function_call as *const u8 as usize;
    let handler_addr = handler as *const Handler as *const u8 as usize;
    let thread_context_addr = thread_context as *const ThreadContext as *const u8 as usize;
    let bridge_function_ptr = build_bridge_function(
        delegate_function_addr,
        handler_addr,
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

    Ok(bridge_function_ptr /*as *const ()*/)
}

pub fn get_or_create_bridge_data(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_public_index: usize,
    offset_bytes: usize,
    data_length_in_bytes: usize,
) -> Result<*const u8, HandlerError> {
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
    Ok(ptr /*as *const ()*/)
}

pub fn get_or_create_bridge_callback_function(
    handler: &Handler,
    thread_context: &mut ThreadContext,
    module_index: usize,
    function_public_index: usize,
) -> Result<*const u8, HandlerError> {
    // get the internal index of function
    let (target_module_index, function_internal_index) = thread_context
        .get_function_target_module_index_and_internal_index(module_index, function_public_index);

    // check if the specified (target_module_index, function_internal_index) already
    // exists in the callback function table
    let opt_callback_function_ptr =
        thread_context.find_bridge_callback_function(target_module_index, function_internal_index);

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
        return Err(HandlerError::new(HandleErrorType::ResultsAmountMissmatch));
    }

    let delegate_function_addr = delegate_callback_function_call as *const u8 as usize;
    let handler_addr = handler as *const Handler as *const u8 as usize;
    let thread_context_addr = thread_context as *const ThreadContext as *const u8 as usize;

    let callback_function_ptr = build_bridge_function(
        delegate_function_addr,
        handler_addr,
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
fn build_bridge_function(
    delegate_function_addr: usize,
    handler_addr: usize,
    thread_context_addr: usize,
    target_module_index: usize,
    function_internal_index: usize,
    params: &[OperandDataType],
    results: &[OperandDataType],
) -> *const u8 {
    let mut mutex_jit_generator = get_jit_generator_without_imported_symbols();
    let jit_generator = mutex_jit_generator.as_mut().unwrap();

    let pointer_type = jit_generator.module.isa().pointer_type();

    // the signature of the delegate function:
    //
    // extern "C" fn delegate_function (
    //     handler_ptr: *const u8,
    //     thread_context_ptr: *mut u8,
    //     target_module_index: usize,
    //     function_internal_index: usize,
    //     params_ptr: *const u8,
    //     results_ptr: *mut u8);

    let mut func_delegate_sig = jit_generator.module.make_signature();
    func_delegate_sig.params.push(AbiParam::new(pointer_type)); // handler_ptr
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

    let mut func_exported_sig = jit_generator.module.make_signature();
    for dt in params {
        func_exported_sig
            .params
            .push(AbiParam::new(convert_vm_operand_data_type_to_jit_type(*dt)));
    }
    if !results.is_empty() {
        func_exported_sig
            .returns
            .push(AbiParam::new(convert_vm_operand_data_type_to_jit_type(
                results[0],
            )));
    }

    // the name of wrapper function needs to be constructed using a
    // "process global unique" id, otherwise duplicate ids will be generated
    // in unit tests due to parallet running, which will eventually cause
    // the wrapper function construction to fail.
    let next_id = unsafe {
        let mut last_id = LAST_BRIDGE_FUNCTION_ID.lock().unwrap();
        let next_id: usize = *last_id;
        *last_id = next_id + 1;
        next_id
    };

    let func_exported_name = format!("exported_{}", next_id);

    let func_exported_declare = jit_generator
        .module
        .declare_function(&func_exported_name, Linkage::Local, &func_exported_sig)
        .unwrap();

    {
        let mut func_exported = Function::with_name_signature(
            UserFuncName::user(0, func_exported_declare.as_u32()),
            func_exported_sig,
        );

        // create two stack slots, one for parameters, one for results.
        let ss0 = func_exported.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            (OPERAND_SIZE_IN_BYTES * params.len()) as u32,
            3,
        ));
        let ss1 = func_exported.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            OPERAND_SIZE_IN_BYTES as u32,
            3,
        ));

        let mut function_builder = FunctionBuilder::new(
            &mut func_exported,
            &mut jit_generator.function_builder_context,
        );

        let block_0 = function_builder.create_block();
        function_builder.append_block_params_for_function_params(block_0);
        function_builder.switch_to_block(block_0);

        for idx in 0..params.len() {
            let value_param = function_builder.block_params(block_0)[idx];
            function_builder.ins().stack_store(
                value_param,
                ss0,
                (idx * OPERAND_SIZE_IN_BYTES) as i32,
            );
        }

        // build params for calling delegate function
        let callee_0 = function_builder
            .ins()
            .iconst(pointer_type, delegate_function_addr as i64);
        let param_0 = function_builder
            .ins()
            .iconst(pointer_type, handler_addr as i64);
        let param_1 = function_builder
            .ins()
            .iconst(pointer_type, thread_context_addr as i64);
        let param_2 = function_builder
            .ins()
            .iconst(types::I32, target_module_index as i64);
        let param_3 = function_builder
            .ins()
            .iconst(types::I32, function_internal_index as i64);
        let param_4 = function_builder.ins().stack_addr(pointer_type, ss0, 0);
        let param_5 = function_builder.ins().stack_addr(pointer_type, ss1, 0);

        // call delegate function
        let sig_ref0 = function_builder.import_signature(func_delegate_sig);
        function_builder.ins().call_indirect(
            sig_ref0,
            callee_0,
            &[param_0, param_1, param_2, param_3, param_4, param_5],
        );

        if !results.is_empty() {
            let value_ret = function_builder.ins().stack_load(
                convert_vm_operand_data_type_to_jit_type(results[0]),
                ss1,
                0,
            );
            function_builder.ins().return_(&[value_ret]);
        } else {
            function_builder.ins().return_(&[]);
        }

        function_builder.seal_all_blocks();
        function_builder.finalize();

        // println!("{}", func_exported.display());

        // generate the (machine/native) code of func_bridge
        jit_generator.context.func = func_exported;

        jit_generator
            .module
            .define_function(func_exported_declare, &mut jit_generator.context)
            .unwrap();
    }

    jit_generator
        .module
        .clear_context(&mut jit_generator.context);

    // link
    jit_generator.module.finalize_definitions().unwrap();

    // get func_bridge ptr
    jit_generator
        .module
        .get_finalized_function(func_exported_declare)
}

// function calling from outside of VM (such as a Rust program
// embeds this VM and call a function of VM)
#[allow(clippy::not_unsafe_ptr_arg_deref)]
extern "C" fn delegate_bridge_function_call(
    handler_ptr: *const u8,
    thread_context_ptr: *mut u8,
    target_module_index: usize,
    function_internal_index: usize,
    params_ptr: *const u8,
    results_ptr: *mut u8,
) {
    // params:
    // | 8 bytes | 8 bytes | ... |
    //
    // results:
    // | 8 bytes |

    let handler = unsafe { &*(handler_ptr as *const Handler) };
    let thread_context = unsafe { &mut *(thread_context_ptr as *mut ThreadContext) };

    let (type_index, local_variable_list_index, code_offset, local_variables_allocate_bytes) =
        thread_context
            .get_function_type_and_local_variable_list_index_and_code_offset_and_local_variables_allocate_bytes(
                target_module_index,
                function_internal_index,
            );
    let type_item = &thread_context.module_common_instances[target_module_index]
        .type_section
        .items[type_index];

    let params_count = type_item.params_count as usize;
    let results_count = type_item.results_count as usize;

    // reset the statck
    thread_context.stack.reset();

    // push arguments
    let stack_push_ptr = thread_context.stack.push_operands_from_memory(params_count);
    unsafe {
        std::ptr::copy(
            params_ptr,
            stack_push_ptr,
            OPERAND_SIZE_IN_BYTES * params_count,
        )
    };

    // create function statck frame
    thread_context.stack.create_frame(
        type_item.params_count,
        type_item.results_count,
        local_variable_list_index as u32,
        local_variables_allocate_bytes,
        Some(ProgramCounter {
            instruction_address: 0,
            function_internal_index: 0,

            // set MSB of 'return module index' to '1' to indicate that it's the END of the
            // current function call.
            module_index: EXIT_CURRENT_HANDLER_LOOP_BIT,
        }),
    );

    // set new PC
    thread_context.pc.module_index = target_module_index;
    thread_context.pc.function_internal_index = function_internal_index;
    thread_context.pc.instruction_address = code_offset;

    // start processing instructions
    match process_continuous_instructions(handler, thread_context) {
        Ok(_) => {}
        Err(e) => panic!("{}", e),
    }

    // pop the results from the stack
    // note:
    //
    // only 0 or 1 return value is allowed for C function.
    if results_count > 0 {
        let result_operands = thread_context.stack.pop_operands_without_bound_check(1);
        unsafe { std::ptr::copy(result_operands.as_ptr(), results_ptr, OPERAND_SIZE_IN_BYTES) };
    }
}

// it's similar to the function 'process_bridge_function_call' except
// that 'process_callback_function_call' will not reset the calling stack.
//
// in the other word, the 'process_bridge_function_call' starts a new 'calling-thread',
// and the 'process_callback_function_call' is 'insert' a 'sub-calling-thread' into
// the current 'calling-thread'.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
extern "C" fn delegate_callback_function_call(
    handler_ptr: *const u8,
    thread_context_ptr: *mut u8,
    target_module_index: usize,
    function_internal_index: usize,
    params_ptr: *const u8,
    results_ptr: *mut u8,
) {
    // params:
    // | 8 bytes | 8 bytes | ... |
    //
    // results:
    // | 8 bytes |

    let handler = unsafe { &*(handler_ptr as *const Handler) };
    let thread_context = unsafe { &mut *(thread_context_ptr as *mut ThreadContext) };

    let (type_index, local_variable_list_index, code_offset, local_variables_allocate_bytes) =
        thread_context
            .get_function_type_and_local_variable_list_index_and_code_offset_and_local_variables_allocate_bytes(
                target_module_index,
                function_internal_index,
            );
    let type_item = &thread_context.module_common_instances[target_module_index]
        .type_section
        .items[type_index];

    let params_count = type_item.params_count as usize;
    let results_count = type_item.results_count as usize;

    // push arguments
    let stack_push_ptr = thread_context.stack.push_operands_from_memory(params_count);
    unsafe {
        std::ptr::copy(
            params_ptr,
            stack_push_ptr,
            OPERAND_SIZE_IN_BYTES * params_count,
        )
    };

    // store the current PC as return PC
    let ProgramCounter {
        instruction_address: return_instruction_address,
        function_internal_index: return_function_internal_index,
        module_index: return_module_index,
    } = thread_context.pc;

    let return_pc = ProgramCounter {
        instruction_address: return_instruction_address,
        function_internal_index: return_function_internal_index,

        // set MSB of 'return module index' to '1' to indicate that it's the END of the
        // current function call.
        module_index: return_module_index | EXIT_CURRENT_HANDLER_LOOP_BIT,
    };

    // module M, function A
    //
    // 0x0000 inst_0     callback function     module N, function B
    // 0x0004 inst_1     interrupt
    // 0x0008 inst_2   ----------------------> 0x0000 inst_0
    //     \------------<----------------\     0x0004 inst_1
    //                                   |     0x0008 inst_2
    // HandleResult::Move(X) --\      ^     0x000c inst_3
    //                            |      |     0x0010 end
    //                            |      |       |
    // 0x000c inst_3   <----------/      \---<---/
    // 0x0010 inst_4
    // 0x0014 inst_5
    // 0x0018 inst_6
    // 0x001c end

    // create function statck frame
    thread_context.stack.create_frame(
        type_item.params_count,
        type_item.results_count,
        local_variable_list_index as u32,
        local_variables_allocate_bytes,
        Some(return_pc),
    );

    // set new PC
    thread_context.pc.module_index = target_module_index;
    thread_context.pc.function_internal_index = function_internal_index;
    thread_context.pc.instruction_address = code_offset;

    // start processing instructions
    match process_continuous_instructions(handler, thread_context) {
        Ok(_) => {}
        Err(e) => panic!("{}", e),
    }

    // pop the results from the stack
    // note:
    //
    // only 0 or 1 return value is allowed for C function.
    if results_count > 0 {
        let result_operands = thread_context.stack.pop_operands_without_bound_check(1);
        unsafe { std::ptr::copy(result_operands.as_ptr(), results_ptr, OPERAND_SIZE_IN_BYTES) };
    }
}
