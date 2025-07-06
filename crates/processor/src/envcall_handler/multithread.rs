// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::{thread, time::Duration};

use anc_context::thread_context::ThreadContext;

use crate::multithread_handler::{
    create_thread, ThreadStartFunction, CHILD_THREADS, CURRENT_THREAD_ID, LAST_THREAD_MESSAGE, RX,
    THREAD_START_DATA, TX,
};

pub const THREAD_RUNNING_STATUS_RUNNING: u32 = 0;
pub const THREAD_RUNNING_STATUS_FINISH: u32 = 1;
pub const THREAD_ERROR_NUMBER_SUCCESS: u32 = 0;
pub const THREAD_ERROR_NUMBER_NOT_FOUND: u32 = 1;

pub fn thread_id(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // `fn () -> i32`
    CURRENT_THREAD_ID.with(|id_cell| {
        let id = *id_cell.borrow();
        thread_context.stack.push_i32_u(id);
    });
}

pub fn thread_create(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // ```
    // fn (function_public_index: i32,
    //     thread_start_data_access_index: i64,
    //     thread_start_data_length: i64) -> i32
    // ```

    // get arguments
    let thread_start_data_length = thread_context.stack.pop_i64_u() as usize;
    let thread_start_data_access_index = thread_context.stack.pop_i64_u() as usize;
    let function_public_index = thread_context.stack.pop_i32_u() as usize;

    // get the current module index
    let module_index = thread_context.pc.module_index;

    // get thread start data
    let thread_start_data_object = thread_context.get_target_data_object(
        module_index,
        thread_start_data_access_index,
        0,
        thread_start_data_length,
    );

    let mut thread_start_data = vec![0_u8; thread_start_data_length];
    thread_start_data_object.accessor.read_idx(
        thread_start_data_access_index,
        0,
        thread_start_data_length,
        thread_start_data.as_mut_ptr(),
    );

    let thread_start_function = ThreadStartFunction {
        module_index,
        function_public_index,
    };

    let child_thread_id = create_thread(thread_start_function, thread_start_data);

    thread_context.stack.push_i32_u(child_thread_id);
}

pub fn thread_start_data_length(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // `fn () -> i64`

    let data_length = THREAD_START_DATA.with(|data_cell| {
        let data = data_cell.borrow();
        data.len()
    });

    thread_context.stack.push_i64_u(data_length as u64);
}

pub fn thread_start_data_read(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // ```
    // fn (module_index: i32,
    //     data_access_index: i64,
    //     offset_of_thread_start_data: i64,
    //     expected_length_in_bytes: i64) -> i64
    // ```

    let expected_length_in_bytes = thread_context.stack.pop_i64_u() as usize;
    let offset_of_thread_start_data = thread_context.stack.pop_i64_u() as usize;
    let data_access_index = thread_context.stack.pop_i64_u() as usize;
    let module_index = thread_context.stack.pop_i32_u() as usize;

    let actual_read_length = THREAD_START_DATA.with(|data_cell| {
        let data = data_cell.borrow();
        let data_length = data.len();

        if offset_of_thread_start_data >= data_length {
            // Offset of thread start data is out of bounds.
            0
        } else {
            let available_length_in_bytes =
                if offset_of_thread_start_data + expected_length_in_bytes > data_length {
                    data_length - offset_of_thread_start_data
                } else {
                    expected_length_in_bytes
                };

            let target_data_object = thread_context.get_target_data_object(
                module_index,
                data_access_index,
                0,
                available_length_in_bytes,
            );

            let src_ptr = data[offset_of_thread_start_data..].as_ptr();
            target_data_object.accessor.write_idx(
                src_ptr,
                data_access_index,
                0,
                available_length_in_bytes,
            );
            available_length_in_bytes
        }
    });

    thread_context.stack.push_i64_u(actual_read_length as u64);
}

pub fn thread_wait_and_collect(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // `fn (child_thread_id: i32) -> (thread_exit_code: i32, thread_error_number: i32)`
    //
    // Returns:
    // - thread_exit_code: The value returned by the "thread start function."
    // - thread_error_number: 0 for success, 1 for thread not found.

    let child_thread_id = thread_context.stack.pop_i32_u();

    let (thread_exit_code, thread_error_number) = CHILD_THREADS.with(|child_threads_cell| {
        let mut child_threads = child_threads_cell.borrow_mut();

        // remove the child thread object from 'child thread collection'
        let opt_child_thread = child_threads.remove(&child_thread_id);

        match opt_child_thread {
            Some(child_thread) => {
                let result = child_thread.join_handle.join().unwrap();
                match result {
                    Ok(thread_exit_code) => (thread_exit_code, THREAD_ERROR_NUMBER_SUCCESS),
                    // there is no way to return the details of ProcessorError in the
                    // child thread, so only the panic can be thrown.
                    Err(e) => panic!("Child thread panic: {}", e),
                }
            }
            None => (0, THREAD_ERROR_NUMBER_NOT_FOUND), // thread not found
        }
    });

    thread_context.stack.push_i32_u(thread_exit_code);
    thread_context.stack.push_i32_u(thread_error_number);
}

pub fn thread_running_status(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // `fn (child_thread_id: i32) -> (running_status: i32, thread_error_number: i32)`
    //
    // Returns:
    // - running_status: 0 = running, 1 = finished
    // - thread_error_number: 0 for success, 1 for thread not found.

    let child_thread_id = thread_context.stack.pop_i32_u();

    let (thread_running_status, thread_error_number) = CHILD_THREADS.with(|child_threads_cell| {
        let child_threads = child_threads_cell.borrow_mut();

        let opt_child_thread = child_threads.get(&child_thread_id);

        match opt_child_thread {
            Some(child_thread) => {
                let thread_running_status = if child_thread.join_handle.is_finished() {
                    THREAD_RUNNING_STATUS_FINISH
                } else {
                    THREAD_RUNNING_STATUS_RUNNING
                };
                (thread_running_status, THREAD_ERROR_NUMBER_SUCCESS)
            }
            None => (0, THREAD_ERROR_NUMBER_NOT_FOUND), // thread not found
        }
    });

    thread_context.stack.push_i32_u(thread_running_status);
    thread_context.stack.push_i32_u(thread_error_number);
}

pub fn thread_terminate(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // `fn (child_thread_id: i32) -> ()`
    //
    // Note: Dropping the sender (TX) will cause the receiver (RX) in the child thread to stop.
    // See Rust's std::sync::mpsc documentation for details.
    //
    // Example:
    // if let Some(child_thread) = opt_child_thread {
    //     drop(child_thread.tx)
    // }

    let child_thread_id = thread_context.stack.pop_i32_u();

    CHILD_THREADS.with(|child_threads_cell| {
        let mut child_threads = child_threads_cell.borrow_mut();

        // remove the child thread object from 'child thread collection'
        let _ = child_threads.remove(&child_thread_id);
    });
}

pub fn thread_send_msg(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // `fn (child_thread_id: i32, module_index: i32, data_access_index: i64, content_length_in_bytes: i64) -> thread_error_number: i32`
    //
    // Returns 0 for success, 1 for failure (if the child thread has finished or does not exist).
    // This function is non-blocking and returns immediately.

    let content_length_in_bytes = thread_context.stack.pop_i64_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();
    let child_thread_id = thread_context.stack.pop_i32_u();

    CHILD_THREADS.with(|child_threads_cell| {
        let child_threads = child_threads_cell.borrow_mut();
        let child_thread_opt = child_threads.get(&child_thread_id);

        let thread_error_number = match child_thread_opt {
            Some(child_thread) => {
                let target_data_object = thread_context.get_target_data_object(
                    module_index as usize,
                    data_access_index as usize,
                    0,
                    content_length_in_bytes as usize,
                );

                let mut data_send = vec![0_u8; content_length_in_bytes as usize];
                target_data_object.accessor.read_idx(
                    data_access_index as usize,
                    0,
                    content_length_in_bytes as usize,
                    data_send.as_mut_ptr(),
                );

                match child_thread.tx.send(data_send) {
                    Ok(_) => THREAD_ERROR_NUMBER_SUCCESS,
                    Err(_) => THREAD_ERROR_NUMBER_NOT_FOUND,
                }
            }
            None => THREAD_ERROR_NUMBER_NOT_FOUND,
        };

        // push 'thread_error_number' to stack
        thread_context.stack.push_i32_u(thread_error_number);
    });
}

pub fn thread_send_msg_to_parent(/* _handler: &Handler, */ thread_context: &mut ThreadContext,) {
    // `fn (module_index: i32, data_access_index: i64, content_length_in_bytes: i64) -> ()`
    //
    // This function is non-blocking and returns immediately.

    let content_length_in_bytes = thread_context.stack.pop_i64_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();

    TX.with(|tx_refcell| {
        let tx_ref = tx_refcell.borrow();
        let tx_opt = tx_ref.as_ref();
        match tx_opt {
            Some(tx) => {
                let target_data_object = thread_context.get_target_data_object(
                    module_index as usize,
                    data_access_index as usize,
                    0,
                    content_length_in_bytes as usize,
                );
                let mut data_send = vec![0_u8; content_length_in_bytes as usize];
                target_data_object.accessor.read_idx(
                    data_access_index as usize,
                    0,
                    content_length_in_bytes as usize,
                    data_send.as_mut_ptr(),
                );

                tx.send(data_send)
                    .expect("Send message to parent thread failed.");
            }
            None => {
                unreachable!("TX of channel to parent thread is not set.")
            }
        };
    });
}

pub fn thread_receive_msg(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // `fn (child_thread_id: i32) -> (length: i64, thread_error_number: i32)`
    //
    // Returns:
    // - length: The length of the received message in bytes.
    // - thread_error_number: 0 for success, 1 for failure (if the child thread has finished or does not exist).

    let child_thread_id = thread_context.stack.pop_i32_u();

    CHILD_THREADS.with(|child_threads_cell| {
        let child_threads = child_threads_cell.borrow_mut();
        let child_thread_opt = child_threads.get(&child_thread_id);

        let (length, thread_error_number) = match child_thread_opt {
            Some(child_thread) => {
                match child_thread.rx.recv() {
                    Ok(data) => {
                        // store the received data
                        LAST_THREAD_MESSAGE.with(|msg_refcell| {
                            let length = data.len();
                            msg_refcell.replace(data);
                            (length, THREAD_ERROR_NUMBER_SUCCESS)
                        })
                    }
                    Err(_) => (0, THREAD_ERROR_NUMBER_NOT_FOUND), // the PIPE may have been closed
                }
            }
            None => (0, THREAD_ERROR_NUMBER_NOT_FOUND),
        };

        // push 'length' and 'thread_error_number' to stack
        thread_context.stack.push_i64_u(length as u64);
        thread_context.stack.push_i32_u(thread_error_number);
    });
}

pub fn thread_receive_msg_from_parent(
    /* _handler: &Handler, */ thread_context: &mut ThreadContext,
) {
    // `fn () -> i64`
    //
    // Returns the length (in bytes) of the new message.

    RX.with(|rx_refcell| {
        let rx_ref = rx_refcell.borrow();
        let rx_opt = rx_ref.as_ref();
        match rx_opt {
            Some(rx) => {
                match rx.recv() {
                    Ok(data) => {
                        // store the received data
                        LAST_THREAD_MESSAGE.with(|msg_refcell| {
                            let length = data.len();
                            msg_refcell.replace(data);

                            // push 'length' to stack
                            thread_context.stack.push_i64_u(length as u64);
                        });
                    }
                    Err(_) => {
                        // ignore errors when receiving messages from the
                        // parent thread, because it means that the current thread
                        // is being terminated, there is no longer any sense in
                        // dealing with errors.
                    }
                };
            }
            None => {
                unreachable!("RX of channel to parent is not set.")
            }
        };
    });
}

pub fn thread_msg_length(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // `fn () -> i64`

    LAST_THREAD_MESSAGE.with(|msg_refcell| {
        let msg_ref = msg_refcell.borrow();
        let length = msg_ref.len();

        // push 'length' to stack
        thread_context.stack.push_i64_u(length as u64);
    });
}

pub fn thread_msg_read(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // `fn (module_index: i32, data_access_index: i64, offset_of_message: i64, expected_size_in_bytes: i64) -> i64`

    let expected_size_in_bytes = thread_context.stack.pop_i64_u() as usize;
    let offset_of_message = thread_context.stack.pop_i64_u() as usize;
    let data_access_index = thread_context.stack.pop_i64_u() as usize;
    let module_index = thread_context.stack.pop_i32_u() as usize;

    let available_length_in_bytes = LAST_THREAD_MESSAGE.with(|msg_refcell| {
        let msg_ref = msg_refcell.borrow();
        let msg_length = msg_ref.len();

        if offset_of_message >= msg_length {
            // Offset of message is out of bounds.
            0
        } else {
            let available_length_in_bytes =
                if offset_of_message + expected_size_in_bytes > msg_length {
                    msg_length - offset_of_message
                } else {
                    expected_size_in_bytes
                };

            let target_data_object = thread_context.get_target_data_object(
                module_index,
                data_access_index,
                0,
                available_length_in_bytes,
            );

            let src_ptr = msg_ref[offset_of_message..].as_ptr();
            target_data_object.accessor.write_idx(
                src_ptr,
                data_access_index,
                0,
                available_length_in_bytes,
            );

            available_length_in_bytes
        }
    });

    // push 'length' to stack
    thread_context
        .stack
        .push_i64_u(available_length_in_bytes as u64);
}

// ref:
// https://linux.die.net/man/2/nanosleep
pub fn thread_sleep(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // Blocks the current thread for the specified number of milliseconds.
    //
    // Signature: `fn (milliseconds: i64) -> ()`

    let milliseconds = thread_context.stack.pop_i64_u();
    thread::sleep(Duration::from_millis(milliseconds));
}

// Note:
// Unit tests for multithread functions are complex to write directly in bytecode.
// These tests are implemented in the 'xiaoxuan-core-assembly' project.
