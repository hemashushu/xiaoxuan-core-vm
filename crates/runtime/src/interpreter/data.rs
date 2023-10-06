// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_thread::thread_context::ThreadContext;

use super::InterpretResult;

pub fn data_load(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn data_long_load(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_load(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

fn do_data_load(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let (datas, _target_module_index, data_internal_index) =
        thread_context.get_current_module_data_internal_index_and_datas_object(data_public_index);
    datas.load_idx_64(data_internal_index, offset_bytes, dst_ptr);

    InterpretResult::Move(8)
}

pub fn data_load32(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load32(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn data_long_load32(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_load32(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

fn do_data_load32(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let (datas, _target_module_index, data_internal_index) =
        thread_context.get_current_module_data_internal_index_and_datas_object(data_public_index);
    datas.load_idx_32(data_internal_index, offset_bytes, dst_ptr);

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
    let (datas, _target_module_index, data_internal_index) =
        thread_context.get_current_module_data_internal_index_and_datas_object(data_public_index);
    datas.load_idx_32_extend_from_i16_s(data_internal_index, offset_bytes, dst_ptr);

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
    let (datas, _target_module_index, data_internal_index) =
        thread_context.get_current_module_data_internal_index_and_datas_object(data_public_index);
    datas.load_idx_32_extend_from_i16_u(data_internal_index, offset_bytes, dst_ptr);

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
    let (datas, _target_module_index, data_internal_index) =
        thread_context.get_current_module_data_internal_index_and_datas_object(data_public_index);
    datas.load_idx_32_extend_from_i8_s(data_internal_index, offset_bytes, dst_ptr);

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
    let (datas, _target_module_index, data_internal_index) =
        thread_context.get_current_module_data_internal_index_and_datas_object(data_public_index);
    datas.load_idx_32_extend_from_i8_u(data_internal_index, offset_bytes, dst_ptr);

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
    let (datas, _target_module_index, data_internal_index) =
        thread_context.get_current_module_data_internal_index_and_datas_object(data_public_index);
    datas.load_idx_32_with_float_check(data_internal_index, offset_bytes, dst_ptr);

    InterpretResult::Move(8)
}

pub fn data_load_f64(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_f64(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn data_long_load_f64(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_load_f64(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

fn do_data_load_f64(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let (datas, _target_module_index, data_internal_index) =
        thread_context.get_current_module_data_internal_index_and_datas_object(data_public_index);
    datas.load_idx_64_with_float_check(data_internal_index, offset_bytes, dst_ptr);

    InterpretResult::Move(8)
}

pub fn data_store(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_store(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn data_long_store(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_store(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

fn do_data_store(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let (datas, _target_module_index, data_internal_index) =
        thread_context.get_current_module_data_internal_index_and_datas_object(data_public_index);
    datas.store_idx_64(src_ptr, data_internal_index, offset_bytes);

    InterpretResult::Move(8)
}

pub fn data_store32(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_store32(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn data_long_store32(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_store32(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

fn do_data_store32(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let (datas, _target_module_index, data_internal_index) =
        thread_context.get_current_module_data_internal_index_and_datas_object(data_public_index);
    datas.store_idx_32(src_ptr, data_internal_index, offset_bytes);

    InterpretResult::Move(8)
}

pub fn data_store16(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_store16(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn data_long_store16(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_store16(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

fn do_data_store16(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let (datas, _target_module_index, data_internal_index) =
        thread_context.get_current_module_data_internal_index_and_datas_object(data_public_index);
    datas.store_idx_16(src_ptr, data_internal_index, offset_bytes);

    InterpretResult::Move(8)
}

pub fn data_store8(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_store8(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

pub fn data_long_store8(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_data_store8(
        thread_context,
        data_public_index as usize,
        offset_bytes as usize,
    )
}

fn do_data_store8(
    thread_context: &mut ThreadContext,
    data_public_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let (datas, _target_module_index, data_internal_index) =
        thread_context.get_current_module_data_internal_index_and_datas_object(data_public_index);
    datas.store_idx_8(src_ptr, data_internal_index, offset_bytes);

    InterpretResult::Move(8)
}

#[cfg(test)]
mod tests {

    use ancvm_binary::{
        module_image::{
            data_section::{DataEntry, UninitDataEntry},
            local_variable_section::LocalVariableEntry,
        },
        utils::{build_module_binary_with_single_function_and_data_sections, BytecodeWriter},
    };

    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    use crate::{
        in_memory_program::InMemoryProgram, interpreter::process_function, program::Program,
    };

    #[test]
    fn test_process_data_load_store() {
        // init_runtime();

        //        read-only data section
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

        // bytecodes
        //
        // 0x0000 data_load32_i8_u     3 1
        // 0x0008 data_load32_i8_u     2 1
        // 0x0010 data_load32_i16_u    0 1
        // 0x0018 data_load32          0 0
        //
        // 0x0020 data_store32         0 2          ;; store 0x19171311
        // 0x0028 data_store16         4 2          ;; store 0xd0c0
        // 0x0030 data_store8          6 2          ;; store 0xe0
        // 0x0038 data_store8          7 2          ;; store 0xf0
        //
        // 0x0040 local_load_f64       0 3
        // 0x0048 data_store           0 4          ;; store f64
        // 0x0050 local_load32_f32     0 2
        // 0x0058 data_store32         0 3          ;; store f32
        //
        // 0x0060 data_load            0 2
        // 0x0068 data_store           0 5          ;; store 0xf0e0d0c0_19171311
        // 0x0070 data_load            0 2
        // 0x0078 data_store32         0 6          ;; store 0x19171311
        //
        // 0x0080 data_load            0 2          ;; load 0xf0e0d0c0_19171311
        // 0x0088 data_load32          4 2          ;; load 0xf0e0d0c0
        // 0x0090 data_load32_i16_u    6 2          ;; load 0xf0e0
        // 0x0098 data_load32_i16_s    6 2          ;; load 0xf0e0
        // 0x00a0 data_load32_i8_u     7 2          ;; load 0xf0
        // 0x00a8 data_load32_i8_s     7 2          ;; load 0xf0
        // 0x00b0 data_load32_f32      0 3          ;; load f32
        // 0x00b8 data_load_f64        0 4          ;; load f64
        // 0x00c0 data_load            0 5          ;; load 0xf0e0d0c0_19171311
        // 0x00c8 data_load32          0 6          ;; load 0x19171311
        // 0x00d0 end
        //
        // (f32, f64) -> (i64,i32,i32,i32,i32,i32, f32,f64 ,i64,i32)

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::data_load32_i8_u, 3, 1)
            .write_opcode_i16_i32(Opcode::data_load32_i8_u, 2, 1)
            .write_opcode_i16_i32(Opcode::data_load32_i16_u, 0, 1)
            .write_opcode_i16_i32(Opcode::data_load32, 0, 0)
            //
            .write_opcode_i16_i32(Opcode::data_store32, 0, 2)
            .write_opcode_i16_i32(Opcode::data_store16, 4, 2)
            .write_opcode_i16_i32(Opcode::data_store8, 6, 2)
            .write_opcode_i16_i32(Opcode::data_store8, 7, 2)
            //
            // the arguments f32 and f64 are at the top of stack, they can access directly
            // here test loading the arguments as local variables.
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 3)
            .write_opcode_i16_i32(Opcode::data_store, 0, 4) // store f64
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 2)
            .write_opcode_i16_i32(Opcode::data_store32, 0, 3) // store f32
            //
            .write_opcode_i16_i32(Opcode::data_load, 0, 2)
            .write_opcode_i16_i32(Opcode::data_store, 0, 5)
            //
            .write_opcode_i16_i32(Opcode::data_load, 0, 2)
            .write_opcode_i16_i32(Opcode::data_store32, 0, 6)
            //
            .write_opcode_i16_i32(Opcode::data_load, 0, 2)
            .write_opcode_i16_i32(Opcode::data_load32, 4, 2)
            .write_opcode_i16_i32(Opcode::data_load32_i16_u, 6, 2)
            .write_opcode_i16_i32(Opcode::data_load32_i16_s, 6, 2)
            .write_opcode_i16_i32(Opcode::data_load32_i8_u, 7, 2)
            .write_opcode_i16_i32(Opcode::data_load32_i8_s, 7, 2)
            //
            .write_opcode_i16_i32(Opcode::data_load32_f32, 0, 3)
            .write_opcode_i16_i32(Opcode::data_load_f64, 0, 4)
            //
            .write_opcode_i16_i32(Opcode::data_load, 0, 5)
            .write_opcode_i16_i32(Opcode::data_load32, 0, 6)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function_and_data_sections(
            vec![
                DataEntry::from_i32(0x19171311),
                DataEntry::from_i32(0xf0e0d0c0),
            ],
            vec![
                DataEntry::from_bytes(vec![0u8; 8], 8),
                DataEntry::from_f32(0.0f32),
                DataEntry::from_f64(0.0f64),
                DataEntry::from_i64(0),
                DataEntry::from_i32(0),
            ],
            vec![],
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
            vec![
                LocalVariableEntry::from_i32(),
                LocalVariableEntry::from_i64(),
            ], // local vars
            code0,
        );

        let program0 = InMemoryProgram::new(vec![binary0]);
        let program_context0 = program0.build_program_context().unwrap();
        let mut thread_context0 = program_context0.new_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &vec![
                ForeignValue::Float32(3.1415926f32),
                ForeignValue::Float64(2.9979e8f64),
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
                ForeignValue::Float32(3.1415926f32),
                ForeignValue::Float64(2.9979e8f64),
                //
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0x19171311u32),
            ]
        );
    }

    #[test]
    fn test_process_data_load_store_uninitialized() {
        // init_runtime();

        //        read-only data section
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

        // bytecodes
        //
        // 0x0000 data_load32_i8_u     3 1
        // 0x0008 data_load32_i8_u     2 1
        // 0x0010 data_load32_i16_u    0 1
        // 0x0018 data_load32          0 0
        //
        // 0x0020 data_store32         0 2          ;; store 0x19171311
        // 0x0028 data_store16         4 2          ;; store 0xd0c0
        // 0x0030 data_store8          6 2          ;; store 0xe0
        // 0x0038 data_store8          7 2          ;; store 0xf0
        // 0x0040 data_store           0 4          ;; store f64
        // 0x0048 data_store32         0 3          ;; store f32
        //
        // 0x0050 data_load            0 2
        // 0x0058 data_store           0 5          ;; store 0xf0e0d0c0_19171311
        // 0x0060 data_load            0 2
        // 0x0068 data_store32         0 6          ;; store 0x19171311
        //
        // 0x0070 data_load            0 2          ;; load 0xf0e0d0c0_19171311
        // 0x0078 data_load32          4 2          ;; load 0xf0e0d0c0
        // 0x0080 data_load32_i16_u    6 2          ;; load 0xf0e0
        // 0x0088 data_load32_i16_s    6 2          ;; load 0xf0e0
        // 0x0090 data_load32_i8_u     7 2          ;; load 0xf0
        // 0x0098 data_load32_i8_s     7 2          ;; load 0xf0
        //
        // 0x00a0 data_load32_f32      0 3          ;; load f32
        // 0x00a8 data_load_f64        0 4          ;; load f64
        // 0x00b0 data_load            0 5          ;; load 0xf0e0d0c0_19171311
        // 0x00b8 data_load32          0 6          ;; load 0x19171311
        // 0x00c0 end
        //
        // (f32, f64) -> (i64,i32,i32,i32,i32,i32, f32,f64 ,i64,i32)

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::data_load32_i8_u, 3, 1)
            .write_opcode_i16_i32(Opcode::data_load32_i8_u, 2, 1)
            .write_opcode_i16_i32(Opcode::data_load32_i16_u, 0, 1)
            .write_opcode_i16_i32(Opcode::data_load32, 0, 0)
            //
            .write_opcode_i16_i32(Opcode::data_store32, 0, 2)
            .write_opcode_i16_i32(Opcode::data_store16, 4, 2)
            .write_opcode_i16_i32(Opcode::data_store8, 6, 2)
            .write_opcode_i16_i32(Opcode::data_store8, 7, 2)
            //
            .write_opcode_i16_i32(Opcode::data_store, 0, 4) // store f64
            .write_opcode_i16_i32(Opcode::data_store32, 0, 3) // store f32
            //
            .write_opcode_i16_i32(Opcode::data_load, 0, 2)
            .write_opcode_i16_i32(Opcode::data_store, 0, 5)
            //
            .write_opcode_i16_i32(Opcode::data_load, 0, 2)
            .write_opcode_i16_i32(Opcode::data_store32, 0, 6)
            //
            .write_opcode_i16_i32(Opcode::data_load, 0, 2)
            .write_opcode_i16_i32(Opcode::data_load32, 4, 2)
            .write_opcode_i16_i32(Opcode::data_load32_i16_u, 6, 2)
            .write_opcode_i16_i32(Opcode::data_load32_i16_s, 6, 2)
            .write_opcode_i16_i32(Opcode::data_load32_i8_u, 7, 2)
            .write_opcode_i16_i32(Opcode::data_load32_i8_s, 7, 2)
            //
            .write_opcode_i16_i32(Opcode::data_load32_f32, 0, 3)
            .write_opcode_i16_i32(Opcode::data_load_f64, 0, 4)
            //
            .write_opcode_i16_i32(Opcode::data_load, 0, 5)
            .write_opcode_i16_i32(Opcode::data_load32, 0, 6)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function_and_data_sections(
            vec![
                DataEntry::from_i32(0x19171311),
                DataEntry::from_i32(0xf0e0d0c0),
            ],
            vec![],
            vec![
                UninitDataEntry::from_bytes(8, 8),
                UninitDataEntry::from_f32(),
                UninitDataEntry::from_f64(),
                UninitDataEntry::from_i64(),
                UninitDataEntry::from_i32(),
            ],
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
            vec![
                LocalVariableEntry::from_bytes(8, 8),
                LocalVariableEntry::from_f32(),
                LocalVariableEntry::from_f64(),
                LocalVariableEntry::from_i64(),
                LocalVariableEntry::from_i32(),
            ], // local vars
            code0,
        );

        let program0 = InMemoryProgram::new(vec![binary0]);
        let program_context0 = program0.build_program_context().unwrap();
        let mut thread_context0 = program_context0.new_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &vec![
                ForeignValue::Float32(3.1415926f32),
                ForeignValue::Float64(2.9979e8f64),
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
                ForeignValue::Float32(3.1415926f32),
                ForeignValue::Float64(2.9979e8f64),
                //
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0x19171311u32),
            ]
        );
    }

    #[test]
    fn test_process_data_long_load_store() {
        // init_runtime();

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

        // bytecodes
        //
        // 0x0000 i32_imm              0x19171311
        // 0x0008 i32_imm              0x0
        // 0x0010 data_long_store32    0
        // 0x0018 i32_imm              0xd0c0
        // 0x0020 i32_imm              0x4
        // 0x0028 data_long_store16    0
        // 0x0030 i32_imm              0xe0
        // 0x0038 i32_imm              0x6
        // 0x0040 data_long_store8     0
        // 0x0048 i32_imm              0xf0
        // 0x0050 i32_imm              0x7
        // 0x0058 data_long_store8     0
        // 0x0060 i32_imm              0x0
        // 0x0068 data_long_load       0
        // 0x0070 i32_imm              0x0
        // 0x0078 data_long_store      1
        // 0x0080 i32_imm              0x0
        // 0x0088 data_long_load       0
        // 0x0090 i32_imm              0x4
        // 0x0098 data_long_load32     0
        // 0x00a0 i32_imm              0x6
        // 0x00a8 data_long_load32_i16_u 0
        // 0x00b0 i32_imm              0x6
        // 0x00b8 data_long_load32_i16_s 0
        // 0x00c0 i32_imm              0x7
        // 0x00c8 data_long_load32_i8_u 0
        // 0x00d0 i32_imm              0x7
        // 0x00d8 data_long_load32_i8_s 0
        // 0x00e0 i32_imm              0x0
        // 0x00e8 data_long_load       1
        // 0x00f0 i32_imm              0x0
        // 0x00f8 data_long_load32     1
        // 0x0100 i32_imm              0x0
        // 0x0108 data_long_load32_i16_u 1
        // 0x0110 i32_imm              0x0
        // 0x0118 data_long_load32_i8_u 1
        // 0x0120 end
        //
        // () -> (i64,i32,i32,i32,i32,i32,  i64,i32,i32,i32)

        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 0x19171311)
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::data_long_store32, 0) // store 32
            //
            .write_opcode_i32(Opcode::i32_imm, 0xd0c0)
            .write_opcode_i32(Opcode::i32_imm, 4)
            .write_opcode_i32(Opcode::data_long_store16, 0) // store 16
            //
            .write_opcode_i32(Opcode::i32_imm, 0xe0)
            .write_opcode_i32(Opcode::i32_imm, 6)
            .write_opcode_i32(Opcode::data_long_store8, 0) // store 8
            //
            .write_opcode_i32(Opcode::i32_imm, 0xf0)
            .write_opcode_i32(Opcode::i32_imm, 7)
            .write_opcode_i32(Opcode::data_long_store8, 0) // store 8
            //
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::data_long_load, 0) // load 64
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::data_long_store, 1) // store 64
            //
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::data_long_load, 0)
            .write_opcode_i32(Opcode::i32_imm, 4)
            .write_opcode_i32(Opcode::data_long_load32, 0)
            .write_opcode_i32(Opcode::i32_imm, 6)
            .write_opcode_i32(Opcode::data_long_load32_i16_u, 0)
            .write_opcode_i32(Opcode::i32_imm, 6)
            .write_opcode_i32(Opcode::data_long_load32_i16_s, 0)
            .write_opcode_i32(Opcode::i32_imm, 7)
            .write_opcode_i32(Opcode::data_long_load32_i8_u, 0)
            .write_opcode_i32(Opcode::i32_imm, 7)
            .write_opcode_i32(Opcode::data_long_load32_i8_s, 0)
            //
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::data_long_load, 1)
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::data_long_load32, 1)
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::data_long_load32_i16_u, 1)
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::data_long_load32_i8_u, 1)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function_and_data_sections(
            vec![],
            vec![],
            vec![
                UninitDataEntry::from_bytes(8, 8),
                UninitDataEntry::from_bytes(8, 8),
            ],
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
        );

        let program0 = InMemoryProgram::new(vec![binary0]);
        let program_context0 = program0.build_program_context().unwrap();
        let mut thread_context0 = program_context0.new_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &vec![]);
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
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0x19171311u32),
                ForeignValue::UInt32(0x00001311u32), // extend from i16 to i32
                ForeignValue::UInt32(0x00000011u32), // extend from i8 to i32
            ]
        );
    }
}
