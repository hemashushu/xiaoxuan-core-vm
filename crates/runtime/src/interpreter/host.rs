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

use ancvm_program::{
    jit_util::build_host_to_vm_function, memory::Memory, thread_context::ThreadContext,
};

use super::{process_callback_function_call, InterpretResult};

pub fn nop(_thread: &mut ThreadContext) -> InterpretResult {
    InterpretResult::Move(2)
}

pub fn panic(_thread: &mut ThreadContext) -> InterpretResult {
    InterpretResult::Panic
}

pub fn unreachable(_thread: &mut ThreadContext) -> InterpretResult {
    InterpretResult::Unreachable
}

pub fn debug(thread: &mut ThreadContext) -> InterpretResult {
    let code = thread.get_param_i32();
    InterpretResult::Debug(code)
}

pub fn host_addr_local(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 reversed_index:i16 local_variable_index:i16)
    let (offset_bytes, reversed_index, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_host_addr_local(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn host_addr_local_long(thread_context: &mut ThreadContext) -> InterpretResult {
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
) -> InterpretResult {
    let final_offset = thread_context
        .get_local_variable_address_by_index_and_offset_with_bounds_check(
            reversed_index,
            local_variable_index,
            offset_bytes,
            0,
        );
    let ptr = thread_context.stack.get_ptr(final_offset);
    store_pointer_to_operand_stack(thread_context, ptr);
    InterpretResult::Move(8)
}

pub fn host_addr_data(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_host_addr_data(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn host_addr_data_long(thread_context: &mut ThreadContext) -> InterpretResult {
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
) -> InterpretResult {
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_current_module_data_target_module_index_and_internal_index_and_data_object_with_bounds_check(
            data_public_index,
            0,
            0,
        );
    let total_offset =
        data_object.get_data_address_by_index_and_offset(data_internal_index, offset_bytes);
    let ptr = data_object.get_ptr(total_offset);
    store_pointer_to_operand_stack(thread_context, ptr);
    InterpretResult::Move(8)
}

pub fn host_addr_heap(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread_context.get_param_i16();
    let heap_address = thread_context.stack.pop_i64_u();

    let total_offset = heap_address as usize + offset_bytes as usize;
    let ptr = thread_context.heap.get_ptr(total_offset);
    store_pointer_to_operand_stack(thread_context, ptr);
    InterpretResult::Move(4)
}

pub fn host_copy_from_heap(thread_context: &mut ThreadContext) -> InterpretResult {
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

    InterpretResult::Move(2)
}

pub fn host_copy_to_heap(thread_context: &mut ThreadContext) -> InterpretResult {
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

    InterpretResult::Move(2)
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

pub fn host_addr_func(thread_context: &mut ThreadContext) -> InterpretResult {
    // (operand func_pub_index:i32) -> i64/i32

    let function_public_index = thread_context.stack.pop_i32_u() as usize;
    let module_index = thread_context.pc.module_index;

    // get the internal index of function
    let (target_module_index, function_internal_index) = thread_context
        .get_function_target_module_index_and_internal_index(module_index, function_public_index);

    let callback_function_ptr =
        get_callback_function_ptr(thread_context, target_module_index, function_internal_index)
            .unwrap();

    store_pointer_to_operand_stack(thread_context, callback_function_ptr);

    InterpretResult::Move(2)
}

fn get_callback_function_ptr(
    thread_context: &mut ThreadContext,
    target_module_index: usize,
    function_internal_index: usize,
) -> Result<*const u8, &'static str> {
    // check if the specified (target_module_index, function_internal_index) already
    // exists in the callback function table
    let opt_callback_function_ptr =
        thread_context.find_callback_function(target_module_index, function_internal_index);

    if let Some(callback_function_ptr) = opt_callback_function_ptr {
        return Ok(callback_function_ptr);
    }

    let type_index = thread_context.program_context.program_modules[target_module_index]
        .func_section
        .items[function_internal_index]
        .type_index;
    let (params, results) = thread_context.program_context.program_modules[target_module_index]
        .type_section
        .get_item_params_and_results(type_index as usize);

    if results.len() > 1 {
        return Err("The specified function has more than 1 return value.");
    }

    let delegate_function_addr = process_callback_function_call as *const u8 as usize;
    let thread_context_addr = thread_context as *const ThreadContext as *const u8 as usize;
    let callback_function_ptr = build_host_to_vm_function(
        delegate_function_addr,
        thread_context_addr,
        target_module_index,
        function_internal_index,
        params,
        results,
    );

    // store the function pointer into table
    thread_context.insert_callback_function(
        target_module_index,
        function_internal_index,
        callback_function_ptr,
    );

    Ok(callback_function_ptr)
}

#[cfg(test)]
mod tests {

    use ancvm_binary::{
        bytecode_writer::BytecodeWriter,
        module_image::{
            data_section::{InitedDataEntry, UninitDataEntry},
            local_variable_section::LocalVariableEntry,
            type_section::TypeEntry,
        },
        utils::{
            helper_build_module_binary_with_functions_and_external_functions,
            helper_build_module_binary_with_single_function,
            helper_build_module_binary_with_single_function_and_data_sections,
            HelperExternalFunctionEntry, HelperFuncEntryWithLocalVars,
        },
    };

    use crate::{in_memory_program_source::InMemoryProgramSource, interpreter::process_function};
    use ancvm_program::{program_settings::ProgramSettings, program_source::ProgramSource};
    use ancvm_types::{opcode::Opcode, DataType, ExternalLibraryType, ForeignValue};

    #[test]
    fn test_interpreter_host_nop() {
        // bytecodes
        //
        // 0x0000 nop
        // 0x0002 end
        //
        // (i32, i32) -> (i32, i32)

        let code0 = BytecodeWriter::new()
            .append_opcode(Opcode::nop)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32], // params
            vec![DataType::I32, DataType::I32], // results
            vec![],                             // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::UInt32(7), ForeignValue::UInt32(11)],
        );
        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::UInt32(7), ForeignValue::UInt32(11)]
        );
    }

    fn read_memory_i64(fv: ForeignValue) -> u64 {
        #[cfg(target_pointer_width = "64")]
        if let ForeignValue::UInt64(addr) = fv {
            let ptr = addr as *const u64;
            unsafe { std::ptr::read(ptr) }
        } else {
            0
        }
        #[cfg(target_pointer_width = "32")]
        if let ForeignValue::UInt32(addr) = fv {
            let ptr = addr as *const u64;
            unsafe { std::ptr::read(ptr) }
        } else {
            0
        }
    }

    fn read_memory_i32(fv: ForeignValue) -> u32 {
        #[cfg(target_pointer_width = "64")]
        if let ForeignValue::UInt64(addr) = fv {
            let ptr = addr as *const u32;
            unsafe { std::ptr::read(ptr) }
        } else {
            0
        }
        #[cfg(target_pointer_width = "32")]
        if let ForeignValue::UInt32(addr) = fv {
            let ptr = addr as *const u32;
            unsafe { std::ptr::read(ptr) }
        } else {
            0
        }
    }

    fn read_memory_i16(fv: ForeignValue) -> u16 {
        #[cfg(target_pointer_width = "64")]
        if let ForeignValue::UInt64(addr) = fv {
            let ptr = addr as *const u16;
            unsafe { std::ptr::read(ptr) }
        } else {
            0
        }
        #[cfg(target_pointer_width = "32")]
        if let ForeignValue::UInt32(addr) = fv {
            let ptr = addr as *const u16;
            unsafe { std::ptr::read(ptr) }
        } else {
            0
        }
    }

    fn read_memory_i8(fv: ForeignValue) -> u8 {
        #[cfg(target_pointer_width = "64")]
        if let ForeignValue::UInt64(addr) = fv {
            let ptr = addr as *const u8;
            unsafe { std::ptr::read(ptr) }
        } else {
            0
        }
        #[cfg(target_pointer_width = "32")]
        if let ForeignValue::UInt32(addr) = fv {
            let ptr = addr as *const u8;
            unsafe { std::ptr::read(ptr) }
        } else {
            0
        }
    }

    #[test]
    fn test_interpreter_host_address() {
        //        read-only data section
        //       |low address    high addr|
        //       |                        |
        // index |0              1        |
        //  type |i32------|    |i32------|
        //
        //  data 11 00 00 00    13 00 00 00
        //
        //        read write data section
        //       |low address             high address|
        //       |                                    |
        // index |2(0)                       3(1)     |
        //  type |i64------------------|    |i32------|
        //
        //  data 17 00 00 00 00 00 00 00    19 00 00 00
        //
        //        uninitialized data section
        //       |low address             high address|
        //       |                                    |
        // index |4(0)           5(1)                 |
        //  type |i32------|    |i64------------------|
        //
        //  data 23 00 00 00    29 00 00 00 00 00 00 00
        //
        //        local variable area
        //       |low address                                       high addr|
        //       |                                                           |
        // index |0       1                           2                      |
        //  type |bytes| |i32------|   |padding--|   |i32------|   |padding--|
        //
        //  data 0.....0 31 00 00 00   00 00 00 00   37 00 00 00   00 00 00 00
        //       ^
        //       | 64 bytes
        //       |because the results will overwrite the stack, so leave enough space for results
        //
        // () -> (i64,i64,i64,i64,  i64,i64,i64,i64)

        let code0 = BytecodeWriter::new()
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x17)
            .append_opcode_i16_i32(Opcode::data_store64, 0, 2)
            //
            .append_opcode_i32(Opcode::i32_imm, 0x19)
            .append_opcode_i16_i32(Opcode::data_store32, 0, 3)
            //
            .append_opcode_i32(Opcode::i32_imm, 0x23)
            .append_opcode_i16_i32(Opcode::data_store32, 0, 4)
            //
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x29)
            .append_opcode_i16_i32(Opcode::data_store64, 0, 5)
            //
            .append_opcode_i32(Opcode::i32_imm, 0x31)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 0, 1)
            .append_opcode_i32(Opcode::i32_imm, 0x37)
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

        #[cfg(target_pointer_width = "64")]
        let result_datatypes = vec![
            DataType::I64,
            DataType::I64,
            DataType::I64,
            DataType::I64,
            DataType::I64,
            DataType::I64,
            DataType::I64,
            DataType::I64,
        ];

        #[cfg(target_pointer_width = "32")]
        let result_datatypes = vec![
            DataType::I32,
            DataType::I32,
            DataType::I32,
            DataType::I32,
            DataType::I32,
            DataType::I32,
            DataType::I32,
            DataType::I32,
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
            vec![InitedDataEntry::from_i32(0x11), InitedDataEntry::from_i32(0x13)],
            vec![
                InitedDataEntry::from_i64(0xee), // init data
                InitedDataEntry::from_i32(0xff), // init data
            ],
            vec![UninitDataEntry::from_i32(), UninitDataEntry::from_i64()],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

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
    fn test_interpreter_host_address_long() {
        //        read-only data section
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
        //       |low address         high addr|
        //       |                             |
        // index |0       1                    |
        //  type |bytes| |bytes----------------|
        //
        //  data 0.....0 23 29 31 37 41 43 47 53
        //       ^       |        |        |  |
        //       |       |4       |5       |6 |7
        //       |
        //       | 64 bytes
        //       |because the results will overwrite the stack, so leave enough space for results
        //
        // () -> (i64,i64,i64,i64,  i64,i64,i64,i64)

        let code0 = BytecodeWriter::new()
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x5347434137312923u64)
            .append_opcode_i16_i16_i16(Opcode::local_store64, 0, 0, 1)
            //
            .append_opcode_i32(Opcode::i32_imm, 0)
            .append_opcode_i32(Opcode::host_addr_data_long, 0)
            .append_opcode_i32(Opcode::i32_imm, 2)
            .append_opcode_i32(Opcode::host_addr_data_long, 0)
            .append_opcode_i32(Opcode::i32_imm, 2)
            .append_opcode_i32(Opcode::host_addr_data_long, 1)
            .append_opcode_i32(Opcode::i32_imm, 3)
            .append_opcode_i32(Opcode::host_addr_data_long, 1)
            //
            .append_opcode_i32(Opcode::i32_imm, 0)
            .append_opcode_i32(Opcode::host_addr_local_long, 1)
            .append_opcode_i32(Opcode::i32_imm, 3)
            .append_opcode_i32(Opcode::host_addr_local_long, 1)
            .append_opcode_i32(Opcode::i32_imm, 6)
            .append_opcode_i16_i32(Opcode::host_addr_local_long, 0, 1)
            .append_opcode_i32(Opcode::i32_imm, 7)
            .append_opcode_i16_i32(Opcode::host_addr_local_long, 0, 1)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        #[cfg(target_pointer_width = "64")]
        let result_datatypes = vec![
            DataType::I64,
            DataType::I64,
            DataType::I64,
            DataType::I64,
            DataType::I64,
            DataType::I64,
            DataType::I64,
            DataType::I64,
        ];

        #[cfg(target_pointer_width = "32")]
        let result_datatypes = vec![
            DataType::I32,
            DataType::I32,
            DataType::I32,
            DataType::I32,
            DataType::I32,
            DataType::I32,
            DataType::I32,
            DataType::I32,
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

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

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
        //
        //        heap
        //       |low address                high addr|
        //       |                                    |
        //  addr |0x100         0x200                 |
        //  type |i32-------|   |i64------------------|
        //
        //  data  02 03 05 07   11 13 17 19 23 29 31 37
        //        ^     ^       ^           ^        ^
        //        |0    |1      |2          |3       |4
        // () -> (i64,i64,i64,i64,i64)

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 1)
            .append_opcode(Opcode::heap_resize)
            .append_opcode(Opcode::drop)
            //
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .append_opcode_i32(Opcode::i32_imm, 0x07050302)
            .append_opcode_i16(Opcode::heap_store32, 0)
            //
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x200)
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x3731292319171311)
            .append_opcode_i16(Opcode::heap_store64, 0)
            //
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .append_opcode_i16(Opcode::host_addr_heap, 0)
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .append_opcode_i16(Opcode::host_addr_heap, 2)
            //
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x200)
            .append_opcode_i16(Opcode::host_addr_heap, 0)
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x200)
            .append_opcode_i16(Opcode::host_addr_heap, 4)
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x200)
            .append_opcode_i16(Opcode::host_addr_heap, 7)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        #[cfg(target_pointer_width = "64")]
        let result_datatypes = vec![
            DataType::I64,
            DataType::I64,
            DataType::I64,
            DataType::I64,
            DataType::I64,
        ];

        #[cfg(target_pointer_width = "32")]
        let result_datatypes = vec![
            DataType::I32,
            DataType::I32,
            DataType::I32,
            DataType::I32,
            DataType::I32,
        ];

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],           // params
            result_datatypes, // results
            vec![],           // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

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

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 1)
            .append_opcode(Opcode::heap_resize)
            .append_opcode(Opcode::drop)
            //
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 0)
            .append_opcode_pesudo_i64(Opcode::i64_imm, 8)
            .append_opcode(Opcode::host_copy_to_heap)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .append_opcode_pesudo_i64(Opcode::i64_imm, 8)
            .append_opcode(Opcode::host_copy_from_heap)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        #[cfg(target_pointer_width = "64")]
        let param_datatypes = vec![DataType::I64, DataType::I64];

        #[cfg(target_pointer_width = "32")]
        let param_datatypes = vec![DataType::I32, DataType::I32];

        let binary0 = helper_build_module_binary_with_single_function(
            param_datatypes, // params
            vec![],          // results
            vec![],          // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let src_buf: &[u8; 8] = b"hello...";
        let dst_buf: [u8; 8] = [0; 8];

        let src_ptr = src_buf.as_ptr();
        let dst_ptr = dst_buf.as_ptr();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::UInt64(src_ptr as usize as u64),
                ForeignValue::UInt64(dst_ptr as usize as u64),
            ],
        );
        result0.unwrap();

        assert_eq!(&dst_buf, b"hello...");
    }

    #[test]
    fn test_interpreter_host_addr_func_and_callback_function() {
        // extern "C" do_something(callback_func, a:i32, b:i32) -> i32 {
        //     callback_func(a) + b
        // }
        //
        // func0 (a:i32, b:i32)->i32 {
        //     do_something(func1, a, b)
        // }
        //
        // func1 (a:i32) -> i32 {   ;; this is the callback function for external function 'do_something'
        //     a*2
        // }

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 1) // func1 index
            .append_opcode(Opcode::host_addr_func) // get host address of the func1
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0) // external func param 1
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1) // external func param 2
            //
            .append_opcode_i32(Opcode::i32_imm, 0) // external func index
            .append_opcode(Opcode::extcall) // call external function
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let code1 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i32(Opcode::i32_imm, 2)
            .append_opcode(Opcode::i32_mul)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_external_functions(
            vec![
                TypeEntry {
                    params: vec![DataType::I64, DataType::I32, DataType::I32],
                    results: vec![DataType::I32],
                }, // do_something
                TypeEntry {
                    params: vec![DataType::I32, DataType::I32],
                    results: vec![DataType::I32],
                }, // func0
                TypeEntry {
                    params: vec![DataType::I32],
                    results: vec![DataType::I32],
                }, // func1
            ], // types
            vec![
                HelperFuncEntryWithLocalVars {
                    type_index: 1,
                    local_variable_item_entries_without_args: vec![],
                    code: code0,
                },
                HelperFuncEntryWithLocalVars {
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
                library_name: "lib-test-0.so.1.0.0".to_string(),
                function_name: "do_something".to_string(),
                type_index: 0,
            }],
        );

        let mut pwd = std::env::current_dir().unwrap();
        if !pwd.ends_with("runtime") {
            // in the VSCode debug mode
            pwd.push("crates");
            pwd.push("runtime");
        }
        pwd.push("tests");
        let program_source_path = pwd.to_str().unwrap();

        let program_source0 = InMemoryProgramSource::with_settings(
            vec![binary0],
            &ProgramSettings::new(program_source_path, true, "", ""),
        );

        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::UInt32(11), ForeignValue::UInt32(13)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(11 * 2 + 13)]);

        let result1 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::UInt32(211), ForeignValue::UInt32(223)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(211 * 2 + 223)]);
    }
}
