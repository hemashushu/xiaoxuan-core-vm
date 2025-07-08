// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

// Bridge Function
// ---------------
//
// The bridge function enables a Rust application to embed the XiaoXuan Core VM and
// its binary image as a library, allowing VM functions to be called as if they were native Rust functions.
//
// Example invocation flow:
//
// ```diagram
//     Rust Application                      XiaoXuan Core VM
// /------------------------\          /------------------------\         XiaoXuan Core Function
// |                        |          | Bridge Function Table  |       /------------------------\
// | int (*add)(int,int)=.. |          | |--------------------| |       |                        |
// | int c = add(11,13);    | ---\     | | mod idx | func idx | | ----> | fn (i32, i32) -> (i32) |
// | printf("%d", c);       |    |     | | 0       | 0        | |       |     i32.add            |
// |                        |    |     | | ...     | ...      | |       | end                    |
// \------------------------/    |     | |--------------------| |       |                        |
//                               |     |                        |       \------------------------/
//                               \---> | Bridge Function Code 0 |
//                                     | 0x0000 0xb8, 0x34,     |
//                                     | 0x0000 0x12, 0x00...   |
//                                     |                        |
//                                     | Bridge Function Code 1 |
//                                     | ...                    |
//                                     |                        |
//                                     \------------------------/
// ```
//
// Note:
// - A 'bridge function' is a native function generated at runtime (JIT).
// - The bridge function table maps module/function indices to these native bridge functions.
//
// Principle of Bridge Function Generation:
// ----------------------------------------
//
// 1. Allocate a memory region (e.g., posix_memalign/mmap on Unix, VirtualAlloc on Windows).
// 2. Set memory permissions to READ+WRITE (optional, as this is often the default).
// 3. Copy the native code for the function into the allocated memory.
// 4. Change memory permissions to READ+EXEC (using mprotect or VirtualProtect).
//
// Example: Creating a simple native function at runtime (x86_64):
//
// ```c
// use libc::{c_void, memset, perror, size_t, sysconf};
// use libc::{memalign, memcpy, mprotect};
//
// fn main() {
//     /**
//      * the function and its native x86_64 code:
//      *  fn f() -> i64 {
//      *      return 0x1234;
//      *  }
//      */
//     let code: [u8; 6] = [
//         0xb8, 0x34, 0x12, 0x00, 0x00, // mov $0x1234,%eax
//         0xc3, // ret
//     ];
//
//     let pagesize = sysconf(libc::_SC_PAGE_SIZE) as size_t;
//     let buffer_length = 4 * pagesize;
//
//     /**
//      * allocate memory block for executable code
//      *
//      *  `void *aligned_alloc(size_t alignment, size_t size);`
//      *  `int posix_memalign(void **memptr, size_t alignment, size_t size);`
//      *  `void *memalign(size_t alignment, size_t size);` (deprecated)
//      *  `mmap with MAP_ANONYMOUT option`
//      *
//      *   ref:
//      *   https://www.gnu.org/software/libc/manual/html_node/Memory_002dmapped-I_002fO.html
//      *   https://www.gnu.org/software/libc/manual/html_node/Aligned-Memory-Blocks.html
//      */
//     let buffer_ptr = memalign(pagesize, buffer_length);
//
//     /**
//      * change the permission for this memory block
//      *
//      *  `int mprotect(void *addr, size_t len, int prot);`
//      *
//      *  ref:
//      *  https://www.gnu.org/software/libc/manual/html_node/Memory-Protection.html
//      */
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
//     /**
//      * flush the i-cache and d-cache (only necessary on non-x86_64 arch)
//      *  e.g.
//      *  macos: sys_icache_invalidate
//      *  windows: FlushInstructionCache
//      *  linux on aarch64: dc civac, dsb ish, ic ivau, dsb ish, ish
//      *  ref:
//      *  - https://community.arm.com/arm-community-blogs/b/architectures-and-processors-blog/posts/caches-and-self-modifying-code
//      */
//
//     /* convert function pointer into function */
//     let func: fn() -> i64 = std::mem::transmute(func_ptr);
//     let val = func();
//     println!("function return: 0x{:x}", val);
// }
// ```
//
// Note: Building native functions for different platforms can be tedious.
// This module uses the 'cranelift-jit' crate to simplify bridge function generation.

pub struct BridgeFunctionTable {
    pub functions_by_modules: Vec<BridgeFunctionsByModule>,
}

impl BridgeFunctionTable {
    pub fn new() -> Self {
        Self {
            functions_by_modules: Vec::new(),
        }
    }
}

impl Default for BridgeFunctionTable {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BridgeFunctionsByModule {
    pub module_index: usize,
    pub bridge_function_items: Vec<BridgeFunctionItem>,
}

pub struct BridgeFunctionItem {
    pub function_internal_index: usize,
    pub bridge_function_ptr: *const u8,
}

impl BridgeFunctionTable {
    pub fn find_bridge_function(
        &self,
        target_module_index: usize,
        target_function_internal_index: usize,
    ) -> Option<*const u8> {
        match self.functions_by_modules
            .iter()
            .find(|module_item| module_item.module_index == target_module_index)
        {
            Some(module_item) => module_item
                .bridge_function_items
                .iter()
                .find(|bridge_function_item| {
                    bridge_function_item.function_internal_index == target_function_internal_index
                })
                .map(|bridge_function_item| bridge_function_item.bridge_function_ptr),
            None => None,
        }
    }
}
