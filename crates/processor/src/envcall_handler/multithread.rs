// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::{thread, time::Duration};

use anc_context::thread_context::ThreadContext;

use crate::{
    handler::Handler,
    multithread_handler::{
        create_thread, ThreadStartFunction, CHILD_THREADS, CURRENT_THREAD_ID, LAST_THREAD_MESSAGE,
        RX, THREAD_START_DATA, TX,
    },
};

pub const THREAD_RUNNING_STATUS_RUNNING: u32 = 0;
pub const THREAD_RUNNING_STATUS_FINISH: u32 = 1;
pub const MESSAGE_SEND_RESULT_SUCCESS: u32 = 0;
pub const MESSAGE_SEND_RESULT_FAILURE: u32 = 1;
pub const MESSAGE_RECEIVE_RESULT_SUCCESS: u32 = 0;
pub const MESSAGE_RECEIVE_RESULT_FAILURE: u32 = 1;

pub fn thread_id(_handler: &Handler, thread_context: &mut ThreadContext) {
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

pub fn thread_create(_handler: &Handler, thread_context: &mut ThreadContext) {
    // ```
    // fn (function_public_index:u32,
    //    thread_start_data_address_in_heap:u32, thread_start_data_length:u32) -> child_thread_id:u32
    // ```

    // get arguments
    let thread_start_data_length = thread_context.stack.pop_i32_u() as usize;
    let thread_start_data_address = thread_context.stack.pop_i32_u() as usize;
    let function_public_index = thread_context.stack.pop_i32_u() as usize;

    // get the current module index
    let module_index = thread_context.pc.module_index;

    // get thread start data
    let thread_start_data = thread_context
        .memory
        .load_data(thread_start_data_address, thread_start_data_length)
        .to_vec();

    let thread_start_function = ThreadStartFunction {
        module_index,
        function_public_index,
    };

    let child_thread_id = create_thread(thread_start_function, thread_start_data);

    thread_context.stack.push_i32_u(child_thread_id);
}

pub fn thread_start_data_length(_handler: &Handler, thread_context: &mut ThreadContext) {
    // 'fn () -> length:u32'

    let data_length = THREAD_START_DATA.with(|data_cell| {
        let data = data_cell.borrow();
        data.len()
    });

    thread_context.stack.push_i32_u(data_length as u32);
}

pub fn thread_start_data_read(_handler: &Handler, thread_context: &mut ThreadContext) {
    // 'fn (offset:u32, length:u32, dst_memory_ptr:u64) -> (actual_read_length: u32)'

    let dst_memory_ptr = thread_context.stack.pop_i64_u() as usize;
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
        let dst_ptr = dst_memory_ptr as *mut u8;

        unsafe { std::ptr::copy(src_ptr, dst_ptr, actual_read_length) };

        actual_read_length
    });

    thread_context.stack.push_i32_u(actual_read_length as u32);
}

pub fn thread_wait_and_collect(_handler: &Handler, thread_context: &mut ThreadContext) {
    // 'fn (child_thread_id:u32) -> (thread_exit_code:u32, thread_not_found:u32)'

    let child_thread_id = thread_context.stack.pop_i32_u();

    let (thread_exit_code, thread_not_found) = CHILD_THREADS.with(|child_threads_cell| {
        let mut child_threads = child_threads_cell.borrow_mut();

        // remove the child thread object from 'child thread collection'
        let opt_child_thread = child_threads.remove(&child_thread_id);

        match opt_child_thread {
            Some(child_thread) => {
                let result = child_thread.join_handle.join().unwrap();
                match result {
                    Ok(thread_exit_code) => (thread_exit_code, 0),
                    // there is no way to return the details of ProcessorError in the
                    // child thread, so only the panic can be thrown.
                    Err(e) => panic!("Child thread panic: {}", e),
                }
            }
            None => (0, 1),
        }
    });

    thread_context.stack.push_i32_u(thread_exit_code);
    thread_context.stack.push_i32_u(thread_not_found);
}

pub fn thread_running_status(_handler: &Handler, thread_context: &mut ThreadContext) {
    // 'fn (child_thread_id:u32) -> (running_status:u32, thread_not_found:u32)'

    let child_thread_id = thread_context.stack.pop_i32_u();

    let (thread_running_status, thread_not_found) = CHILD_THREADS.with(|child_threads_cell| {
        let child_threads = child_threads_cell.borrow_mut();

        let opt_child_thread = child_threads.get(&child_thread_id);

        match opt_child_thread {
            Some(child_thread) => {
                let thread_running_status = if child_thread.join_handle.is_finished() {
                    THREAD_RUNNING_STATUS_FINISH
                } else {
                    THREAD_RUNNING_STATUS_RUNNING
                };
                (thread_running_status, 0)
            }
            None => (0, 1),
        }
    });

    thread_context.stack.push_i32_u(thread_running_status);
    thread_context.stack.push_i32_u(thread_not_found);
}

pub fn thread_terminate(_handler: &Handler, thread_context: &mut ThreadContext) {
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

pub fn thread_receive_msg_from_parent(_handler: &Handler, thread_context: &mut ThreadContext) {
    // 'fn () -> length:u32'
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

pub fn thread_send_msg_to_parent(_handler: &Handler, thread_context: &mut ThreadContext) {
    // 'fn (src_memory_ptr:u64, length:u32) -> ()'

    let length = thread_context.stack.pop_i32_u();
    let src_memory_ptr = thread_context.stack.pop_i64_u();

    TX.with(|tx_refcell| {
        let tx_ref = tx_refcell.borrow();
        let tx_opt = tx_ref.as_ref();
        match tx_opt {
            Some(tx) => {
                let src_ptr = src_memory_ptr as *const u8;
                let data_slice = unsafe { std::slice::from_raw_parts(src_ptr, length as usize) };
                let data_vec = data_slice.to_vec();

                tx.send(data_vec)
                    .expect("Send message to parent thread failed.");
            }
            None => {
                unreachable!("TX is not set.")
            }
        };
    });
}

pub fn thread_receive_msg(_handler: &Handler, thread_context: &mut ThreadContext) {
    // 'fn (child_thread_id:u32) -> (length:u32, receive_result:u32)'

    let child_thread_id = thread_context.stack.pop_i32_u();

    CHILD_THREADS.with(|child_threads_cell| {
        let child_threads = child_threads_cell.borrow_mut();
        let child_thread_opt = child_threads.get(&child_thread_id);

        let (length, receive_result) = match child_thread_opt {
            Some(child_thread) => {
                match child_thread.rx.recv() {
                    Ok(data) => {
                        // store the received data
                        LAST_THREAD_MESSAGE.with(|msg_refcell| {
                            let length = data.len();
                            msg_refcell.replace(data);
                            (length, MESSAGE_RECEIVE_RESULT_SUCCESS)
                        })
                    }
                    Err(_) => (0, MESSAGE_RECEIVE_RESULT_FAILURE), // the PIPE may have closed
                }
            }
            None => (0, MESSAGE_RECEIVE_RESULT_FAILURE),
        };

        // push 'length' and 'receive_result' to stack
        thread_context.stack.push_i32_u(length as u32);
        thread_context.stack.push_i32_u(receive_result);
    });
}

pub fn thread_send_msg(_handler: &Handler, thread_context: &mut ThreadContext) {
    // 'fn (child_thread_id:u32, src_memory_ptr:u64, length:u32) -> send_result:u32'

    let length = thread_context.stack.pop_i32_u();
    let src_memory_ptr = thread_context.stack.pop_i64_u();
    let child_thread_id = thread_context.stack.pop_i32_u();

    CHILD_THREADS.with(|child_threads_cell| {
        let child_threads = child_threads_cell.borrow_mut();
        let child_thread_opt = child_threads.get(&child_thread_id);

        let send_result = match child_thread_opt {
            Some(child_thread) => {
                let src_ptr = src_memory_ptr as *const u8;
                let data_slice = unsafe { std::slice::from_raw_parts(src_ptr, length as usize) };
                let data_vec = data_slice.to_vec();

                match child_thread.tx.send(data_vec) {
                    Ok(_) => MESSAGE_SEND_RESULT_SUCCESS,
                    Err(_) => MESSAGE_SEND_RESULT_FAILURE,
                }
            }
            None => MESSAGE_SEND_RESULT_FAILURE,
        };

        // push 'send_result' to stack
        thread_context.stack.push_i32_u(send_result);
    });
}

pub fn thread_msg_length(_handler: &Handler, thread_context: &mut ThreadContext) {
    // 'fn () -> length:u32'

    LAST_THREAD_MESSAGE.with(|msg_refcell| {
        let msg_ref = msg_refcell.borrow();
        let length = msg_ref.len();

        // push 'length' to stack
        thread_context.stack.push_i32_u(length as u32);
    });
}

pub fn thread_msg_read(_handler: &Handler, thread_context: &mut ThreadContext) {
    // 'fn (offset:u32, length:u32, dst_memory_ptr:u64) -> actual_read_length:u32'

    let dst_memory_ptr = thread_context.stack.pop_i64_u() as usize;
    let length = thread_context.stack.pop_i32_u() as usize;
    let offset = thread_context.stack.pop_i32_u() as usize;

    let actual_read_length = LAST_THREAD_MESSAGE.with(|msg_refcell| {
        let msg_ref = msg_refcell.borrow();
        let msg_length = msg_ref.len();
        let actual_read_length = if offset + length > msg_length {
            msg_length - offset
        } else {
            length
        };

        let src_ptr = msg_ref[offset..].as_ptr();
        let dst_ptr = dst_memory_ptr as *mut u8;

        unsafe { std::ptr::copy(src_ptr, dst_ptr, actual_read_length) };

        actual_read_length
    });

    // push 'length' to stack
    thread_context.stack.push_i32_u(actual_read_length as u32);
}

// ref:
// https://linux.die.net/man/2/nanosleep
pub fn thread_sleep(_handler: &Handler, thread_context: &mut ThreadContext) {
    // `fn (milliseconds:u64) -> ()`

    let milliseconds = thread_context.stack.pop_i64_u();
    thread::sleep(Duration::from_millis(milliseconds));
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use anc_context::process_resource::ProgramSource;
    use anc_image::{
        bytecode_writer::BytecodeWriterHelper,
        entry::LocalVariableEntry,
        utils::{
            helper_build_module_binary_with_functions_and_blocks,
            helper_build_module_binary_with_single_function, HelperFunctionEntry,
        },
    };
    use anc_isa::{opcode::Opcode, OperandDataType};

    use crate::{
        envcall_num::EnvCallNum, in_memory_program_source::InMemoryProgramSource,
        multithread_process::start_program,
    };

    // note::
    // unit tests for following functions would be too complicated
    // if written directly in bytecode, so leave these unit tests to
    // to done in project 'xiaoxuan-core-assembly'.
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

    #[test]
    fn test_envcall_multithread_main_thread_id() {
        // the signature of 'thread start function' must be
        // () -> i32

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::envcall, EnvCallNum::thread_id as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[],                     // params
            &[OperandDataType::I32], // results
            &[],                     // local variables
            code0,
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let result0 = start_program(&process_context0, "", vec![]);

        const MAIN_THREAD_ID: u32 = 0;
        assert_eq!(result0.unwrap(), MAIN_THREAD_ID);
    }

    #[test]
    fn test_envcall_multithread_child_thread_id() {
        // the signature of 'thread start function' must be
        // () -> i32

        let code0 = BytecodeWriterHelper::new()
            // envcall/thread_create params
            .append_opcode_i32(Opcode::imm_i32, 1) // function_public_index
            .append_opcode_i32(Opcode::imm_i32, 0) // thread_start_data_address, no start data
            .append_opcode_i32(Opcode::imm_i32, 0) // thread_start_data_length, no start data
            .append_opcode_i32(Opcode::envcall, EnvCallNum::thread_create as u32)
            // now the operand(s) on the top of stack is: (child thread id)
            .append_opcode_i32(Opcode::envcall, EnvCallNum::thread_wait_and_collect as u32)
            // now the operand(s) on the top of stack is: (child thread exit code, thread result)
            // .append_opcode(Opcode::drop)
            .append_opcode_i16_i16_i16(Opcode::local_store_i32, 0, 0, 0)
            // now the operand(s) on the top of stack is: (child thread exit code)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code1 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::envcall, EnvCallNum::thread_id as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_blocks(
            &[
                HelperFunctionEntry {
                    params: vec![],                      // params
                    results: vec![OperandDataType::I32], // results
                    local_variable_item_entries_without_args: vec![LocalVariableEntry::from_i32()], // local variables (for dropping operands)
                    code: code0,
                },
                HelperFunctionEntry {
                    params: vec![],                                   // params
                    results: vec![OperandDataType::I32],              // results
                    local_variable_item_entries_without_args: vec![], // local variables
                    code: code1,
                },
            ],
            &[],
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let result0 = start_program(&process_context0, "", vec![]);

        // note that the main thread id is '0',
        // the first child thread id is '1'.
        const FIRST_CHILD_THREAD_ID: u32 = 1;
        assert_eq!(result0.unwrap(), FIRST_CHILD_THREAD_ID);
    }

    #[test]
    fn test_envcall_multithread_thread_create() {
        // the signature of 'thread start function' must be
        // () -> i32

        let code0 = BytecodeWriterHelper::new()
            // envcall/thread_create params
            .append_opcode_i32(Opcode::imm_i32, 1) // function_public_index
            .append_opcode_i32(Opcode::imm_i32, 0) // thread_start_data_address, no start data
            .append_opcode_i32(Opcode::imm_i32, 0) // thread_start_data_length, no start data
            .append_opcode_i32(Opcode::envcall, EnvCallNum::thread_create as u32)
            // now the operand(s) on the top of stack is: (child thread id)
            .append_opcode_i32(Opcode::envcall, EnvCallNum::thread_wait_and_collect as u32)
            // now the operand(s) on the top of stack is: (child thread exit code, thread result)
            // .append_opcode(Opcode::drop)
            .append_opcode_i16_i16_i16(Opcode::local_store_i32, 0, 0, 0)
            // now the operand(s) on the top of stack is: (child thread exit code)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code1 = BytecodeWriterHelper::new()
            // set the thread exit code
            .append_opcode_i32(Opcode::imm_i32, 0x13)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_blocks(
            &[
                HelperFunctionEntry {
                    params: vec![],                      // params
                    results: vec![OperandDataType::I32], // results
                    local_variable_item_entries_without_args: vec![LocalVariableEntry::from_i32()], // local variables (for dropping operands)
                    code: code0,
                },
                HelperFunctionEntry {
                    params: vec![],                                   // params
                    results: vec![OperandDataType::I32],              // results
                    local_variable_item_entries_without_args: vec![], // local variables
                    code: code1,
                },
            ],
            &[],
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let result0 = start_program(&process_context0, "", vec![]);
        assert_eq!(result0.unwrap(), 0x13);
    }

    #[test]
    fn test_envcall_multithread_thread_sleep() {
        // the signature of 'thread start function' must be
        // () -> i32

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i64(Opcode::imm_i64, 1000)
            .append_opcode_i32(Opcode::envcall, EnvCallNum::thread_sleep as u32)
            .append_opcode_i32(Opcode::imm_i32, 0x13)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[],                     // params
            &[OperandDataType::I32], // results
            &[],                     // local variables
            code0,
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();

        let instant_a = Instant::now();
        let result0 = start_program(&process_context0, "", vec![]);
        assert_eq!(result0.unwrap(), 0x13);

        let instant_b = Instant::now();
        let duration = instant_b.duration_since(instant_a);
        let ms = duration.as_millis() as u64;
        assert!(ms > 500);
    }
}
