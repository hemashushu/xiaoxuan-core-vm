// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_program::{memory::Memory, thread_context::ThreadContext, ProgramSourceType};
use ancvm_types::ForeignValue;

use crate::{
    in_memory_program_source::InMemoryProgramSource,
    multithread_program::{
        create_thread, MultithreadProgram, MT_PROGRAM_OBJECT_ADDRESS, MT_PROGRAM_SOURCE_TYPE,
    },
    CHILD_THREADS, CURRENT_THREAD_ID, THREAD_START_DATA,
};

pub const RESULT_SUCCESS: u32 = 0;
pub const RESULT_FAILURE: u32 = 1;
pub const WAIT_SUCCESS: u32 = 0;
pub const WAIT_NOT_FOUND: u32 = 1;
// pub const RUNNING_STATUS_RUNNING: u32 = 0;
// pub const RUNNING_STATUS_FINISH: u32 = 1;
// pub const RUNNING_STATUS_NOT_FOUND: u32 = 2;

pub fn thread_id(thread_context: &mut ThreadContext) {
    // `fn () -> thread_id:u64`

    // the function `thread::id()` is unstable, so here implement ourself.
    //
    // ref:
    //
    // ```test
    // error[E0658]: use of unstable library feature 'thread_id_value'
    // #![feature(thread_id_value)]
    // ```

    CURRENT_THREAD_ID.with(|id_cell| {
        let id = *id_cell.borrow();
        thread_context.stack.push_i32_u(id);
    });
}

pub fn thread_start_data_read(thread_context: &mut ThreadContext) {
    // 'fn (offset:u32, length:u32, dst_address:u64) -> result:u32'

    let dst_address = thread_context.stack.pop_i64_u();
    let length = thread_context.stack.pop_i32_u();
    let offset = thread_context.stack.pop_i32_u();

    THREAD_START_DATA.with(|data_cell| {
        let data = data_cell.borrow();
        if (offset + length) as usize > data.len() {
            // out of range
            thread_context.stack.push_i32_u(RESULT_FAILURE);
            return;
        }

        let src_ptr = data[offset as usize..].as_ptr();
        let dst_ptr = thread_context.heap.get_mut_ptr(dst_address as usize);
        unsafe { std::ptr::copy(src_ptr, dst_ptr, length as usize) };

        thread_context.stack.push_i32_u(RESULT_SUCCESS);
    });
}

pub fn thread_create(thread_context: &mut ThreadContext) {
    // '``
    // fn (module_index:u32, func_public_index:u32,
    //    thread_start_data_address:u32, thread_start_data_length:u32) -> child_thread_id:u32
    // ```

    let mut mt_program_object_address: usize = 0;
    let mut mt_program_source_type: ProgramSourceType = ProgramSourceType::InMemory;

    MT_PROGRAM_OBJECT_ADDRESS.with(|addr_cell| {
        mt_program_object_address = *addr_cell.borrow();
    });

    MT_PROGRAM_SOURCE_TYPE.with(|source_type_cell| {
        mt_program_source_type = *source_type_cell.borrow();
    });

    // get the 'MultithreadProgram' ref
    let mt_program_object_ptr = match mt_program_source_type {
        ancvm_program::ProgramSourceType::InMemory => {
            mt_program_object_address as *const u8
                as *const MultithreadProgram<InMemoryProgramSource>
        }
        ancvm_program::ProgramSourceType::File => {
            // TODO::
            todo!()
        }
    };

    let mt_program = unsafe { &*mt_program_object_ptr };

    // get arguments
    let thread_start_data_length = thread_context.stack.pop_i32_u();
    let thread_start_data_address = thread_context.stack.pop_i32_u();
    let func_public_index = thread_context.stack.pop_i32_u();
    let module_index = thread_context.stack.pop_i32_u();

    // get thread start data
    let thread_start_data = thread_context
        .heap
        .load_data(
            thread_start_data_address as usize,
            thread_start_data_length as usize,
        )
        .to_vec();

    let child_thread_id = create_thread(
        mt_program,
        module_index as usize,
        func_public_index as usize,
        thread_start_data,
    );

    thread_context.stack.push_i32_u(child_thread_id);
}

pub fn thread_wait_for_finish(thread_context: &mut ThreadContext) {
    // 'fn (child_thread_id:u32) -> (status:u32, thread_exit_code:u32)'

    let child_thread_id = thread_context.stack.pop_i32_u();

    CHILD_THREADS.with(|child_threads_cell| {
        let mut child_threads = child_threads_cell.borrow_mut();

        // remove the child thread object from 'child thread collection'
        let opt_child_thread = child_threads.remove(&child_thread_id);

        match opt_child_thread {
            Some(child_thread) => {
                let result = child_thread.join_handle.join().unwrap();
                match result {
                    Ok(values) => {
                        assert!(values.len() == 1);
                        match values[0] {
                            ForeignValue::UInt32(thread_exit_code) => {
                                thread_context.stack.push_i32_u(WAIT_SUCCESS);
                                thread_context.stack.push_i32_u(thread_exit_code);
                            }
                            _ => {
                                panic!("Error thread start function return type.")
                            }
                        }
                    }
                    Err(_) => {
                        thread_context.stack.push_i32_u(WAIT_SUCCESS);
                        thread_context.stack.push_i32_u(1);
                    }
                }
            }
            None => {
                thread_context.stack.push_i32_u(WAIT_NOT_FOUND);
                thread_context.stack.push_i32_u(0);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use ancvm_binary::utils::{
        build_module_binary_with_functions_and_blocks, build_module_binary_with_single_function,
        BytecodeWriter, HelperFuncEntryWithSignatureAndLocalVars,
    };
    use ancvm_types::{envcallcode::EnvCallCode, opcode::Opcode, DataType, ForeignValue};

    use crate::{
        in_memory_program_source::InMemoryProgramSource,
        interpreter::envcall::multithread::WAIT_SUCCESS,
        multithread_program::{create_thread, MultithreadProgram},
        CHILD_THREADS,
    };

    #[test]
    fn test_multithread_thread_id() {
        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::envcall, EnvCallCode::thread_id as u32)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I32], // params
            vec![DataType::I32], // results
            vec![],              // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let multithread_program0 = MultithreadProgram::new(program_source0);
        let child_thread_id0 = create_thread(&multithread_program0, 0, 0, vec![]);

        const FIRST_CHILD_THREAD_ID: u32 = 1;

        let r0 = CHILD_THREADS.with(|child_threads_cell| {
            let mut child_threads = child_threads_cell.borrow_mut();
            let opt_child_thread = child_threads.remove(&child_thread_id0);
            let child_thread = opt_child_thread.unwrap();

            let result0 = child_thread.join_handle.join().unwrap();
            result0
        });

        assert_eq!(
            r0.unwrap(),
            vec![ForeignValue::UInt32(FIRST_CHILD_THREAD_ID)]
        );
    }

    #[test]
    fn test_multithread_thread_start_data_read() {
        let code0 = BytecodeWriter::new()
            // resize heap to 1 page
            .write_opcode_i32(Opcode::i32_imm, 1)
            .write_opcode(Opcode::heap_resize)
            // read the thread start data to heap
            .write_opcode_i32(Opcode::i32_imm, 0) // offset
            .write_opcode_i32(Opcode::i32_imm, 4) // length
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0) // dst address
            .write_opcode_i32(Opcode::envcall, EnvCallCode::thread_start_data_read as u32)
            // read data from heap to stack
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0) // heap addr
            .write_opcode_i16(Opcode::heap_load32, 0) // offset
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I32], // params
            vec![DataType::I32], // results
            vec![],              // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let multithread_program0 = MultithreadProgram::new(program_source0);
        let child_thread_id0 =
            create_thread(&multithread_program0, 0, 0, vec![0x11, 0x22, 0x33, 0x44]);

        CHILD_THREADS.with(|child_threads_cell| {
            let mut child_threads = child_threads_cell.borrow_mut();
            let opt_child_thread = child_threads.remove(&child_thread_id0);
            let child_thread = opt_child_thread.unwrap();

            let result0 = child_thread.join_handle.join().unwrap();

            assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(0x44332211)]);
        });
    }

    #[test]
    fn test_multithread_thread_create() {
        let code0 = BytecodeWriter::new()
            // envcall/thread_create params
            .write_opcode_i32(Opcode::i32_imm, 0) // module_index
            .write_opcode_i32(Opcode::i32_imm, 1) // func_public_index
            .write_opcode_i32(Opcode::i32_imm, 0) // thread_start_data_address
            .write_opcode_i32(Opcode::i32_imm, 0) // thread_start_data_length
            .write_opcode_i32(Opcode::envcall, EnvCallCode::thread_create as u32)
            // now the operand on the top of stack is the child thread id
            .write_opcode_i32(Opcode::envcall, EnvCallCode::thread_wait_for_finish as u32)
            // now the operand on the top of stack is the (wait status, child thread exit code)
            .write_opcode(Opcode::end)
            .to_bytes();

        let code1 = BytecodeWriter::new()
            // set the thread exit code
            .write_opcode_i32(Opcode::i32_imm, 0x13171923)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_functions_and_blocks(
            vec![
                HelperFuncEntryWithSignatureAndLocalVars {
                    params: vec![DataType::I32],                      // params
                    results: vec![DataType::I32, DataType::I32],      // results
                    local_variable_item_entries_without_args: vec![], // local vars
                    code: code0,
                },
                HelperFuncEntryWithSignatureAndLocalVars {
                    params: vec![DataType::I32],                      // params
                    results: vec![DataType::I32],                     // results
                    local_variable_item_entries_without_args: vec![], // local vars
                    code: code1,
                },
            ],
            vec![],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let multithread_program0 = MultithreadProgram::new(program_source0);
        let child_thread_id0 = create_thread(&multithread_program0, 0, 0, vec![]);

        CHILD_THREADS.with(|child_threads_cell| {
            let mut child_threads = child_threads_cell.borrow_mut();
            let opt_child_thread = child_threads.remove(&child_thread_id0);
            let child_thread = opt_child_thread.unwrap();

            let result0 = child_thread.join_handle.join().unwrap();

            assert_eq!(
                result0.unwrap(),
                vec![
                    ForeignValue::UInt32(WAIT_SUCCESS),
                    ForeignValue::UInt32(0x13171923)
                ]
            );
        });
    }
}
