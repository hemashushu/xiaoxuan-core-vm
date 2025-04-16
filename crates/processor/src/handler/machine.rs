// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;

use crate::TERMINATE_CODE_FAILED_TO_CREATE_DELEGATE_FUNCTION;

use super::{HandleResult, Handler};

pub fn terminate(_handler: &Handler, thread: &mut ThreadContext) -> HandleResult {
    let terminate_code = thread.get_param_i32() as i32;
    HandleResult::Terminate(terminate_code)
}

pub fn get_function(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param function_public_index:i32) -> (module_index:i32, function_public_index:i32)
    let function_public_index = thread_context.get_param_i32();
    let module_index = thread_context.pc.module_index as u32;
    thread_context.stack.push_i32_u(module_index);
    thread_context.stack.push_i32_u(function_public_index);
    HandleResult::Move(8)
}

pub fn get_data(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param data_public_index:i32) -> (module_index:i32, data_public_index:i32)
    let data_public_index = thread_context.get_param_i32();
    let module_index = thread_context.pc.module_index as u32;
    thread_context.stack.push_i32_u(module_index);
    thread_context.stack.push_i32_u(data_public_index);
    HandleResult::Move(8)
}

// DEPRECATED
// // pub fn host_addr_local(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
// //     // (param layers:i16 offset_bytes:i16 local_variable_index:i16) -> pointer
// //     let (layers, offset_bytes, local_variable_index) =
// //         thread_context.get_param_i16_i16_i16();
// //     do_host_addr_local(
// //         thread_context,
// //         layers,
// //         local_variable_index as usize,
// //         offset_bytes as usize,
// //     )
// // }
// //
// // pub fn host_addr_local_extend(
// //     _handler: &Handler,
// //     thread_context: &mut ThreadContext,
// // ) -> HandleResult {
// //     // (param layers:i16 local_variable_index:i32) (operand offset_bytes:i64) -> pointer
// //     let (layers, local_variable_index) = thread_context.get_param_i16_i32();
// //     let offset_bytes = thread_context.stack.pop_i64_u();
// //     do_host_addr_local(
// //         thread_context,
// //         layers,
// //         local_variable_index as usize,
// //         offset_bytes as usize,
// //     )
// // }
// //
// // fn do_host_addr_local(
// //     thread_context: &mut ThreadContext,
// //     layers: u16,
// //     local_variable_index: usize,
// //     offset_bytes: usize,
// // ) -> HandleResult {
// //     let start_address = thread_context.get_local_variable_start_address(
// //         layers,
// //         local_variable_index,
// //         offset_bytes,
// //         0,
// //     );
// //     let ptr = thread_context.stack.get_ptr(start_address, offset_bytes);
// //     store_pointer_to_operand_stack(thread_context, ptr);
// //     HandleResult::Move(8)
// // }

pub fn host_addr_data(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
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
    _handler: &Handler,
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
    _handler: &Handler,
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

// DEPRECATED
// //
// // pub fn host_addr_memory(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
// //     // (param offset_bytes:i16) (operand memory_address:i64) -> i64
// //     let offset_bytes = thread_context.get_param_i16();
// //     let memory_address = thread_context.stack.pop_i64_u();
// //
// //     let total_offset = memory_address as usize + offset_bytes as usize;
// //     let ptr = thread_context.memory.get_ptr(total_offset);
// //     store_pointer_to_operand_stack(thread_context, ptr);
// //     HandleResult::Move(4)
// // }
// //
// // pub fn host_copy_from_memory(
// //     _handler: &Handler,
// //     thread_context: &mut ThreadContext,
// // ) -> HandleResult {
// //     // copy data from VM heap to host memory
// //     // () (operand dst_pointer:i64 src_addr:i64 count:i64) -> ()
// //
// //     let count = thread_context.stack.pop_i64_u();
// //     let src_memory_address = thread_context.stack.pop_i64_u();
// //     let dst_host_ptr = thread_context.stack.pop_i64_u();
// //
// //     let src_heap_ptr = thread_context.memory.get_ptr(src_memory_address as usize);
// //     unsafe { std::ptr::copy(src_heap_ptr, dst_host_ptr as *mut u8, count as usize) };
// //
// //     HandleResult::Move(2)
// // }
// //
// // pub fn host_copy_to_memory(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
// //     // copy data from host memory to VM heap
// //     // () (operand dst_addr:i64 src_pointer:i64 count:i64) -> ()
// //
// //     let count = thread_context.stack.pop_i64_u();
// //     let src_host_ptr = thread_context.stack.pop_i64_u();
// //     let dst_memory_address = thread_context.stack.pop_i64_u();
// //
// //     let dst_heap_ptr = thread_context
// //         .memory
// //         .get_mut_ptr(dst_memory_address as usize);
// //     unsafe { std::ptr::copy(src_host_ptr as *const u8, dst_heap_ptr, count as usize) };
// //
// //     HandleResult::Move(2)
// // }
// //
// // pub fn host_external_memory_copy(
// //     _handler: &Handler,
// //     thread_context: &mut ThreadContext,
// // ) -> HandleResult {
// //     // copy data between host memory
// //     // (operand dst_pointer:i64 src_pointer:i64 count:i64)
// //
// //     let count = thread_context.stack.pop_i64_u();
// //     let src_host_ptr = thread_context.stack.pop_i64_u();
// //     let dst_host_ptr = thread_context.stack.pop_i64_u();
// //
// //     unsafe {
// //         std::ptr::copy(
// //             src_host_ptr as *const u8,
// //             dst_host_ptr as *mut u8,
// //             count as usize,
// //         )
// //     };
// //
// //     HandleResult::Move(2)
// // }

// pub fn host_addr_function(handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
//     // (param function_public_index:i32) -> i64/i32
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
// }

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
        entry::{
            ExternalLibraryEntry,  ReadOnlyDataEntry, ReadWriteDataEntry,
            UninitDataEntry,
        },
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
        handler::Handler, in_memory_program_source::InMemoryProgramSource,
        process::process_function,
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

        let handler = Handler::new();
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();

        let mut thread_context0 = process_context0.create_thread_context();
        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);

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
        // index |2(0)                       3(1)     |
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
        // index |4(0)           5(1)                 |
        //  type |i32------|    |i64------------------|
        //
        //  data 00 00 00 00    00 00 00 00 00 00 00 00
        //       |              |
        //   r/w |4             |5
        //
// DEPRECATED
//        //        local variable area
//        //        ===================
//        //
//        //       |low address                                       high addr|
//        // local |                                                           |
//        // index |0       1                           2                      |
//        //  type |bytes| |i32------|   |padding--|   |i32------|   |padding--|
//        //
//        //  data 0.....0 31 00 00 00   00 00 00 00   37 00 00 00   00 00 00 00
//        //       ^       |                           |
//        //   r/w |       |6                          |7
//        //       |
//        //       | 64 bytes, the space for storing function results.
//        //       | because the results will overwrite the stack, so it need to
//        //       | leave enough space for results, then the data of local variables
//        //       | can be still read after function is finish.
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
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 3)
            .append_opcode_i16_i32(Opcode::data_store_i32, 0, 4)
            // write "0xee" to ".data" index 2
            .append_opcode_i64(Opcode::imm_i64, 0xee)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 2)
            // write "0xff" to ".data" index 3
            .append_opcode_i32(Opcode::imm_i32, 0xff)
            .append_opcode_i16_i32(Opcode::data_store_i32, 0, 3)
            // DEPRECATED
            // // //
            // // .append_opcode_i32(Opcode::imm_i32, 0x31)
            // // .append_opcode_i16_i16_i16(Opcode::local_store_i32, 0, 0, 1)
            // // //
            // // .append_opcode_i32(Opcode::imm_i32, 0x37)
            // // .append_opcode_i16_i16_i16(Opcode::local_store_i32, 0, 0, 2)
            //
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 0)
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 1)
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 2)
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 3)
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 4)
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 5)
            // DEPRECATED
            // // .append_opcode_i16_i16_i16(Opcode::host_addr_local, 0, 0, 1)
            // // .append_opcode_i16_i16_i16(Opcode::host_addr_local, 0, 0, 2)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", format_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                // DEPRECATED
                // // OperandDataType::I64,
                // // OperandDataType::I64,
            ], // results
            &[
                // DEPRECATE
                // // LocalVariableEntry::from_bytes(64, 8), // space
                // // LocalVariableEntry::from_i32(),
                // // LocalVariableEntry::from_i32(),
            ], // local variables
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

        let handler = Handler::new();
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        let fvs = result0.unwrap();

        assert_eq!(read_memory_i32(fvs[0]), 0x11);
        assert_eq!(read_memory_i32(fvs[1]), 0x13);
        assert_eq!(read_memory_i64(fvs[2]), 0xee);
        assert_eq!(read_memory_i32(fvs[3]), 0xff);
        assert_eq!(read_memory_i32(fvs[4]), 0x19);
        assert_eq!(read_memory_i64(fvs[5]), 0x17);


        // DEPRECATED
        // // // note:
        // // // depending on the implementation of the stack (the stack frame and local variables),
        // // // the following 'assert_eq' may fail,
        // // // because the local variables (as well as their host addresses) will no longer valid
        // // // when a function exits.
        // // assert_eq!(read_memory_i32(fvs[6]), 0x31);
        // // assert_eq!(read_memory_i32(fvs[7]), 0x37);
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
        // index |2(0)                       3(1)     |
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
        // index |4(0)           5(1)                 |
        //  type |i32------|    |i64------------------|
        //
        //  data 00 00 00 00    00 00 00 00 00 00 00 00
        //       |              |
        //   r/w |4             |5
        //
        // () -> (i64,i64,i64,i64,i64,i64)
        //        -----------------------
        //        | addr of data
        //
        // read the values of data through the host address.

        let code0 = BytecodeWriterHelper::new()
            // copy ".data" index 2 to ".bss" index 5
            .append_opcode_i16_i32(Opcode::data_load_extend_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 5)
            // copy ".data" index 3 to ".bss" index 4
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 3)
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

        // println!("{}", format_bytecode_as_text(&code0));

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
            &[
            ], // local variables
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

        let handler = Handler::new();
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        let fvs = result0.unwrap();

        assert_eq!(read_memory_i32(fvs[0]), 0x11);
        assert_eq!(read_memory_i32(fvs[1]), 0x13);
        assert_eq!(read_memory_i64(fvs[2]), 0xee);
        assert_eq!(read_memory_i32(fvs[3]), 0xff);
        assert_eq!(read_memory_i32(fvs[4]), 0x19);
        assert_eq!(read_memory_i64(fvs[5]), 0x17);
    }

    #[test]
    fn test_handler_host_address_of_data_dynamic_base() {
        // note:
        // only limited testing is done here.
        // corss-module tests are left to higher-level module
        // because the code is too complex due to the need to
        // generate and link multiple modules.

        //        read-only data section
        //        ======================
        //
        //       |low address  high addr|
        //   pub |                      |
        // index |0           1         |
        //  data 02 03 05 07  11 13 17 19
        //       |            |     |
        //   r/w |0           |1    |2

        //        read-write data section
        //        ======================
        //
        //       |low address  high addr|
        //   pub |                      |
        // index |2                     |
        //  data 23 29 31 37  41 43 47 53
        //       |            |     |
        //   r/w |3           |4    |5

        // () -> (i64,i64,i64,i64,i64,i64)
        //       -------------------------
        //             addr of data

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 0) // data index
            .append_opcode_i32(Opcode::imm_i64, 0) // offset
            .append_opcode(Opcode::host_addr_data_dynamic)
            //
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 1) // data index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset
            .append_opcode(Opcode::host_addr_data_dynamic)
            //
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 1) // data index
            .append_opcode_i64(Opcode::imm_i64, 2) // offset
            .append_opcode(Opcode::host_addr_data_dynamic)
            //
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 1) // data index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset
            .append_opcode(Opcode::host_addr_data_dynamic)
            //
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 1) // data index
            .append_opcode_i64(Opcode::imm_i64, 4) // offset
            .append_opcode(Opcode::host_addr_data_dynamic)
            //
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 1) // data index
            .append_opcode_i64(Opcode::imm_i64, 6) // offset
            .append_opcode(Opcode::host_addr_data_dynamic)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", format_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                //
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
            ], // results
            &[], // local variables
            code0,
            &[
                ReadOnlyDataEntry::from_bytes(vec![0x02u8, 0x03, 0x05, 0x07], 4), // init data
                ReadOnlyDataEntry::from_bytes(vec![0x11u8, 0x13, 0x17, 0x19], 4), // init data
            ], // init data
            &[ReadWriteDataEntry::from_i64(0x5347434137312923u64)],
            &[],
        );

        let handler = Handler::new();
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        let fvs = result0.unwrap();

        assert_eq!(read_memory_i8(fvs[0]), 0x02);
        assert_eq!(read_memory_i8(fvs[1]), 0x11);
        assert_eq!(read_memory_i8(fvs[2]), 0x17);
        //
        assert_eq!(read_memory_i8(fvs[3]), 0x23);
        assert_eq!(read_memory_i8(fvs[4]), 0x41);
        assert_eq!(read_memory_i8(fvs[5]), 0x47);
    }

    #[test]
    fn test_handler_host_address_of_allocated_memory() {
        // todo
    }

    //     DEPRECATED
    //     #[test]
    //     fn test_handler_host_address_memory() {
    //         //        heap
    //         //       |low address                high addr|
    //         //       |                                    |
    //         //  addr |0x100         0x200                 |
    //         //  type |i32-------|   |i64------------------|
    //         //
    //         //  data  02 03 05 07   11 13 17 19 23 29 31 37
    //         //        ^     ^       ^           ^        ^
    //         //        |0    |1      |2          |3       |4
    //         //
    //         // () -> (i64,i64,i64,i64,i64)
    //
    //         let code0 = BytecodeWriterHelper::new()
    //             .append_opcode_i32(Opcode::imm_i32, 1)
    //             .append_opcode(Opcode::memory_resize)
    //             // .append_opcode(Opcode::drop)
    //             //
    //             .append_opcode_i64(Opcode::imm_i64, 0x100)
    //             .append_opcode_i32(Opcode::imm_i32, 0x07050302)
    //             .append_opcode_i16(Opcode::memory_store_i32, 0)
    //             //
    //             .append_opcode_i64(Opcode::imm_i64, 0x200)
    //             .append_opcode_i64(Opcode::imm_i64, 0x3731292319171311)
    //             .append_opcode_i16(Opcode::memory_store_i64, 0)
    //             //
    //             .append_opcode_i64(Opcode::imm_i64, 0x100)
    //             .append_opcode_i16(Opcode::host_addr_memory, 0)
    //             .append_opcode_i64(Opcode::imm_i64, 0x100)
    //             .append_opcode_i16(Opcode::host_addr_memory, 2)
    //             //
    //             .append_opcode_i64(Opcode::imm_i64, 0x200)
    //             .append_opcode_i16(Opcode::host_addr_memory, 0)
    //             .append_opcode_i64(Opcode::imm_i64, 0x200)
    //             .append_opcode_i16(Opcode::host_addr_memory, 4)
    //             .append_opcode_i64(Opcode::imm_i64, 0x200)
    //             .append_opcode_i16(Opcode::host_addr_memory, 7)
    //             //
    //             .append_opcode(Opcode::end)
    //             .to_bytes();
    //
    //         println!("{}", format_bytecode_as_text(&code0));
    //
    //         let binary0 = helper_build_module_binary_with_single_function(
    //             &[], // params
    //             &[
    //                 OperandDataType::I64,
    //                 OperandDataType::I64,
    //                 OperandDataType::I64,
    //                 OperandDataType::I64,
    //                 OperandDataType::I64,
    //             ], // results
    //             &[], // local variables
    //             code0,
    //         );
    //
    //         let handler = Handler::new();
    //         let resource0 = InMemoryProgramSource::new(vec![binary0]);
    //         let process_context0 = resource0.create_process_context().unwrap();
    //         let mut thread_context0 = process_context0.create_thread_context();
    //
    //         let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
    //         let fvs = result0.unwrap();
    //
    //         assert_eq!(read_memory_i32(fvs[0]), 0x07050302);
    //         assert_eq!(read_memory_i16(fvs[1]), 0x0705);
    //         assert_eq!(read_memory_i64(fvs[2]), 0x3731292319171311);
    //         assert_eq!(read_memory_i32(fvs[3]), 0x37312923);
    //         assert_eq!(read_memory_i8(fvs[4]), 0x37);
    //     }
    //
    //     #[test]
    //     fn test_handler_host_memory_and_vm_memory_copy() {
    //         // fn(src_ptr, dst_ptr) -> ()
    //
    //         // copy src_ptr -> VM heap 0x100 with 8 bytes
    //         // copy VM heap 0x100 -> dst_ptr with 8 bytes
    //         //
    //         //               0x100                        dst_ptr
    //         //            vm |01234567| --> copy --> host |01234567|
    //         //                ^
    //         //       /--copy--/
    //         //       |
    //         // host |01234567|
    //         //      src_ptr
    //
    //         let code0 = BytecodeWriterHelper::new()
    //             .append_opcode_i32(Opcode::imm_i32, 1)
    //             .append_opcode(Opcode::memory_resize)
    //             // .append_opcode(Opcode::drop)
    //             //
    //             .append_opcode_i64(Opcode::imm_i64, 0x100)
    //             .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0)
    //             .append_opcode_i64(Opcode::imm_i64, 8)
    //             .append_opcode(Opcode::host_copy_to_memory)
    //             //
    //             .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
    //             .append_opcode_i64(Opcode::imm_i64, 0x100)
    //             .append_opcode_i64(Opcode::imm_i64, 8)
    //             .append_opcode(Opcode::host_copy_from_memory)
    //             //
    //             .append_opcode(Opcode::end)
    //             .to_bytes();
    //
    //         let binary0 = helper_build_module_binary_with_single_function(
    //             &[OperandDataType::I64, OperandDataType::I64], // params
    //             &[],                                           // results
    //             &[],                                           // local variables
    //             code0,
    //         );
    //
    //         let handler = Handler::new();
    //         let resource0 = InMemoryProgramSource::new(vec![binary0]);
    //         let process_context0 = resource0.create_process_context().unwrap();
    //         let mut thread_context0 = process_context0.create_thread_context();
    //
    //         let src_buf: &[u8; 8] = b"hello.vm";
    //         let dst_buf: [u8; 8] = [0; 8];
    //
    //         let src_ptr = src_buf.as_ptr();
    //         let dst_ptr = dst_buf.as_ptr();
    //
    //         let result0 = process_function(
    //             &handler,
    //             &mut thread_context0,
    //             0,
    //             0,
    //             &[
    //                 ForeignValue::U64(src_ptr as usize as u64),
    //                 ForeignValue::U64(dst_ptr as usize as u64),
    //             ],
    //         );
    //         result0.unwrap();
    //
    //         assert_eq!(&dst_buf, b"hello.vm");
    //     }
    //
    //     #[test]
    //     fn test_handler_host_external_memory_copy() {
    //         // fn(src_ptr, dst_ptr) -> ()
    //
    //         let code0 = BytecodeWriterHelper::new()
    //             .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1) // dst ptr
    //             .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0) // src ptr
    //             .append_opcode_i64(Opcode::imm_i64, 8) // length
    //             .append_opcode(Opcode::host_external_memory_copy)
    //             //
    //             .append_opcode(Opcode::end)
    //             .to_bytes();
    //
    //         let binary0 = helper_build_module_binary_with_single_function(
    //             &[OperandDataType::I64, OperandDataType::I64], // params
    //             &[],                                           // results
    //             &[LocalVariableEntry::from_i64()],             // local variables
    //             code0,
    //         );
    //
    //         let handler = Handler::new();
    //         let resource0 = InMemoryProgramSource::new(vec![binary0]);
    //         let process_context0 = resource0.create_process_context().unwrap();
    //         let mut thread_context0 = process_context0.create_thread_context();
    //
    //         let src_buf: &[u8; 8] = b"whatever";
    //         let dst_buf: [u8; 8] = [0; 8];
    //
    //         let src_ptr = src_buf.as_ptr();
    //         let dst_ptr = dst_buf.as_ptr();
    //
    //         let result0 = process_function(
    //             &handler,
    //             &mut thread_context0,
    //             0,
    //             0,
    //             &[
    //                 ForeignValue::U64(src_ptr as usize as u64),
    //                 ForeignValue::U64(dst_ptr as usize as u64),
    //             ],
    //         );
    //         result0.unwrap();
    //
    //         assert_eq!(&dst_buf, b"whatever");
    //     }

    #[test]
    fn test_handler_host_addr_function_and_callback_function() {
        // the external function (a C function) in "libtest0.so.1":
        //
        // int do_something(int (*callback_function)(int), int a, int b)
        // {
        //     int s = (callback_function)(a);
        //     return s + b;
        // }
        //
        // VM functions
        //
        // ;; entry function
        // fn function0 (a:i32, b:i32)->i32 {
        //     do_something(function1, a, b)
        // }
        //
        // ;; used as callback function for external function 'do_something'
        // fn function1 (a:i32) -> i32 {
        //     a*2
        // }
        //
        // calling path:
        // (11,13) ->
        //   function0 (VM) ->
        //     do_something (external function) ->
        //       function1 (call from external) ->
        //     return to do_something ->
        //   return to function0 ->
        // return (11*2+13)

        // VM function 0
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::host_addr_function, 1) // get host address of the func1
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 0) // external func param 1
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 1) // external func param 2
            //
            .append_opcode_i32(Opcode::extcall, 0) // call external function, external function index = 0
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        // VM function 1
        let code1 = BytecodeWriterHelper::new()
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 0)
            .append_opcode_i32(Opcode::imm_i32, 2)
            .append_opcode(Opcode::mul_i32)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_data_and_external_functions(
            &[
                HelperFunctionEntry {
                    // type_index: 1,
                    params: vec![OperandDataType::I32, OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code0,
                },
                HelperFunctionEntry {
                    // type_index: 2,
                    params: vec![OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                    local_variable_item_entries_without_args: vec![],
                    code: code1,
                },
            ],
            &[],
            &[],
            &[],
            &[ExternalLibraryEntry::new(
                "libtest0".to_owned(),
                Box::new(ExternalLibraryDependency::Local(Box::new(
                    DependencyLocal {
                        path: "lib/libtest0.so.1".to_owned(),
                        condition: DependencyCondition::True,
                        parameters: HashMap::default(),
                    },
                ))),
            )],
            &[HelperExternalFunctionEntry {
                name: "do_something".to_string(),
                external_library_index: 0,
                params: vec![
                    OperandDataType::I64,
                    OperandDataType::I32,
                    OperandDataType::I32,
                ],
                result: Some(OperandDataType::I32),
            }],
        );

        let mut pwd = std::env::current_dir().unwrap();
        // let pkg_name = env!("CARGO_PKG_NAME");
        let crate_folder_name = "processor";
        if !pwd.ends_with(crate_folder_name) {
            // in the VSCode `Debug` environment, the `current_dir()`
            // the project root folder.
            // while in both `$ cargo test` and VSCode `Run Test` environment
            // the `current_dir()` return the current crate path.
            pwd.push("crates");
            pwd.push(crate_folder_name);
        }
        pwd.push("tests");
        // let application_path = pwd.to_str().unwrap();

        let handler = Handler::new();
        let resource0 = InMemoryProgramSource::with_property(
            vec![binary0],
            ProcessProperty::new(
                pwd,
                ProgramSourceType::Module,
                vec![],
                HashMap::<String, String>::new(),
            ),
        );
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &handler,
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(11), ForeignValue::U32(13)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(11 * 2 + 13)]);

        let result1 = process_function(
            &handler,
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(211), ForeignValue::U32(223)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::U32(211 * 2 + 223)]);
    }
}
