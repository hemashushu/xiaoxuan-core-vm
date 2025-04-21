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

use crate::{
    handler::Handler, process::process_function, ProcessorError, ProcessorErrorType,
    GenericError,
};

// these values should be 'process global', but the unit test
// runs on parallel and overwrite these values each thread.
// change to 'thread local' to avoid this.
thread_local! {
    // pointer to the "process context" and "handler" objects
    pub static PROCESS_CONTEXT_ADDRESS: RefCell<usize> = const{ RefCell::new(0) };
    pub static HANDLER_ADDRESS: RefCell<usize> =  const{  RefCell::new(0) };
}

// the Tx and Rx
// -------------
//
// threads communicate through message pipe, the raw type of message is u8 array,
// so the message can be:
//
// - primitive data
// - a struct
// - an array
// - (the address of) a function
// - (the address of) a closure function
thread_local! {
    // the collection of child threads
    pub static CHILD_THREADS:RefCell<BTreeMap<u32, ChildThread>> = const {RefCell::new(BTreeMap::new())};

    // an incremented only integer that is used to generate the child thread id.
    pub static CHILD_THREAD_NEXT_ID:RefCell<u32> = const {RefCell::new(0)};

    pub static CURRENT_THREAD_ID:RefCell<u32> = const {RefCell::new(0)};

    // the receiver that connects to the parent thread
    //
    // note:
    // "parent thread" is the creator of the current thread
    pub static RX:RefCell<Option<Receiver<Vec<u8>>>> =const { RefCell::new(None)};

    // the sender that connect to the parent thread
    pub static TX:RefCell<Option<Sender<Vec<u8>>>> = const {RefCell::new(None)};

    // the data (an u8 array) that comes from the parent thread
    pub static THREAD_START_DATA:RefCell<Vec<u8>> = const {RefCell::new(vec![])};

    // a temporary memory for storing the message comed from other threads.
    //
    // the data comes from other thread (includes the parent thread and child threads) is
    // stored in this temporary memory each time the function `thread_receive_msg` or
    // `thread_receive_msg_from_parent` is called.
    //
    //
    // ```diagram
    //
    //              | thread_receive_msg_from_parent
    //              | thread_receive_msg
    //              |
    // /---------\  |                   /-----------\
    // | message |--------------------->| temporary |
    // | box     |                      | memory    |
    // \---------/                      \-----------/
    //                                       |
    //                                       | thread_msg_read
    //                                       v
    //                                 writable data
    //
    // ```
    //
    // note:
    // this temporary memory will be replaced with new message each time the function
    // `thread_receive_msg` or `thread_receive_msg_from_parent` is called,
    // a function `thread_msg_read` should be called asap to read the message.
    pub static LAST_THREAD_MESSAGE:RefCell<Vec<u8>> = const {RefCell::new(vec![])};
}

pub struct ChildThread {
    // the child thread on host will return the 'thread_exit_code'
    pub join_handle: JoinHandle<Result<u32, GenericError>>,

    // the receiver that connects to the child thread
    pub rx: Receiver<Vec<u8>>,

    // the sender that connects to the child thread
    pub tx: Sender<Vec<u8>>,
}

#[derive(Debug, Clone, Copy)]
pub struct ThreadStartFunction {
    pub module_index: usize,
    pub function_public_index: usize,
}

/// the signature of the 'thread start function' must be:
/// 'fn () -> exit_code:u32'
///
pub fn create_thread(
    // entry_point_name: &str,
    thread_start_function: ThreadStartFunction,
    thread_start_data: Vec<u8>,
) -> u32 {
    let mut process_context_address: usize = 0;
    let mut handler_address: usize = 0;
    let mut next_thread_id: u32 = 0;

    HANDLER_ADDRESS.with(|data| {
        handler_address = *data.borrow();
    });

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

    // the default stack size is 2MB
    // https://doc.rust-lang.org/stable/std/thread/index.html#stack-size
    // change to a smaller size
    let thread_builder = std::thread::Builder::new().stack_size(HOST_THREAD_STACK_SIZE);

    // let entry_point_name_string = entry_point_name.to_owned();
    let join_handle = thread_builder
        .spawn(move || {
            // store the address of process_context and handler
            HANDLER_ADDRESS.with(|data| {
                *data.borrow_mut() = handler_address;
            });

            PROCESS_CONTEXT_ADDRESS.with(|data| {
                *data.borrow_mut() = process_context_address;
            });

            // set up the local properties
            CURRENT_THREAD_ID.with(|id_cell| {
                *id_cell.borrow_mut() = next_thread_id;
            });

            RX.with(|rx| {
                rx.replace(Some(child_rx));
            });

            TX.with(|tx| {
                tx.replace(Some(child_tx));
            });

            // store the thread additional data
            // let thread_start_data_length = thread_start_data.len();

            THREAD_START_DATA.with(|data| {
                data.replace(thread_start_data);
            });

            let handler_ptr = handler_address as *const u8 as *const Handler;
            let process_context_ptr = process_context_address as *const u8 as *const ProcessContext;

            let handler = unsafe { &*handler_ptr };
            let process_context = unsafe { &*process_context_ptr };

            let mut thread_context = process_context.create_thread_context();

            // let function_public_index = if let Some(idx) = thread_context
            //     .module_linking_instance
            //     .entry_point_section
            //     .get_function_public_index(&entry_point_name_string)
            // {
            //     idx
            // } else {
            //     return Err(
            //         Box::new(HandlerError::new(HandleErrorType::EntryPointNotFound(
            //             entry_point_name_string,
            //         ))) as GenericError,
            //     );
            // };
            //
            // const MAIN_MODULE_INDEX: usize = 0;
            let result_foreign_values = process_function(
                handler,
                &mut thread_context,
                thread_start_function.module_index,
                thread_start_function.function_public_index,
                // the specified function should only has no parameters
                &[],
            );

            // returns Result<Vec<ForeignValue>, Box<ProcessorError>>.
            //
            // the 'thread start function' should only return one value,
            // it is the user-defined thread exit code.
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
                            Err(ProcessorError::new(
                                ProcessorErrorType::DataTypeMissmatch,
                            ))
                        }
                    }
                }
                Err(e) => Err(e),
            };

            result.map_err(|entry_error| Box::new(entry_error) as GenericError)
        })
        .unwrap();

    let child_thread = ChildThread {
        join_handle,
        rx: parent_rx,
        tx: parent_tx,
    };

    CHILD_THREADS.with(|child_threads| {
        child_threads
            .borrow_mut()
            .insert(next_thread_id, child_thread);
    });

    next_thread_id
}
