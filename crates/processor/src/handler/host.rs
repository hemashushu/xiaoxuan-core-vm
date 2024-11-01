// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// about the callback function
//
// on the XiaoXuan Core application, pass VM function as a callback function to the external C library.
//
//                                      runtime (native)
//                                   /------------------------\
//                                   |                        |
//                                   | external func list     |
//                                   | |--------------------| |
//                                   | | idx | lib  | name  | |
//                              /--> | | 0   | ".." | ".."  | |
//                              |    | |--------------------| |
//                              |    |                        |
//                              |    | wrapper func code 0    |
//  XiaoXuan core application   |    | 0x0000 0xb8, 0x34,     |
// /------------------------\   |    | 0x000a 0x12, 0x00...   | --\
// |                        |   |    |                        |   |
// | fn $demo () -> ()      |   |    |                        |   |
// |   extcall do_something | --/    | callback func table    |   |
// | end                    |        | |--------------------| |   |
// |                        |        | | mod idx | func idx | |   |      libxyz.so
// | fn $callback () -> ()  | <----- | | 0       | 0        | |   |    /----------------------\
// |   ...                  |        | | ...     | ...      | |   \--> | void do_something (  |
// | end                    |        | |--------------------| |        |     void* () cb) {   |
// |                        |        |                        |        |     ...              |
// \------------------------/        | bridge func code 0     | <----- |     (cb)(11, 13)     |
//                                   | 0x0000 0xb8, 0x34,     |        | }                    |
//                                   | 0x000a 0x12, 0x00...   |        |                      |
//                                   |                        |        \----------------------/
//                                   | bridge func code 1     |
//                                   | ...                    |
//                                   |                        |
//                                   \------------------------/
//

use ancvm_context::{memory::Memory, thread_context::ThreadContext};

use crate::delegate::build_callback_function;

use super::HandleResult;

pub fn panic(_thread: &mut ThreadContext) -> HandleResult {
    HandleResult::Panic
}

pub fn unreachable(thread: &mut ThreadContext) -> HandleResult {
    let code = thread.get_param_i32();
    HandleResult::Unreachable(code)
}

pub fn debug(thread: &mut ThreadContext) -> HandleResult {
    let code = thread.get_param_i32();
    HandleResult::Debug(code)
}

pub fn host_addr_local(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_host_addr_local(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn host_addr_local_offset(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
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

pub fn host_addr_data(thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_host_addr_data(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn host_addr_data_offset(thread_context: &mut ThreadContext) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
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

pub fn host_addr_heap(thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread_context.get_param_i16();
    let heap_address = thread_context.stack.pop_i64_u();

    let total_offset = heap_address as usize + offset_bytes as usize;
    let ptr = thread_context.heap.get_ptr(total_offset);
    store_pointer_to_operand_stack(thread_context, ptr);
    HandleResult::Move(4)
}

pub fn host_copy_heap_to_memory(thread_context: &mut ThreadContext) -> HandleResult {
    // copy data from VM heap to host memory
    // (operand dst_pointer:i64 src_offset:i64 length_in_bytes:i64) -> ()

    let length_in_bytes = thread_context.stack.pop_i64_u();
    let src_heap_address = thread_context.stack.pop_i64_u();
    let dst_host_ptr = thread_context.stack.pop_i64_u();

    let src_heap_ptr = thread_context.heap.get_ptr(src_heap_address as usize);
    unsafe {
        std::ptr::copy(
            src_heap_ptr,
            dst_host_ptr as *mut u8,
            length_in_bytes as usize,
        )
    };

    HandleResult::Move(2)
}

pub fn host_copy_memory_to_heap(thread_context: &mut ThreadContext) -> HandleResult {
    // copy data from host memory to VM heap
    // (operand dst_offset:i64 src_pointer:i64 length_in_bytes:i64)

    let length_in_bytes = thread_context.stack.pop_i64_u();
    let src_host_ptr = thread_context.stack.pop_i64_u();
    let dst_heap_address = thread_context.stack.pop_i64_u();

    let dst_heap_ptr = thread_context.heap.get_mut_ptr(dst_heap_address as usize);
    unsafe {
        std::ptr::copy(
            src_host_ptr as *const u8,
            dst_heap_ptr,
            length_in_bytes as usize,
        )
    };

    HandleResult::Move(2)
}

pub fn host_memory_copy(thread_context: &mut ThreadContext) -> HandleResult {
    // copy data between host memory
    // (operand dst_pointer:i64 src_pointer:i64 length_in_bytes:i64)

    let length_in_bytes = thread_context.stack.pop_i64_u();
    let src_host_ptr = thread_context.stack.pop_i64_u();
    let dst_host_ptr = thread_context.stack.pop_i64_u();

    unsafe {
        std::ptr::copy(
            src_host_ptr as *const u8,
            dst_host_ptr as *mut u8,
            length_in_bytes as usize,
        )
    };

    HandleResult::Move(2)
}

fn store_pointer_to_operand_stack(thread_context: &mut ThreadContext, ptr: *const u8) {
    #[cfg(target_pointer_width = "64")]
    {
        let address = ptr as u64;
        thread_context.stack.push_i64_u(address);
    }

    #[cfg(target_pointer_width = "32")]
    {
        let address = ptr as u32;
        thread_context.stack.push_i32_u(address);
    }
}

pub fn host_addr_function(thread_context: &mut ThreadContext) -> HandleResult {
    // (param function_public_index:i32) -> i64/i32

    let function_public_index = thread_context.get_param_i32() as usize;
    let module_index = thread_context.pc.module_index;

    if let Ok(callback_function_ptr) =
        build_callback_function(thread_context, module_index, function_public_index)
    {
        store_pointer_to_operand_stack(thread_context, callback_function_ptr);
        HandleResult::Move(8)
    } else {
        HandleResult::Panic
    }
}

#[cfg(test)]
mod tests {

    use ancvm_binary::{
        bytecode_reader::format_bytecode_as_text,
        bytecode_writer::BytecodeWriter,
        utils::{
            helper_build_module_binary_with_functions_and_external_functions,
            helper_build_module_binary_with_single_function,
            helper_build_module_binary_with_single_function_and_data_sections,
            HelperExternalFunctionEntry, HelperFunctionWithCodeAndLocalVariablesEntry,
        },
    };

    use crate::{
        in_memory_program_resource::InMemoryProgramResource, interpreter::process_function,
        InterpreterError, InterpreterErrorType,
    };
    use ancvm_context::{program_resource::ProgramResource, environment::ProgramSettings};
    use ancvm_isa::{
        entry::{InitedDataEntry, LocalVariableEntry, TypeEntry, UninitDataEntry},
        opcode::Opcode,
        OperandDataType, ExternalLibraryType, ForeignValue,
    };

    fn read_memory_i64(fv: ForeignValue) -> u64 {
        #[cfg(target_pointer_width = "64")]
        if let ForeignValue::U64(addr) = fv {
            let ptr = addr as *const u64;
            unsafe { std::ptr::read(ptr) }
        } else {
            panic!("The data type of the foreign value does not match.")
        }
        #[cfg(target_pointer_width = "32")]
        if let ForeignValue::U32(addr) = fv {
            let ptr = addr as *const u64;
            unsafe { std::ptr::read(ptr) }
        } else {
            panic!("The data type of the foreign value does not match.")
        }
    }

    fn read_memory_i32(fv: ForeignValue) -> u32 {
        #[cfg(target_pointer_width = "64")]
        if let ForeignValue::U64(addr) = fv {
            let ptr = addr as *const u32;
            unsafe { std::ptr::read(ptr) }
        } else {
            panic!("The data type of the foreign value does not match.")
        }
        #[cfg(target_pointer_width = "32")]
        if let ForeignValue::U32(addr) = fv {
            let ptr = addr as *const u32;
            unsafe { std::ptr::read(ptr) }
        } else {
            panic!("The data type of the foreign value does not match.")
        }
    }

    fn read_memory_i16(fv: ForeignValue) -> u16 {
        #[cfg(target_pointer_width = "64")]
        if let ForeignValue::U64(addr) = fv {
            let ptr = addr as *const u16;
            unsafe { std::ptr::read(ptr) }
        } else {
            panic!("The data type of the foreign value does not match.")
        }
        #[cfg(target_pointer_width = "32")]
        if let ForeignValue::U32(addr) = fv {
            let ptr = addr as *const u16;
            unsafe { std::ptr::read(ptr) }
        } else {
            panic!("The data type of the foreign value does not match.")
        }
    }

    fn read_memory_i8(fv: ForeignValue) -> u8 {
        #[cfg(target_pointer_width = "64")]
        if let ForeignValue::U64(addr) = fv {
            let ptr = addr as *const u8;
            unsafe { std::ptr::read(ptr) }
        } else {
            panic!("The data type of the foreign value does not match.")
        }
        #[cfg(target_pointer_width = "32")]
        if let ForeignValue::U32(addr) = fv {
            let ptr = addr as *const u8;
            unsafe { std::ptr::read(ptr) }
        } else {
            panic!("The data type of the foreign value does not match.")
        }
    }

    #[test]
    fn test_interpreter_host_panic() {
        // () -> ()

        let code0 = BytecodeWriterHelper::new()
            .append_opcode(Opcode::nop)
            .append_opcode(Opcode::panic)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![], // params
            vec![], // results
            vec![], // local vars
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();

        let mut thread_context0 = process_context0.create_thread_context();
        let result0 = process_function(&mut thread_context0, 0, 0, &[]);

        assert!(matches!(
            result0,
            Err(InterpreterError {
                error_type: InterpreterErrorType::Panic
            })
        ));
    }

    #[test]
    fn test_interpreter_host_debug() {
        // () -> ()

        let code0 = BytecodeWriterHelper::new()
            .append_opcode(Opcode::nop)
            .append_opcode_i32(Opcode::debug, 0x101)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![], // params
            vec![], // results
            vec![], // local vars
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();

        let mut thread_context0 = process_context0.create_thread_context();
        let result0 = process_function(&mut thread_context0, 0, 0, &[]);

        assert!(matches!(
            result0,
            Err(InterpreterError {
                error_type: InterpreterErrorType::Debug(0x101)
            })
        ));
    }

    #[test]
    fn test_interpreter_host_unreachable() {
        // () -> ()

        let code0 = BytecodeWriterHelper::new()
            .append_opcode(Opcode::nop)
            .append_opcode_i32(Opcode::unreachable, 0x103)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![], // params
            vec![], // results
            vec![], // local vars
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();

        let mut thread_context0 = process_context0.create_thread_context();
        let result0 = process_function(&mut thread_context0, 0, 0, &[]);

        assert!(matches!(
            result0,
            Err(InterpreterError {
                error_type: InterpreterErrorType::Unreachable(0x103)
            })
        ));
    }

    #[test]
    fn test_interpreter_host_address_of_data_and_local_vars() {
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
        //        | addr of data           | addr of local vars
        //
        // read the values of data and local vars through the host address.

        // bytecode:
        //
        // 0x0000  81 01 00 00  17 00 00 00    i64.imm           low:0x00000017  high:0x00000000
        //         00 00 00 00
        // 0x000c  08 03 00 00  02 00 00 00    data.store64      off:0x00  idx:2
        // 0x0014  80 01 00 00  19 00 00 00    i32.imm           0x00000019
        // 0x001c  09 03 00 00  03 00 00 00    data.store32      off:0x00  idx:3
        // 0x0024  80 01 00 00  23 00 00 00    i32.imm           0x00000023
        // 0x002c  09 03 00 00  04 00 00 00    data.store32      off:0x00  idx:4
        // 0x0034  81 01 00 00  29 00 00 00    i64.imm           low:0x00000029  high:0x00000000
        //         00 00 00 00
        // 0x0040  08 03 00 00  05 00 00 00    data.store64      off:0x00  idx:5
        // 0x0048  80 01 00 00  31 00 00 00    i32.imm           0x00000031
        // 0x0050  09 02 00 00  00 00 01 00    local.store32     rev:0   off:0x00  idx:1
        // 0x0058  80 01 00 00  37 00 00 00    i32.imm           0x00000037
        // 0x0060  09 02 00 00  00 00 02 00    local.store32     rev:0   off:0x00  idx:2
        // 0x0068  05 0c 00 00  00 00 00 00    host.addr_data    off:0x00  idx:0
        // 0x0070  05 0c 00 00  01 00 00 00    host.addr_data    off:0x00  idx:1
        // 0x0078  05 0c 00 00  02 00 00 00    host.addr_data    off:0x00  idx:2
        // 0x0080  05 0c 00 00  03 00 00 00    host.addr_data    off:0x00  idx:3
        // 0x0088  05 0c 00 00  04 00 00 00    host.addr_data    off:0x00  idx:4
        // 0x0090  05 0c 00 00  05 00 00 00    host.addr_data    off:0x00  idx:5
        // 0x0098  03 0c 00 00  00 00 01 00    host.addr_local   rev:0   off:0x00  idx:1
        // 0x00a0  03 0c 00 00  00 00 02 00    host.addr_local   rev:0   off:0x00  idx:2
        // 0x00a8  00 0a                       end

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x17)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 2)
            //
            .append_opcode_i32(Opcode::imm_i32, 0x19)
            .append_opcode_i16_i32(Opcode::data_store32, 0, 3)
            //
            .append_opcode_i32(Opcode::imm_i32, 0x23)
            .append_opcode_i16_i32(Opcode::data_store32, 0, 4)
            //
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x29)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 5)
            //
            .append_opcode_i32(Opcode::imm_i32, 0x31)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 0, 1)
            .append_opcode_i32(Opcode::imm_i32, 0x37)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 0, 2)
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

        println!("{}", format_bytecode_as_text(&code0));

        #[cfg(target_pointer_width = "64")]
        let result_datatypes = vec![
            OperandDataType::I64,
            OperandDataType::I64,
            OperandDataType::I64,
            OperandDataType::I64,
            OperandDataType::I64,
            OperandDataType::I64,
            OperandDataType::I64,
            OperandDataType::I64,
        ];

        #[cfg(target_pointer_width = "32")]
        let result_datatypes = vec![
            OperandDataType::I32,
            OperandDataType::I32,
            OperandDataType::I32,
            OperandDataType::I32,
            OperandDataType::I32,
            OperandDataType::I32,
            OperandDataType::I32,
            OperandDataType::I32,
        ];

        let binary0 = helper_build_module_binary_with_single_function_and_data_sections(
            vec![],           // params
            result_datatypes, // results
            vec![
                LocalVariableEntry::from_bytes(64, 8), // space
                LocalVariableEntry::from_i32(),
                LocalVariableEntry::from_i32(),
            ], // local vars
            code0,
            vec![
                InitedDataEntry::from_i32(0x11),
                InitedDataEntry::from_i32(0x13),
            ],
            vec![
                InitedDataEntry::from_i64(0xee), // init data
                InitedDataEntry::from_i32(0xff), // init data
            ],
            vec![UninitDataEntry::from_i32(), UninitDataEntry::from_i64()],
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
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
    fn test_interpreter_host_address_offset_of_data_and_local_vars() {
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
        //        | addr of data   | addr of local vars
        //
        // read the values of data and local vars through the host address.

        // bytecode:
        //
        // 0x0000  81 01 00 00  23 29 31 37    i64.imm           low:0x37312923  high:0x53474341
        //         41 43 47 53
        // 0x000c  08 02 00 00  00 00 01 00    local.store64     rev:0   off:0x00  idx:1
        // 0x0014  80 01 00 00  00 00 00 00    i32.imm           0x00000000
        // 0x001c  06 0c 00 00  00 00 00 00    host.addr_data_offset  idx:0
        // 0x0024  80 01 00 00  02 00 00 00    i32.imm           0x00000002
        // 0x002c  06 0c 00 00  00 00 00 00    host.addr_data_offset  idx:0
        // 0x0034  80 01 00 00  02 00 00 00    i32.imm           0x00000002
        // 0x003c  06 0c 00 00  01 00 00 00    host.addr_data_offset  idx:1
        // 0x0044  80 01 00 00  03 00 00 00    i32.imm           0x00000003
        // 0x004c  06 0c 00 00  01 00 00 00    host.addr_data_offset  idx:1
        // 0x0054  80 01 00 00  00 00 00 00    i32.imm           0x00000000
        // 0x005c  04 0c 00 00  01 00 00 00    host.addr_local_offset  rev:0   idx:1
        // 0x0064  80 01 00 00  03 00 00 00    i32.imm           0x00000003
        // 0x006c  04 0c 00 00  01 00 00 00    host.addr_local_offset  rev:0   idx:1
        // 0x0074  80 01 00 00  06 00 00 00    i32.imm           0x00000006
        // 0x007c  04 0c 00 00  01 00 00 00    host.addr_local_offset  rev:0   idx:1
        // 0x0084  80 01 00 00  07 00 00 00    i32.imm           0x00000007
        // 0x008c  04 0c 00 00  01 00 00 00    host.addr_local_offset  rev:0   idx:1
        // 0x0094  00 0a                       end

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x5347434137312923u64)
            .append_opcode_i16_i16_i16(Opcode::local_store_i64, 0, 0, 1)
            //
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i32(Opcode::host_addr_data_offset, 0)
            .append_opcode_i32(Opcode::imm_i32, 2)
            .append_opcode_i32(Opcode::host_addr_data_offset, 0)
            .append_opcode_i32(Opcode::imm_i32, 2)
            .append_opcode_i32(Opcode::host_addr_data_offset, 1)
            .append_opcode_i32(Opcode::imm_i32, 3)
            .append_opcode_i32(Opcode::host_addr_data_offset, 1)
            //
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i16_i32(Opcode::host_addr_local_offset, 0, 1)
            .append_opcode_i32(Opcode::imm_i32, 3)
            .append_opcode_i16_i32(Opcode::host_addr_local_offset, 0, 1)
            .append_opcode_i32(Opcode::imm_i32, 6)
            .append_opcode_i16_i32(Opcode::host_addr_local_offset, 0, 1)
            .append_opcode_i32(Opcode::imm_i32, 7)
            .append_opcode_i16_i32(Opcode::host_addr_local_offset, 0, 1)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", format_bytecode_as_text(&code0));

        #[cfg(target_pointer_width = "64")]
        let result_datatypes = vec![
            OperandDataType::I64,
            OperandDataType::I64,
            OperandDataType::I64,
            OperandDataType::I64,
            OperandDataType::I64,
            OperandDataType::I64,
            OperandDataType::I64,
            OperandDataType::I64,
        ];

        #[cfg(target_pointer_width = "32")]
        let result_datatypes = vec![
            OperandDataType::I32,
            OperandDataType::I32,
            OperandDataType::I32,
            OperandDataType::I32,
            OperandDataType::I32,
            OperandDataType::I32,
            OperandDataType::I32,
            OperandDataType::I32,
        ];

        let binary0 = helper_build_module_binary_with_single_function_and_data_sections(
            vec![],           // params
            result_datatypes, // results
            vec![
                LocalVariableEntry::from_bytes(64, 8), // space
                LocalVariableEntry::from_bytes(8, 8),
            ], // local vars
            code0,
            vec![
                InitedDataEntry::from_bytes(vec![0x02u8, 0x03, 0x05, 0x07], 4), // init data
                InitedDataEntry::from_bytes(vec![0x11u8, 0x13, 0x17, 0x19], 4), // init data
            ], // init data
            vec![],
            vec![],
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
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
    fn test_interpreter_host_address_heap() {
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

        // bytecode:
        //
        // 0x0000  80 01 00 00  01 00 00 00    i32.imm           0x00000001
        // 0x0008  83 04                       heap.resize
        // 0x000a  02 01                       drop
        // 0x000c  81 01 00 00  00 01 00 00    i64.imm           low:0x00000100  high:0x00000000
        //         00 00 00 00
        // 0x0018  80 01 00 00  02 03 05 07    i32.imm           0x07050302
        // 0x0020  09 04 00 00                 heap.store32      off:0x00
        // 0x0024  81 01 00 00  00 02 00 00    i64.imm           low:0x00000200  high:0x00000000
        //         00 00 00 00
        // 0x0030  81 01 00 00  11 13 17 19    i64.imm           low:0x19171311  high:0x37312923
        //         23 29 31 37
        // 0x003c  08 04 00 00                 heap.store64      off:0x00
        // 0x0040  81 01 00 00  00 01 00 00    i64.imm           low:0x00000100  high:0x00000000
        //         00 00 00 00
        // 0x004c  07 0c 00 00                 host.addr_heap    off:0x00
        // 0x0050  81 01 00 00  00 01 00 00    i64.imm           low:0x00000100  high:0x00000000
        //         00 00 00 00
        // 0x005c  07 0c 02 00                 host.addr_heap    off:0x02
        // 0x0060  81 01 00 00  00 02 00 00    i64.imm           low:0x00000200  high:0x00000000
        //         00 00 00 00
        // 0x006c  07 0c 00 00                 host.addr_heap    off:0x00
        // 0x0070  81 01 00 00  00 02 00 00    i64.imm           low:0x00000200  high:0x00000000
        //         00 00 00 00
        // 0x007c  07 0c 04 00                 host.addr_heap    off:0x04
        // 0x0080  81 01 00 00  00 02 00 00    i64.imm           low:0x00000200  high:0x00000000
        //         00 00 00 00
        // 0x008c  07 0c 07 00                 host.addr_heap    off:0x07
        // 0x0090  00 0a                       end

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 1)
            .append_opcode(Opcode::heap_resize)
            // .append_opcode(Opcode::drop)
            //
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i32(Opcode::imm_i32, 0x07050302)
            .append_opcode_i16(Opcode::heap_store32, 0)
            //
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x200)
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x3731292319171311)
            .append_opcode_i16(Opcode::heap_store_i64, 0)
            //
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::host_addr_heap, 0)
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::host_addr_heap, 2)
            //
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x200)
            .append_opcode_i16(Opcode::host_addr_heap, 0)
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x200)
            .append_opcode_i16(Opcode::host_addr_heap, 4)
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x200)
            .append_opcode_i16(Opcode::host_addr_heap, 7)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        println!("{}", format_bytecode_as_text(&code0));

        #[cfg(target_pointer_width = "64")]
        let result_datatypes = vec![
            OperandDataType::I64,
            OperandDataType::I64,
            OperandDataType::I64,
            OperandDataType::I64,
            OperandDataType::I64,
        ];

        #[cfg(target_pointer_width = "32")]
        let result_datatypes = vec![
            OperandDataType::I32,
            OperandDataType::I32,
            OperandDataType::I32,
            OperandDataType::I32,
            OperandDataType::I32,
        ];

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],           // params
            result_datatypes, // results
            vec![],           // local vars
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        let fvs = result0.unwrap();

        assert_eq!(read_memory_i32(fvs[0]), 0x07050302);
        assert_eq!(read_memory_i16(fvs[1]), 0x0705);
        assert_eq!(read_memory_i64(fvs[2]), 0x3731292319171311);
        assert_eq!(read_memory_i32(fvs[3]), 0x37312923);
        assert_eq!(read_memory_i8(fvs[4]), 0x37);
    }

    #[test]
    fn test_interpreter_host_heap_copy() {
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
            .append_opcode(Opcode::heap_resize)
            // .append_opcode(Opcode::drop)
            //
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0)
            .append_opcode_pesudo_i64(Opcode::imm_i64, 8)
            .append_opcode(Opcode::host_copy_memory_to_heap)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1)
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x100)
            .append_opcode_pesudo_i64(Opcode::imm_i64, 8)
            .append_opcode(Opcode::host_copy_heap_to_memory)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        #[cfg(target_pointer_width = "64")]
        let param_datatypes = vec![OperandDataType::I64, OperandDataType::I64];

        #[cfg(target_pointer_width = "32")]
        let param_datatypes = vec![OperandDataType::I32, OperandDataType::I32];

        let binary0 = helper_build_module_binary_with_single_function(
            param_datatypes, // params
            vec![],          // results
            vec![],          // local vars
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let src_buf: &[u8; 8] = b"hello.vm";
        let dst_buf: [u8; 8] = [0; 8];

        let src_ptr = src_buf.as_ptr();
        let dst_ptr = dst_buf.as_ptr();

        let result0 = process_function(
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
    fn test_interpreter_host_memory_copy() {
        // fn(src_ptr, dst_ptr) -> ()

        // copy src_ptr -> dst_ptr
        //
        // host src_ptr  local var     host dst_ptr
        // |01234567| -> |45670123| -> |45670123|

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i16_i16(Opcode::host_addr_local, 0, 4, 2) // dst ptr
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0) // src ptr
            .append_opcode_pesudo_i64(Opcode::imm_i64, 4) // length
            .append_opcode(Opcode::host_memory_copy)
            //
            .append_opcode_i16_i16_i16(Opcode::host_addr_local, 0, 0, 2) // dst ptr
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0) // src ptr
            .append_opcode_i16(Opcode::i64_inc, 4)
            .append_opcode_pesudo_i64(Opcode::imm_i64, 4) // length
            .append_opcode(Opcode::host_memory_copy)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 1) // dst ptr
            .append_opcode_i16_i16_i16(Opcode::host_addr_local, 0, 0, 2) // src ptr
            .append_opcode_pesudo_i64(Opcode::imm_i64, 8) // length
            .append_opcode(Opcode::host_memory_copy)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        #[cfg(target_pointer_width = "64")]
        let param_datatypes = vec![OperandDataType::I64, OperandDataType::I64];

        #[cfg(target_pointer_width = "32")]
        let param_datatypes = vec![OperandDataType::I32, OperandDataType::I32];

        let binary0 = helper_build_module_binary_with_single_function(
            param_datatypes,                      // params
            vec![],                               // results
            vec![LocalVariableEntry::from_i64()], // local vars
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let src_buf: &[u8; 8] = b"whatever";
        let dst_buf: [u8; 8] = [0; 8];

        let src_ptr = src_buf.as_ptr();
        let dst_ptr = dst_buf.as_ptr();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::U64(src_ptr as usize as u64),
                ForeignValue::U64(dst_ptr as usize as u64),
            ],
        );
        result0.unwrap();

        assert_eq!(&dst_buf, b"everwhat");
    }

    #[test]
    fn test_interpreter_host_addr_function_and_callback_function() {
        // C function in "libtest0.so.1"
        // ===============================
        // int do_something(int (*callback_func)(int), int a, int b)
        // {
        //     int s = (callback_func)(a);
        //     return s + b;
        // }
        //
        // VM functions
        // ============
        //
        // fn func0 (a:i32, b:i32)->i32 {
        //     do_something(func1, a, b)
        // }
        //
        // ;; used as callback function for external function 'do_something'
        // fn func1 (a:i32) -> i32 {
        //     a*2
        // }
        //
        // calling path:
        // (11,13) -> func0(VM) -> do_something(C) -> func1(VM) -> do_something(C) -> func0(VM) -> (11*2+13)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::host_addr_function, 1) // get host address of the func1
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0) // external func param 1
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1) // external func param 2
            //
            .append_opcode_i32(Opcode::extcall, 0) // call external function, external func index = 0
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let code1 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i32(Opcode::imm_i32, 2)
            .append_opcode(Opcode::i32_mul)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_external_functions(
            vec![
                TypeEntry {
                    params: vec![OperandDataType::I64, OperandDataType::I32, OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                }, // do_something
                TypeEntry {
                    params: vec![OperandDataType::I32, OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                }, // func0
                TypeEntry {
                    params: vec![OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                }, // func1
            ], // types
            vec![
                HelperFunctionWithCodeAndLocalVariablesEntry {
                    type_index: 1,
                    local_variable_item_entries_without_args: vec![],
                    code: code0,
                },
                HelperFunctionWithCodeAndLocalVariablesEntry {
                    type_index: 2,
                    local_variable_item_entries_without_args: vec![],
                    code: code1,
                },
            ],
            vec![],
            vec![],
            vec![],
            vec![HelperExternalFunctionEntry {
                external_library_type: ExternalLibraryType::User,
                library_name: "libtest0.so.1".to_string(),
                function_name: "do_something".to_string(),
                type_index: 0,
            }],
        );

        let mut pwd = std::env::current_dir().unwrap();
        let pkg_name = env!("CARGO_PKG_NAME");
        if !pwd.ends_with(pkg_name) {
            // in the VSCode `Debug` environment, the `current_dir()`
            // the project root folder.
            // while in both `$ cargo test` and VSCode `Run Test` environment
            // the `current_dir()` return the current crate path.
            pwd.push("crates");
            pwd.push(pkg_name);
        }
        pwd.push("tests");
        let program_source_path = pwd.to_str().unwrap();

        let program_resource0 = InMemoryProgramResource::with_settings(
            vec![binary0],
            &ProgramSettings::new(program_source_path, true, "", ""),
        );

        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(11), ForeignValue::U32(13)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(11 * 2 + 13)]);

        let result1 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(211), ForeignValue::U32(223)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::U32(211 * 2 + 223)]);
    }
}
