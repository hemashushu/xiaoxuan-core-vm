// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.


// the bridge function
// -------------------
//
// on C/Rust application, embeds the XiaoXuan Core VM and script as a library
// and calls VM function as if it is a native function.
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

// 'bridge function' is actually a native function which is created at runtime (it is similar to JIT),
// the principle of building native funtion at runtime is quite simple:
// 1. allocates a block/region of memeory (posix_memalign/mmap, VirtualAlloc(windows))
// 2. set the memory permission to READ+WRITE (optional, vecause this is the default permissions)
// 3. copy the native code of function to the memory
// 4. set the memory permission to READ+EXEC (by function `mprotect`, `VirtualProtect(windows)`)
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
// however, to build native functions on various arch and platforms is a boring job,
// to make life easy, this module uses crate 'cranelift-jit' :D.

// function calling from outside of VM (such as a Rust program
// embeds this VM and call a function of VM)
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn process_bridge_function_call(
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

    // // initialize interpreters
    // init_interpreters();
    let interpreter = Handler::new(); // todo: should be obtained from prt

    let thread_context = unsafe { &mut *(thread_context_ptr as *mut ThreadContext) };

    let (type_index, local_list_index, code_offset, local_variables_allocate_bytes) =
        thread_context
            .get_function_type_and_local_list_index_and_code_offset_and_local_variables_allocate_bytes(
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
        local_list_index as u32,
        local_variables_allocate_bytes,
        Some(ProgramCounter {
            instruction_address: 0,
            function_internal_index: 0,

            // set MSB of 'return module index' to '1' to indicate that it's the END of the
            // current function call.
            module_index: 0 | EXIT_CURRENT_HANDLER_LOOP_BIT,
        }),
    );

    // set new PC
    thread_context.pc.module_index = target_module_index;
    thread_context.pc.function_internal_index = function_internal_index;
    thread_context.pc.instruction_address = code_offset;

    // start processing instructions
    match process_continuous_instructions(&interpreter, thread_context) {
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
pub extern "C" fn process_callback_function_call(
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

    let interpreter = Handler::new(); // todo: should be obtained from prt

    let thread_context = unsafe { &mut *(thread_context_ptr as *mut ThreadContext) };

    let (type_index, local_list_index, code_offset, local_variables_allocate_bytes) =
        thread_context
            .get_function_type_and_local_list_index_and_code_offset_and_local_variables_allocate_bytes(
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
        local_list_index as u32,
        local_variables_allocate_bytes,
        Some(return_pc),
    );

    // set new PC
    thread_context.pc.module_index = target_module_index;
    thread_context.pc.function_internal_index = function_internal_index;
    thread_context.pc.instruction_address = code_offset;

    // start processing instructions
    match process_continuous_instructions(&interpreter, thread_context) {
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


// pub fn get_bridge_function<T>(
//     thread_context: &mut ThreadContext,
//     module_name: &str,
//     function_name: &str,
// ) -> Result<T, HandlerError> {
//     let (module_index, function_public_index) = thread_context
//         .find_function_public_index_by_name(module_name, function_name)
//         .ok_or(HandlerError::new(HandlerErrorType::ItemNotFound))?;
//
//     let function_ptr = build_bridge_function(thread_context, module_index, function_public_index)?;
//     let function = unsafe { std::mem::transmute_copy(&function_ptr) };
//     Ok(function)
// }
//
// pub fn get_bridge_data<T>(
//     thread_context: &mut ThreadContext,
//     module_name: &str,
//     data_name: &str,
// ) -> Result<*const T, HandlerError>
// where
//     T: Sized,
// {
//     let (module_index, data_public_index) = thread_context
//         .find_data_public_index_by_name(module_name, data_name)
//         .ok_or(HandlerError::new(HandlerErrorType::ItemNotFound))?;
//
//     let data_ptr = build_bridge_data(
//         thread_context,
//         module_index,
//         data_public_index,
//         0,
//         std::mem::size_of::<T>(),
//     )?;
//
//     Ok(data_ptr as *const T)
// }