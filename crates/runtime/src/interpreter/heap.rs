// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_program::{
    memory::Memory, resizeable_memory::ResizeableMemory, thread_context::ThreadContext,
};

use super::InterpretResult;

pub fn heap_load(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    thread_context.heap.load_64(total_offset, dst_ptr);

    InterpretResult::Move(4)
}

pub fn heap_load32(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    thread_context.heap.load_32(total_offset, dst_ptr);

    InterpretResult::Move(4)
}

pub fn heap_load32_i16_s(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    thread_context
        .heap
        .load_32_extend_from_i16_s(total_offset, dst_ptr);

    InterpretResult::Move(4)
}

pub fn heap_load32_i16_u(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    thread_context
        .heap
        .load_32_extend_from_i16_u(total_offset, dst_ptr);

    InterpretResult::Move(4)
}

pub fn heap_load32_i8_s(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    thread_context
        .heap
        .load_32_extend_from_i8_s(total_offset, dst_ptr);

    InterpretResult::Move(4)
}

pub fn heap_load32_i8_u(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    thread_context
        .heap
        .load_32_extend_from_i8_u(total_offset, dst_ptr);

    InterpretResult::Move(4)
}

pub fn heap_load_f64(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    thread_context
        .heap
        .load_64_with_float_check(total_offset, dst_ptr);

    InterpretResult::Move(4)
}

pub fn heap_load32_f32(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    thread_context
        .heap
        .load_32_with_float_check(total_offset, dst_ptr);

    InterpretResult::Move(4)
}

pub fn heap_store(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64 number:i64)
    let offset_bytes = thread_context.get_param_i16();

    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let address = thread_context.stack.pop_i64_u();
    let total_offset = address as usize + offset_bytes as usize;
    thread_context.heap.store_64(src_ptr, total_offset);

    InterpretResult::Move(4)
}

pub fn heap_store32(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64 number:i64)
    let offset_bytes = thread_context.get_param_i16();

    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let address = thread_context.stack.pop_i64_u();
    let total_offset = address as usize + offset_bytes as usize;
    thread_context.heap.store_32(src_ptr, total_offset);

    InterpretResult::Move(4)
}

pub fn heap_store16(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64 number:i64)
    let offset_bytes = thread_context.get_param_i16();

    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let address = thread_context.stack.pop_i64_u();
    let total_offset = address as usize + offset_bytes as usize;
    thread_context.heap.store_16(src_ptr, total_offset);

    InterpretResult::Move(4)
}

pub fn heap_store8(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64 number:i64)
    let offset_bytes = thread_context.get_param_i16();

    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let address = thread_context.stack.pop_i64_u();
    let total_offset = address as usize + offset_bytes as usize;
    thread_context.heap.store_8(src_ptr, total_offset);

    InterpretResult::Move(4)
}

pub fn heap_capacity(thread_context: &mut ThreadContext) -> InterpretResult {
    // () -> pages:i64
    let pages = thread_context.heap.get_capacity_in_pages();
    thread_context.stack.push_i64_u(pages as u64);

    InterpretResult::Move(2)
}

pub fn heap_resize(thread_context: &mut ThreadContext) -> InterpretResult {
    // (operand pages:i64) -> new_pages:i64
    let pages = thread_context.stack.pop_i64_u();
    let new_pages = thread_context.heap.resize(pages as usize);
    thread_context.stack.push_i64_u(new_pages as u64);

    InterpretResult::Move(2)
}

pub fn heap_fill(thread_context: &mut ThreadContext) -> InterpretResult {
    // (operand address:i64, value:i8, count:i64) -> ()
    let count = thread_context.stack.pop_i64_u() as usize;
    let value = thread_context.stack.pop_i32_u() as u8;
    let address = thread_context.stack.pop_i64_u() as usize;

    thread_context.heap.fill(address, value, count);

    InterpretResult::Move(2)
}

pub fn heap_copy(thread_context: &mut ThreadContext) -> InterpretResult {
    // (operand dst_address:i64, src_address:i64, length_in_bytes:i64) -> ()

    let length_in_bytes = thread_context.stack.pop_i64_u() as usize;
    let src_address = thread_context.stack.pop_i64_u() as usize;
    let dst_address = thread_context.stack.pop_i64_u() as usize;

    thread_context
        .heap
        .copy(dst_address, src_address, length_in_bytes);

    InterpretResult::Move(2)
}

#[cfg(test)]
mod tests {
    use crate::{in_memory_program_source::InMemoryProgramSource, interpreter::process_function};
    use ancvm_binary::utils::{build_module_binary_with_single_function, BytecodeWriter};
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    #[test]
    fn test_process_heap_load_store() {
        //       |low address                                                              high address|
        //       |                                                                                     |
        // index |0x100                              0x200  0x300  0x400                     0x500     |
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

        let code0 = BytecodeWriter::new()
            // note that the init size of heap is 0
            // change the capacity of heap before test
            .write_opcode_i32(Opcode::i32_imm, 1)
            .write_opcode(Opcode::heap_resize)
            .write_opcode(Opcode::drop)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i32(Opcode::i32_imm, 0x19171311)
            .write_opcode_i16(Opcode::heap_store32, 0)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i32(Opcode::i32_imm, 0xd0c0)
            .write_opcode_i16(Opcode::heap_store16, 4)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i32(Opcode::i32_imm, 0xe0)
            .write_opcode_i16(Opcode::heap_store32, 6)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i32(Opcode::i32_imm, 0xf0)
            .write_opcode_i16(Opcode::heap_store32, 7)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x300)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .write_opcode_i16(Opcode::heap_store, 0) // store f64
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x200)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode_i16(Opcode::heap_store32, 0) // store f32
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x400)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_load, 0)
            .write_opcode_i16(Opcode::heap_store, 0)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x500)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_load, 0)
            .write_opcode_i16(Opcode::heap_store32, 0)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_load, 0)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_load32, 4)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_load32_i16_u, 6)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_load32_i16_s, 6)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_load32_i8_u, 7)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_load32_i8_s, 7)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x200)
            .write_opcode_i16(Opcode::heap_load32_f32, 0)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x300)
            .write_opcode_i16(Opcode::heap_load_f64, 0)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x400)
            .write_opcode_i16(Opcode::heap_load, 0)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x500)
            .write_opcode_i16(Opcode::heap_load32, 0)
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
            &[
                // https://baseconvert.com/ieee-754-floating-point
                // https://www.binaryconvert.com/convert_float.html
                ForeignValue::Float32(std::f32::consts::PI), // 3.1415926f32
                // 0x40490FDA
                // 218,15,73,64
                ForeignValue::Float64(std::f64::consts::E), // deprecated 2.9979e8f64
                                                            // 0x41B1DE6EB0000000
                                                            // 0,0,0,176,110,222,177,65
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
                ForeignValue::Float32(std::f32::consts::PI), // 3.1415926f32
                ForeignValue::Float64(std::f64::consts::E),  // deprecated 2.9979e8f64
                //
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0x19171311u32),
            ]
        );
    }

    #[test]
    fn test_process_heap_capacity() {

        // () -> (i64, i64, i64, i64, i64)

        let code0 = BytecodeWriter::new()
            // get the capacity
            .write_opcode(Opcode::heap_capacity)
            // resize - increase
            .write_opcode_i32(Opcode::i32_imm, 2)
            .write_opcode(Opcode::heap_resize)
            // resize - increase
            .write_opcode_i32(Opcode::i32_imm, 4)
            .write_opcode(Opcode::heap_resize)
            // resize - decrease
            .write_opcode_i32(Opcode::i32_imm, 1)
            .write_opcode(Opcode::heap_resize)
            // get the capcity
            .write_opcode(Opcode::heap_capacity)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![], // params
            vec![
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
            ], // results
            vec![], // local varslist which
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);

        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt64(0),
                ForeignValue::UInt64(2),
                ForeignValue::UInt64(4),
                ForeignValue::UInt64(1),
                ForeignValue::UInt64(1),
            ]
        );
    }
}
