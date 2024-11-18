// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::process_context::ProcessContext;
use ancvm_isa::{ForeignValue, GenericError};

use crate::{
    handler::Handler,
    multithread_handler::{
        create_thread, CHILD_THREADS, HANDLER_ADDRESS, PROCESS_CONTEXT_ADDRESS, THREAD_START_DATA,
    },
    process::process_function,
    HandleErrorType, HandlerError,
};

// pub fn start_with_multi_thread<T>(resource: T, thread_start_data: Vec<u8>) -> Result<u64, GenericError>
// where
//     T: Resource + std::marker::Send + std::marker::Sync + 'static,
pub fn start_with_multiple_thread(
    process_context: &ProcessContext,
    thread_start_data: Vec<u8>,
) -> Result<u64, GenericError> {
    let handler = Handler::new();
    let handler_address = &handler as *const Handler as *const u8 as usize;
    let process_context_address = process_context as *const ProcessContext as *const u8 as usize;

    HANDLER_ADDRESS.with(|data| {
        *data.borrow_mut() = handler_address;
    });

    PROCESS_CONTEXT_ADDRESS.with(|data| {
        *data.borrow_mut() = process_context_address;
    });

    // let multithread_instance = Multithread::new(resource);
    let main_thread_id = create_thread(None, thread_start_data);

    CHILD_THREADS.with(|child_threads_cell| {
        let mut child_threads = child_threads_cell.borrow_mut();
        let opt_child_thread = child_threads.remove(&main_thread_id);
        let child_thread = opt_child_thread.unwrap();
        child_thread.join_handle.join().unwrap()
    })
}

// pub fn start_with_single_thread<T>(resource: T, thread_start_data: Vec<u8>) -> Result<u64, GenericError>
// where
//     T: Resource + std::marker::Send + std::marker::Sync + 'static,
pub fn start_with_single_thread(
    process_context: &ProcessContext,
    thread_start_data: Vec<u8>,
) -> Result<u64, GenericError> {
    let handler = Handler::new();
    let handler_address = &handler as *const Handler as *const u8 as usize;
    let process_context_address = process_context as *const ProcessContext as *const u8 as usize;

    HANDLER_ADDRESS.with(|data| {
        *data.borrow_mut() = handler_address;
    });

    PROCESS_CONTEXT_ADDRESS.with(|data| {
        *data.borrow_mut() = process_context_address;
    });

    THREAD_START_DATA.with(|data| {
        data.replace(thread_start_data);
    });

    // let process_context = resource.create_process_context().unwrap();
    let mut thread_context = process_context.create_thread_context();

    // use the function 'entry' as the startup function
    const MAIN_MODULE_INDEX: usize = 0;
    let entry_function_public_index = thread_context
        .module_index_instance
        .index_property_section
        .entry_function_public_index as usize;

    // the signature of the 'entry function' must be:
    // 'fn () -> exit_code:u64'

    let result_foreign_values = process_function(
        &handler,
        &mut thread_context,
        MAIN_MODULE_INDEX,
        entry_function_public_index,
        &[],
    );

    match result_foreign_values {
        Ok(foreign_values) => {
            if foreign_values.len() != 1 {
                return Err(
                    Box::new(HandlerError::new(HandleErrorType::ResultsAmountMissmatch))
                        as GenericError,
                );
            }

            if let ForeignValue::U64(exit_code) = foreign_values[0] {
                Ok(exit_code)
            } else {
                Err(Box::new(HandlerError::new(HandleErrorType::DataTypeMissmatch)) as GenericError)
            }
        }
        Err(e) => Err(Box::new(e) as GenericError),
    }
}

#[cfg(test)]
mod tests {
    use ancvm_context::resource::Resource;
    use ancvm_image::{
        bytecode_writer::BytecodeWriterHelper,
        utils::helper_build_module_binary_with_single_function,
    };
    use ancvm_isa::{opcode::Opcode, OperandDataType};

    use crate::{
        in_memory_resource::InMemoryResource,
        multithread_process::{start_with_multiple_thread, start_with_single_thread},
    };

    #[test]
    fn test_envcall_multithread_start_with_multithread() {
        // the signature of 'thread start function' must be
        // () -> (i64)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i64(Opcode::imm_i64, 0x11)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                     // params
            vec![OperandDataType::I64], // results
            vec![],                     // local variables
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let result0 = start_with_multiple_thread(&process_context0, vec![]);

        const EXPECT_THREAD_EXIT_CODE: u64 = 0x11;
        assert_eq!(result0.unwrap(), EXPECT_THREAD_EXIT_CODE);
    }

    #[test]
    fn test_envcall_multithread_start_with_single_thread() {
        // the signature of 'thread start function' must be
        // () -> (i64)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i64(Opcode::imm_i64, 0x11)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                     // params
            vec![OperandDataType::I64], // results
            vec![],                     // local variables
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let result0 = start_with_single_thread(&process_context0, vec![]);

        const EXPECT_THREAD_EXIT_CODE: u64 = 0x11;
        assert_eq!(result0.unwrap(), EXPECT_THREAD_EXIT_CODE);
    }
}
