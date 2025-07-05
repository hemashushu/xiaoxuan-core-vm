// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

// when the XiaoXuan Core VM is embed into a Rust (or C) application as a library,
// the application can call the VM function through the bridge function as if it calls a native function.
//
// call external functon from Rust application example:
//
// ref:
// https://doc.rust-lang.org/nomicon/ffi.html
// https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html
// https://doc.rust-lang.org/stable/reference/items/functions.html

use anc_context::thread_context::ThreadContext;

use crate::{
    bridge_handler::{get_or_create_bridge_data, get_or_create_bridge_function},
    handler::Handler,
    ProcessorErrorType, ProcessorError,
};

// create a new bridge function and map it to the specified VM function.
//
// return the existing one if the bridge function corresponding
// to the specified function has already been created.
pub fn get_function<T>(
    handler: &Handler,
    thread_context: &mut ThreadContext,
    module_name: &str,
    function_name: &str,
) -> Result<T, ProcessorError> {
    todo!(); // find_function_by_full_name returns target function internal index
    let (module_index, function_public_index) = thread_context
        .find_function_by_full_name(module_name, function_name)
        .ok_or(ProcessorError::new(ProcessorErrorType::ItemNotFound))?;

    let function_ptr = get_or_create_bridge_function(
        handler,
        thread_context,
        module_index,
        function_public_index,
    )?;
    let function = unsafe { std::mem::transmute_copy(&function_ptr) };
    Ok(function)
}

pub fn get_data<T>(
    thread_context: &mut ThreadContext,
    module_name: &str,
    data_name: &str,
) -> Result<*const T, ProcessorError>
where
    T: Sized,
{
    let (module_index, data_public_index) = thread_context
        .find_data_public_index_by_name_path(module_name, data_name)
        .ok_or(ProcessorError::new(ProcessorErrorType::ItemNotFound))?;

    let data_ptr = get_or_create_bridge_data(
        thread_context,
        module_index,
        data_public_index,
        0,
        std::mem::size_of::<T>(),
    )?;

    Ok(data_ptr as *const T)
}

pub fn get_data_mut<T>(
    thread_context: &mut ThreadContext,
    module_name: &str,
    data_name: &str,
) -> Result<*mut T, ProcessorError>
where
    T: Sized,
{
    let (module_index, data_public_index) = thread_context
        .find_data_public_index_by_name_path(module_name, data_name)
        .ok_or(ProcessorError::new(ProcessorErrorType::ItemNotFound))?;

    let data_ptr = get_or_create_bridge_data(
        thread_context,
        module_index,
        data_public_index,
        0,
        std::mem::size_of::<T>(),
    )?;

    Ok(data_ptr as *mut T)
}


// //     pub fn find_bridge_function(
// //         &self,
// //         target_module_index: usize,
// //         function_internal_index: usize,
// //     ) -> Option<*const u8> {
// //         find_delegate_function(
// //             &self.bridge_function_module_items,
// //             target_module_index,
// //             function_internal_index,
// //         )
// //     }
// //
// //     pub fn find_bridge_callback_function(
// //         &self,
// //         target_module_index: usize,
// //         function_internal_index: usize,
// //     ) -> Option<*const u8> {
// //         find_delegate_function(
// //             &self.callback_function_entries,
// //             target_module_index,
// //             function_internal_index,
// //         )
// //     }
// //
// //     pub fn insert_bridge_function(
// //         &mut self,
// //         target_module_index: usize,
// //         function_internal_index: usize,
// //         bridge_function_ptr: *const u8,
// //     ) {
// //         insert_delegate_function(
// //             &mut self.bridge_function_module_items,
// //             target_module_index,
// //             function_internal_index,
// //             bridge_function_ptr,
// //         );
// //     }
// //
// // pub fn insert_callback_function(
// //     &mut self,
// //     target_module_index: usize,
// //     function_internal_index: usize,
// //     bridge_function_ptr: *const u8,
// // ) {
// //     insert_delegate_function(
// //         &mut self.callback_function_entries,
// //         target_module_index,
// //         function_internal_index,
// //         bridge_function_ptr,
// //     );
// // }


#[cfg(test)]
mod tests {
    use anc_context::process_resource::ProgramSource;
    use anc_image::{
        bytecode_writer::BytecodeWriterHelper,
        entry::InitedDataEntry,
        utils::{
            helper_build_module_binary_with_single_function,
            helper_build_module_binary_with_single_function_and_data,
        },
    };
    use anc_isa::{opcode::Opcode, ForeignValue, OperandDataType};

    use crate::{
        bridge_process::{get_data, get_data_mut, get_function},
        handler::Handler,
        in_memory_program_source::InMemoryProgramSource,
        process::process_function,
    };

    #[test]
    fn test_get_function() {
        // fn add(a:i32, b:i32) -> (i32) {
        //   a+b
        // }
        //
        // bytecode
        //
        // 0x0000 local_load32         0 0 0
        // 0x0008 local_load32         0 0 1
        // 0x0010 add_i32
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0, 0)
            .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0, 1)
            .append_opcode(Opcode::add_i32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[OperandDataType::I32, OperandDataType::I32], // params
            &[OperandDataType::I32],                       // results
            &[],                                           // local variables
            code0,
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let fn_add = get_function::<extern "C" fn(i32, i32) -> i32>(
            &handler,
            &mut thread_context0,
            "main",
            "func0",
        )
        .unwrap();

        assert_eq!(fn_add(11, 13), 24);
        assert_eq!(fn_add(23, 29), 52);
    }

    #[test]
    fn test_get_data() {
        let code0 = BytecodeWriterHelper::new()
            // (param offset_bytes:i16 data_public_index:i32) -> i64
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 0, 1)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[],                     // params
            &[OperandDataType::I32], // results
            &[],                     // local variables
            code0,
            &[InitedDataEntry::from_i32(0x11)],
            &[InitedDataEntry::from_i32(0x13)],
            &[],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let data0_ptr = get_data::<i32>(&mut thread_context0, "main", "data0").unwrap();
        let data1_ptr = get_data_mut::<i32>(&mut thread_context0, "main", "data1").unwrap();

        assert_eq!(unsafe { *data0_ptr }, 0x11);
        assert_eq!(unsafe { *data1_ptr }, 0x13);

        // update data1
        unsafe {
            *data1_ptr = 0x17;
        }

        let result0 = process_function( /* &handler, */ &mut thread_context0, 0, 0, &[]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(0x17),]);
    }
}
