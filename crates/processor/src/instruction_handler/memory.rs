// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::{
    ThreadContext, MEMORY_DATA_ACCESS_INDEX_MASK, MEMORY_DATA_ACCESS_INDEX_MSB,
};

use super::HandleResult;

pub fn memory_allocate(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand size_in_bytes:i64 alignment_in_bytes:i32) -> i64
    let alignment_in_bytes = thread_context.stack.pop_i32_u();
    let size_in_bytes = thread_context.stack.pop_i64_u();
    let data_internal_index = thread_context
        .allocator
        .allocate(size_in_bytes as usize, alignment_in_bytes as usize);

    // Set the MSB to indicate it's a memory allocation
    // This is to differentiate between memory allocations and data sections.
    // Memory allocations will have the MSB set, while data sections will not.
    // This allows us to use the same data access index for both memory allocations and data sections
    // without having to use a separate data access index for memory allocations.
    let data_access_index = data_internal_index | MEMORY_DATA_ACCESS_INDEX_MSB;

    thread_context.stack.push_i64_u(data_access_index as u64);

    HandleResult::Move(2)
}

pub fn memory_reallocate(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand data_access_index:i64 new_size_in_bytes:i64 alignment_in_bytes:i32) -> i64

    let alignment_in_bytes = thread_context.stack.pop_i32_u();
    let new_size_in_bytes = thread_context.stack.pop_i64_u();
    let data_access_index = thread_context.stack.pop_i64_u();

    // Clear the MSB to get the original address
    let data_internal_index = data_access_index as usize & MEMORY_DATA_ACCESS_INDEX_MASK;

    let reallocated_data_internal_index = thread_context.allocator.reallocate(
        data_internal_index,
        new_size_in_bytes as usize,
        alignment_in_bytes as usize,
    );

    let reallocated_data_access_index =
        reallocated_data_internal_index | MEMORY_DATA_ACCESS_INDEX_MSB;

    thread_context
        .stack
        .push_i64_u(reallocated_data_access_index as u64);

    HandleResult::Move(2)
}

pub fn memory_free(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand data_access_index:i64) -> ()

    let data_access_index = thread_context.stack.pop_i64_u();

    // Clear the MSB to get the original address
    let data_internal_index = data_access_index as usize & MEMORY_DATA_ACCESS_INDEX_MASK;

    thread_context.allocator.free(data_internal_index);
    HandleResult::Move(2)
}

pub fn memory_fill(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand
    //     data_module_index:i32
    //     data_access_index:i64
    //     offset_in_bytes:i64
    //     size_in_bytes:i64
    //     value:i8) -> ()

    let value = thread_context.stack.pop_i32_u() as u8;
    let size_in_bytes = thread_context.stack.pop_i64_u() as usize;
    let offset_in_bytes = thread_context.stack.pop_i64_u() as usize;
    let data_access_index = thread_context.stack.pop_i64_u() as usize;
    let data_module_index = thread_context.stack.pop_i32_u() as usize;

    let target_data_object = thread_context.get_target_data_object(
        data_module_index,
        data_access_index,
        offset_in_bytes,
        size_in_bytes,
    );

    let address = target_data_object
        .accessor
        .get_start_address_by_index(target_data_object.data_internal_index_in_section);
    let dst = target_data_object
        .accessor
        .get_mut_ptr(address, offset_in_bytes);

    unsafe { std::ptr::write_bytes(dst, value, size_in_bytes) };
    HandleResult::Move(2)
}

pub fn memory_copy(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand
    //     source_data_module_index:i32
    //     source_data_access_index:i64
    //     source_offset_in_bytes:i64
    //     dest_data_module_index:i32
    //     dest_data_access_index:i64
    //     dest_offset_in_bytes:i64
    //     size_in_bytes:i64) -> ()

    let size_in_bytes = thread_context.stack.pop_i64_u() as usize;
    let dest_offset_in_bytes = thread_context.stack.pop_i64_u() as usize;
    let dest_data_access_index = thread_context.stack.pop_i64_u() as usize;
    let dest_data_module_index = thread_context.stack.pop_i32_u() as usize;
    let source_offset_in_bytes = thread_context.stack.pop_i64_u() as usize;
    let source_data_access_index = thread_context.stack.pop_i64_u() as usize;
    let source_data_module_index = thread_context.stack.pop_i32_u() as usize;

    let source_data_object = thread_context.get_target_data_object(
        source_data_module_index,
        source_data_access_index,
        source_offset_in_bytes,
        size_in_bytes,
    );

    let source_address = source_data_object
        .accessor
        .get_start_address_by_index(source_data_object.data_internal_index_in_section);
    let source_ptr = source_data_object
        .accessor
        .get_ptr(source_address, source_offset_in_bytes);

    let dest_data_object = thread_context.get_target_data_object(
        dest_data_module_index,
        dest_data_access_index,
        dest_offset_in_bytes,
        size_in_bytes,
    );

    let dest_address = dest_data_object
        .accessor
        .get_start_address_by_index(dest_data_object.data_internal_index_in_section);
    let dest_ptr = dest_data_object
        .accessor
        .get_mut_ptr(dest_address, dest_offset_in_bytes);

    unsafe { std::ptr::copy(source_ptr, dest_ptr, size_in_bytes) };

    HandleResult::Move(2)
}

#[cfg(test)]
mod tests {

    use anc_context::program_source::ProgramSource;
    use anc_image::{
        bytecode_writer::BytecodeWriterHelper,
        entry::{ReadOnlyDataEntry, UninitDataEntry},
        utils::helper_build_module_binary_with_single_function_and_data,
    };
    use anc_isa::{opcode::Opcode, ForeignValue, OperandDataType};
    use pretty_assertions::assert_eq;

    use crate::{in_memory_program_source::InMemoryProgramSource, process::process_function};

    #[test]
    fn test_memory_allocate_reallocate_and_free() {
        // API:
        // - memory_allocate    // () (operand size_in_bytes:i64 alignment_in_bytes:i32) -> i64
        // - memory_reallocate  // () (operand data_access_index:i64 new_size_in_bytes:i64 alignment_in_bytes:i32) -> i64
        // - memory_free        // () (operand data_access_index:i64) -> ()
        // - memory_load_i64    // () (operand module_index:i32 data_access_index:i64 offset_bytes:i64) -> i64
        // - memory_load_i32_u  // () (operand module_index:i32 data_access_index:i64 offset_bytes:i64) -> i32
        // - memory_load_i16_u  // () (operand module_index:i32 data_access_index:i64 offset_bytes:i64) -> i16
        // - memory_store_i32   // () (operand value:i32 module_index:i32 data_access_index:i64 offset_bytes:i64) -> (remain_values)
        //
        // - local_load_i64     // (param layers:i16 local_variable_index:i32) -> i64
        // - local_store_i64    // (param layers:i16 local_variable_index:i32) (operand value:i64) -> (remain_values)

        // [ 4 bytes ] [4 bytes ] <-- allocated memory
        //  ^           ^
        //  |           | 5. write 0x19171311_u32 to high 4 bytes
        //  |
        //  | 1. allocate 4 bytes
        //  | 2. write 0x07050302_u32
        //  | 3. read i32
        //  | 4. re-allocate to 8 bytes
        //  |
        // [--------------------]
        //  ^
        //  | 6. read i64
        //  |
        // [ 2 bytes ]
        //  ^
        //  | 7. re-allocate to 2 bytes
        //  | 8. read i16
        // [--------------------]
        //  ^
        //  | 9. free memory

        let code0 = BytecodeWriterHelper::new()
            // 1. allocate memory for 4 bytes
            .append_opcode_i64(Opcode::imm_i64, 4) // size
            .append_opcode_i32(Opcode::imm_i32, 4) // align
            .append_opcode(Opcode::memory_allocate)
            .append_opcode_i16_i32(Opcode::local_store_i64, 0, 0) // store the allocated memory address in local variable 0
            // 2. write 0x07050302_u32
            .append_opcode_i32(Opcode::imm_i32, 0x07050302_u32) // value 0
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // load the allocated memory address from local variable 0
            .append_opcode_i64(Opcode::imm_i64, 0) // offset
            .append_opcode(Opcode::memory_store_i32)
            // 3. read i32
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // load the allocated memory address from local variable 0
            .append_opcode_i64(Opcode::imm_i64, 0) // offset
            .append_opcode(Opcode::memory_load_i32_u)
            // 4. re-allocate to 8 bytes
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // load the allocated memory address from local variable 0
            .append_opcode_i64(Opcode::imm_i64, 8) // new size
            .append_opcode_i32(Opcode::imm_i32, 8) // align
            .append_opcode(Opcode::memory_reallocate)
            .append_opcode_i16_i32(Opcode::local_store_i64, 0, 0) // store the re-allocated memory address in local variable 0
            // 5. write 0x19171311_u32 to high 4 bytes
            .append_opcode_i32(Opcode::imm_i32, 0x19171311_u32) // value 1
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // load the re-allocated memory address from local variable 0
            .append_opcode_i64(Opcode::imm_i64, 4) // offset
            .append_opcode(Opcode::memory_store_i32)
            // 6. read i64
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // load the re-allocated memory address from local variable 0
            .append_opcode_i64(Opcode::imm_i64, 0) // offset
            .append_opcode(Opcode::memory_load_i64)
            // 7. re-allocate to 2 bytes
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // load the re-allocated memory address from local variable 0
            .append_opcode_i64(Opcode::imm_i64, 2) // new size
            .append_opcode_i32(Opcode::imm_i32, 2) // align
            .append_opcode(Opcode::memory_reallocate)
            .append_opcode_i16_i32(Opcode::local_store_i64, 0, 0) // store the re-allocated memory address in local variable 0
            // 8. read i16
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // load the re-allocated memory address from local variable 0
            .append_opcode_i64(Opcode::imm_i64, 0) // offset
            .append_opcode(Opcode::memory_load_i16_u)
            // 9. free memory
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // load the allocated memory address from local variable 0
            .append_opcode(Opcode::memory_free)
            // end
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[
                OperandDataType::I32,
                OperandDataType::I64,
                OperandDataType::I32,
            ], // results
            &[
                OperandDataType::I64, // for storing the allocated memory address
            ], // local variables
            code0,
            &[], // read_only_data_entries
            &[], //read_write_data_entries
            &[], //uninit_uninit_data_entries
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);

        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U32(0x07050302_u32),
                ForeignValue::U64(0x19171311_07050302_u64),
                ForeignValue::U32(0x0302_u32),
            ]
        );
    }

    #[test]
    fn test_memory_fill() {
        // API:
        // - memory_fill
        // () (operand
        //     data_module_index:i32
        //     data_access_index:i64
        //     offset_in_bytes:i64
        //     size_in_bytes:i64
        //     value:i8) -> ()

        // [ 8 bytes pre-defined data ] [ 8 bytes allocated memory ]
        //  ^                            ^
        //  |                            |
        //  | 1. fill 8 bytes with 0x11  | 4. allocate 8 bytes
        //  | 2. fill 4 bytes with 0x22  | 5. fill 8 bytes with 0x33
        //  |    at offset 0             | 6. fill 4 bytes with 0x44
        //  | 3. read i64                |    at offset 4
        //  |                            | 7. read i64
        //  |----------------------------|
        //  | 8. free memory

        let code0 = BytecodeWriterHelper::new()
            // 1. fill 8 bytes with 0x11 at offset 0
            .append_opcode_i32(Opcode::imm_i32, 0) // data module index
            .append_opcode_i64(Opcode::imm_i64, 0) // data access index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i64(Opcode::imm_i64, 8) // size in bytes
            .append_opcode_i32(Opcode::imm_i32, 0x11) // value to fill
            .append_opcode(Opcode::memory_fill)
            // 2. fill 4 bytes with 0x22 at offset 0
            .append_opcode_i32(Opcode::imm_i32, 0) // data module
            .append_opcode_i64(Opcode::imm_i64, 0) // data access index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i64(Opcode::imm_i64, 4) // size
            .append_opcode_i32(Opcode::imm_i32, 0x22) // value to fill
            .append_opcode(Opcode::memory_fill)
            // 3. read i64
            .append_opcode_i32(Opcode::imm_i32, 0) // data module
            .append_opcode_i64(Opcode::imm_i64, 0) // data access index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset
            .append_opcode(Opcode::memory_load_i64)
            // 4. allocate 8 bytes
            .append_opcode_i64(Opcode::imm_i64, 8) // size
            .append_opcode_i32(Opcode::imm_i32, 8) // align
            .append_opcode(Opcode::memory_allocate)
            .append_opcode_i16_i32(Opcode::local_store_i64, 0, 0) // store the allocated memory address in local variable 0
            // 5. fill 8 bytes with 0x33
            .append_opcode_i32(Opcode::imm_i32, 0) // data module index
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // load the allocated memory address from local variable 0
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i64(Opcode::imm_i64, 8) // size in bytes
            .append_opcode_i32(Opcode::imm_i32, 0x33) // value to fill
            .append_opcode(Opcode::memory_fill)
            // 6. fill 4 bytes with 0x44 at offset 4
            .append_opcode_i32(Opcode::imm_i32, 0) // data module
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // load the allocated memory address from local variable 0
            .append_opcode_i64(Opcode::imm_i64, 4) // offset in bytes
            .append_opcode_i64(Opcode::imm_i64, 4) // size in bytes
            .append_opcode_i32(Opcode::imm_i32, 0x44) // value to fill
            .append_opcode(Opcode::memory_fill)
            // 7. read i64
            .append_opcode_i32(Opcode::imm_i32, 0) // data module
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // load the allocated memory address from local variable 0
            .append_opcode_i64(Opcode::imm_i64, 0) // offset
            .append_opcode(Opcode::memory_load_i64)
            // 8. free memory
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // load the allocated memory address from local variable 0
            .append_opcode(Opcode::memory_free)
            // end
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[
                OperandDataType::I64, // for reading pre-defined data
                OperandDataType::I64, // for reading dynamically allocated memory
            ], // results
            &[
                OperandDataType::I64, // for storing the allocated memory address
            ], // local variables
            code0,
            &[],                            // read_only_data_entries
            &[],                            // read_write_data_entries
            &[UninitDataEntry::from_i64()], // uninit_uninit_data_entries
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);

        // ```diagram
        //                   |-- address low, offset 0
        //                   v
        // 0x11111111_22222222_u64
        // 0x44444444_33333333_u64
        //          ^
        //          |-- address high, offset 4
        // ```

        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U64(0x11111111_22222222_u64), // 1. read i64 from pre-defined data
                ForeignValue::U64(0x44444444_33333333_u64), // 2. read i64 from allocated memory
            ]
        );
    }

    #[test]
    fn test_memory_copy() {
        // API:
        // - memory_copy
        // () (operand
        //     source_data_module_index:i32
        //     source_data_access_index:i64
        //     source_offset_in_bytes:i64
        //     dest_data_module_index:i32
        //     dest_data_access_index:i64
        //     dest_offset_in_bytes:i64
        //     size_in_bytes:i64) -> ()

        // 1. allocate 8 bytes of memory
        //
        // 0x19171311_07050302  <-- read-only data, index 0
        //   --------
        //          | 2. copy high 4 bytes from read-only data to allocated memory
        //          \-----\
        //                |
        //                v
        // [ high ]  [ low ]  <-- 8 bytes allocated memory
        //                |
        //                | 3. copy 8 bytes from allocated memory to another data item
        //                \--\
        //                   |
        //                   v
        // 0x00000000_00000000  <-- uninitialized data, index 1
        //
        // 4. read i64 from the allocated memory
        // 5. read i64 from the uninitialized data
        // 6. free the allocated memory

        let code0 = BytecodeWriterHelper::new()
            // 1. allocate 8 bytes of memory
            .append_opcode_i64(Opcode::imm_i64, 8) // size
            .append_opcode_i32(Opcode::imm_i32, 8) // align
            .append_opcode(Opcode::memory_allocate)
            .append_opcode_i16_i32(Opcode::local_store_i64, 0, 0) // store the allocated memory address in local variable 0
            // 2. copy high 4 bytes from read-only data to allocated memory
            .append_opcode_i32(Opcode::imm_i32, 0) // source data module index (read-only data)
            .append_opcode_i64(Opcode::imm_i64, 0) // source data access index (read-only data)
            .append_opcode_i64(Opcode::imm_i64, 4) // source offset in bytes
            .append_opcode_i32(Opcode::imm_i32, 0) // dest data module index (allocated memory)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // load the allocated memory address from local variable 0
            .append_opcode_i64(Opcode::imm_i64, 0) // dest offset in bytes
            .append_opcode_i64(Opcode::imm_i64, 4) // size in bytes
            .append_opcode(Opcode::memory_copy)
            // 3. copy 8 bytes from allocated memory
            .append_opcode_i32(Opcode::imm_i32, 0) // source data module index (allocated memory)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // load the allocated memory address from local variable 0
            .append_opcode_i64(Opcode::imm_i64, 0) // source offset in bytes
            .append_opcode_i32(Opcode::imm_i32, 0) // dest data module index (uninitialized data)
            .append_opcode_i64(Opcode::imm_i64, 1) // dest data access index (uninitialized data)
            .append_opcode_i64(Opcode::imm_i64, 0) // dest offset in bytes
            .append_opcode_i64(Opcode::imm_i64, 8) // size in bytes
            .append_opcode(Opcode::memory_copy)
            // 4. read i64 from the allocated memory
            .append_opcode_i32(Opcode::imm_i32, 0) // data module index (allocated memory)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // load the allocated memory address from local variable 0
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::memory_load_i64)
            // 5. read i64 from the uninitialized data
            .append_opcode_i32(Opcode::imm_i32, 0) // data module index (uninitialized data)
            .append_opcode_i64(Opcode::imm_i64, 1) // data access index (uninitialized data)
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::memory_load_i64)
            // 6. free the allocated memory
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0) // load the allocated memory address from local variable 0
            .append_opcode(Opcode::memory_free)
            // end
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[
                OperandDataType::I64, // for reading pre-defined data
                OperandDataType::I64, // for reading dynamically allocated memory
            ], // results
            &[
                OperandDataType::I64, // for storing the allocated memory address
            ], // local variables
            code0,
            &[ReadOnlyDataEntry::from_i64(0x19171311_07050302_u64)], // read_only_data_entries
            &[],                                                     // read_write_data_entries
            &[UninitDataEntry::from_i64()],                          // uninit_uninit_data_entries
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);

        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U64(0x19171311_u64), // 4. read i64 from the allocated memory
                ForeignValue::U64(0x19171311),     // 5. read i64 from the uninitialized data
            ]
        );
    }
}
