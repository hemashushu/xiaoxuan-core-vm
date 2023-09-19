// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::{memory::Memory, thread::Thread};

use super::InterpretResult;

pub fn local_load(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 local_variable_index:i32)
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    do_local_load(thread, local_variable_index as usize, offset_bytes as usize)
}

pub fn local_long_load(thread: &mut Thread) -> InterpretResult {
    // (param local_variable_index:i32) (operand offset_bytes:i32)
    let local_variable_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_local_load(thread, local_variable_index as usize, offset_bytes as usize)
}

fn do_local_load(
    thread: &mut Thread,
    local_variable_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    // there are two ways to transfer data from memory to stack, one way
    // is to read data from memory to a (integer) variable and then
    // push the variable onto the stack, e.g.
    //
    // ```rust
    // let data = stack.read_u64(data_address);
    // stack.push_u64(data);
    // ```
    //
    // the another way is the 'memcpy'.
    // the latter has a higher efficiency because it eliminates data conversion,
    // so the second method is adopted.

    let dst_ptr = thread.stack.push_from_memory();
    let data_address =
        thread.get_local_variable_address_by_index_and_offset(local_variable_index, offset_bytes);
    thread.stack.load_64(data_address, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn local_load32(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 local_variable_index:i32)
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    do_local_load32(thread, local_variable_index as usize, offset_bytes as usize)
}

pub fn local_long_load32(thread: &mut Thread) -> InterpretResult {
    // (param local_variable_index:i32) (operand offset_bytes:i32)
    let local_variable_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_local_load32(thread, local_variable_index as usize, offset_bytes as usize)
}

fn do_local_load32(
    thread: &mut Thread,
    local_variable_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread.stack.push_from_memory();
    let data_address =
        thread.get_local_variable_address_by_index_and_offset(local_variable_index, offset_bytes);
    thread.stack.load_32(data_address, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn local_load32_i16_s(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 local_variable_index:i32)
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    do_local_load32_i16_s(thread, local_variable_index as usize, offset_bytes as usize)
}

pub fn local_long_load32_i16_s(thread: &mut Thread) -> InterpretResult {
    // (param local_variable_index:i32) (operand offset_bytes:i32)
    let local_variable_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_local_load32_i16_s(thread, local_variable_index as usize, offset_bytes as usize)
}

fn do_local_load32_i16_s(
    thread: &mut Thread,
    local_variable_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread.stack.push_from_memory();
    let data_address =
        thread.get_local_variable_address_by_index_and_offset(local_variable_index, offset_bytes);
    thread
        .stack
        .load_32_extend_from_i16_s(data_address, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn local_load32_i16_u(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 local_variable_index:i32)
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    do_local_load32_i16_u(thread, local_variable_index as usize, offset_bytes as usize)
}

pub fn local_long_load32_i16_u(thread: &mut Thread) -> InterpretResult {
    // (param local_variable_index:i32) (operand offset_bytes:i32)
    let local_variable_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_local_load32_i16_u(thread, local_variable_index as usize, offset_bytes as usize)
}

fn do_local_load32_i16_u(
    thread: &mut Thread,
    local_variable_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread.stack.push_from_memory();
    let data_address =
        thread.get_local_variable_address_by_index_and_offset(local_variable_index, offset_bytes);
    thread
        .stack
        .load_32_extend_from_i16_u(data_address, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn local_load32_i8_s(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 local_variable_index:i32)
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    do_local_load32_i8_s(thread, local_variable_index as usize, offset_bytes as usize)
}

pub fn local_long_load32_i8_s(thread: &mut Thread) -> InterpretResult {
    // (param local_variable_index:i32) (operand offset_bytes:i32)
    let local_variable_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_local_load32_i8_s(thread, local_variable_index as usize, offset_bytes as usize)
}

fn do_local_load32_i8_s(
    thread: &mut Thread,
    local_variable_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread.stack.push_from_memory();
    let data_address =
        thread.get_local_variable_address_by_index_and_offset(local_variable_index, offset_bytes);
    thread.stack.load_32_extend_from_i8_s(data_address, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn local_load32_i8_u(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 local_variable_index:i32)
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    do_local_load32_i8_u(thread, local_variable_index as usize, offset_bytes as usize)
}

pub fn local_long_load32_i8_u(thread: &mut Thread) -> InterpretResult {
    // (param local_variable_index:i32) (operand offset_bytes:i32)
    let local_variable_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_local_load32_i8_u(thread, local_variable_index as usize, offset_bytes as usize)
}

fn do_local_load32_i8_u(
    thread: &mut Thread,
    local_variable_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread.stack.push_from_memory();
    let data_address =
        thread.get_local_variable_address_by_index_and_offset(local_variable_index, offset_bytes);
    thread.stack.load_32_extend_from_i8_u(data_address, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn local_load32_f32(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 local_variable_index:i32)
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    do_local_load32_f32(thread, local_variable_index as usize, offset_bytes as usize)
}

pub fn local_long_load32_f32(thread: &mut Thread) -> InterpretResult {
    // (param local_variable_index:i32) (operand offset_bytes:i32)
    let local_variable_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_local_load32_f32(thread, local_variable_index as usize, offset_bytes as usize)
}

fn do_local_load32_f32(
    thread: &mut Thread,
    local_variable_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread.stack.push_from_memory();
    let data_address =
        thread.get_local_variable_address_by_index_and_offset(local_variable_index, offset_bytes);
    thread.stack.load_32_with_float_check(data_address, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn local_load_f64(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 local_variable_index:i32)
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    do_local_load_f64(thread, local_variable_index as usize, offset_bytes as usize)
}

pub fn local_long_load_f64(thread: &mut Thread) -> InterpretResult {
    // (param local_variable_index:i32) (operand offset_bytes:i32)
    let local_variable_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_local_load_f64(thread, local_variable_index as usize, offset_bytes as usize)
}

fn do_local_load_f64(
    thread: &mut Thread,
    local_variable_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread.stack.push_from_memory();
    let data_address =
        thread.get_local_variable_address_by_index_and_offset(local_variable_index, offset_bytes);
    thread.stack.load_64_with_float_check(data_address, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn local_store(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 local_variable_index:i32)
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    do_local_store(thread, local_variable_index as usize, offset_bytes as usize)
}

pub fn local_long_store(thread: &mut Thread) -> InterpretResult {
    // (param local_variable_index:i32) (operand offset_bytes:i32)
    let local_variable_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_local_store(thread, local_variable_index as usize, offset_bytes as usize)
}

fn do_local_store(
    thread: &mut Thread,
    local_variable_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let src_ptr = thread.stack.pop_to_memory();
    let data_address =
        thread.get_local_variable_address_by_index_and_offset(local_variable_index, offset_bytes);
    thread.stack.store_64(src_ptr, data_address);

    InterpretResult::MoveOn(8)
}

pub fn local_store32(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 local_variable_index:i32)
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    do_local_store32(thread, local_variable_index as usize, offset_bytes as usize)
}

pub fn local_long_store32(thread: &mut Thread) -> InterpretResult {
    // (param local_variable_index:i32) (operand offset_bytes:i32)
    let local_variable_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_local_store32(thread, local_variable_index as usize, offset_bytes as usize)
}

fn do_local_store32(
    thread: &mut Thread,
    local_variable_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let src_ptr = thread.stack.pop_to_memory();
    let data_address =
        thread.get_local_variable_address_by_index_and_offset(local_variable_index, offset_bytes);
    thread.stack.store_32(src_ptr, data_address);

    InterpretResult::MoveOn(8)
}

pub fn local_store16(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 local_variable_index:i32)
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    do_local_store16(thread, local_variable_index as usize, offset_bytes as usize)
}

pub fn local_long_store16(thread: &mut Thread) -> InterpretResult {
    // (param local_variable_index:i32) (operand offset_bytes:i32)
    let local_variable_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_local_store16(thread, local_variable_index as usize, offset_bytes as usize)
}

fn do_local_store16(
    thread: &mut Thread,
    local_variable_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let src_ptr = thread.stack.pop_to_memory();
    let data_address =
        thread.get_local_variable_address_by_index_and_offset(local_variable_index, offset_bytes);
    thread.stack.store_16(src_ptr, data_address);

    InterpretResult::MoveOn(8)
}

pub fn local_store8(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 local_variable_index:i32)
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    do_local_store8(thread, local_variable_index as usize, offset_bytes as usize)
}

pub fn local_long_store8(thread: &mut Thread) -> InterpretResult {
    // (param local_variable_index:i32) (operand offset_bytes:i32)
    let local_variable_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_local_store8(thread, local_variable_index as usize, offset_bytes as usize)
}

fn do_local_store8(
    thread: &mut Thread,
    local_variable_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let src_ptr = thread.stack.pop_to_memory();
    let data_address =
        thread.get_local_variable_address_by_index_and_offset(local_variable_index, offset_bytes);
    thread.stack.store_8(src_ptr, data_address);

    InterpretResult::MoveOn(8)
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        load_modules_binary,
        module_image::local_variable_section::VariableItemEntry,
        utils::{build_module_binary_with_single_function, BytecodeWriter},
    };
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    use crate::{init_runtime, interpreter::process_function, thread::Thread};

    #[test]
    fn test_process_local_load_store() {
        //       |low address                                                              high address|
        //       |                                                                                     |
        // index |0                                  1      2      3                         4         |
        //  type |bytes-------------------|         |f32|  |f64|  |i64------------------|   |i32-------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 12 17 19
        //       |           |        |  |          |      |      ^                          ^
        //       |store32    |store16 |  |          |sf32  |sf64  |                          |
        //        step0       step1   |  |          |step5 |step4 |                          |
        //                      store8|  |          |      |      |                          |
        //       |              step2    |store8    |      |      |store64                   |store32
        //       |                        step3     |      |      |                          |
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
        // 0x0000 i32_imm              0x19171311
        // 0x0008 local_store32        0 0          ;; store 0x19171311
        // 0x0010 i32_imm              0xd0c0
        // 0x0018 local_store16        4 0          ;; store 0xd0c0
        // 0x0020 i32_imm              0xe0
        // 0x0028 local_store8         6 0          ;; store 0xe0
        // 0x0030 i32_imm              0xf0
        // 0x0038 local_store8         7 0          ;; store 0xf0
        //
        // 0x0040 local_store          0 2          ;; store f64
        // 0x0048 local_store32        0 1          ;; store f32
        //
        // 0x0050 local_load           0 0
        // 0x0058 local_store          0 3          ;; store 0xf0e0d0c0_19171311
        // 0x0060 local_load           0 0
        // 0x0068 local_store32        0 4          ;; store 0x19171311
        //
        // 0x0070 local_load           0 0          ;; load 0xf0e0d0c0_19171311
        // 0x0078 local_load32         4 0          ;; load 0xf0e0d0c0
        // 0x0080 local_load32_i16_u   6 0          ;; load 0xf0e0
        // 0x0088 local_load32_i16_s   6 0          ;; load 0xf0e0
        // 0x0090 local_load32_i8_u    7 0          ;; load 0xf0
        // 0x0098 local_load32_i8_s    7 0          ;; load 0xf0
        //
        // 0x00a0 local_load32_f32     0 1          ;; load f32
        // 0x00a8 local_load_f64       0 2          ;; load f64
        // 0x00b0 local_load           0 3          ;; load 0xf0e0d0c0_19171311
        // 0x00b8 local_load32         0 4          ;; load 0x19171311
        // 0x00c0 end
        //
        // (f32, f64) -> (i64,i32,i32,i32,i32,i32, f32,f64 ,i64,i32)

        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 0x19171311)
            .write_opcode_i16_i32(Opcode::local_store32, 0, 0)
            .write_opcode_i32(Opcode::i32_imm, 0xd0c0)
            .write_opcode_i16_i32(Opcode::local_store16, 4, 0)
            .write_opcode_i32(Opcode::i32_imm, 0xe0)
            .write_opcode_i16_i32(Opcode::local_store8, 6, 0)
            .write_opcode_i32(Opcode::i32_imm, 0xf0)
            .write_opcode_i16_i32(Opcode::local_store8, 7, 0)
            //
            // here access arguments directly
            // note that the general method is using 'local_load' instruction
            .write_opcode_i16_i32(Opcode::local_store, 0, 2) // store f64
            .write_opcode_i16_i32(Opcode::local_store32, 0, 1) // store f32
            //
            .write_opcode_i16_i32(Opcode::local_load, 0, 0)
            .write_opcode_i16_i32(Opcode::local_store, 0, 3)
            //
            .write_opcode_i16_i32(Opcode::local_load, 0, 0)
            .write_opcode_i16_i32(Opcode::local_store32, 0, 4)
            //
            .write_opcode_i16_i32(Opcode::local_load, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load32, 4, 0)
            .write_opcode_i16_i32(Opcode::local_load32_i16_u, 6, 0)
            .write_opcode_i16_i32(Opcode::local_load32_i16_s, 6, 0)
            .write_opcode_i16_i32(Opcode::local_load32_i8_u, 7, 0)
            .write_opcode_i16_i32(Opcode::local_load32_i8_s, 7, 0)
            //
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 1)
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 2)
            //
            .write_opcode_i16_i32(Opcode::local_load, 0, 3)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 4)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
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
            code0,
            vec![
                VariableItemEntry::from_bytes(8, 8),
                VariableItemEntry::from_f32(),
                VariableItemEntry::from_f64(),
                VariableItemEntry::from_i64(),
                VariableItemEntry::from_i32(),
            ], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        init_runtime();
        let result0 = process_function(
            &mut thread0,
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
    fn test_process_local_long_load_store() {
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
        // 0x0010 local_long_store32   0
        // 0x0018 i32_imm              0xd0c0
        // 0x0020 i32_imm              0x4
        // 0x0028 local_long_store16   0
        // 0x0030 i32_imm              0xe0
        // 0x0038 i32_imm              0x6
        // 0x0040 local_long_store8    0
        // 0x0048 i32_imm              0xf0
        // 0x0050 i32_imm              0x7
        // 0x0058 local_long_store8    0
        // 0x0060 i32_imm              0x0
        // 0x0068 local_long_load      0
        // 0x0070 i32_imm              0x0
        // 0x0078 local_long_store     1
        // 0x0080 i32_imm              0x0
        // 0x0088 local_long_load      0
        // 0x0090 i32_imm              0x4
        // 0x0098 local_long_load32    0
        // 0x00a0 i32_imm              0x6
        // 0x00a8 local_long_load32_i16_u 0
        // 0x00b0 i32_imm              0x6
        // 0x00b8 local_long_load32_i16_s 0
        // 0x00c0 i32_imm              0x7
        // 0x00c8 local_long_load32_i8_u 0
        // 0x00d0 i32_imm              0x7
        // 0x00d8 local_long_load32_i8_s 0
        // 0x00e0 i32_imm              0x0
        // 0x00e8 local_long_load      1
        // 0x00f0 i32_imm              0x0
        // 0x00f8 local_long_load32    1
        // 0x0100 i32_imm              0x0
        // 0x0108 local_long_load32_i16_u 1
        // 0x0110 i32_imm              0x0
        // 0x0118 local_long_load32_i8_u 1
        // 0x0120 end
        //
        // () -> (i64,i32,i32,i32,i32,i32,  i64,i32,i32,i32)

        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 0x19171311)
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::local_long_store32, 0) // store 32
            //
            .write_opcode_i32(Opcode::i32_imm, 0xd0c0)
            .write_opcode_i32(Opcode::i32_imm, 4)
            .write_opcode_i32(Opcode::local_long_store16, 0) // store 16
            //
            .write_opcode_i32(Opcode::i32_imm, 0xe0)
            .write_opcode_i32(Opcode::i32_imm, 6)
            .write_opcode_i32(Opcode::local_long_store8, 0) // store 8
            //
            .write_opcode_i32(Opcode::i32_imm, 0xf0)
            .write_opcode_i32(Opcode::i32_imm, 7)
            .write_opcode_i32(Opcode::local_long_store8, 0) // store 8
            //
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::local_long_load, 0) // load 64
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::local_long_store, 1) // store 64
            //
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::local_long_load, 0)
            .write_opcode_i32(Opcode::i32_imm, 4)
            .write_opcode_i32(Opcode::local_long_load32, 0)
            .write_opcode_i32(Opcode::i32_imm, 6)
            .write_opcode_i32(Opcode::local_long_load32_i16_u, 0)
            .write_opcode_i32(Opcode::i32_imm, 6)
            .write_opcode_i32(Opcode::local_long_load32_i16_s, 0)
            .write_opcode_i32(Opcode::i32_imm, 7)
            .write_opcode_i32(Opcode::local_long_load32_i8_u, 0)
            .write_opcode_i32(Opcode::i32_imm, 7)
            .write_opcode_i32(Opcode::local_long_load32_i8_s, 0)
            //
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::local_long_load, 1)
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::local_long_load32, 1)
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::local_long_load32_i16_u, 1)
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::local_long_load32_i8_u, 1)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
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
            code0,
            vec![
                VariableItemEntry::from_bytes(8, 8),
                VariableItemEntry::from_bytes(8, 8),
            ], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        init_runtime();
        let result0 = process_function(&mut thread0, 0, 0, &vec![]);
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
