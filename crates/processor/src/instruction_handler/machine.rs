// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;

use super::HandleResult;

pub fn terminate(/* _handler: &Handler, */ thread: &mut ThreadContext) -> HandleResult {
    let terminate_code = thread.get_param_i32() as i32;
    HandleResult::Terminate(terminate_code)
}

pub fn get_function(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param function_public_index:i32) -> (module_index:i32, function_public_index:i32)
    let function_public_index = thread_context.get_param_i32();
    let module_index = thread_context.pc.module_index as u32;
    thread_context.stack.push_i32_u(module_index);
    thread_context.stack.push_i32_u(function_public_index);
    HandleResult::Move(8)
}

pub fn get_data(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param data_public_index:i32) -> (module_index:i32, data_public_index:i32)
    let data_public_index = thread_context.get_param_i32();
    let module_index = thread_context.pc.module_index as u32;
    thread_context.stack.push_i32_u(module_index);
    thread_context.stack.push_i32_u(data_public_index);
    HandleResult::Move(8)
}

pub fn host_addr_data(/* _handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32) -> pointer
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_host_addr_data(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn host_addr_data_extend(
/*    _handler: &Handler, */
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i64) -> pointer
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i64_u();
    do_host_addr_data(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn host_addr_data_dynamic(
/*    _handler: &Handler, */
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // () (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> pointer
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_public_index = thread_context.stack.pop_i32_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_host_addr_data(
        thread_context,
        module_index as usize,
        data_public_index as usize,
        offset_bytes as usize,
        2,
    )
}

fn do_host_addr_data(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_public_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let target_data_object =
        thread_context.get_target_data_object(module_index, data_public_index, 0, 0);
    let start_address = target_data_object
        .accessor
        .get_start_address_by_index(target_data_object.data_internal_index_in_section);
    let ptr = target_data_object
        .accessor
        .get_ptr(start_address, offset_bytes);
    store_pointer_to_operand_stack(thread_context, ptr);
    HandleResult::Move(instruction_length_in_bytes)
}

pub fn host_addr_function( /* handler: &Handler, */ thread_context: &mut ThreadContext) -> HandleResult {
    // (param function_public_index:i32) -> pointer
    //
    //     let function_public_index = thread_context.get_param_i32() as usize;
    //     let module_index = thread_context.pc.module_index;
    //
    //     if let Ok(callback_function_ptr) = get_or_create_bridge_callback_function(
    //         handler,
    //         thread_context,
    //         module_index,
    //         function_public_index,
    //     ) {
    //         store_pointer_to_operand_stack(thread_context, callback_function_ptr);
    //         HandleResult::Move(8)
    //     } else {
    //         HandleResult::Terminate(TERMINATE_CODE_FAILED_TO_CREATE_DELEGATE_FUNCTION)
    //     }
    todo!()
}

pub fn host_addr_function_dynamic(
    // handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // () (operand function_module_index:i32 function_public_index:i32) -> pointer
    todo!()
}

fn store_pointer_to_operand_stack(thread_context: &mut ThreadContext, ptr: *const u8) {
    let address = ptr as u64;
    thread_context.stack.push_i64_u(address);
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anc_context::{
        process_property::{ProcessProperty, ProgramSourceType},
        program_source::ProgramSource,
    };
    use anc_image::{
        bytecode_reader::format_bytecode_as_text,
        bytecode_writer::BytecodeWriterHelper,
        entry::{ExternalLibraryEntry, ReadOnlyDataEntry, ReadWriteDataEntry, UninitDataEntry},
        utils::{
            helper_build_module_binary_with_functions_and_data_and_external_functions,
            helper_build_module_binary_with_single_function,
            helper_build_module_binary_with_single_function_and_data, HelperExternalFunctionEntry,
            HelperFunctionEntry,
        },
    };
    use anc_isa::{
        opcode::Opcode, DependencyCondition, DependencyLocal, ExternalLibraryDependency,
        ForeignValue, OperandDataType,
    };

    use crate::{
        in_memory_program_source::InMemoryProgramSource,
        process::process_function, ProcessorError, ProcessorErrorType, TERMINATE_CODE_UNREACHABLE,
    };

    fn read_memory_i64(fv: ForeignValue) -> u64 {
        if let ForeignValue::U64(addr) = fv {
            let ptr = addr as *const u64;
            unsafe { std::ptr::read(ptr) }
        } else {
            panic!("The data type of the foreign value does not match.")
        }
    }

    fn read_memory_i32(fv: ForeignValue) -> u32 {
        if let ForeignValue::U64(addr) = fv {
            let ptr = addr as *const u32;
            unsafe { std::ptr::read(ptr) }
        } else {
            panic!("The data type of the foreign value does not match.")
        }
    }

    fn read_memory_i16(fv: ForeignValue) -> u16 {
        if let ForeignValue::U64(addr) = fv {
            let ptr = addr as *const u16;
            unsafe { std::ptr::read(ptr) }
        } else {
            panic!("The data type of the foreign value does not match.")
        }
    }

    fn read_memory_i8(fv: ForeignValue) -> u8 {
        if let ForeignValue::U64(addr) = fv {
            let ptr = addr as *const u8;
            unsafe { std::ptr::read(ptr) }
        } else {
            panic!("The data type of the foreign value does not match.")
        }
    }

    #[test]
    fn test_handler_fundamental_terminate() {
        // () -> ()
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::terminate, TERMINATE_CODE_UNREACHABLE as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[], // params
            &[], // results
            &[], // local variables
            code0,
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();

        let mut thread_context0 = process_context0.create_thread_context();
        let result0 = process_function( /* &handler, */ &mut thread_context0, 0, 0, &[]);

        assert!(matches!(
            result0,
            Err(ProcessorError {
                error_type: ProcessorErrorType::Terminate(TERMINATE_CODE_UNREACHABLE)
            })
        ));
    }

    #[test]
    fn test_handler_host_address_of_data() {
        //        read-only data section
        //        ======================
        //
        //       |low address    high addr|
        //   pub |                        |
        // index |0              1        |
        //  type |i32------|    |i32------|
        //
        //  data 11 00 00 00    13 00 00 00
        //       |              |
        //   r/w |0             |1
        //
        //        read write data section
        //        =======================
        //
        //       |low address             high address|
        //   pub |                                    |
        // index |2                          3        |
        //  type |i64------------------|    |i32------|
        //
        //  data 17 00 00 00 00 00 00 00    19 00 00 00
        //       |                          |
        //   r/w |2                         |3
        //
        //        uninitialized data section
        //        ==========================
        //
        //       |low address             high address|
        //   pub |                                    |
        // index |4              5                    |
        //  type |i32------|    |i64------------------|
        //
        //  data 00 00 00 00    00 00 00 00 00 00 00 00
        //       |              |
        //   r/w |4             |5
        //
        //
        // () -> (i64,i64,i64,i64,i64,i64)
        //        -----------------------
        //        | addr of data
        //
        // read the values of data through the host address.

        let code0 = BytecodeWriterHelper::new()
            // copy ".data" index 2 to ".bss" index 5
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 5)
            // copy ".data" index 3 to ".bss" index 4
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 0, 3)
            .append_opcode_i16_i32(Opcode::data_store_i32, 0, 4)
            // write "0xee" to ".data" index 2
            .append_opcode_i64(Opcode::imm_i64, 0xee)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 2)
            // write "0xff" to ".data" index 3
            .append_opcode_i32(Opcode::imm_i32, 0xff)
            .append_opcode_i16_i32(Opcode::data_store_i32, 0, 3)
            //
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 0)
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 1)
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 2)
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 3)
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 4)
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 5)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
            ], // results
            &[], // local variables
            code0,
            &[
                ReadOnlyDataEntry::from_i32(0x11), // ro, data idx: 0
                ReadOnlyDataEntry::from_i32(0x13), // ro, data idx: 1
            ],
            &[
                ReadWriteDataEntry::from_i64(0x17), // rw, data idx: 2
                ReadWriteDataEntry::from_i32(0x19), // rw, data idx: 3
            ],
            &[
                UninitDataEntry::from_i32(), // bss, data idx: 4
                UninitDataEntry::from_i64(), // bss, data idx: 5
            ],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function( /* &handler, */ &mut thread_context0, 0, 0, &[]);
        let fvs = result0.unwrap();

        assert_eq!(read_memory_i32(fvs[0]), 0x11);
        assert_eq!(read_memory_i32(fvs[1]), 0x13);
        assert_eq!(read_memory_i64(fvs[2]), 0xee);
        assert_eq!(read_memory_i32(fvs[3]), 0xff);
        assert_eq!(read_memory_i32(fvs[4]), 0x19);
        assert_eq!(read_memory_i64(fvs[5]), 0x17);
    }

    #[test]
    fn test_handler_host_address_of_data_extend() {
        //        read-only data section
        //        ======================
        //
        //       |low address    high addr|
        //   pub |                        |
        // index |0              1        |
        //  type |i32------|    |i32------|
        //
        //  data 11 00 00 00    13 00 00 00
        //       |              |
        //   r/w |0             |1
        //
        //        read write data section
        //        =======================
        //
        //       |low address             high address|
        //   pub |                                    |
        // index |2                          3        |
        //  type |i64------------------|    |i32------|
        //
        //  data 17 00 00 00 00 00 00 00    19 00 00 00
        //       |                          |
        //   r/w |2                         |3
        //
        //        uninitialized data section
        //        ==========================
        //
        //       |low address             high address|
        //   pub |                                    |
        // index |4              5                    |
        //  type |i32------|    |i64------------------|
        //
        //  data 00 00 00 00    00 00 00 00 00 00 00 00
        //       |              |
        //   r/w |4             |5
        //
        //
        // () -> (i64,i64,i64,i64,i64,i64)
        //        -----------------------
        //        | addr of data
        //
        // read the values of data through the host address.

        let code0 = BytecodeWriterHelper::new()
            // copy ".data" index 2 to ".bss" index 5
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 5)
            // copy ".data" index 3 to ".bss" index 4
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 0, 3)
            .append_opcode_i16_i32(Opcode::data_store_i32, 0, 4)
            // write "0xee" to ".data" index 2
            .append_opcode_i64(Opcode::imm_i64, 0xee)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 2)
            // write "0xff" to ".data" index 3
            .append_opcode_i32(Opcode::imm_i32, 0xff)
            .append_opcode_i16_i32(Opcode::data_store_i32, 0, 3)
            //
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i32(Opcode::host_addr_data_extend, 0)
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i32(Opcode::host_addr_data_extend, 1)
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i32(Opcode::host_addr_data_extend, 2)
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i32(Opcode::host_addr_data_extend, 3)
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i32(Opcode::host_addr_data_extend, 4)
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i32(Opcode::host_addr_data_extend, 5)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
            ], // results
            &[], // local variables
            code0,
            &[
                ReadOnlyDataEntry::from_i32(0x11), // ro, data idx: 0
                ReadOnlyDataEntry::from_i32(0x13), // ro, data idx: 1
            ],
            &[
                ReadWriteDataEntry::from_i64(0x17), // rw, data idx: 2
                ReadWriteDataEntry::from_i32(0x19), // rw, data idx: 3
            ],
            &[
                UninitDataEntry::from_i32(), // bss, data idx: 4
                UninitDataEntry::from_i64(), // bss, data idx: 5
            ],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function( /* &handler, */ &mut thread_context0, 0, 0, &[]);
        let fvs = result0.unwrap();

        assert_eq!(read_memory_i32(fvs[0]), 0x11);
        assert_eq!(read_memory_i32(fvs[1]), 0x13);
        assert_eq!(read_memory_i64(fvs[2]), 0xee);
        assert_eq!(read_memory_i32(fvs[3]), 0xff);
        assert_eq!(read_memory_i32(fvs[4]), 0x19);
        assert_eq!(read_memory_i64(fvs[5]), 0x17);
    }

    #[test]
    fn test_handler_host_address_of_data_dynamic() {
        //        read-only data section
        //        ======================
        //
        //       |low address    high addr|
        //   pub |                        |
        // index |0              1        |
        //  type |i32------|    |i32------|
        //
        //  data 11 00 00 00    13 00 00 00
        //       |              |
        //   r/w |0             |1
        //
        //        read write data section
        //        =======================
        //
        //       |low address             high address|
        //   pub |                                    |
        // index |2                          3        |
        //  type |i64------------------|    |i32------|
        //
        //  data 17 00 00 00 00 00 00 00    19 00 00 00
        //       |                          |
        //   r/w |2                         |3
        //
        //        uninitialized data section
        //        ==========================
        //
        //       |low address             high address|
        //   pub |                                    |
        // index |4              5                    |
        //  type |i32------|    |i64------------------|
        //
        //  data 00 00 00 00    00 00 00 00 00 00 00 00
        //       |              |
        //   r/w |4             |5
        //
        //
        // () -> (i64,i64,i64,i64,i64,i64)
        //        -----------------------
        //        | addr of data
        //
        // read the values of data through the host address.

        let code0 = BytecodeWriterHelper::new()
            // copy ".data" index 2 to ".bss" index 5
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 5)
            // copy ".data" index 3 to ".bss" index 4
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 0, 3)
            .append_opcode_i16_i32(Opcode::data_store_i32, 0, 4)
            // write "0xee" to ".data" index 2
            .append_opcode_i64(Opcode::imm_i64, 0xee)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 2)
            // write "0xff" to ".data" index 3
            .append_opcode_i32(Opcode::imm_i32, 0xff)
            .append_opcode_i16_i32(Opcode::data_store_i32, 0, 3)
            //
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 0) // data public index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::host_addr_data_dynamic)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 1) // data public index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::host_addr_data_dynamic)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 2) // data public index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::host_addr_data_dynamic)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 3) // data public index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::host_addr_data_dynamic)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 4) // data public index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::host_addr_data_dynamic)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 5) // data public index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::host_addr_data_dynamic)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
            ], // results
            &[], // local variables
            code0,
            &[
                ReadOnlyDataEntry::from_i32(0x11), // ro, data idx: 0
                ReadOnlyDataEntry::from_i32(0x13), // ro, data idx: 1
            ],
            &[
                ReadWriteDataEntry::from_i64(0x17), // rw, data idx: 2
                ReadWriteDataEntry::from_i32(0x19), // rw, data idx: 3
            ],
            &[
                UninitDataEntry::from_i32(), // bss, data idx: 4
                UninitDataEntry::from_i64(), // bss, data idx: 5
            ],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function( /* &handler, */ &mut thread_context0, 0, 0, &[]);
        let fvs = result0.unwrap();

        assert_eq!(read_memory_i32(fvs[0]), 0x11);
        assert_eq!(read_memory_i32(fvs[1]), 0x13);
        assert_eq!(read_memory_i64(fvs[2]), 0xee);
        assert_eq!(read_memory_i32(fvs[3]), 0xff);
        assert_eq!(read_memory_i32(fvs[4]), 0x19);
        assert_eq!(read_memory_i64(fvs[5]), 0x17);
    }

    #[test]
    fn test_handler_host_address_of_allocated_memory() {
        // todo
    }

//     #[test]
//     fn test_handler_host_addr_function() {}

//     #[test]
//     fn test_handler_host_addr_function_and_callback_function() {
//         // `do_something` is an external function (a C function) in "libtest0.so.1":
//         //
//         // ```c
//         // int do_something(int (*callback_function)(int), int a, int b)
//         // {
//         //     int s = (callback_function)(a);
//         //     return s + b;
//         // }
//         // ```
//         //
//         // `function0` and `function1` are VM functions:
//         //
//         // ```code
//         // ;; the entry function
//         // fn function0 (a:i32, b:i32)->i32 {
//         //     call(do_something, host_addr_function(function1), a, b)
//         // }
//         //
//         // ;; used as callback function for external function 'do_something'
//         // fn function1 (a:i32) -> i32 {
//         //     a*2
//         // }
//         // ```
//         //
//         // the calling path:
//         //
//         // ```diagram
//         // (11,13) ->
//         //   function0 (VM function) ->
//         //     do_something (external function) ->
//         //       function1 (VM function) ->
//         //     return to do_something ->
//         //   return to function0 ->
//         // returns 35
//         // ```
//
//         // VM function 0
//         let code0 = BytecodeWriterHelper::new()
//             .append_opcode_i32(Opcode::host_addr_function, 1) // get host address of the func1, for external func param 0
//             .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0) // for external func param 1
//             .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 1) // for external func param 2
//             //
//             .append_opcode_i32(Opcode::extcall, 0) // call external function, external function index = 0
//             //
//             .append_opcode(Opcode::end)
//             .to_bytes();
//
//         // VM function 1
//         let code1 = BytecodeWriterHelper::new()
//             .append_opcode_i16_i32(Opcode::local_load_i32_u, 0, 0)
//             .append_opcode_i32(Opcode::imm_i32, 2)
//             .append_opcode(Opcode::mul_i32)
//             //
//             .append_opcode(Opcode::end)
//             .to_bytes();
//
//         let binary0 = helper_build_module_binary_with_functions_and_data_and_external_functions(
//             &[
//                 HelperFunctionEntry {
//                     // type_index: 1,
//                     params: vec![OperandDataType::I32, OperandDataType::I32],
//                     results: vec![OperandDataType::I32],
//                     local_variable_item_entries_without_args: vec![],
//                     code: code0,
//                 },
//                 HelperFunctionEntry {
//                     // type_index: 2,
//                     params: vec![OperandDataType::I32],
//                     results: vec![OperandDataType::I32],
//                     local_variable_item_entries_without_args: vec![],
//                     code: code1,
//                 },
//             ],
//             &[],
//             &[],
//             &[],
//             &[ExternalLibraryEntry::new(
//                 "libtest0".to_owned(),
//                 Box::new(ExternalLibraryDependency::Local(Box::new(
//                     DependencyLocal {
//                         path: "lib/libtest0.so.1".to_owned(),
//                         condition: DependencyCondition::True,
//                         parameters: HashMap::default(),
//                     },
//                 ))),
//             )],
//             &[HelperExternalFunctionEntry {
//                 name: "do_something".to_string(),
//                 external_library_index: 0,
//                 params: vec![
//                     OperandDataType::I64,
//                     OperandDataType::I32,
//                     OperandDataType::I32,
//                 ],
//                 result: Some(OperandDataType::I32),
//             }],
//         );
//
//         let mut pwd = std::env::current_dir().unwrap();
//         // let pkg_name = env!("CARGO_PKG_NAME");
//         let crate_folder_name = "processor";
//         if !pwd.ends_with(crate_folder_name) {
//             // in the VSCode `Debug` environment, the `current_dir()`
//             // the project root folder.
//             // while in both `$ cargo test` and VSCode `Run Test` environment
//             // the `current_dir()` return the current crate path.
//             pwd.push("crates");
//             pwd.push(crate_folder_name);
//         }
//         pwd.push("tests");
//         // let application_path = pwd.to_str().unwrap();
//
//         /* let handler = Handler::new(); */
//         let resource0 = InMemoryProgramSource::with_property(
//             vec![binary0],
//             ProcessProperty::new(
//                 pwd,
//                 ProgramSourceType::Module,
//                 vec![],
//                 HashMap::<String, String>::new(),
//             ),
//         );
//         let process_context0 = resource0.create_process_context().unwrap();
//         let mut thread_context0 = process_context0.create_thread_context();
//
//         let result0 = process_function(
//             &handler,
//             &mut thread_context0,
//             0,
//             0,
//             &[ForeignValue::U32(11), ForeignValue::U32(13)],
//         );
//         assert_eq!(result0.unwrap(), vec![ForeignValue::U32(11 * 2 + 13)]);
//
//         let result1 = process_function(
//             &handler,
//             &mut thread_context0,
//             0,
//             0,
//             &[ForeignValue::U32(211), ForeignValue::U32(223)],
//         );
//         assert_eq!(result1.unwrap(), vec![ForeignValue::U32(211 * 2 + 223)]);
//     }
}
