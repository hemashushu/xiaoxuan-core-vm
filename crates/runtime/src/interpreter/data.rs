// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_program::thread_context::ThreadContext;

use super::InterpretResult;

const DATA_LENGTH_IN_BYTES_64_BIT: usize = 8;
const DATA_LENGTH_IN_BYTES_32_BIT: usize = 4;
const DATA_LENGTH_IN_BYTES_16_BIT: usize = 2;
const DATA_LENGTH_IN_BYTES_8_BIT: usize = 1;

pub fn data_load64_i64(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load64_i64(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn data_long_load64_i64(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_load64_i64(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

fn do_data_load64_i64(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_current_module_data_target_module_index_and_internal_index_and_data_object_with_bounds_check(
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_64_BIT,
        );
    data_object.load_idx_64(data_internal_index, offset_bytes, dst_ptr);

    InterpretResult::Move(8)
}

pub fn data_load32_i32(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load32_i32(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn data_long_load32_i32(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_load32_i32(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

fn do_data_load32_i32(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_current_module_data_target_module_index_and_internal_index_and_data_object_with_bounds_check(
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_32_BIT,
        );
    data_object.load_idx_32(data_internal_index, offset_bytes, dst_ptr);

    InterpretResult::Move(8)
}

pub fn data_load32_i16_s(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load32_i16_s(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn data_long_load32_i16_s(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_load32_i16_s(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

fn do_data_load32_i16_s(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_current_module_data_target_module_index_and_internal_index_and_data_object_with_bounds_check(
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_16_BIT,
        );
    data_object.load_idx_32_extend_from_i16_s(data_internal_index, offset_bytes, dst_ptr);

    InterpretResult::Move(8)
}

pub fn data_load32_i16_u(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load32_i16_u(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn data_long_load32_i16_u(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_load32_i16_u(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

fn do_data_load32_i16_u(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_current_module_data_target_module_index_and_internal_index_and_data_object_with_bounds_check(
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_16_BIT,
        );
    data_object.load_idx_32_extend_from_i16_u(data_internal_index, offset_bytes, dst_ptr);

    InterpretResult::Move(8)
}

pub fn data_load32_i8_s(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load32_i8_s(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn data_long_load32_i8_s(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_load32_i8_s(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

fn do_data_load32_i8_s(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_current_module_data_target_module_index_and_internal_index_and_data_object_with_bounds_check(
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_8_BIT,
        );
    data_object.load_idx_32_extend_from_i8_s(data_internal_index, offset_bytes, dst_ptr);

    InterpretResult::Move(8)
}

pub fn data_load32_i8_u(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load32_i8_u(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn data_long_load32_i8_u(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_load32_i8_u(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

fn do_data_load32_i8_u(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_current_module_data_target_module_index_and_internal_index_and_data_object_with_bounds_check(
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_8_BIT,
        );
    data_object.load_idx_32_extend_from_i8_u(data_internal_index, offset_bytes, dst_ptr);

    InterpretResult::Move(8)
}

pub fn data_load32_f32(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load32_f32(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn data_long_load32_f32(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_load32_f32(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

fn do_data_load32_f32(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_current_module_data_target_module_index_and_internal_index_and_data_object_with_bounds_check(
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_32_BIT,
        );
    data_object.load_idx_32_with_float_check(data_internal_index, offset_bytes, dst_ptr);

    InterpretResult::Move(8)
}

pub fn data_load64_f64(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load64_f64(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn data_long_load64_f64(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_load64_f64(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

fn do_data_load64_f64(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_current_module_data_target_module_index_and_internal_index_and_data_object_with_bounds_check(
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_64_BIT,
        );
    data_object.load_idx_64_with_float_check(data_internal_index, offset_bytes, dst_ptr);

    InterpretResult::Move(8)
}

pub fn data_store64(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_data_store64(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

pub fn data_long_store64(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_store64(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

fn do_data_store64(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
    src_ptr: *const u8,
) -> InterpretResult {
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_current_module_data_target_module_index_and_internal_index_and_data_object_with_bounds_check(
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_64_BIT,
        );
    data_object.store_idx_64(src_ptr, data_internal_index, offset_bytes);

    InterpretResult::Move(8)
}

pub fn data_store32(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_data_store32(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

pub fn data_long_store32(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_store32(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

fn do_data_store32(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
    src_ptr: *const u8,
) -> InterpretResult {
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_current_module_data_target_module_index_and_internal_index_and_data_object_with_bounds_check(
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_32_BIT,
        );
    data_object.store_idx_32(src_ptr, data_internal_index, offset_bytes);

    InterpretResult::Move(8)
}

pub fn data_store16(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_data_store16(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

pub fn data_long_store16(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_store16(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

fn do_data_store16(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
    src_ptr: *const u8,
) -> InterpretResult {
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_current_module_data_target_module_index_and_internal_index_and_data_object_with_bounds_check(
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_16_BIT,
        );
    data_object.store_idx_16(src_ptr, data_internal_index, offset_bytes);

    InterpretResult::Move(8)
}

pub fn data_store8(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_data_store8(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

pub fn data_long_store8(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_store8(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

fn do_data_store8(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
    src_ptr: *const u8,
) -> InterpretResult {
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_current_module_data_target_module_index_and_internal_index_and_data_object_with_bounds_check(
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_8_BIT,
        );
    data_object.store_idx_8(src_ptr, data_internal_index, offset_bytes);

    InterpretResult::Move(8)
}

#[cfg(test)]
mod tests {

    use ancvm_binary::{
        bytecode_writer::BytecodeWriter,
        module_image::data_section::{InitedDataEntry, UninitDataEntry},
        utils::helper_build_module_binary_with_single_function_and_data_sections,
    };

    use crate::{in_memory_program_source::InMemoryProgramSource, interpreter::process_function};
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    #[test]
    fn test_interpreter_data_load_and_store() {
        //        read-only data section
        //        ======================
        //
        //       |low address    high addr|
        //       |                        |
        // index |0           1           |
        //  type |i32------| |i32---------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0
        //       |           |        |  |
        //       |           |        |  |load8u (step 1)
        //       |           |        |load8u (step 2)
        //       |           |load16u (step 3)
        //       |load32 (step 4)
        //
        //        read-write data section
        //        =======================
        //
        //       |low address                                                              high address|
        //       |                                                                                     |
        // index |2(0)                               3(1)   4(2)   5(3)                      6(4)      |
        //  type |bytes-------------------|         |f32|  |f64|  |i64------------------|   |i32-------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 12 17 19
        //       |           |        |  |          |      |      ^                          ^
        //       |store32    |store16 |  |          |sf32  |sf64  |                          |
        //                            |  |          |stepN'|stepN |                          |
        //                      store8|  |          |      |      |                          |
        //       |                       |store8    |      |      |store64                   |store32
        //       |                                  |      |      |                          |
        //       \----->--load64-->---------------------------->--/-->-------------------->--/
        //
        //       11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 12 17 19
        //       |           |        |  |load8u    |      |      |                          |
        //       |           |        |  |load8s  loadf32  |      |                          |
        //       |           |        |                  loadf64  |                          |
        //       |           |        |load16u                    |                          |
        //       |           |        |load16s                 load64                      load32
        //       |           |
        //       |load64     |load32
        //
        // (f32, f64) -> (i64,i32,i32,i32,i32,i32, f32,f64 ,i64,i32)

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i32(Opcode::data_load32_i8_u, 3, 1)
            .append_opcode_i16_i32(Opcode::data_load32_i8_u, 2, 1)
            .append_opcode_i16_i32(Opcode::data_load32_i16_u, 0, 1)
            .append_opcode_i16_i32(Opcode::data_load32_i32, 0, 0)
            //
            .append_opcode_i16_i32(Opcode::data_store32, 0, 2)
            .append_opcode_i16_i32(Opcode::data_store16, 4, 2)
            .append_opcode_i16_i32(Opcode::data_store8, 6, 2)
            .append_opcode_i16_i32(Opcode::data_store8, 7, 2)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode_i16_i32(Opcode::data_store64, 0, 4) // store f64
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode_i16_i32(Opcode::data_store32, 0, 3) // store f32
            //
            .append_opcode_i16_i32(Opcode::data_load64_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::data_store64, 0, 5)
            //
            .append_opcode_i16_i32(Opcode::data_load64_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::data_store32, 0, 6)
            //
            .append_opcode_i16_i32(Opcode::data_load64_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::data_load32_i32, 4, 2)
            .append_opcode_i16_i32(Opcode::data_load32_i16_u, 6, 2)
            .append_opcode_i16_i32(Opcode::data_load32_i16_s, 6, 2)
            .append_opcode_i16_i32(Opcode::data_load32_i8_u, 7, 2)
            .append_opcode_i16_i32(Opcode::data_load32_i8_s, 7, 2)
            //
            .append_opcode_i16_i32(Opcode::data_load32_f32, 0, 3)
            .append_opcode_i16_i32(Opcode::data_load64_f64, 0, 4)
            //
            .append_opcode_i16_i32(Opcode::data_load64_i64, 0, 5)
            .append_opcode_i16_i32(Opcode::data_load32_i32, 0, 6)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data_sections(
            vec![DataType::F32, DataType::F64], // params
            vec![
                DataType::I64,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::F32,
                DataType::F64,
                DataType::I64,
                DataType::I32,
            ], // results
            vec![],                             // local vars
            code0,
            vec![
                InitedDataEntry::from_i32(0x19171311),
                InitedDataEntry::from_i32(0xf0e0d0c0),
            ],
            vec![
                InitedDataEntry::from_bytes(vec![0u8; 8], 8),
                InitedDataEntry::from_f32(0.0f32),
                InitedDataEntry::from_f64(0.0f64),
                InitedDataEntry::from_i64(0),
                InitedDataEntry::from_i32(0),
            ],
            vec![],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::Float32(std::f32::consts::PI),
                ForeignValue::Float64(std::f64::consts::E),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0xf0e0d0c0u32),
                ForeignValue::UInt32(0xf0e0u32),
                ForeignValue::UInt32(0xfffff0e0u32), // extend from i16 to i32
                ForeignValue::UInt32(0xf0u32),
                ForeignValue::UInt32(0xfffffff0u32), // extend from i8 to i32
                //
                ForeignValue::Float32(std::f32::consts::PI),
                ForeignValue::Float64(std::f64::consts::E),
                //
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0x19171311u32),
            ]
        );
    }

    #[test]
    fn test_interpreter_data_load_and_store_uninitialized() {
        //        read-only data section
        //        ======================
        //
        //       |low address    high addr|
        //       |                        |
        // index |0           1           |
        //  type |i32------| |i32---------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0
        //       |           |        |  |
        //       |           |        |  |load8u (step 1)
        //       |           |        |load8u (step 2)
        //       |           |load16u (step 3)
        //       |load32 (step 4)
        //
        //        uninitialized data section
        //        ==========================
        //
        //       |low address                                                              high address|
        //       |                                                                                     |
        // index |2(0)                               3(1)   4(2)   5(3)                      6(4)      |
        //  type |bytes-------------------|         |f32|  |f64|  |i64------------------|   |i32-------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 12 17 19
        //       |           |        |  |          |      |      ^                          ^
        //       |store32    |store16 |  |          |sf32  |sf64  |                          |
        //                            |  |          |stepN'|stepN |                          |
        //                      store8|  |          |      |      |                          |
        //       |                       |store8    |      |      |store64                   |store32
        //       |                                  |      |      |                          |
        //       \----->--load64-->---------------------------->--/-->-------------------->--/
        //
        //       11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 12 17 19
        //       |           |        |  |load8u    |      |      |                          |
        //       |           |        |  |load8s  loadf32  |      |                          |
        //       |           |        |                  loadf64  |                          |
        //       |           |        |load16u                    |                          |
        //       |           |        |load16s                 load64                      load32
        //       |           |
        //       |load64     |load32
        //
        // (f32, f64) -> (i64,i32,i32,i32,i32,i32, f32,f64 ,i64,i32)

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i32(Opcode::data_load32_i8_u, 3, 1)
            .append_opcode_i16_i32(Opcode::data_load32_i8_u, 2, 1)
            .append_opcode_i16_i32(Opcode::data_load32_i16_u, 0, 1)
            .append_opcode_i16_i32(Opcode::data_load32_i32, 0, 0)
            //
            .append_opcode_i16_i32(Opcode::data_store32, 0, 2)
            .append_opcode_i16_i32(Opcode::data_store16, 4, 2)
            .append_opcode_i16_i32(Opcode::data_store8, 6, 2)
            .append_opcode_i16_i32(Opcode::data_store8, 7, 2)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode_i16_i32(Opcode::data_store64, 0, 4) // store f64
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode_i16_i32(Opcode::data_store32, 0, 3) // store f32
            //
            .append_opcode_i16_i32(Opcode::data_load64_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::data_store64, 0, 5)
            //
            .append_opcode_i16_i32(Opcode::data_load64_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::data_store32, 0, 6)
            //
            .append_opcode_i16_i32(Opcode::data_load64_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::data_load32_i32, 4, 2)
            .append_opcode_i16_i32(Opcode::data_load32_i16_u, 6, 2)
            .append_opcode_i16_i32(Opcode::data_load32_i16_s, 6, 2)
            .append_opcode_i16_i32(Opcode::data_load32_i8_u, 7, 2)
            .append_opcode_i16_i32(Opcode::data_load32_i8_s, 7, 2)
            //
            .append_opcode_i16_i32(Opcode::data_load32_f32, 0, 3)
            .append_opcode_i16_i32(Opcode::data_load64_f64, 0, 4)
            //
            .append_opcode_i16_i32(Opcode::data_load64_i64, 0, 5)
            .append_opcode_i16_i32(Opcode::data_load32_i32, 0, 6)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data_sections(
            vec![DataType::F32, DataType::F64], // params
            vec![
                DataType::I64,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::F32,
                DataType::F64,
                DataType::I64,
                DataType::I32,
            ], // results
            vec![],                             // local vars
            code0,
            vec![
                InitedDataEntry::from_i32(0x19171311),
                InitedDataEntry::from_i32(0xf0e0d0c0),
            ],
            vec![],
            vec![
                UninitDataEntry::from_bytes(8, 8),
                UninitDataEntry::from_f32(),
                UninitDataEntry::from_f64(),
                UninitDataEntry::from_i64(),
                UninitDataEntry::from_i32(),
            ],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::Float32(std::f32::consts::PI),
                ForeignValue::Float64(std::f64::consts::E),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0xf0e0d0c0u32),
                ForeignValue::UInt32(0xf0e0u32),
                ForeignValue::UInt32(0xfffff0e0u32), // extend from i16 to i32
                ForeignValue::UInt32(0xf0u32),
                ForeignValue::UInt32(0xfffffff0u32), // extend from i8 to i32
                // group 1
                ForeignValue::Float32(std::f32::consts::PI),
                ForeignValue::Float64(std::f64::consts::E),
                // group 2
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0x19171311u32),
            ]
        );
    }

    #[test]
    fn test_interpreter_data_long_load_and_store() {
        //       |low address                                 high address|
        //       |                                                        |
        // index |0                                  1                    |
        //  type |bytes-------------------|         |bytes----------------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0         11 13 17 19 c0 d0 e0 f0
        //       |           |        |  |          ^
        //       |store32    |store16 |  |          |
        //        step0       step1   |  |          |
        //                      store8|  |          |
        //       |              step2    |store8    |store64
        //       |                        step3     |
        //       \----->--load64-->-----------------/
        //
        //       11 13 17 19 c0 d0    e0 f0         11 13 17 19 c0 d0 e0 f0
        //       |           |        |  |load8u    |
        //       |           |        |  |load8s    |load64
        //       |           |        |             |load32
        //       |           |        |load16u      |load16u
        //       |           |        |load16s      |load8u
        //       |           |
        //       |load64     |load32
        //
        // () -> (i64,i32,i32,i32,i32,i32,  i64,i32,i32,i32)

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 0)
            .append_opcode_i32(Opcode::i32_imm, 0x19171311)
            .append_opcode_i32(Opcode::data_long_store32, 0) // store 32
            //
            .append_opcode_i32(Opcode::i32_imm, 4)
            .append_opcode_i32(Opcode::i32_imm, 0xd0c0)
            .append_opcode_i32(Opcode::data_long_store16, 0) // store 16
            //
            .append_opcode_i32(Opcode::i32_imm, 6)
            .append_opcode_i32(Opcode::i32_imm, 0xe0)
            .append_opcode_i32(Opcode::data_long_store8, 0) // store 8
            //
            .append_opcode_i32(Opcode::i32_imm, 7)
            .append_opcode_i32(Opcode::i32_imm, 0xf0)
            .append_opcode_i32(Opcode::data_long_store8, 0) // store 8
            //
            .append_opcode_i32(Opcode::i32_imm, 0)
            .append_opcode_i32(Opcode::i32_imm, 0)
            .append_opcode_i32(Opcode::data_long_load64_i64, 0) // load 64
            .append_opcode_i32(Opcode::data_long_store64, 1) // store 64
            //
            .append_opcode_i32(Opcode::i32_imm, 0)
            .append_opcode_i32(Opcode::data_long_load64_i64, 0)
            .append_opcode_i32(Opcode::i32_imm, 4)
            .append_opcode_i32(Opcode::data_long_load32_i32, 0)
            .append_opcode_i32(Opcode::i32_imm, 6)
            .append_opcode_i32(Opcode::data_long_load32_i16_u, 0)
            .append_opcode_i32(Opcode::i32_imm, 6)
            .append_opcode_i32(Opcode::data_long_load32_i16_s, 0)
            .append_opcode_i32(Opcode::i32_imm, 7)
            .append_opcode_i32(Opcode::data_long_load32_i8_u, 0)
            .append_opcode_i32(Opcode::i32_imm, 7)
            .append_opcode_i32(Opcode::data_long_load32_i8_s, 0)
            //
            .append_opcode_i32(Opcode::i32_imm, 0)
            .append_opcode_i32(Opcode::data_long_load64_i64, 1)
            .append_opcode_i32(Opcode::i32_imm, 0)
            .append_opcode_i32(Opcode::data_long_load32_i32, 1)
            .append_opcode_i32(Opcode::i32_imm, 0)
            .append_opcode_i32(Opcode::data_long_load32_i16_u, 1)
            .append_opcode_i32(Opcode::i32_imm, 0)
            .append_opcode_i32(Opcode::data_long_load32_i8_u, 1)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data_sections(
            vec![], // params
            vec![
                DataType::I64,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I64,
                DataType::I32,
                DataType::I32,
                DataType::I32,
            ], // results
            vec![], // local vars
            code0,
            vec![],
            vec![],
            vec![
                UninitDataEntry::from_bytes(8, 8),
                UninitDataEntry::from_bytes(8, 8),
            ],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0xf0e0d0c0u32),
                ForeignValue::UInt32(0xf0e0u32),
                ForeignValue::UInt32(0xfffff0e0u32), // extend from i16 to i32
                ForeignValue::UInt32(0xf0u32),
                ForeignValue::UInt32(0xfffffff0u32), // extend from i8 to i32
                // group 1
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0x19171311u32),
                ForeignValue::UInt32(0x00001311u32), // extend from i16 to i32
                ForeignValue::UInt32(0x00000011u32), // extend from i8 to i32
            ]
        );
    }

    #[test]
    // #[should_panic]
    fn test_interpreter_data_bounds_check0() {
        let prev_hook = std::panic::take_hook(); // let panic silent
        std::panic::set_hook(Box::new(|_| {}));

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i32(Opcode::data_load32_i32, 2, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data_sections(
            vec![], // params
            vec![], // results
            vec![], // local vars
            code0,
            vec![],
            vec![InitedDataEntry::from_i32(11)],
            vec![],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();

        let result = std::panic::catch_unwind(move || {
            let mut thread_context0 = program0.create_thread_context();
            let _ = process_function(&mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    // #[should_panic]
    fn test_interpreter_data_bounds_check1() {
        let prev_hook = std::panic::take_hook(); // let panic silent
        std::panic::set_hook(Box::new(|_| {}));

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i32(Opcode::data_load64_i64, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data_sections(
            vec![], // params
            vec![], // results
            vec![], // local vars
            code0,
            vec![],
            vec![InitedDataEntry::from_i32(11)],
            vec![],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();

        let result = std::panic::catch_unwind(move || {
            let mut thread_context0 = program0.create_thread_context();
            let _ = process_function(&mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    // #[should_panic]
    fn test_interpreter_data_bounds_check2() {
        let prev_hook = std::panic::take_hook(); // let panic silent
        std::panic::set_hook(Box::new(|_| {}));

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 11)
            .append_opcode_i16_i32(Opcode::data_store32, 2, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data_sections(
            vec![], // params
            vec![], // results
            vec![], // local vars
            code0,
            vec![],
            vec![InitedDataEntry::from_i32(11)],
            vec![],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();

        let result = std::panic::catch_unwind(move || {
            let mut thread_context0 = program0.create_thread_context();
            let _ = process_function(&mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    // #[should_panic]
    fn test_interpreter_data_bounds_check3() {
        let prev_hook = std::panic::take_hook(); // let panic silent
        std::panic::set_hook(Box::new(|_| {}));

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 2) // offset
            .append_opcode_i32(Opcode::data_long_load32_i32, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data_sections(
            vec![], // params
            vec![], // results
            vec![], // local vars
            code0,
            vec![],
            vec![InitedDataEntry::from_i32(11)],
            vec![],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();

        let result = std::panic::catch_unwind(move || {
            let mut thread_context0 = program0.create_thread_context();
            let _ = process_function(&mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }
}
