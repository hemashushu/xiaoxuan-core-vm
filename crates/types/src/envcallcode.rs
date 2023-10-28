// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

#[repr(u32)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum EnvCallCode {
    //
    // runtime functions
    //

    // runtime info
    runtime_name = 0x100,       // get the VM runtime code name
                                // `fn (buf_ptr: i64) -> name_len:i32`

    runtime_version,            // get the VM runtime version
                                // `fn () -> version:i64`
                                //
                                // 0x0000_0000_0000_0000
                                //        |    |    |
                                //        |    |    |patch version
                                //        |    |minor
                                //        |major

    runtime_features,           // get name list of the runtime features, separated by commas, e.g.
                                // "syscall,extcall"
                                //
                                // `fn (buf_ptr: i64) -> feature_list_len:i32`

    runtime_feature_check,      // `fn (name_ptr:i64, name_len:i32) -> bool`

    // host info
    host_arch,                  // x86_64/aarch64/riscv64 ...
    host_os,                    // linux/macos/windows/freebsd/android/ios ...
    host_family,                // unix/windows ...
    host_endian,                // little/big
    host_pointer_width,         // 32/64 ...

    // ref:
    // https://doc.rust-lang.org/reference/conditional-compilation.html#target_arch

    // thread

    // the XiaoXuan VM thread model:
    //
    //     thread                                             pipe              upstrem thread
    //   /--------------------------------------\          /-----------\      /----------\
    //   |                                      |-\        |  receive  |      |          |
    //   |                                 RX <---------------------------------- TX     |
    //   |                                 TX ----------------------------------> RX     |
    //   |                                      | |        |  send     |      |          |
    //   |                                      | |        \-----------/      \----------/
    //   |  /----------\  /------\  /-------\   | |
    //   |  | backpack |  | heap |  | stack |   | |
    //   |  \----------/  \------/  \-------/   | |
    //   |                                      | |
    //   |  /-----------------\                 | |
    //   |  | SP, FP, PC      |                 | |          runtime
    //   |  \-----------------/                 | |        /-------------\
    //   |                                      | | -----> | interpreter |
    //   |    module image                      | | <----- |             |
    //   |  /-----------------------\           | |        \-------------/
    //   |  | read-write data       |-\         | |
    //   |  | uninit. data          | |         | |         a set of stateless
    //   |  |-----------------------| |         | |         functions
    //   |  | read-only data (ref)  | |         | |
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
    // threads can only comunicate through PIPE.
    //
    // in the XiaoXuan VM, all 'objects' are thread-safe.

    // about the message pipe:
    //
    // threads communicate through message pipe, the raw type of message is u8 array, so it can be:
    // - primitive data
    // - a struct
    // - an array
    // - (the address of) a function
    // - (the address of) a closure function
    thread_id,                  // get the current thread id
                                // 'fn () -> thread_id:u32'

    thread_create,              // craete a new thread and run the specified function.
                                //
                                // '``
                                // fn (module_index:u32, func_public_index:u32,
                                //    thread_start_data_address:u32, thread_start_data_length:u32) -> child_thread_id:u32
                                // ```
                                //
                                // the value of 'thread_start_data_address' is the address of a data block in the heap
                                //
                                // the signature of the 'thread start function' MUST be:
                                // 'fn (thread_start_data_length:u32) -> result_code:u32'

    thread_wait_for_finish,     // wait for the specified (child) thread to finish, return the results of the starting function
                                // 'fn (child_thread_id:u32) -> (wait_status:u32, thread_exit_code:u32)'
                                // - wait_status: 0=success, 1=not_found
                                // - thread_exit_code: 0=thread exit with success, 1=thread exit with failure
                                //
                                // when the child thread finish, it will be removed from
                                // the 'child thread collection' automatically.

    thread_running_status,      // check whether the specified (child) thread is finish
                                // 'fn (child_thread_id:u32) -> running_status:u32'
                                // - running_status:  0=running, 1=finish, 2=not_found

    thread_exit,                // drop the specified (child) thread
                                // 'fn (child_thread_id:u32)'

    thread_msg_recive,          // receive message from the upstream (parent) thread
                                // 'fn ()->u64'
                                //
                                // receiving new message from the upstream (parent) thread, the current thread
                                // will block until a new message arrives or the pipe is closed.
                                //
                                // the length (in bytes) of new message is return.
                                //
                                // when the pipe is closed, the child thread will be terminated as well,
                                // the child thread will be removed from the parent's 'child thread collection'
                                // automatically.

    thread_msg_send,            // send message (from heap) to the upstream (parent) thread
                                // 'fn (src_address:u64, length:u32) -> result:u32'
                                //
                                // result: 0=success, 1=failed.

    thread_msg_receive_from,    // receive message from the specified (child) thread
    thread_msg_send_to,         // send message to the specified (child) thread

    thread_start_data_read,     // read/copy the thread start data to heap
                                // 'fn (offset:u32, length:u32, dst_address:u64) -> result:u32'
                                //
                                // result: 0=success, 1=failed.

    thread_msg_read,            // read/copy the last received message to heap
                                // 'fn (offset:u32, length:u32, dst_address:u64) -> result:u32'
                                //
                                // result: 0=success, 1=failed.

    // ref:
    // - https://doc.rust-lang.org/std/sync/mpsc/index.html
    // - https://doc.rust-lang.org/stable/rust-by-example/std_misc/channels.html
    // - https://smallcultfollowing.com/babysteps/blog/2015/12/18/rayon-data-parallelism-in-rust/

    // regex
    regex_create,               // ref: https://github.com/rust-lang/regex
    regex_match,
    regex_test,
    regex_remove,

    //
    // I/O functions
    //

    /*
    print_char = 0x200,     // `fn (fd:i32 char:i32)`
    print_i32,              // `fn (fd:i32 value:i32)`
    print_i64,              // `fn (fd:i32 value:i64)`
    */

    //
    // delegate to std I/O
    //

    /*
    write,                  // `fn (fd:i32 addr:i64 length:i64)`
    */
}

pub const MAX_ECALLCODE_NUMBER: usize = 0x400;
