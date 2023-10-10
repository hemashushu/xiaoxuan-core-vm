// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_program::{jit_util::build_host_to_vm_function, thread_context::ThreadContext};

use crate::interpreter::process_bridge_function_call;

// about the briage function:
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
// 4. set the memory permission to READ+EXEC (mprotect, VirtualProtect(windows))
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
// however, to build native functions on various arch and platforms is boring job,
// to make life easy, this module uses crate 'cranelift-jit' :D.

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
        thread_context.find_bridge_function(target_module_index, function_internal_index);

    if let Some(bridge_function_ptr) = opt_bridge_function_ptr {
        return Ok(unsafe { std::mem::transmute_copy::<*const u8, T>(&bridge_function_ptr) });
    }

    let type_index = thread_context.program_context.program_modules[target_module_index]
        .func_section
        .items[function_internal_index]
        .type_index;
    let (params, results) = thread_context.program_context.program_modules[target_module_index]
        .type_section
        .get_item_params_and_results(type_index as usize);

    if results.len() > 1 {
        return Err("Only functions with one return value are allowed.");
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

    Ok(unsafe { std::mem::transmute_copy::<*const u8, T>(&bridge_function_ptr) })
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
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{opcode::Opcode, DataType};

    use crate::{bridge::get_function, in_memory_program_source::InMemoryProgramSource};

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

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.new_thread_context();

        let fn_add: extern "C" fn(i32, i32) -> i32 =
            get_function(&mut thread_context0, "main", "add").unwrap();

        assert_eq!(fn_add(11, 13), 24);
        assert_eq!(fn_add(23, 29), 52);
    }
}
