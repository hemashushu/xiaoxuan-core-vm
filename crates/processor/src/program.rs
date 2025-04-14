// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::process_context::ProcessContext;
use anc_isa::ForeignValue;

use crate::{
    handler::Handler,
    multithread_handler::{HANDLER_ADDRESS, PROCESS_CONTEXT_ADDRESS, THREAD_START_DATA},
    process::process_function,
    ProcessorError, ProcessorErrorType,
};

// pub fn start_with_multiple_thread(
//     process_context: &ProcessContext,
//     entry_point_name: &str,
//     thread_start_data: Vec<u8>,
// ) -> Result<u32, GenericError> {
//     let handler = Handler::new();
//     let handler_address = &handler as *const Handler as *const u8 as usize;
//     let process_context_address = process_context as *const ProcessContext as *const u8 as usize;
//
//     HANDLER_ADDRESS.with(|data| {
//         *data.borrow_mut() = handler_address;
//     });
//
//     PROCESS_CONTEXT_ADDRESS.with(|data| {
//         *data.borrow_mut() = process_context_address;
//     });
//
//     let main_thread_id = create_thread(entry_point_name, thread_start_data);
//
//     CHILD_THREADS.with(|child_threads_cell| {
//         let mut child_threads = child_threads_cell.borrow_mut();
//         let opt_child_thread = child_threads.remove(&main_thread_id);
//         let child_thread = opt_child_thread.unwrap();
//         child_thread.join_handle.join().unwrap()
//     })
// }

/// internal entry point names:
///
/// - internal entry point name: "_start"
///   executes function: '{app_module_name}::_start' (the default entry point)
///   user CLI unit name: "" (empty string)
///
/// - internal entry point name: "{submodule_name}"
///   executes function: '{app_module_name}::app::{submodule_name}::_start' (the additional executable units)
///   user CLI unit name: ":{submodule_name}"
///
/// - internal entry point name: "{submodule_name}::test_*"
///   executes function: '{app_module_name}::tests::{submodule_name}::test_*' (unit tests)
///   user CLI unit name: name path prefix, e.g. "{submodule_name}", "{submodule_name}::test_get_"
///
/// The type of entry function:
///
/// the entry functions (includes the default entry, additional executable units and unit test functions)
/// should returns i32, as well as the thread start function also returns i32.
pub fn start_program(
    process_context: &ProcessContext,
    internal_entry_point_name: &str,
    thread_start_data: Vec<u8>,
) -> Result<u32, ProcessorError> {
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

    let mut thread_context = process_context.create_thread_context();

    // use the function 'entry' as the startup function

    let function_public_index = if let Some(idx) = thread_context
        .module_linking_instance
        .entry_point_section
        .get_function_public_index(internal_entry_point_name)
    {
        idx
    } else {
        return Err(ProcessorError::new(
            ProcessorErrorType::EntryPointNotFound(internal_entry_point_name.to_owned()),
        ));
    };

    // the signature of the 'entry function' must be:
    // 'fn () -> exit_code:u32'

    const MAIN_MODULE_INDEX: usize = 0;

    let result_foreign_values = process_function(
        &handler,
        &mut thread_context,
        MAIN_MODULE_INDEX,
        function_public_index,
        &[],
    );

    match result_foreign_values {
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
    }
}

#[cfg(test)]
mod tests {
    use anc_context::process_resource::ProgramSource;
    use anc_image::{
        bytecode_writer::BytecodeWriterHelper,
        utils::helper_build_module_binary_with_single_function,
    };
    use anc_isa::{opcode::Opcode, OperandDataType};

    use crate::{
        in_memory_program_source::InMemoryProgramSource, multithread_process::start_program,
    };

    // #[test]
    // fn test_envcall_multithread_start_with_multithread() {
    //     // the signature of 'thread start function' must be
    //     // () -> i32
    //
    //     let code0 = BytecodeWriterHelper::new()
    //         .append_opcode_i32(Opcode::imm_i32, 0x11)
    //         .append_opcode(Opcode::end)
    //         .to_bytes();
    //
    //     let binary0 = helper_build_module_binary_with_single_function(
    //         vec![],                     // params
    //         vec![OperandDataType::I32], // results
    //         vec![],                     // local variables
    //         code0,
    //     );
    //
    //     let resource0 = InMemoryResource::new(vec![binary0]);
    //     let process_context0 = resource0.create_process_context().unwrap();
    //     let result0 = start_with_multiple_thread(&process_context0, "", vec![]);
    //
    //     const EXPECT_THREAD_EXIT_CODE: u32 = 0x11;
    //     assert_eq!(result0.unwrap(), EXPECT_THREAD_EXIT_CODE);
    // }

    #[test]
    fn test_envcall_multithread_start_program() {
        // the signature of 'thread start function' must be
        // () -> (i32)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 0x11)
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

        const EXPECT_THREAD_EXIT_CODE: u32 = 0x11;
        assert_eq!(result0.unwrap(), EXPECT_THREAD_EXIT_CODE);
    }
}
