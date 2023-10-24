// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::{cell::RefCell, sync::Arc};

use ancvm_program::{program_source::ProgramSource, ProgramSourceType};
use ancvm_types::{ForeignValue, RuntimeError};

use crate::{
    interpreter::process_function, ChildThread, CHILD_THREADS, CHILD_THREAD_MAX_ID,
    CURRENT_THREAD_ID, RX, THREAD_START_DATA, TX,
};

// these values should be 'process-global', but the unit test
// runs on parallel, so change to thread-local to avoid overwrite by unit tests
thread_local! {
    pub static MT_PROGRAM_OBJECT_ADDRESS: RefCell<usize> = RefCell::new(0);
    pub static MT_PROGRAM_SOURCE_TYPE: RefCell<ProgramSourceType> = RefCell::new(ProgramSourceType::InMemory);
}

pub struct MultithreadProgram<T>
where
    T: ProgramSource, //  + ?Sized,
{
    pub program_source: Arc<T>,
}

impl<T> MultithreadProgram<T>
where
    T: ProgramSource + std::marker::Send + std::marker::Sync + 'static,
{
    pub fn new(program_source: T) -> Self {
        Self {
            program_source: Arc::new(program_source),
        }
    }
}

pub fn create_thread<T>(
    mt_program: &MultithreadProgram<T>, // dyn ProgramSource + std::marker::Send + std::marker::Sync + 'static,
    module_index: usize,
    func_public_index: usize,
    thread_start_data: Vec<u8>,
) -> u32
where
    T: ProgramSource + std::marker::Send + std::marker::Sync + 'static,
{
    let mt_program_object_address =
        mt_program as *const MultithreadProgram<_> as *const u8 as usize;
    let mt_program_source_type = mt_program.program_source.get_source_type();

    let mut next_thread_id: u32 = 0;

    CHILD_THREAD_MAX_ID.with(|max_id_cell| {
        let max_id = *max_id_cell.borrow();
        next_thread_id = max_id + 1;
        *max_id_cell.borrow_mut() = next_thread_id;
    });

    let (parent_tx, child_rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let (child_tx, parent_rx) = std::sync::mpsc::channel::<Vec<u8>>();

    let cloned_program_source = Arc::clone(&mt_program.program_source);

    const HOST_THREAD_STACK_SIZE: usize = 128 * 1024; // 128 KB

    // the default stack size is 2MB
    // https://doc.rust-lang.org/stable/std/thread/index.html#stack-size
    // change to a smaller size
    let thread_builder = std::thread::Builder::new().stack_size(HOST_THREAD_STACK_SIZE);

    let join_handle = thread_builder
        .spawn(move || {
            // store the information of program object
            MT_PROGRAM_OBJECT_ADDRESS.with(|addr_cell| {
                *addr_cell.borrow_mut() = mt_program_object_address;
            });

            MT_PROGRAM_SOURCE_TYPE.with(|source_type_cell| {
                *source_type_cell.borrow_mut() = mt_program_source_type;
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
            let thread_start_data_length = thread_start_data.len();

            THREAD_START_DATA.with(|data| {
                data.replace(thread_start_data);
            });

            let rst_program = cloned_program_source.build_program();
            match rst_program {
                Ok(program) => {
                    let mut thread_context = program.create_thread_context();
                    let rst_foreign_values = process_function(
                        &mut thread_context,
                        module_index,
                        func_public_index,
                        // the specified function should only has one or zero parameters, the value of argument
                        // is the length of 'thread_start_data'.
                        &[ForeignValue::UInt32(thread_start_data_length as u32)],
                    );

                    // Result<Vec<ForeignValue>, Box<dyn RuntimeError + Send>>
                    match rst_foreign_values {
                        Ok(foreign_values) => Ok(foreign_values),
                        Err(e) => Err(Box::new(e) as Box<dyn RuntimeError + Send>),
                    }
                }
                Err(e) => {
                    // Result<Vec<ForeignValue>, Box<dyn RuntimeError + Send>>
                    Err(Box::new(e) as Box<dyn RuntimeError + Send>)
                }
            }
        })
        .unwrap();

    let child_thread = ChildThread {
        join_handle,
        rx: RefCell::new(Some(parent_rx)),
        tx: RefCell::new(Some(parent_tx)),
    };

    CHILD_THREADS.with(|child_threads| {
        child_threads
            .borrow_mut()
            .insert(next_thread_id, child_thread);
    });

    next_thread_id
}
