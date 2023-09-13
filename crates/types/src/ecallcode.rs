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
                            // (buf_ptr: u64) -> name_len:u32
                            //
    runtime_version,        // get the VM runtime version
                            // () -> version:u64
                            // 0x0000_0000_0000_0000
                            //        |    |    |
                            //        |    |    |patch version
                            //        |    |minor
                            //        |major
                            //
    runtime_features,       // get a list of feature names separated by commas, e.g.
                            // "scall,ccall,shared_memory"
                            // (buf_ptr: u64) -> feature_list_len:u32

    // heap memory

    heap_fill,          // fill the specified memory region with specified value    (param start_addr:i64, count:i64, value:i8)
    heap_copy,          // copy the specified memory region to specified address    (param src_addr:i64, dst_addr:i64, length:i64)
    heap_size,          // return the amount of the thread-local
                        // memory (i.e. heap) pages, each page is 32 KiB by default
    heap_grow,          // increase the heap size                                   (param pages:i64)
                        // return the new size (in pages)

    // thread

    // the XiaoXuan VM thread model:
    //
    //                            thread
    //                          /--------------------------------------\
    //                          |                                      |-\
    //                          |  /---------------\                   | |  msg_send
    //                          |  | child threads |-----------------------------\
    //                          |  \---------------/  /-----------\    | |       |
    //                          |                     | parent th |----| |-------|-\
    //                          |                     \-----------/    | |       | |
    //                          |                                      | |       | |
    //                          |  /---------------\                   | | <-----/ |
    //                          |  | message box   | <---------------------------/
    //    (UNDECIDED)           |  \---------------/                   | |
    // /---------------\  load  |                                      | |
    // |               | -----> |  /----------\  /------\  /-------\   | |
    // | shared memory |        |  | backpack |  | heap |  | stack |   | |
    // |               | <----- |  \----------/  \------/  \-------/   | |
    // \---------------/ store  |                                      | |
    //                          |  /-----------------\     status      | |
    //   module                 |  | read-write data |-\   /----\      | |
    // /----------------\       |  | uninit. data    | |   | SP |      | |
    // | read-only data |-\     |  \-----------------/ |   | FP |      | |
    // |----------------| |     |    \-----------------/   | PC |      | |
    // | types          | |     |                          \----/      | |
    // | functions      | |     |  /-------------\                     | |
    // \----------------/ |--------| module ref. |                     | |
    //   \----------------/     |  \-------------/                     | |
    //       modules            |                                      | |
    //                          \--------------------------------------/ |
    //                            \--------------------------------------/
    //                                           threads
    //
    // note that the heap, stack, data sections, backpack and messagebox are all thread-local,
    // by default the XiaoXuan has no 'global' data or variables.
    // threads can comunicate through message box or shared-memory.

    thread_id,          // get the current thread id
    thread_create,      // craete a new thread, return the mailbox id
    thread_msg_send,    // (param mailbox_id:i32)
    thread_msg_reply,   // reply message to parent thread
    thread_exit,        //
                        // ref:
                        // - https://doc.rust-lang.org/std/sync/mpsc/index.html
                        // - https://doc.rust-lang.org/stable/rust-by-example/std_misc/channels.html
                        // - https://smallcultfollowing.com/babysteps/blog/2015/12/18/rayon-data-parallelism-in-rust/

    // backpack
    //
    // 'backpack' is a thread-local data hash map

    backpack_add,            // (param is_ref_type:i8, data:bytes)   add a data item to the backpack
    backpack_get,            // (param bag_item_id:i32)
    backpack_remove,         // (param bag_item_id:i32)

    // foreign function call

    create_host_function,
                        // create a new host function and map it to a module function.
                        // (param module_index:i32 func_index:i32)
                        //
                        // it's commonly used for creating a callback function pointer for external C function.
                        //
                        // note:
                        // - a host function will be created when `create_host_function` is executed, as well as
                        //   the specified VM function will be appended to the "function pointer table" to
                        //   prevent duplicate creation.
                        // - the new created function is refered to a (thread id, module idx, function idx) tuple.

    // regex

    regex_create,       // ref: https://github.com/rust-lang/regex
    regex_match,
    regex_test,
    regex_remove,

    //
    // I/O functions
    //

    write_char = 0x200, // (param fd:i32 char:i32)
    write_bytes,        // (param fd:i32 addr:i64 length:i64)
    write_i32,          // (param fd:i32 value:i32)
    write_i64,          // (param fd:i32 value:i64)
}

pub const MAX_ECALLCODE_NUMBER:usize = 0x300;