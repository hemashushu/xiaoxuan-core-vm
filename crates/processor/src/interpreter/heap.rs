// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::{
    memory::Memory, resizeable_memory::ResizeableMemory, thread_context::ThreadContext,
};

use super::InterpretResult;

pub fn heap_load_i64(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    thread_context.heap.load_64(total_offset, dst_ptr);

    InterpretResult::Move(4)
}

pub fn heap_load32_i32(thread_context: &mut ThreadContext) -> InterpretResult {
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

pub fn heap_load64_f64(thread_context: &mut ThreadContext) -> InterpretResult {
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

pub fn heap_store_i64(thread_context: &mut ThreadContext) -> InterpretResult {
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
    use crate::{in_memory_program_resource::InMemoryProgramResource, interpreter::process_function};
    use ancvm_binary::{
        bytecode_writer::BytecodeWriter, utils::helper_build_module_binary_with_single_function,
    };
    use ancvm_context::program_resource::ProgramResource;
    use ancvm_isa::{opcode::Opcode, OperandDataType, ForeignValue};

    #[test]
    fn test_interpreter_heap_capacity() {
        // () -> (i64, i64, i64, i64, i64)

        let code0 = BytecodeWriter::new()
            // get the capacity
            .append_opcode(Opcode::heap_capacity)
            // resize - increase
            .append_opcode_i32(Opcode::imm_i32, 2)
            .append_opcode(Opcode::heap_resize)
            // resize - increase
            .append_opcode_i32(Opcode::imm_i32, 4)
            .append_opcode(Opcode::heap_resize)
            // resize - decrease
            .append_opcode_i32(Opcode::imm_i32, 1)
            .append_opcode(Opcode::heap_resize)
            // get the capcity
            .append_opcode(Opcode::heap_capacity)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![], // params
            vec![
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
                OperandDataType::I64,
            ], // results
            vec![], // local vars
            code0,
        );

        let program_resource0 = InMemoryProgramResource::new(vec![binary0]);
        let process_context0 = program_resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);

        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U64(0),
                ForeignValue::U64(2),
                ForeignValue::U64(4),
                ForeignValue::U64(1),
                ForeignValue::U64(1),
            ]
        );
    }

    #[test]
    fn test_interpreter_heap_load_and_store() {
        //       |low address                                                              high address|
        //       |                                                                                     |
        // index |0x100                              0x200  0x300  0x400                     0x500     |
        //  type |bytes-------------------|         |f32|  |f64|  |i64------------------|   |i32-------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 13 17 19
        //       |           |        |  |          |      |      ^                          ^
        //       |store32    |store16 |  |          |sf32  |sf64  |                          |
        //        step0       step1   |  |          |step5 |step4 |                          |
        //                      store8|  |          |      |      |                          |
        //       |              step2    |store8    |      |      |store64                   |store32
        //       |                        step3     |      |      |                          |
        //       \----->--load64-->---------------------------->--/-->-------------------->--/
        //
        //       11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 13 17 19
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

            // init heap size
            .append_opcode_i32(Opcode::imm_i32, 1)
            .append_opcode(Opcode::heap_resize)
            // .append_opcode(Opcode::drop)
            //
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i32(Opcode::imm_i32, 0x19171311)
            .append_opcode_i16(Opcode::heap_store32, 0)
            //
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i32(Opcode::imm_i32, 0xd0c0)
            .append_opcode_i16(Opcode::heap_store16, 4)
            //
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i32(Opcode::imm_i32, 0xe0)
            .append_opcode_i16(Opcode::heap_store8, 6)
            //
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i32(Opcode::imm_i32, 0xf0)
            .append_opcode_i16(Opcode::heap_store8, 7)
            //
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x300)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode_i16(Opcode::heap_store_i64, 0) // store f64
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x200)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode_i16(Opcode::heap_store32, 0) // store f32
            //
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x400)
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::heap_load_i64, 0)
            .append_opcode_i16(Opcode::heap_store_i64, 0)
            //
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x500)
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::heap_load_i64, 0)
            .append_opcode_i16(Opcode::heap_store32, 0)
            //
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::heap_load_i64, 0)
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::heap_load32_i32, 4)
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::heap_load32_i16_u, 6)
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::heap_load32_i16_s, 6)
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::heap_load32_i8_u, 7)
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::heap_load32_i8_s, 7)
            //
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x200)
            .append_opcode_i16(Opcode::heap_load32_f32, 0)
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x300)
            .append_opcode_i16(Opcode::heap_load64_f64, 0)
            //
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x400)
            .append_opcode_i16(Opcode::heap_load_i64, 0)
            .append_opcode_pesudo_i64(Opcode::imm_i64, 0x500)
            .append_opcode_i16(Opcode::heap_load32_i32, 0)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![OperandDataType::F32, OperandDataType::F64], // params
            vec![
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::F32,
                OperandDataType::F64,
                OperandDataType::I64,
                OperandDataType::I32,
            ], // results
            vec![],                             // local vars
            code0,
        );

        let program_resource0 = InMemoryProgramResource::new(vec![binary0]);
        let process_context0 = program_resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[
                // https://baseconvert.com/ieee-754-floating-point
                // https://www.binaryconvert.com/convert_float.html
                ForeignValue::F32(std::f32::consts::PI),
                ForeignValue::F64(std::f64::consts::E),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::U64(0xf0e0d0c0_19171311u64),
                ForeignValue::U32(0xf0e0d0c0u32),
                ForeignValue::U32(0xf0e0u32),
                ForeignValue::U32(0xfffff0e0u32), // extend from i16 to i32
                ForeignValue::U32(0xf0u32),
                ForeignValue::U32(0xfffffff0u32), // extend from i8 to i32
                // group 1
                ForeignValue::F32(std::f32::consts::PI),
                ForeignValue::F64(std::f64::consts::E),
                // group 2
                ForeignValue::U64(0xf0e0d0c0_19171311u64),
                ForeignValue::U32(0x19171311u32),
            ]
        );
    }
}
