// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::{
    cell::RefCell,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
    thread::JoinHandle,
};

use ancvm_program::program_source::ProgramSource;
use ancvm_types::{ForeignValue, RuntimeError};

use crate::{interpreter::process_function, CHILD_THREAD_MAX_ID, CURRENT_THREAD_ID, RX, TX};

pub struct MultithreadProgram<T>
where
    T: ProgramSource,
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

    pub fn create_thread(
        &self,
        module_index: usize,
        func_public_index: usize,
        arguments: Vec<ForeignValue>,
    ) -> ChildThread
    // JoinHandle<Result<Vec<ForeignValue>, Box<dyn RuntimeError + Send>>>
    where
        T: ProgramSource + std::marker::Send + std::marker::Sync + 'static,
    {
        let mut next_thread_id: u32 = 0;

        CHILD_THREAD_MAX_ID.with(|max_id_cell| {
            let max_id = *max_id_cell.borrow();
            next_thread_id = max_id + 1;
            *max_id_cell.borrow_mut() = next_thread_id;
        });

        let (parent_tx, child_rx) = std::sync::mpsc::channel::<Vec<u8>>();
        let (child_tx, parent_rx) = std::sync::mpsc::channel::<Vec<u8>>();

        let cloned_next_thread_id = next_thread_id;
        let cloned_program_source = Arc::clone(&self.program_source);

        let join_handle = std::thread::spawn(move || {
            // set up the local properties
            CURRENT_THREAD_ID.with(|id_cell| {
                *id_cell.borrow_mut() = cloned_next_thread_id;
            });

            RX.with(|rx| {
                rx.replace(Some(child_rx));
            });

            TX.with(|tx| {
                tx.replace(Some(child_tx));
            });

            let rst_program = cloned_program_source.build_program();
            match rst_program {
                Ok(program) => {
                    let mut thread_context = program.create_thread_context();
                    let rst_foreign_values = process_function(
                        &mut thread_context,
                        module_index,
                        func_public_index,
                        &arguments,
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
        });

        ChildThread {
            join_handle,
            rx: RefCell::new(Some(parent_rx)),
            tx: RefCell::new(Some(parent_tx)),
        }
    }
}

pub struct ChildThread {
    pub join_handle: JoinHandle<Result<Vec<ForeignValue>, Box<dyn RuntimeError + Send>>>,
    pub rx: RefCell<Option<Receiver<Vec<u8>>>>,
    pub tx: RefCell<Option<Sender<Vec<u8>>>>,
}

#[cfg(test)]
mod tests {
    use ancvm_binary::utils::{build_module_binary_with_single_function, BytecodeWriter};
    use ancvm_types::{ecallcode::ECallCode, opcode::Opcode, DataType, ForeignValue};

    use crate::{
        in_memory_program_source::InMemoryProgramSource, multithread_program::MultithreadProgram,
    };

    #[test]
    fn test_multithread_base() {
        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i32_add)
            .write_opcode_i32(Opcode::ecall, ECallCode::thread_id as u32)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32], // params
            vec![DataType::I32, DataType::I32], // results
            vec![],                             // local varslist which
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let multithread_program0 = MultithreadProgram::new(program_source0);
        let child_thread0 = multithread_program0.create_thread(
            0,
            0,
            vec![ForeignValue::UInt32(11), ForeignValue::UInt32(13)],
        );

        const FIRST_CHILD_THREAD_ID: u32 = 1;

        let result0 = child_thread0.join_handle.join().unwrap();
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(11 + 13),
                ForeignValue::UInt32(FIRST_CHILD_THREAD_ID)
            ]
        );
    }
}
