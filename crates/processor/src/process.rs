// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::{program_resource::ProgramResource, thread_context::ThreadContext};
use ancvm_isa::{ForeignValue, GenericError};

use crate::{
    delegate::{build_bridge_data, build_bridge_function},
    interpreter::process_function,
    multithread_program::{create_thread, MultithreadProgram},
    InterpreterError, InterpreterErrorType, CHILD_THREADS, THREAD_START_DATA,
};

// process_function

pub fn start_program_in_multithread<T>(
    program_resource: T,
    thread_start_data: Vec<u8>,
) -> Result<u64, GenericError>
where
    T: ProgramResource + std::marker::Send + std::marker::Sync + 'static,
{
    let multithread_program = MultithreadProgram::new(program_resource);
    let main_thread_id = create_thread(&multithread_program, None, thread_start_data);

    CHILD_THREADS.with(|child_threads_cell| {
        let mut child_threads = child_threads_cell.borrow_mut();
        let opt_child_thread = child_threads.remove(&main_thread_id);
        let child_thread = opt_child_thread.unwrap();
        child_thread.join_handle.join().unwrap()
    })
}

pub fn start_program_in_single_thread<T>(
    program_resource: T,
    thread_start_data: Vec<u8>,
) -> Result<u64, GenericError>
where
    T: ProgramResource + std::marker::Send + std::marker::Sync + 'static,
{
    THREAD_START_DATA.with(|data| {
        data.replace(thread_start_data);
    });

    let process_context = program_resource.create_process_context().unwrap();
    let mut thread_context = process_context.create_thread_context();

    // use the function 'entry' as the startup function
    const MAIN_MODULE_INDEX: usize = 0;
    let entry_function_public_index = thread_context
        .index_instance
        .property_section
        .entry_function_public_index as usize;

    // the signature of the 'entry function' must be:
    // 'fn () -> exit_code:u64'

    let result_foreign_values = process_function(
        &mut thread_context,
        MAIN_MODULE_INDEX,
        entry_function_public_index,
        &[],
    );

    match result_foreign_values {
        Ok(foreign_values) => {
            if foreign_values.len() != 1 {
                return Err(Box::new(InterpreterError::new(
                    InterpreterErrorType::ResultsAmountMissmatch,
                )) as GenericError);
            }

            if let ForeignValue::U64(exit_code) = foreign_values[0] {
                Ok(exit_code)
            } else {
                Err(Box::new(InterpreterError::new(
                    InterpreterErrorType::DataTypeMissmatch,
                )) as GenericError)
            }
        }
        Err(e) => Err(Box::new(e) as GenericError),
    }
}

pub fn call_function(
    thread_context: &mut ThreadContext,
    module_index: usize,
    function_public_index: usize,
    arguments: &[ForeignValue],
) -> Result<Vec<ForeignValue>, InterpreterError> {
    process_function(
        thread_context,
        module_index,
        function_public_index,
        arguments,
    )
}

pub fn get_bridge_function<T>(
    thread_context: &mut ThreadContext,
    module_name: &str,
    function_name: &str,
) -> Result<T, InterpreterError> {
    let (module_index, function_public_index) = thread_context
        .find_function_public_index_by_name(module_name, function_name)
        .ok_or(InterpreterError::new(InterpreterErrorType::ItemNotFound))?;

    let function_ptr = build_bridge_function(thread_context, module_index, function_public_index)?;
    let function = unsafe { std::mem::transmute_copy(&function_ptr) };
    Ok(function)
}

pub fn get_bridge_data<T>(
    thread_context: &mut ThreadContext,
    module_name: &str,
    data_name: &str,
) -> Result<*const T, InterpreterError>
where
    T: Sized,
{
    let (module_index, data_public_index) = thread_context
        .find_data_public_index_by_name(module_name, data_name)
        .ok_or(InterpreterError::new(InterpreterErrorType::ItemNotFound))?;

    let data_ptr = build_bridge_data(
        thread_context,
        module_index,
        data_public_index,
        0,
        std::mem::size_of::<T>(),
    )?;

    Ok(data_ptr as *const T)
}

#[cfg(test)]
mod tests {
    // todo
}
