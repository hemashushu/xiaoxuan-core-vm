// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::{jit_util::build_host_to_vm_function, thread_context::ThreadContext};

use crate::{InterpreterError, InterpreterErrorType};

// the bridge function
// -------------------
//
// on C/Rust application, embeds the XiaoXuan Core VM and script as a library
// and calls VM function as if it is a native function.
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
    let bridge_function_ptr = build_host_to_vm_function(
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

pub fn build_callback_function(
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
    let callback_function_ptr = build_host_to_vm_function(
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

#[cfg(test)]
mod tests {
    use ancvm_context::program_resource::ProgramResource;
    use ancvm_image::{
        bytecode_writer::{BytecodeWriter, BytecodeWriterHelper},
        utils::helper_build_module_binary_with_single_function,
    };
    use ancvm_isa::{opcode::Opcode, OperandDataType};

    use crate::{
        delegate::build_bridge_function, in_memory_program_resource::InMemoryProgramResource,
    };

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
            vec![],                                           // local vars
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
