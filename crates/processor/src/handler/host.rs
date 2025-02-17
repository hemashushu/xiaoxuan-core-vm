// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use anc_context::{memory_access::MemoryAccess, thread_context::ThreadContext};

use crate::{
    bridge_handler::get_or_create_bridge_callback_function,
    PANIC_CODE_BRIDGE_FUNCTION_CREATE_FAILURE,
};

use super::{HandleResult, Handler};

pub fn panic(_handler: &Handler, thread: &mut ThreadContext) -> HandleResult {
    let code = thread.get_param_i32();
    HandleResult::Panic(code)
}

// pub fn unreachable(thread: &mut ThreadContext) -> HandleResult {
//     let code = thread.get_param_i32();
//     HandleResult::Unreachable(code)
// }
//
// pub fn debug(thread: &mut ThreadContext) -> HandleResult {
//     let code = thread.get_param_i32();
//     HandleResult::Debug(code)
// }

pub fn host_addr_local(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> i64
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_host_addr_local(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn host_addr_local_extend(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32) -> i64
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_host_addr_local(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_host_addr_local(
    thread_context: &mut ThreadContext,
    reversed_index: u16,
    local_variable_index: usize,
    offset_bytes: usize,
) -> HandleResult {
    let final_offset = thread_context
        .get_local_variable_address_by_index_and_offset_with_bounds_check(
            reversed_index,
            local_variable_index,
            offset_bytes,
            0,
        );
    let ptr = thread_context.stack.get_ptr(final_offset);
    store_pointer_to_operand_stack(thread_context, ptr);
    HandleResult::Move(8)
}

pub fn host_addr_data(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32) -> i64
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_host_addr_data(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn host_addr_data_extend(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i32) -> i64
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_host_addr_data(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

fn do_host_addr_data(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
) -> HandleResult {
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_data_target_module_index_and_internal_index_and_data_object_with_bounds_check(
            thread_context.pc.module_index,
            data_public_index,
            0,
            0,
        );
    let total_offset =
        data_object.get_data_address_by_index_and_offset(data_internal_index, offset_bytes);
    let ptr = data_object.get_ptr(total_offset);
    store_pointer_to_operand_stack(thread_context, ptr);
    HandleResult::Move(8)
}

pub fn host_addr_memory(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16) (operand memory_address:i64) -> i64
    let offset_bytes = thread_context.get_param_i16();
    let memory_address = thread_context.stack.pop_i64_u();

    let total_offset = memory_address as usize + offset_bytes as usize;
    let ptr = thread_context.memory.get_ptr(total_offset);
    store_pointer_to_operand_stack(thread_context, ptr);
    HandleResult::Move(4)
}

pub fn host_copy_from_memory(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // copy data from VM heap to host memory
    // () (operand dst_pointer:i64 src_addr:i64 count:i64) -> ()

    let count = thread_context.stack.pop_i64_u();
    let src_memory_address = thread_context.stack.pop_i64_u();
    let dst_host_ptr = thread_context.stack.pop_i64_u();

    let src_heap_ptr = thread_context.memory.get_ptr(src_memory_address as usize);
    unsafe { std::ptr::copy(src_heap_ptr, dst_host_ptr as *mut u8, count as usize) };

    HandleResult::Move(2)
}

pub fn host_copy_to_memory(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // copy data from host memory to VM heap
    // () (operand dst_addr:i64 src_pointer:i64 count:i64) -> ()

    let count = thread_context.stack.pop_i64_u();
    let src_host_ptr = thread_context.stack.pop_i64_u();
    let dst_memory_address = thread_context.stack.pop_i64_u();

    let dst_heap_ptr = thread_context
        .memory
        .get_mut_ptr(dst_memory_address as usize);
    unsafe { std::ptr::copy(src_host_ptr as *const u8, dst_heap_ptr, count as usize) };

    HandleResult::Move(2)
}

pub fn host_external_memory_copy(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // copy data between host memory
    // (operand dst_pointer:i64 src_pointer:i64 count:i64)

    let count = thread_context.stack.pop_i64_u();
    let src_host_ptr = thread_context.stack.pop_i64_u();
    let dst_host_ptr = thread_context.stack.pop_i64_u();

    unsafe {
        std::ptr::copy(
            src_host_ptr as *const u8,
            dst_host_ptr as *mut u8,
            count as usize,
        )
    };

    HandleResult::Move(2)
}

fn store_pointer_to_operand_stack(thread_context: &mut ThreadContext, ptr: *const u8) {
    let address = ptr as u64;
    thread_context.stack.push_i64_u(address);
}

pub fn host_addr_function(handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param function_public_index:i32) -> i64/i32

    let function_public_index = thread_context.get_param_i32() as usize;
    let module_index = thread_context.pc.module_index;

    if let Ok(callback_function_ptr) = get_or_create_bridge_callback_function(
        handler,
        thread_context,
        module_index,
        function_public_index,
    ) {
        store_pointer_to_operand_stack(thread_context, callback_function_ptr);
        HandleResult::Move(8)
    } else {
        HandleResult::Panic(PANIC_CODE_BRIDGE_FUNCTION_CREATE_FAILURE)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anc_context::{process_property::ProcessProperty, process_resource::ProcessResource};
    use anc_image::{
        bytecode_reader::format_bytecode_as_text,
        bytecode_writer::BytecodeWriterHelper,
        entry::{ExternalLibraryEntry, InitedDataEntry, LocalVariableEntry, UninitDataEntry},
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
        handler::Handler, in_memory_process_resource::InMemoryProcessResource,
        process::process_function, HandleErrorType, HandlerError,
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
    fn test_handler_host_panic() {
        // () -> ()
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::panic, 0x101)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[], // params
            &[], // results
            &[], // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryProcessResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();

        let mut thread_context0 = process_context0.create_thread_context();
        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);

        assert!(matches!(
            result0,
            Err(HandlerError {
                error_type: HandleErrorType::Panic(0x101)
            })
        ));
    }

    #[test]
    fn test_handler_host_address_of_data_and_local_variables() {
        //        read-only data section
        //        ======================
        //
        //       |low address    high addr|
        //       |                        |
        // index |0              1        |
        //  type |i32------|    |i32------|
        //
        //  data 11 00 00 00    13 00 00 00
        //
        //        read write data section
        //        =======================
        //
        //       |low address             high address|
        //       |                                    |
        // index |2(0)                       3(1)     |
        //  type |i64------------------|    |i32------|
        //
        //  data 17 00 00 00 00 00 00 00    19 00 00 00
        //
        //        uninitialized data section
        //        ==========================
        //
        //       |low address             high address|
        //       |                                    |
        // index |4(0)           5(1)                 |
        //  type |i32------|    |i64------------------|
        //
        //  data 23 00 00 00    29 00 00 00 00 00 00 00
        //
        //        local variable area
        //        ===================
        //
        //       |low address                                       high addr|
        //       |                                                           |
        // index |0       1                           2                      |
        //  type |bytes| |i32------|   |padding--|   |i32------|   |padding--|
        //
        //  data 0.....0 31 00 00 00   00 00 00 00   37 00 00 00   00 00 00 00
        //       ^
        //       | 64 bytes, the space for storing function results.
        //       | because the results will overwrite the stack, so it need to
        //       | leave enough space for results, then the data of local variables
        //       | can be still read after function is finish.
        //
        // () -> (i64,i64,i64,i64,i64,i64, i64,i64)
        //        -----------------------  -------
        //        | addr of data           | addr of local variables
        //
        // read the values of data and local variables through the host address.

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i64(Opcode::imm_i64, 0x17)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 2)
            //
            .append_opcode_i32(Opcode::imm_i32, 0x19)
            .append_opcode_i16_i32(Opcode::data_store_i32, 0, 3)
            //
            .append_opcode_i32(Opcode::imm_i32, 0x23)
            .append_opcode_i16_i32(Opcode::data_store_i32, 0, 4)
            //
            .append_opcode_i64(Opcode::imm_i64, 0x29)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 5)
            //
            .append_opcode_i32(Opcode::imm_i32, 0x31)
            .append_opcode_i16_i16_i16(Opcode::local_store_i32, 0, 0, 1)
            .append_opcode_i32(Opcode::imm_i32, 0x37)
            .append_opcode_i16_i16_i16(Opcode::local_store_i32, 0, 0, 2)
            //
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 0)
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 1)
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 2)
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 3)
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 4)
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 5)
            //
            .append_opcode_i16_i16_i16(Opcode::host_addr_local, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::host_addr_local, 0, 0, 2)
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
                OperandDataType::I64,
                OperandDataType::I64,
            ], // results
            &[
                LocalVariableEntry::from_bytes(64, 8), // space
                LocalVariableEntry::from_i32(),
                LocalVariableEntry::from_i32(),
            ], // local variables
            code0,
            &[
                InitedDataEntry::from_i32(0x11), // ro, data idx: 0
                InitedDataEntry::from_i32(0x13), // ro, data idx: 1
            ],
            &[
                InitedDataEntry::from_i64(0xee), // rw, data idx: 2
                InitedDataEntry::from_i32(0xff), // rw, data idx: 3
            ],
            &[
                UninitDataEntry::from_i32(), // bss, data idx: 4
                UninitDataEntry::from_i64(), // bss, data idx: 5
            ],
        );

        let handler = Handler::new();
        let resource0 = InMemoryProcessResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        let fvs = result0.unwrap();

        assert_eq!(read_memory_i32(fvs[0]), 0x11);
        assert_eq!(read_memory_i32(fvs[1]), 0x13);
        assert_eq!(read_memory_i64(fvs[2]), 0x17);
        assert_eq!(read_memory_i32(fvs[3]), 0x19);
        assert_eq!(read_memory_i32(fvs[4]), 0x23);
        assert_eq!(read_memory_i64(fvs[5]), 0x29);

        // note:
        // depending on the implementation of the stack (the stack frame and local variables),
        // the following 'assert_eq' may fail,
        // because the local variables (as well as their host addresses) will no longer valid
        // when a function exits.

        assert_eq!(read_memory_i32(fvs[6]), 0x31);
        assert_eq!(read_memory_i32(fvs[7]), 0x37);
    }

    #[test]
    fn test_handler_host_address_of_data_and_local_variables_extend() {
        //        read-only data section
        //        ======================
        //
        //       |low address  high addr|
        //       |                      |
        // index |0            1        |
        //  type |bytes----|  |byte-----|
        //
        //  data 02 03 05 07  11 13 17 19
        //       |     |            |  |
        //       |0    |1           |2 |3
        //
        //        local variable area
        //        ===================
        //
        //       |low address         high addr|
        //       |                             |
        // index |0       1                    |
        //  type |bytes| |bytes----------------|
        //
        //  data 0.....0 23 29 31 37 41 43 47 53
        //       ^       |        |        |  |
        //       |       |4       |5       |6 |7
        //       |
        //       | 64 bytes, the space for storing function results.
        //       | because the results will overwrite the stack, so it need to
        //       | leave enough space for results, then the data of local variables
        //       | can be still read after function is finish.
        //
        // () -> (i64,i64,i64,i64, i64,i64, i64,i64)
        //        ---------------- ----------------
        //        | addr of data   | addr of local variables
        //
        // read the values of data and local variables through the host address.

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i64(Opcode::imm_i64, 0x5347434137312923u64)
            .append_opcode_i16_i16_i16(Opcode::local_store_i64, 0, 0, 1)
            //
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i32(Opcode::host_addr_data_extend, 0)
            .append_opcode_i32(Opcode::imm_i32, 2)
            .append_opcode_i32(Opcode::host_addr_data_extend, 0)
            .append_opcode_i32(Opcode::imm_i32, 2)
            .append_opcode_i32(Opcode::host_addr_data_extend, 1)
            .append_opcode_i32(Opcode::imm_i32, 3)
            .append_opcode_i32(Opcode::host_addr_data_extend, 1)
            //
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i16_i32(Opcode::host_addr_local_extend, 0, 1)
            .append_opcode_i32(Opcode::imm_i32, 3)
            .append_opcode_i16_i32(Opcode::host_addr_local_extend, 0, 1)
            .append_opcode_i32(Opcode::imm_i32, 6)
            .append_opcode_i16_i32(Opcode::host_addr_local_extend, 0, 1)
            .append_opcode_i32(Opcode::imm_i32, 7)
            .append_opcode_i16_i32(Opcode::host_addr_local_extend, 0, 1)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        println!("{}", format_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
            ], // results
            &[
                LocalVariableEntry::from_bytes(64, 8), // space
                LocalVariableEntry::from_bytes(8, 8),
            ], // local variables
            code0,
            &[
                InitedDataEntry::from_bytes(vec![0x02u8, 0x03, 0x05, 0x07], 4), // init data
                InitedDataEntry::from_bytes(vec![0x11u8, 0x13, 0x17, 0x19], 4), // init data
            ], // init data
            &[],
            &[],
        );

        let handler = Handler::new();
        let resource0 = InMemoryProcessResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        let fvs = result0.unwrap();

        assert_eq!(read_memory_i8(fvs[0]), 0x02);
        assert_eq!(read_memory_i8(fvs[1]), 0x05);
        assert_eq!(read_memory_i8(fvs[2]), 0x17);
        assert_eq!(read_memory_i8(fvs[3]), 0x19);

        // note:
        // depending on the implementation of the stack (the stack frame and local variables),
        // the following 'assert_eq' may fail,
        // because the local variables (as well as their host addresses) will no longer valid
        // when a function exits.

        assert_eq!(read_memory_i8(fvs[4]), 0x23);
        assert_eq!(read_memory_i8(fvs[5]), 0x37);
        assert_eq!(read_memory_i8(fvs[6]), 0x47);
        assert_eq!(read_memory_i8(fvs[7]), 0x53);
    }

    #[test]
    fn test_handler_host_address_memory() {
        //        heap
        //       |low address                high addr|
        //       |                                    |
        //  addr |0x100         0x200                 |
        //  type |i32-------|   |i64------------------|
        //
        //  data  02 03 05 07   11 13 17 19 23 29 31 37
        //        ^     ^       ^           ^        ^
        //        |0    |1      |2          |3       |4
        //
        // () -> (i64,i64,i64,i64,i64)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 1)
            .append_opcode(Opcode::memory_resize)
            // .append_opcode(Opcode::drop)
            //
            .append_opcode_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i32(Opcode::imm_i32, 0x07050302)
            .append_opcode_i16(Opcode::memory_store_i32, 0)
            //
            .append_opcode_i64(Opcode::imm_i64, 0x200)
            .append_opcode_i64(Opcode::imm_i64, 0x3731292319171311)
            .append_opcode_i16(Opcode::memory_store_i64, 0)
            //
            .append_opcode_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::host_addr_memory, 0)
            .append_opcode_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::host_addr_memory, 2)
            //
            .append_opcode_i64(Opcode::imm_i64, 0x200)
            .append_opcode_i16(Opcode::host_addr_memory, 0)
            .append_opcode_i64(Opcode::imm_i64, 0x200)
            .append_opcode_i16(Opcode::host_addr_memory, 4)
            .append_opcode_i64(Opcode::imm_i64, 0x200)
            .append_opcode_i16(Opcode::host_addr_memory, 7)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        println!("{}", format_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function(
            &[], // params
            &[
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
            ], // results
            &[], // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryProcessResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        let fvs = result0.unwrap();

        assert_eq!(read_memory_i32(fvs[0]), 0x07050302);
        assert_eq!(read_memory_i16(fvs[1]), 0x0705);
        assert_eq!(read_memory_i64(fvs[2]), 0x3731292319171311);
        assert_eq!(read_memory_i32(fvs[3]), 0x37312923);
        assert_eq!(read_memory_i8(fvs[4]), 0x37);
    }

    #[test]
    fn test_handler_host_memory_and_vm_memory_copy() {
        // fn(src_ptr, dst_ptr) -> ()

        // copy src_ptr -> VM heap 0x100 with 8 bytes
        // copy VM heap 0x100 -> dst_ptr with 8 bytes
        //
        //               0x100                        dst_ptr
        //            vm |01234567| --> copy --> host |01234567|
        //                ^
        //       /--copy--/
        //       |
        // host |01234567|
        //      src_ptr

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 1)
            .append_opcode(Opcode::memory_resize)
            // .append_opcode(Opcode::drop)
            //
            .append_opcode_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0)
            .append_opcode_i64(Opcode::imm_i64, 8)
            .append_opcode(Opcode::host_copy_to_memory)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i64(Opcode::imm_i64, 8)
            .append_opcode(Opcode::host_copy_from_memory)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[OperandDataType::I64, OperandDataType::I64], // params
            &[],                                           // results
            &[],                                           // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryProcessResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let src_buf: &[u8; 8] = b"hello.vm";
        let dst_buf: [u8; 8] = [0; 8];

        let src_ptr = src_buf.as_ptr();
        let dst_ptr = dst_buf.as_ptr();

        let result0 = process_function(
            &handler,
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::U64(src_ptr as usize as u64),
                ForeignValue::U64(dst_ptr as usize as u64),
            ],
        );
        result0.unwrap();

        assert_eq!(&dst_buf, b"hello.vm");
    }

    #[test]
    fn test_handler_host_external_memory_copy() {
        // fn(src_ptr, dst_ptr) -> ()

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1) // dst ptr
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0) // src ptr
            .append_opcode_i64(Opcode::imm_i64, 8) // length
            .append_opcode(Opcode::host_external_memory_copy)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[OperandDataType::I64, OperandDataType::I64], // params
            &[],                                           // results
            &[LocalVariableEntry::from_i64()],             // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryProcessResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let src_buf: &[u8; 8] = b"whatever";
        let dst_buf: [u8; 8] = [0; 8];

        let src_ptr = src_buf.as_ptr();
        let dst_ptr = dst_buf.as_ptr();

        let result0 = process_function(
            &handler,
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::U64(src_ptr as usize as u64),
                ForeignValue::U64(dst_ptr as usize as u64),
            ],
        );
        result0.unwrap();

        assert_eq!(&dst_buf, b"whatever");
    }

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
        let resource0 = InMemoryProcessResource::with_property(
            vec![binary0],
            &ProcessProperty::new(
                pwd,
                false,
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
