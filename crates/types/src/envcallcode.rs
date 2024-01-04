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

    // todo
    // runtime_features,           // get name list of the runtime features, separated by commas, e.g.
                                // "syscall,extcall"
                                //
                                // `fn (buf_ptr: i64) -> feature_list_len:i32`

    // runtime_feature_check,      // `fn (name_ptr:i64, name_len:i32) -> bool`

    // init and fini functions
    count_start_function = 0x110,   // get the amount of the constructor functions
    count_exit_function,            // get the amount of the destructor functions
    get_start_function_item,        // get 'the public index of constructor function ' by index
    get_exit_function_item,         // get 'the public index of destructor function ' by index

    // host info
    // todo

    // host_info_arch = 0x110,      // x86_64/aarch64/riscv64 ...
    // host_info_os,                // linux/macos/windows/freebsd/android/ios ...
    // host_info_family,            // unix/windows ...
    // host_info_endian,            // little/big
    // host_info_pointer_width,     // 32/64 ...

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
    thread_id = 0x120,          // get the current thread id
                                // 'fn () -> thread_id:u32'

    thread_create,              // craete a new thread and run the specified function (named 'thread start function')
                                // in the current module.
                                //
                                // ```
                                // fn (function_public_index:u32,
                                //    thread_start_data_address_in_heap:u32,
                                //    thread_start_data_length:u32) -> child_thread_id:u32
                                // ```
                                //
                                // the value of 'thread_start_data_address' is the address of a data block in the heap, these
                                // data will be copied to the new thread (temporary on the VM host thread-local memory).
                                // the new thread can read the data through the function 'thread_start_data_read'.
                                //
                                // the signature of the 'thread start function' MUST exactly be:
                                // 'fn () -> exit_code:u64'
                                //
                                // it's similar to the 'int main(int argc, char* argv[])'
                                //
                                // the meaning of the 'exit_code' is defined by user,
                                // but in general convention, the 'exit_code' of the 'entry' function is defined as
                                // 0=thread exit with success, 1=thread exit with failure

    thread_start_data_length,   // get the length of the thread start data
                                // 'fn () -> length:u32'

    thread_start_data_read,     // read/copy the 'thread start data' from the host temporary memory to 'local variable'/data/'VM heap'
                                // 'fn (offset:u32, length:u32, dst_memory_ptr:u64) -> (actual_read_length: u32)'
                                //
                                // results:
                                // - actual_read_length: the length of data that actually read

    thread_wait_and_collect,    // wait for the specified (child) thread to finish and collect resources of child thread,
                                // return the exit code of the 'thread start function'.
                                //
                                // 'fn (child_thread_id:u32) -> (thread_exit_code:u64/u32, thread_result:u32)'
                                //
                                // returns:
                                // - thread_exit_code: the meaning of the 'exit_code' is defined by user, the data type
                                //   of 'thread_exit_code' is platform dependent, e.g. u32 on 32-bit platforms,
                                //   u64 on 64-bit platforms.
                                // - thread_result: 0=success, 1=failure (or thread_not_found)
                                //
                                // the caller will be blocked if the child thread is running, when the child thread finish,
                                // the 'thread_wait_and_collect' gets the (thread_exit_code, thread_result), and the child thread
                                // will be removed from the 'child thread collection'.
                                //
                                // note that if the child thread is finish before the parent thread calls the
                                // function 'thread_wait_and_collect', in which case the resource of child thread
                                // will NOT be released, and it is store in the 'child thread collection' until
                                // the parent thread call 'thread_wait_and_collect'.
                                //
                                // in the other word, 'thread_wait_and_collect' is equivaent to the function 'thread.join()'
                                // in the other programming language, it is used to wait for child thread to stop or
                                // to collect the resources of the child thread.

    thread_running_status,      // check whether the specified (child) thread is finish
                                // 'fn (child_thread_id:u32) -> (running_status:u32, thread_result:u32)'
                                //
                                // returns:
                                // - running_status:  0=running, 1=finish
                                // - thread_result: 0=success, 1=failure (thread_not_found)

    thread_terminate,           // drop the specified (child) thread
                                // 'fn (child_thread_id:u32) -> ()'

    // receive message from the upstream (parent) thread,
    // the length (in bytes) of new message is return.
    // 'fn () -> length:u32'
    //
    // this function will always block the current thread if there is no data available.
    //
    // when the pipe is closed, the child thread will be terminated as well,
    // the child thread will be removed from the parent's 'child thread collection'
    // automatically.
    //
    // so this function does not return 'thread_result' but just ignore errors,
    // because the error means that the current thread
    // is being terminated, there is no longer any sense in
    // dealing with errors.
    thread_receive_msg_from_parent,

    // send message (from heap) to the upstream (parent) thread.
    // this method will never block the current thread.
    // 'fn (src_memory_ptr:u64, length:u32) -> thread_result:u32'
    //
    // returns:
    // - thread_result: 0=success, 1=failed.
    thread_send_msg_to_parent,

    // the first start thread called the 'main thread', a thread
    // can create one or more new threads, these threads called 'child thread's,
    // and the creator called 'parent thread'.
    // when the parent thread exit, all its 'child thread's will exit also.
    //
    // a PIPE is created between the parent thread and child thread, they can
    // comunicates through 'thread_msg_recive/thread_msg_send' and
    // 'thread_msg_receive_from/thread_msg_send_to'. however, there is no direct PIPE
    // between child threads.
    //
    // the example of the thread tree:
    //
    // main thread
    //   |-- child thread 0
    //   |     |-- child thread 0-0
    //   |     |-- child thread 0-1
    //   |     |     |-- child thread 0-1-0
    //   |     |
    //   |     |-- child thread 0-2
    //   |
    //   |-- child thread 1
    //   |

    thread_receive_msg,         // receive message from the specified (child) thread.
                                // this function will always block the current thread if there is no data available.
                                // the 'thread_result' will be '1=failure' when the PIPE (or the child thread) is close or
                                // the specified child thread does not exist.
                                // 'fn (child_thread_id:u32) -> (length:u32, thread_result:u32)'

    thread_send_msg,            // send message to the specified (child) thread.
                                // this function will never block the current thread.
                                // 'fn (child_thread_id:u32, src_memory_ptr:u64, length:u32) -> thread_result:u32'
                                //
                                // returns:
                                // - thread_result: 0=success, 1=failure (thread_not_found).

    thread_msg_length,          // get the length of the last received message
                                // 'fn () -> length:u32'
                                //
    thread_msg_read,            // read/copy the last received message from the host temporary to 'local variable'/data/'VM heap'
                                // 'fn (offset:u32, length:u32, dst_memory_ptr:u64) -> (actual_read_length:u32)'
                                //
                                // returns:
                                // - actual_read_length: the actual length of the read data

    thread_sleep,               // block the current for the specified milliseconds.
                                // 'fn (milliseconds:u64)

    // ref:
    // - https://doc.rust-lang.org/std/sync/mpsc/index.html
    // - https://doc.rust-lang.org/stable/rust-by-example/std_misc/channels.html
    // - https://smallcultfollowing.com/babysteps/blog/2015/12/18/rayon-data-parallelism-in-rust/

    // time
    time_now = 0x130,           // get the current time
                                // `fn () -> (seconds:u64, nano_seconds:u32)`
                                // valid values of 'nano_seconds' are [0, 999_999_999]

    // regex
    regex_create = 0x140,       // ref: https://github.com/rust-lang/regex
    regex_match,
    regex_test,
    regex_remove,
}

pub const MAX_ECALLCODE_NUMBER: usize = 0x400;
