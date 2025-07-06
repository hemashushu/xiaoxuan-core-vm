// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::{
    cell::RefCell,
    collections::BTreeMap,
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
};

use anc_context::process_context::ProcessContext;
use anc_isa::ForeignValue;

use crate::{process::process_function, GenericError, ProcessorError, ProcessorErrorType};

thread_local! {
    // PROCESS_CONTEXT_ADDRESS should be process-global, but unit tests run in parallel
    // and may overwrite these values in each thread. Using thread-local storage avoids conflicts.

    // Pointer to the process context object.
    pub static PROCESS_CONTEXT_ADDRESS: RefCell<usize> = const{ RefCell::new(0) };
    // pub static HANDLER_ADDRESS: RefCell<usize> =  const{  RefCell::new(0) };

    // Collection of child threads for the current thread.
    pub static CHILD_THREADS:RefCell<BTreeMap<u32, ChildThread>> = const {RefCell::new(BTreeMap::new())};

    // Monotonically increasing integer used to generate unique child thread IDs.
    pub static CHILD_THREAD_NEXT_ID:RefCell<u32> = const {RefCell::new(0)};

    // The current thread's ID.
    pub static CURRENT_THREAD_ID:RefCell<u32> = const {RefCell::new(0)};

    // Message passing (Tx and Rx)
    // ---------------------------
    // Threads communicate via message pipes. The raw message type is a u8 array, so messages can be:
    // - Primitive data
    // - Structs
    // - Arrays
    // - (The address of) a function
    // - (The address of) a closure function

    // Receiver for messages from the parent thread.
    // Note: The "parent thread" is the creator of the current thread.
    pub static RX:RefCell<Option<Receiver<Vec<u8>>>> = const { RefCell::new(None) };

    // Sender for messages to the parent thread.
    pub static TX:RefCell<Option<Sender<Vec<u8>>>> = const { RefCell::new(None) };

    // Data (u8 array) sent from the parent thread at thread start.
    pub static THREAD_START_DATA:RefCell<Vec<u8>> = const {RefCell::new(vec![])};

    // Temporary buffer ("letter paper") for the last received message.
    //
    // This buffer is replaced with a new message each time
    // `thread_receive_msg` or `thread_receive_msg_from_parent` is called
    // and the message box is not empty. To read the message, call `thread_msg_read`.
    //
    // ```diagram
    //
    //        parent thread or child thread      current thread
    //        |                                  |
    //        |                                  |
    //        |                             /---------\
    // 1. `thread_send_msg()` ------------> | message |
    //                                      | box     |
    //                                      \---------/
    //                                          |
    //                                          | 2. `thread_receive_msg_from_parent()`
    //                                          |    or `thread_receive_msg()`
    //                                          |    (blocks if message box is empty)
    //                                          v
    //                                     /--------\
    //                                     | letter |
    //                                     | paper  |
    //                                     \--------/
    //                                          |
    //                                          | 3. `thread_msg_read()`
    //                                          v
    //                                     memory or data
    // ```
    pub static LAST_THREAD_MESSAGE:RefCell<Vec<u8>> = const {RefCell::new(vec![])};
}

pub struct ChildThread {
    // Join handle for the child thread. Returns the thread's exit code (u32) or an error.
    pub join_handle: JoinHandle<Result<u32, GenericError>>,

    // Receiver for messages sent from the child thread to the parent.
    pub rx: Receiver<Vec<u8>>,

    // Sender for messages sent from the parent to the child thread.
    pub tx: Sender<Vec<u8>>,
}

#[derive(Debug, Clone, Copy)]
pub struct ThreadStartFunction {
    pub module_index: usize,
    pub function_public_index: usize,
}

/// The signature of the "thread start function" must be:
/// `fn () -> exit_code: u32`
///
/// Returns the new thread's ID.
pub fn create_thread(
    thread_start_function: ThreadStartFunction,
    thread_start_data: Vec<u8>,
) -> u32 {
    let mut process_context_address: usize = 0;
    let mut next_thread_id: u32 = 0;

    PROCESS_CONTEXT_ADDRESS.with(|data| {
        process_context_address = *data.borrow();
    });

    CHILD_THREAD_NEXT_ID.with(|max_id_cell| {
        let last_thread_id = *max_id_cell.borrow();
        next_thread_id = last_thread_id + 1;
        *max_id_cell.borrow_mut() = next_thread_id;
    });

    let (parent_tx, child_rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let (child_tx, parent_rx) = std::sync::mpsc::channel::<Vec<u8>>();

    const HOST_THREAD_STACK_SIZE: usize = 128 * 1024; // 128 KB

    // Set up the thread builder with a reduced stack size (default is 2MB; here set to 128KB).
    // See: https://doc.rust-lang.org/stable/std/thread/index.html#stack-size
    let thread_builder = std::thread::Builder::new().stack_size(HOST_THREAD_STACK_SIZE);

    let join_handle = thread_builder
        .spawn(move || {
            // Store the process context address in thread-local storage.
            PROCESS_CONTEXT_ADDRESS.with(|data| {
                *data.borrow_mut() = process_context_address;
            });

            // Set up thread-local properties for the new thread.
            CURRENT_THREAD_ID.with(|id_cell| {
                *id_cell.borrow_mut() = next_thread_id;
            });

            RX.with(|rx| {
                rx.replace(Some(child_rx));
            });

            TX.with(|tx| {
                tx.replace(Some(child_tx));
            });

            // Store the thread's start data.
            THREAD_START_DATA.with(|data| {
                data.replace(thread_start_data);
            });

            // SAFETY: The process context pointer is valid for the lifetime of the process.
            let process_context_ptr = process_context_address as *const u8 as *const ProcessContext;
            let process_context = unsafe { &*process_context_ptr };

            let mut thread_context = process_context.create_thread_context();

            let result_foreign_values = process_function(
                &mut thread_context,
                thread_start_function.module_index,
                thread_start_function.function_public_index,
                // The thread start function must not take any parameters.
                &[],
            );

            // Returns `Result<Vec<ForeignValue>, Box<ProcessorError>>`.
            // The thread start function must return exactly one value (the exit code).
            let result = match result_foreign_values {
                Ok(foreign_values) => {
                    if foreign_values.len() != 1 {
                        Err(ProcessorError::new(
                            ProcessorErrorType::ResultsAmountMissmatch,
                        ))
                    } else {
                        if let ForeignValue::U32(exit_code) = foreign_values[0] {
                            Ok(exit_code)
                        } else {
                            Err(ProcessorError::new(ProcessorErrorType::DataTypeMissmatch))
                        }
                    }
                }
                Err(e) => Err(e),
            };

            // Map the error type for the join handle.
            result.map_err(|entry_error| Box::new(entry_error) as GenericError)
        })
        .unwrap();

    let child_thread = ChildThread {
        join_handle,
        rx: parent_rx, // Receiver for messages from the child thread
        tx: parent_tx, // Sender for messages to the child thread
    };

    // Register the new child thread in the thread-local child thread collection.
    CHILD_THREADS.with(|child_threads| {
        child_threads
            .borrow_mut()
            .insert(next_thread_id, child_thread);
    });

    // Return the new thread's ID.
    next_thread_id
}
