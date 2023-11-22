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
    CHILD_THREADS, CURRENT_THREAD_ID, THREAD_START_DATA,
};

pub const THREAD_RESULT_SUCCESS: u32 = 0;
pub const THREAD_RESULT_FAILURE: u32 = 1;

pub const RUNNING_STATUS_RUNNING: u32 = 0;
pub const RUNNING_STATUS_FINISH: u32 = 1;

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
    let thread_start_data_length = thread_context.stack.pop_i32_u();
    let thread_start_data_address = thread_context.stack.pop_i32_u();
    let func_public_index = thread_context.stack.pop_i32_u();

    // get the current module index
    let module_index = thread_context.pc.module_index;

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

pub fn thread_start_data_length(thread_context: &mut ThreadContext) {
    // 'fn () -> length:u32'

    let data_length = THREAD_START_DATA.with(|data_cell| {
        let data = data_cell.borrow();
        data.len()
    });

    thread_context.stack.push_i32_u(data_length as u32);
}

pub fn thread_start_data_read(thread_context: &mut ThreadContext) {
    // 'fn (offset:u32, length:u32, dst_address:u64) -> (actual_read_length: u32, thread_result:u32)'

    let dst_address = thread_context.stack.pop_i64_u() as usize;
    let length = thread_context.stack.pop_i32_u() as usize;
    let offset = thread_context.stack.pop_i32_u() as usize;

    let read_length = THREAD_START_DATA.with(|data_cell| {
        let data = data_cell.borrow();
        let read_length = if (offset + length) > data.len() {
            // out of range
            data.len() - offset
        } else {
            length
        };

        let src_ptr = data[offset..].as_ptr();
        let dst_ptr = thread_context.heap.get_mut_ptr(dst_address);
        unsafe { std::ptr::copy(src_ptr, dst_ptr, length as usize) };

        read_length
    });

    thread_context.stack.push_i32_u(read_length as u32);
    thread_context.stack.push_i32_u(THREAD_RESULT_SUCCESS);
}

pub fn thread_wait_and_collect(thread_context: &mut ThreadContext) {
    // 'fn (child_thread_id:u32) -> (thread_exit_code:u32, thread_result:u32)'
    //
    // returns:
    // - thread_exit_code: the meaning of the 'exit_code' is defined by user,
    //   but in general convention, 0=thread exit with success, 1=thread exit with failure
    // - thread_result: 0=success, 1=failed (or not_found)

    let child_thread_id = thread_context.stack.pop_i32_u();

    let (exit_code, thread_result) = CHILD_THREADS.with(|child_threads_cell| {
        let mut child_threads = child_threads_cell.borrow_mut();

        // remove the child thread object from 'child thread collection'
        let opt_child_thread = child_threads.remove(&child_thread_id);

        match opt_child_thread {
            Some(child_thread) => {
                let result = child_thread.join_handle.join().unwrap();
                match result {
                    Ok(exit_code) => (exit_code, THREAD_RESULT_SUCCESS),
                    Err(_) => (0, THREAD_RESULT_FAILURE),
                }
            }
            None => (0, THREAD_RESULT_FAILURE),
        }
    });

    thread_context.stack.push_i32_u(exit_code);
    thread_context.stack.push_i32_u(thread_result);
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
        multithread_program::process_function_in_multithread,
    };

    #[test]
    fn test_multithread_thread_process_function_in_multithread() {
        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 0x11)
            .append_opcode(Opcode::end)
            .to_bytes();

        // the signature of 'thread start function' must be
        // () -> (i32)
        let binary0 = helper_build_module_binary_with_single_function(
            vec![],              // params
            vec![DataType::I32], // results
            vec![],              // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let result0 = process_function_in_multithread(program_source0, vec![]);

        assert_eq!(result0.unwrap(), 0x11);
    }

    #[test]
    fn test_multithread_thread_id() {
        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::envcall, EnvCallCode::thread_id as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        // the signature of 'thread start function' must be
        // () -> (i32)
        let binary0 = helper_build_module_binary_with_single_function(
            vec![],              // params
            vec![DataType::I32], // results
            vec![],              // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let result0 = process_function_in_multithread(program_source0, vec![]);

        const FIRST_CHILD_THREAD_ID: u32 = 1;
        assert_eq!(result0.unwrap(), FIRST_CHILD_THREAD_ID);
    }

    #[test]
    fn test_multithread_thread_create() {
        let code0 = BytecodeWriter::new()
            // envcall/thread_create params
            .append_opcode_i32(Opcode::i32_imm, 1) // func_public_index
            .append_opcode_i32(Opcode::i32_imm, 0) // thread_start_data_address
            .append_opcode_i32(Opcode::i32_imm, 0) // thread_start_data_length
            .append_opcode_i32(Opcode::envcall, EnvCallCode::thread_create as u32)
            // now the operand on the top of stack is the child thread id
            .append_opcode_i32(Opcode::envcall, EnvCallCode::thread_wait_and_collect as u32)
            // now the operand on the top of stack is the (child thread exit code, thread result)
            .append_opcode(Opcode::drop)
            // now the operand on the top of stack is the (child thread exit code)
            .append_opcode(Opcode::end)
            .to_bytes();

        let code1 = BytecodeWriter::new()
            // set the thread exit code
            .append_opcode_i32(Opcode::i32_imm, 0x13)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_blocks(
            vec![
                HelperFuncEntryWithSignatureAndLocalVars {
                    params: vec![],                                   // params
                    results: vec![DataType::I32],                     // results
                    local_variable_item_entries_without_args: vec![], // local vars
                    code: code0,
                },
                // the signature of 'thread start function' must be
                // () -> (i32)
                HelperFuncEntryWithSignatureAndLocalVars {
                    params: vec![],                                   // params
                    results: vec![DataType::I32],                     // results
                    local_variable_item_entries_without_args: vec![], // local vars
                    code: code1,
                },
            ],
            vec![],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let result0 = process_function_in_multithread(program_source0, vec![]);

        assert_eq!(result0.unwrap(), 0x13);
    }

    #[test]
    fn test_multithread_thread_start_data() {
        //                           0          8   (offset in byte)
        // start data:              |============|
        //                                |
        //                                v copy
        // heap:              0x100 |============|
        //                           |          |
        // start data length --\     |    /-----/
        //                     |     |    |
        //                     V     V    V
        // local var u32:    | u16 | u8 | u8 |
        //                   -----------------
        //                   low     |      high
        //                           \---> exit code 0x37_11_00_08
        let code0 = BytecodeWriter::new()
            // resize heap to 1 page
            .append_opcode_i32(Opcode::i32_imm, 1)
            .append_opcode(Opcode::heap_resize)
            .append_opcode(Opcode::drop)
            //
            // get the length of the 'thread start data'
            .append_opcode_i32(
                Opcode::envcall,
                EnvCallCode::thread_start_data_length as u32,
            )
            // copy to local var offset 0 as i16
            .append_opcode_i16_i16_i16(Opcode::local_store16, 0, 0, 0)
            //
            // read the thread start data and copy to heap 0x100
            .append_opcode_i32(Opcode::i32_imm, 0) // offset
            .append_opcode_i32(Opcode::i32_imm, 8) // length
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x100) // dst address
            .append_opcode_i32(Opcode::envcall, EnvCallCode::thread_start_data_read as u32)
            //
            // copy heap data byte 0 to local var offset 2
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .append_opcode_i16(Opcode::heap_load32_i8_u, 0)
            .append_opcode_i16_i16_i16(Opcode::local_store8, 0, 2, 0)
            //
            // copy heap data byte 8 to local var offset 3
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .append_opcode_i16(Opcode::heap_load32_i8_u, 7)
            .append_opcode_i16_i16_i16(Opcode::local_store8, 0, 3, 0)
            //
            // read local var
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                               // params
            vec![DataType::I32],                  // results
            vec![LocalVariableEntry::from_i32()], // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let result0 = process_function_in_multithread(
            program_source0,
            vec![0x11, 0x13, 0x17, 0x19, 0x23, 0x29, 0x31, 0x37],
        );
        assert_eq!(result0.unwrap(), 0x37_11_00_08);
    }
}
