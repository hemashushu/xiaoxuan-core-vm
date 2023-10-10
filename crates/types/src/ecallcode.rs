// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

#[repr(u32)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum ECallCode {
    //
    // runtime functions
    //

    // info

    runtime_name = 0x100,   // get the VM runtime code name
                            // `fn (buf_ptr: i64) -> name_len:i32`
                            //
    runtime_version,        // get the VM runtime version
                            // `fn () -> version:i64`
                            //
                            // 0x0000_0000_0000_0000
                            //        |    |    |
                            //        |    |    |patch version
                            //        |    |minor
                            //        |major
                            //
    features,               // get a list of feature names separated by commas, e.g.
                            // "syscall,extcall"
                            //
                            // `fn (buf_ptr: i64) -> feature_list_len:i32`
    check_feature,          // `fn (name_ptr:i64, name_len:i32) -> bool`

    // heap memory

    heap_fill,              // fill the specified memory region with the specified (i8) value
                            // `fn (offset:i64, value:i8, count:i64)`
                            //
    heap_copy,              // copy the specified memory region to the specified location
                            // `fn (dst_offset:i64, src_offset:i64, length_in_bytes:i64)`
                            //
    heap_capacity,          // return the amount of pages of the thread-local
                            // memory (i.e. heap), each page is 32 KiB by default
                            // `fn () -> pages:i64`
                            //
    heap_resize,            // increase or decrease the heap size
                            // return the new capacity (in pages)
                            // `fn (pages:i64) -> new_pages:i64`

    // thread

    // the XiaoXuan VM thread model:
    //
    //     thread
    //   /--------------------------------------\
    //   |                                      |-\
    //   |  /---------------\                   | |  pipe
    //   |  | child threads |-----------------------------\
    //   |  \---------------/  /-----------\    | |       |
    //   |                     | parent th |----| |-------|-\
    //   |                     \-----------/    | |       | |
    //   |                                      | |       | |
    //   |  /---------------\                   | | <-----/ |
    //   |  | message box   | <-----------------------------/
    //   |  \---------------/                   | |
    //   |                                      | |
    //   |  /----------\  /------\  /-------\   | |
    //   |  | backpack |  | heap |  | stack |   | |
    //   |  \----------/  \------/  \-------/   | |
    //   |                                      | |
    //   |  /-----------------\                 | |
    //   |  | SP, FP, PC      |                 | |         processor
    //   |  \-----------------/                 | |        /-------------\
    //   |                                      | | -----> | interpreter |
    //   |    module image                      | | <----- |             |
    //   |  /-----------------------\           | |        \-------------/
    //   |  | read-only data (ref)  |-\         | |
    //   |  | read-write data       | |         | |         a set of stateless
    //   |  | uninit. data          | |         | |         functions
    //   |  |-----------------------| |         | |
    //   |  | types (ref)           | |         | |
    //   |  | functions (ref)       | |         | |
    //   |  | local vars (ref)      | |         | |
    //   |  \-----------------------/ |         | |
    //   |    \----------------------/          | |
    //   |       module images                  | |
    //   |                                      | |
    //   \--------------------------------------/ |
    //     \--------------------------------------/
    //        threads
    //
    // note that the heap, stack, data sections, backpack and messagebox are all thread-local,
    // by default the XiaoXuan has no 'global' data or variables.
    // threads can comunicate through message box or shared-memory.

    thread_id,              // get the current thread id
    thread_create,          // craete a new thread, return the mailbox id
    thread_msg_send,        // (param mailbox_id:i32)
    thread_msg_reply,       // reply message to parent thread
    thread_exit,            //
                            // ref:
                            // - https://doc.rust-lang.org/std/sync/mpsc/index.html
                            // - https://doc.rust-lang.org/stable/rust-by-example/std_misc/channels.html
                            // - https://smallcultfollowing.com/babysteps/blog/2015/12/18/rayon-data-parallelism-in-rust/

    // backpack
    //
    // 'backpack' is a thread-local data hash map

    backpack_add,           // add a data item to backpack
                            // `fn (is_ref_type:i8, data:bytes)`
    backpack_get,           // `fn (bag_item_id:i32)`
    backpack_remove,        // `fn (bag_item_id:i32)`

    // regex

    regex_create,           // ref: https://github.com/rust-lang/regex
    regex_match,
    regex_test,
    regex_remove,

    // system

    syscall,                // syscall

    // `fn (syscall_num:i32, params_count: i32)` -> (return_value:i64, error_no:i32)
    //
    // the syscall arguments should be pushed on the stack first, e.g.
    //
    // | params_count   |
    // | syscall_num    |
    // | arg6           |
    // | arg5           |
    // | arg4           |
    // | arg3           |
    // | arg2           |                  | error no       |
    // | arg1           |     return -->   | return value   |
    // | ...            |                  | ...            |
    // \----------------/ <-- stack start  \----------------/
    //
    // when a syscall complete, the return value is store into the 'rax' register,
    // if the operation fails, the value is a negative value (rax < 0).
    // there is no 'errno' if invoke syscall by assembly directly.

    extcall,                // external function call
                            // `fn (external_func_index:i32)`

    // note that both 'scall' and 'extcall' are optional, they may be
    // unavailable in some environment.
    // the supported feature list can be obtained through the instruction 'ecall' with code 'features'.

    host_addr_func,         // create a new host function and map it to a VM function.
                            // this host function named 'bridge funcion'
                            //
                            // `fn (func_pub_index:i32)`

    // return the existing bridge function if the bridge function corresponding
    // to the specified VM function has already been created.


    // it's commonly used for creating a callback function pointer for external C function.
    //
    // note:
    // - a bridge function (host function) will be created when `create_host_function` is executed,
    //   as well as the specified VM function will be appended to the "host function bridge table" to
    //   prevent duplicate creation.
    // - a bridge function is refered to a (module idx, function idx) tuple.
    // - the bridge function is created via JIT codegen.
    // - when the external C function calls the bridge function, a new thread is created.
    //
    // when the XiaoXUan VM is embed into a C or Rust application as a library, the C or Rust application
    // can call the VM function through the bridge function as if it calls a native function.
    //
    // call bridge functon from Rust application example:
    //
    // ref:
    // https://doc.rust-lang.org/nomicon/ffi.html
    // https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html
    // https://doc.rust-lang.org/stable/reference/items/functions.html
    //
    // ```rust
    // fn main() {
    //     let func_ptr = ... (pointer of the bridge function)
    //     let func_addr = ... (virtual memory address of the bridge function)
    //
    //     /** mock pointer and address
    //     let func_ptr = cb_func as *const extern "C" fn(usize, usize);
    //     let func_ptr = cb_func as *const u8;
    //     let func_addr = func_ptr as usize;
    //     */
    //
    //     println!("{:p}", func_ptr);
    //     println!("0x{:x}", func_addr);
    //
    //     let func_from_ptr: fn(usize, usize) = unsafe { std::mem::transmute(func_ptr) };
    //     (func_from_ptr)(11, 13);
    //
    //     let ptr_from_addr = func_addr as *const ();
    //     let func_from_addr: fn(usize, usize) = unsafe { std::mem::transmute(ptr_from_addr) };
    //     (func_from_addr)(17, 19);
    // }
    //
    // #[no_mangle]
    // pub extern "C" fn cb_func(a1: usize, a2: usize) {
    //     println!("numbers: {},{}", a1, a2);
    // }
    // ```
    //
    // call bridge functon from C application example:
    //
    // ```c
    // int main(void)
    // {
    //     void *func_ptr = ...
    //     int (*func_from_ptr)(int, int) = (int (*)(int, int))func_ptr;
    //     printf("1+2=%d\n", (*func_from_ptr)(1, 2));
    //     exit(EXIT_SUCCESS);
    // }
    // ```

    //
    // I/O functions
    //

    print_char = 0x200,     // `fn (fd:i32 char:i32)`
    print_i32,              // `fn (fd:i32 value:i32)`
    print_i64,              // `fn (fd:i32 value:i64)`

    //
    // delegate to std I/O
    //
    write,                  // `fn (fd:i32 addr:i64 length:i64)`
}

pub const MAX_ECALLCODE_NUMBER:usize = 0x400;