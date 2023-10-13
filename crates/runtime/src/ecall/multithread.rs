// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_program::thread_context::ThreadContext;

use crate::CURRENT_THREAD_ID;

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

#[cfg(test)]
mod tests {
    use ancvm_binary::utils::{build_module_binary_with_single_function, BytecodeWriter};
    use ancvm_types::{ecallcode::ECallCode, opcode::Opcode, DataType, ForeignValue};

    use crate::{
        in_memory_program_source::InMemoryProgramSource,
        multithread_program::{create_thread, MultithreadProgram},
        CHILD_THREADS,
    };

    #[test]
    fn test_multithread_thread_id() {
        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::ecall, ECallCode::thread_id as u32)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I32], // params
            vec![DataType::I32], // results
            vec![],              // local varslist which
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let multithread_program0 = MultithreadProgram::new(program_source0);
        let child_thread_id0 = create_thread(&multithread_program0, 0, 0, vec![]);

        const FIRST_CHILD_THREAD_ID: u32 = 1;

        CHILD_THREADS.with(|child_threads_cell| {
            let mut child_threads = child_threads_cell.borrow_mut();
            let opt_child_thread = child_threads.remove(&child_thread_id0);
            let child_thread = opt_child_thread.unwrap();

            let result0 = child_thread.join_handle.join().unwrap();

            assert_eq!(
                result0.unwrap(),
                vec![ForeignValue::UInt32(FIRST_CHILD_THREAD_ID)]
            );
        });
    }
}
