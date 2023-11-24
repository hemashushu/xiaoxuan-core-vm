// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_program::{memory::Memory, thread_context::ThreadContext, ProgramSourceType};

use crate::{
    in_memory_program_source::InMemoryProgramSource,
    multithread_program::{
        create_thread, MultithreadProgram, MT_PROGRAM_OBJECT_ADDRESS, MT_PROGRAM_SOURCE_TYPE,
    },
    CHILD_THREADS, CURRENT_THREAD_ID, LAST_MESSAGE, RX, THREAD_START_DATA, TX,
};

pub const THREAD_RESULT_SUCCESS: u32 = 0;
pub const THREAD_RESULT_FAILURE: u32 = 1;

pub const THREAD_RUNNING_STATUS_RUNNING: u32 = 0;
pub const THREAD_RUNNING_STATUS_FINISH: u32 = 1;

pub fn thread_id(thread_context: &mut ThreadContext) {
    // `fn () -> thread_id:u64`

    // the Rust std function `thread::id()` is unstable, ref:
    //
    // ```test
    // error[E0658]: use of unstable library feature 'thread_id_value'
    // #![feature(thread_id_value)]
    // ```
    //
    // so i implement ourself 'thread_id'.

    CURRENT_THREAD_ID.with(|id_cell| {
        let id = *id_cell.borrow();
        thread_context.stack.push_i32_u(id);
    });
}

pub fn thread_create(thread_context: &mut ThreadContext) {
    // ```
    // fn (func_public_index:u32,
    //    thread_start_data_address_in_heap:u32, thread_start_data_length:u32) -> child_thread_id:u32
    // ```

    let mt_program_object_address: usize =
        MT_PROGRAM_OBJECT_ADDRESS.with(|addr_cell| *addr_cell.borrow());

    let mt_program_source_type: ProgramSourceType =
        MT_PROGRAM_SOURCE_TYPE.with(|source_type_cell| *source_type_cell.borrow());

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
    let thread_start_data_length = thread_context.stack.pop_i32_u() as usize;
    let thread_start_data_address = thread_context.stack.pop_i32_u() as usize;
    let func_public_index = thread_context.stack.pop_i32_u() as usize;

    // get the current module index
    let module_index = thread_context.pc.module_index;

    // get thread start data
    let thread_start_data = thread_context
        .heap
        .load_data(thread_start_data_address, thread_start_data_length)
        .to_vec();

    let child_thread_id = create_thread(
        mt_program,
        module_index,
        func_public_index,
        thread_start_data,
    );

    thread_context.stack.push_i32_u(child_thread_id);
}

pub fn thread_start_data_length(thread_context: &mut ThreadContext) {
    // 'fn () -> length:u32'

    let data_length = THREAD_START_DATA.with(|data_cell| {
        let data = data_cell.borrow();
        data.len()
    });

    thread_context.stack.push_i32_u(data_length as u32);
}

pub fn thread_start_data_read(thread_context: &mut ThreadContext) {
    // 'fn (offset:u32, length:u32, dst_address_in_heap:u64) -> (actual_read_length: u32)'

    let dst_address_in_heap = thread_context.stack.pop_i64_u() as usize;
    let length = thread_context.stack.pop_i32_u() as usize;
    let offset = thread_context.stack.pop_i32_u() as usize;

    let actual_read_length = THREAD_START_DATA.with(|data_cell| {
        let data = data_cell.borrow();
        let data_length = data.len();
        let actual_read_length = if offset + length > data_length {
            data_length - offset
        } else {
            length
        };

        let src_ptr = data[offset..].as_ptr();
        thread_context
            .heap
            .store_from(src_ptr, dst_address_in_heap, actual_read_length);

        actual_read_length
    });

    thread_context.stack.push_i32_u(actual_read_length as u32);
}

pub fn thread_wait_and_collect(thread_context: &mut ThreadContext) {
    // 'fn (child_thread_id:u32) -> (thread_exit_code:u64, thread_result:u32)'

    let child_thread_id = thread_context.stack.pop_i32_u();

    let (thread_exit_code, thread_result) = CHILD_THREADS.with(|child_threads_cell| {
        let mut child_threads = child_threads_cell.borrow_mut();

        // remove the child thread object from 'child thread collection'
        let opt_child_thread = child_threads.remove(&child_thread_id);

        match opt_child_thread {
            Some(child_thread) => {
                let result = child_thread.join_handle.join().unwrap();
                match result {
                    Ok(thread_exit_code) => (thread_exit_code, THREAD_RESULT_SUCCESS),
                    Err(_) => (0, THREAD_RESULT_FAILURE),
                }
            }
            None => (0, THREAD_RESULT_FAILURE),
        }
    });

    thread_context.stack.push_i64_u(thread_exit_code);
    thread_context.stack.push_i32_u(thread_result);
}

pub fn thread_running_status(thread_context: &mut ThreadContext) {
    // 'fn (child_thread_id:u32) -> (running_status:u32, thread_result:u32)'

    let child_thread_id = thread_context.stack.pop_i32_u();

    let (thread_running_status, thread_result) = CHILD_THREADS.with(|child_threads_cell| {
        let child_threads = child_threads_cell.borrow_mut();

        let opt_child_thread = child_threads.get(&child_thread_id);

        match opt_child_thread {
            Some(child_thread) => {
                let thread_running_status = if child_thread.join_handle.is_finished() {
                    THREAD_RUNNING_STATUS_FINISH
                } else {
                    THREAD_RUNNING_STATUS_RUNNING
                };
                (thread_running_status, THREAD_RESULT_SUCCESS)
            }
            None => (0, THREAD_RESULT_FAILURE),
        }
    });

    thread_context.stack.push_i32_u(thread_running_status);
    thread_context.stack.push_i32_u(thread_result);
}

pub fn thread_terminate(thread_context: &mut ThreadContext) {
    // 'fn (child_thread_id:u32) -> ()'

    let child_thread_id = thread_context.stack.pop_i32_u();

    CHILD_THREADS.with(|child_threads_cell| {
        let mut child_threads = child_threads_cell.borrow_mut();

        // remove the child thread object from 'child thread collection'
        let _ = child_threads.remove(&child_thread_id);

        // drop the TX to stop the RX in the child thread.
        //
        // ref:
        // - https://doc.rust-lang.org/std/sync/mpsc/index.html
        // - https://doc.rust-lang.org/std/sync/mpsc/struct.Sender.html
        // - https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html
        //
        // ```rust
        // if let Some(child_thread) = opt_child_thread {
        //     drop(child_thread.tx)
        // }
        // ```
    });
}

pub fn thread_receive_msg_from_parent(thread_context: &mut ThreadContext) {
    // 'fn () -> length:u32'
    RX.with(|rx_refcell| {
        let rx_ref = rx_refcell.borrow();
        let rx_opt = rx_ref.as_ref();
        match rx_opt {
            Some(rx) => {
                match rx.recv() {
                    Ok(data) => {
                        // store the received data
                        LAST_MESSAGE.with(|msg_refcell| {
                            let length = data.len();
                            msg_refcell.replace(data);

                            // push 'length' to stack
                            thread_context.stack.push_i32_u(length as u32);
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
                unreachable!("RX is not set.")
            }
        };
    });
}

pub fn thread_send_msg_to_parent(thread_context: &mut ThreadContext) {
    // 'fn (src_address_in_heap:u64, length:u32) -> thread_result:u32'

    let length = thread_context.stack.pop_i32_u();
    let src_address_in_heap = thread_context.stack.pop_i64_u();

    TX.with(|tx_refcell| {
        let tx_ref = tx_refcell.borrow();
        let tx_opt = tx_ref.as_ref();
        let thread_result = match tx_opt {
            Some(tx) => {
                let data = thread_context
                    .heap
                    .load_data(src_address_in_heap as usize, length as usize);
                let data_vec = data.to_vec();

                match tx.send(data_vec) {
                    Ok(_) => THREAD_RESULT_SUCCESS,
                    Err(_) => THREAD_RESULT_FAILURE,
                }
            }
            None => {
                unreachable!("TX is not set.")
            }
        };

        thread_context.stack.push_i32_u(thread_result);
    });
}

pub fn thread_receive_msg(thread_context: &mut ThreadContext) {
    // 'fn (child_thread_id:u32) -> (length:u32, thread_result:u32)'

    let child_thread_id = thread_context.stack.pop_i32_u();

    CHILD_THREADS.with(|child_threads_cell| {
        let child_threads = child_threads_cell.borrow_mut();
        let child_thread_opt = child_threads.get(&child_thread_id);

        let (length, thread_result) = match child_thread_opt {
            Some(child_thread) => {
                match child_thread.rx.recv() {
                    Ok(data) => {
                        // store the received data
                        LAST_MESSAGE.with(|msg_refcell| {
                            let length = data.len();
                            msg_refcell.replace(data);
                            (length, THREAD_RESULT_SUCCESS)
                        })
                    }
                    Err(_) => (0, THREAD_RESULT_FAILURE),
                }
            }
            None => (0, THREAD_RESULT_FAILURE),
        };

        // push 'length' and 'thread_result' to stack
        thread_context.stack.push_i32_u(length as u32);
        thread_context.stack.push_i32_u(thread_result);
    });
}

pub fn thread_send_msg(thread_context: &mut ThreadContext) {
    // 'fn (child_thread_id:u32, src_address_in_heap:u64, length:u32) -> thread_result:u32'

    let length = thread_context.stack.pop_i32_u();
    let src_address_in_heap = thread_context.stack.pop_i64_u();
    let child_thread_id = thread_context.stack.pop_i32_u();

    CHILD_THREADS.with(|child_threads_cell| {
        let child_threads = child_threads_cell.borrow_mut();
        let child_thread_opt = child_threads.get(&child_thread_id);

        let thread_result = match child_thread_opt {
            Some(child_thread) => {
                let data = thread_context
                    .heap
                    .load_data(src_address_in_heap as usize, length as usize);
                let data_vec = data.to_vec();

                match child_thread.tx.send(data_vec) {
                    Ok(_) => THREAD_RESULT_SUCCESS,
                    Err(_) => THREAD_RESULT_FAILURE,
                }
            }
            None => THREAD_RESULT_FAILURE,
        };

        // push 'thread_result' to stack
        thread_context.stack.push_i32_u(thread_result);
    });
}

pub fn thread_msg_length(thread_context: &mut ThreadContext) {
    // 'fn () -> length:u32'

    LAST_MESSAGE.with(|msg_refcell| {
        let msg_ref = msg_refcell.borrow();
        let length = msg_ref.len();

        // push 'length' to stack
        thread_context.stack.push_i32_u(length as u32);
    });
}

pub fn thread_msg_read(thread_context: &mut ThreadContext) {
    // 'fn (offset:u32, length:u32, dst_address_in_heap:u64) -> actual_read_length:u32'

    let dst_address_in_heap = thread_context.stack.pop_i64_u() as usize;
    let length = thread_context.stack.pop_i32_u() as usize;
    let offset = thread_context.stack.pop_i32_u() as usize;

    let actual_read_length = LAST_MESSAGE.with(|msg_refcell| {
        let msg_ref = msg_refcell.borrow();
        let msg_length = msg_ref.len();
        let actual_read_length = if offset + length > msg_length {
            msg_length - offset
        } else {
            length
        };

        let msg_ptr = msg_ref[offset..].as_ptr();

        thread_context
            .heap
            .store_from(msg_ptr, dst_address_in_heap, actual_read_length);

        actual_read_length
    });

    // push 'length' to stack
    thread_context.stack.push_i32_u(actual_read_length as u32);
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        bytecode_writer::BytecodeWriter,
        module_image::local_variable_section::LocalVariableEntry,
        utils::{
            helper_build_module_binary_with_functions_and_blocks,
            helper_build_module_binary_with_single_function,
            HelperFuncEntryWithSignatureAndLocalVars,
        },
    };
    use ancvm_types::{envcallcode::EnvCallCode, opcode::Opcode, DataType};

    use crate::{
        in_memory_program_source::InMemoryProgramSource,
        multithread_program::run_program_in_multithread,
    };

    #[test]
    fn test_envcall_multithread_run_program_in_multithread() {
        // the signature of 'thread start function' must be
        // () -> (i64)

        let code0 = BytecodeWriter::new()
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x11)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],              // params
            vec![DataType::I64], // results
            vec![],              // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let result0 = run_program_in_multithread(program_source0, vec![]);

        const EXPECT_THREAD_EXIT_CODE: u64 = 0x11;
        assert_eq!(result0.unwrap(), EXPECT_THREAD_EXIT_CODE);
    }

    #[test]
    fn test_envcall_multithread_thread_id() {
        // the signature of 'thread start function' must be
        // () -> (i64)

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::envcall, EnvCallCode::thread_id as u32)
            .append_opcode(Opcode::i64_extend_i32_u)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],              // params
            vec![DataType::I64], // results
            vec![],              // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let result0 = run_program_in_multithread(program_source0, vec![]);

        const FIRST_CHILD_THREAD_ID: u64 = 1;
        assert_eq!(result0.unwrap(), FIRST_CHILD_THREAD_ID);
    }

    #[test]
    fn test_envcall_multithread_thread_create() {
        // the signature of 'thread start function' must be
        // () -> (i64)

        let code0 = BytecodeWriter::new()
            // envcall/thread_create params
            .append_opcode_i32(Opcode::i32_imm, 1) // func_public_index
            .append_opcode_i32(Opcode::i32_imm, 0) // thread_start_data_address, no start data
            .append_opcode_i32(Opcode::i32_imm, 0) // thread_start_data_length, no start data
            .append_opcode_i32(Opcode::envcall, EnvCallCode::thread_create as u32)
            // now the operand(s) on the top of stack is: (child thread id)
            .append_opcode_i32(Opcode::envcall, EnvCallCode::thread_wait_and_collect as u32)
            // now the operand(s) on the top of stack is: (child thread exit code, thread result)
            .append_opcode(Opcode::drop)
            // now the operand(s) on the top of stack is: (child thread exit code)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code1 = BytecodeWriter::new()
            // set the thread exit code
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x13)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_blocks(
            vec![
                HelperFuncEntryWithSignatureAndLocalVars {
                    params: vec![],                                   // params
                    results: vec![DataType::I64],                     // results
                    local_variable_item_entries_without_args: vec![], // local vars
                    code: code0,
                },
                HelperFuncEntryWithSignatureAndLocalVars {
                    params: vec![],                                   // params
                    results: vec![DataType::I64],                     // results
                    local_variable_item_entries_without_args: vec![], // local vars
                    code: code1,
                },
            ],
            vec![],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let result0 = run_program_in_multithread(program_source0, vec![]);
        assert_eq!(result0.unwrap(), 0x13);
    }

    // note::
    // the unit tests for following functions would be too complicated to
    // write directly in bytecode, so leave these unit tests to
    // the project 'xiaoxuan-core-assembly'.
    //
    // - thread_start_data_length
    // - thread_start_data_read
    // - thread_running_status
    // - thread_drop
    // - thread_receive_msg_from_parent
    // - thread_send_msg_to_parent
    // - thread_receive_msg
    // - thread_send_msg
    // - thread_msg_length
    // - thread_msg_read
}
