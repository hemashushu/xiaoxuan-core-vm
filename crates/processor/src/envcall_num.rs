// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

#[repr(u32)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum EnvCallNum {
    // runtime
    //
    // get the VM runtime edition
    // `fn (buf_ptr: i64) -> len:i32`
    runtime_edition = 0x0100,

    // get the VM runtime version
    // `fn () -> version:i64`
    //
    // 0x0000_0000_0000_0000
    //        |    |    |
    //        |    |    |patch version
    //        |    |minor
    //        |major
    runtime_version,

    // get name list of the runtime features, separated by commas, e.g.
    // "syscall,extcall"
    //
    // `fn (buf_ptr: i64) -> feature_list_len:i32`
    //
    // runtime_features,

    // `fn (name_ptr:i64, name_len:i32) -> bool`
    //
    // runtime_feature_check,

    // host
    //
    host_arch,   // x86_64/aarch64/riscv64 ...
    host_os,     // linux/macos/windows/freebsd/android/ios ...
    host_family, // unix/windows ...
    host_endian, // little/big
    host_width,  // 32bit/64bit ... the size of data/pointer

    // ref:
    // https://doc.rust-lang.org/reference/conditional-compilation.html#target_arch

    // env
    //
    env_count,
    env_list,
    env_get,
    env_set,
    env_remove,

    // time
    //
    // get the current time (elapse from epoch)
    // `fn () -> (seconds:u64, nano_seconds:u64)`
    // valid values of 'nano_seconds' are [0, 999_999_999]
    time_now,

    // random
    random_init,
    random_int,

    // thread
    //
    // get the current thread id
    // it's '0' for the main thread, and '1' for the first child thread.
    //
    // 'fn () -> thread_id:u32'
    thread_id = 0x0200,

    // thread model
    // ------------
    //
    // the first start thread is called the 'main thread', a thread
    // can create one or more new threads, these threads are called 'child threads',
    // and the creator is called 'parent thread'.
    // when the parent thread exits, all its 'child threads' will also exit.
    //
    // a PIPE is created between the parent thread and child thread, they can
    // comunicate through 'thread_msg_recive/thread_msg_send' and
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
    //
    //     child thread                              pipe         parent thread
    //   /--------------------------------\      /-----------\    /----------\
    //   |                                |-\    |  receive  |    |          |
    //   |                           RX <---------------------------- TX     |
    //   |                           TX ----------------------------> RX     |
    //   |                                | |    |  send     |    |          |
    //   |                                | |    \-----------/    \----------/
    //   |  /-------------\  /-------\    | |
    //   |  | memory/heap |  | stack |    | |
    //   |  \-------------/  \-------/    | |
    //   |                                | |
    //   |  /-----------------\           | |
    //   |  | SP, FP, PC      |           | |
    //   |  \-----------------/           | |
    //   |                                | |
    //   |    module image                | |
    //   |  /-----------------------\     | |
    //   |  | read-write data       |-\   | |
    //   |  | uninit. data          | |   | |
    //   |  |-----------------------| |   | |
    //   |  | read-only data (ref)  | |   | |
    //   |  | types (ref)           | |   | |
    //   |  | functions (ref)       | |   | |
    //   |  | local variables (ref) | |   | |
    //   |  \-----------------------/ |   | |
    //   |    \----------------------/    | |
    //   |       module images            | |
    //   |                                | |
    //   \--------------------------------/ |
    //     \--------------------------------/
    //        child threads
    //
    // note that the memory/heap, stack, data sections are all thread-local,
    // by default the XiaoXuan has no 'global' data or variables.
    // threads can only comunicate through PIPE.
    //
    // in the XiaoXuan Core VM, all 'objects' are thread-safe.
    //
    // the message pipe
    // ----------------
    //
    // threads communicate through message pipe, the raw type of message is u8 array,
    // so the message can be:
    // - primitive data
    // - a struct
    // - an array
    // - (the address of) a function
    // - (the address of) a closure function

    // create a new thread and execute the specified function (called a 'thread start function')
    // in the current module.
    //
    // ```
    // fn (function_public_index:u32,
    //    thread_start_data_address_in_heap:u32,
    //    thread_start_data_length:u32) -> child_thread_id:u32
    // ```
    //
    // the value of 'thread_start_data_address' is the address of a data block in the memory, this
    // data is copied to the new thread (temporarily on the VM host thread-local memory).
    // the new thread can read the data using the function 'thread_start_data_read'.
    //
    // the signature of the 'thread start function' MUST be exactly:
    // 'fn () -> exit_code:u32'
    //
    // it's similar to the 'int main(int argc, char* argv[])'
    //
    // the meaning of the 'exit_code' is defined by the user,
    // but by general convention, the 'exit_code' of the 'entry' function is defined as:
    // - 0, thread exit on success
    // - 1, thread exit on failure
    thread_create,

    // get the length of the thread start data
    // 'fn () -> length:u32'
    thread_start_data_length,

    // read/copy the 'thread start data' from the host temporary memory to 'local variable'/data/'VM heap'
    // 'fn (offset:u32, length:u32, dst_memory_ptr:u64) -> (actual_read_length: u32)'
    //
    // results:
    // - actual_read_length: the length of data that actually read
    thread_start_data_read,

    // wait for the specified child thread to finish and collect resources from the child thread,
    // return the exit code of the 'thread start function'.
    //
    // 'fn (child_thread_id:u32) -> (thread_exit_code:u32, thread_result:u32)'
    //
    // returns:
    // - thread_exit_code: the meaning of the 'exit_code' is user defined.
    // - thread_result: 0=success, 1=failure (or thread_not_found)
    //
    // the caller will be blocked if the child thread is running, when the child thread finishes,
    // the 'thread_wait_and_collect' will get the (thread_exit_code, thread_result), and the child thread
    // will be removed from the 'child thread collection'.
    //
    // note that if the child thread is finished before the parent thread calls the
    // function 'thread_wait_and_collect', in this case the resource of the child thread
    // will NOT be released, and it will be store in the 'child thread collection' until
    // the parent thread calls 'thread_wait_and_collect'.
    //
    // in the other word, 'thread_wait_and_collect' is equivalent to the function 'thread.join()'
    // in the other programming language, it is used to wait for the child thread to stop or
    // to collect the resources of the child thread.
    thread_wait_and_collect,

    // check whether the specified (child) thread is finish
    // 'fn (child_thread_id:u32) -> (running_status:u32, thread_result:u32)'
    //
    // returns:
    // - running_status:  0=running, 1=finish
    // - thread_result: 0=success, 1=failure (thread_not_found)
    thread_running_status,

    // drop the specified (child) thread
    // 'fn (child_thread_id:u32) -> ()'
    thread_terminate,

    // receives a message from the parent thread,
    // it returns the length (in bytes) of the new message.
    //
    // 'fn () -> length:u32'
    //
    // this function will always block the current thread if there is no data available.
    //
    // when the pipe is closed, the child thread is also closed,
    // the child thread is automatically removed from the parent's 'child thread collection'.
    // so this function does not return 'thread_result', it simply ignores errors,
    // because the error means that the current thread will be terminated,
    // there is no point in dealing with errors anymore.
    thread_receive_msg_from_parent,

    // send message (from memory/heap) to the parent thread.
    // this method will never block the current thread.
    // 'fn (src_memory_ptr:u64, length:u32) -> thread_result:u32'
    //
    // returns:
    // - thread_result: 0=success, 1=failed.
    thread_send_msg_to_parent,

    // receive message from the specified (child) thread.
    // this function will always block the current thread if there is no data available.
    // the 'thread_result' will be '1=failure' when the PIPE (or the child thread) is close or
    // the specified child thread does not exist.
    // 'fn (child_thread_id:u32) -> (length:u32, thread_result:u32)'
    thread_receive_msg,

    // send message to the specified (child) thread.
    // this function will never block the current thread.
    // 'fn (child_thread_id:u32, src_memory_ptr:u64, length:u32) -> thread_result:u32'
    //
    // returns:
    // - thread_result: 0=success, 1=failure (thread_not_found).
    thread_send_msg,

    // get the length of the last received message
    // 'fn () -> length:u32'
    //
    thread_msg_length,

    // read/copy the last received message from the host temporary to 'local variable' (data or 'VM memory/heap')
    // 'fn (offset:u32, length:u32, dst_memory_ptr:u64) -> (actual_read_length:u32)'
    //
    // returns:
    // - actual_read_length: the actual length of the read data
    thread_msg_read,

    // block the current for the specified milliseconds.
    // 'fn (milliseconds:u64)
    thread_sleep,

    // ref:
    // - https://doc.rust-lang.org/std/sync/mpsc/index.html
    // - https://doc.rust-lang.org/stable/rust-by-example/std_misc/channels.html
    // - https://smallcultfollowing.com/babysteps/blog/2015/12/18/rayon-data-parallelism-in-rust/

    // regex
    //
    // ref: https://github.com/rust-lang/regex
    regex_create,
    regex_match,
    regex_test,
    regex_remove,
}

pub const MAX_ENVCALL_CODE_NUMBER: usize = 0x0300;
