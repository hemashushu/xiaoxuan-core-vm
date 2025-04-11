// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::{
    memory_access::MemoryAccess, resizeable_memory::ResizeableMemory, thread_context::ThreadContext,
};

use super::{HandleResult, Handler};

pub fn memory_load_i64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16) (operand memory_address:i64) -> i64
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    thread_context.memory.load_i64(total_offset, dst_ptr);

    HandleResult::Move(4)
}

pub fn memory_load_i32_s(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16) (operand memory_address:i64) -> i64
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    thread_context.memory.load_i32_s(total_offset, dst_ptr);

    HandleResult::Move(4)
}

pub fn memory_load_i32_u(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16) (operand memory_address:i64) -> i64
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    thread_context.memory.load_i32_u(total_offset, dst_ptr);

    HandleResult::Move(4)
}

pub fn memory_load_i16_s(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16) (operand memory_address:i64) -> i64
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    thread_context.memory.load_i16_s(total_offset, dst_ptr);

    HandleResult::Move(4)
}

pub fn memory_load_i16_u(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16) (operand memory_address:i64) -> i64
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    thread_context.memory.load_i16_u(total_offset, dst_ptr);

    HandleResult::Move(4)
}

pub fn memory_load_i8_s(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16) (operand memory_address:i64) -> i64
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    thread_context.memory.load_i8_s(total_offset, dst_ptr);

    HandleResult::Move(4)
}

pub fn memory_load_i8_u(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16) (operand memory_address:i64) -> i64
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    thread_context.memory.load_i8_u(total_offset, dst_ptr);

    HandleResult::Move(4)
}

pub fn memory_load_f64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16) (operand memory_address:i64) -> i64
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    thread_context.memory.load_f64(total_offset, dst_ptr);

    HandleResult::Move(4)
}

pub fn memory_load_f32(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16) (operand memory_address:i64) -> i64
    let offset_bytes = thread_context.get_param_i16();
    let address = thread_context.stack.pop_i64_u();

    let total_offset = address as usize + offset_bytes as usize;
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    thread_context.memory.load_f32(total_offset, dst_ptr);

    HandleResult::Move(4)
}

pub fn memory_store_i64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16) (operand memory_address:i64 value:i64) -> ()
    let offset_bytes = thread_context.get_param_i16();

    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    let address = thread_context.stack.pop_i64_u();
    let total_offset = address as usize + offset_bytes as usize;
    thread_context.memory.store_i64(src_ptr, total_offset);

    HandleResult::Move(4)
}

pub fn memory_store_i32(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16) (operand memory_address:i64 value:i64) -> ()
    let offset_bytes = thread_context.get_param_i16();

    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    let address = thread_context.stack.pop_i64_u();
    let total_offset = address as usize + offset_bytes as usize;
    thread_context.memory.store_i32(src_ptr, total_offset);

    HandleResult::Move(4)
}

pub fn memory_store_i16(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16) (operand memory_address:i64 value:i64) -> ()
    let offset_bytes = thread_context.get_param_i16();

    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    let address = thread_context.stack.pop_i64_u();
    let total_offset = address as usize + offset_bytes as usize;
    thread_context.memory.store_i16(src_ptr, total_offset);

    HandleResult::Move(4)
}

pub fn memory_store_i8(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16) (operand memory_address:i64 value:i64) -> ()
    let offset_bytes = thread_context.get_param_i16();

    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    let address = thread_context.stack.pop_i64_u();
    let total_offset = address as usize + offset_bytes as usize;
    thread_context.memory.store_i8(src_ptr, total_offset);

    HandleResult::Move(4)
}

pub fn memory_capacity(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // () -> pages:i64
    let pages = thread_context.memory.get_capacity_in_pages();
    thread_context.stack.push_i64_u(pages as u64);

    HandleResult::Move(2)
}

pub fn memory_resize(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand pages:i64) -> new_pages:i64
    let pages = thread_context.stack.pop_i64_u();
    let new_pages = thread_context.memory.resize(pages as usize);
    thread_context.stack.push_i64_u(new_pages as u64);

    HandleResult::Move(2)
}

pub fn memory_fill(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand address:i64, value:i8, count:i64) -> ()
    let count = thread_context.stack.pop_i64_u() as usize;
    let value = thread_context.stack.pop_i32_u() as u8;
    let address = thread_context.stack.pop_i64_u() as usize;

    thread_context.memory.fill(address, value, count);

    HandleResult::Move(2)
}

pub fn memory_copy(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand dst_address:i64, src_address:i64, count:i64) -> ()
    let count = thread_context.stack.pop_i64_u() as usize;
    let src_address = thread_context.stack.pop_i64_u() as usize;
    let dst_address = thread_context.stack.pop_i64_u() as usize;

    thread_context.memory.copy(dst_address, src_address, count);

    HandleResult::Move(2)
}

#[cfg(test)]
mod tests {
    use crate::{
        handler::Handler, in_memory_program_source::InMemoryProgramSource,
        process::process_function,
    };
    use anc_context::process_resource::ProgramSource;
    use anc_image::{
        bytecode_writer::BytecodeWriterHelper,
        utils::helper_build_module_binary_with_single_function,
    };
    use anc_isa::{opcode::Opcode, ForeignValue, OperandDataType};

    #[test]
    fn test_handler_memory_capacity() {
        // () -> (i64, i64, i64, i64, i64)

        let code0 = BytecodeWriterHelper::new()
            // get the capacity
            .append_opcode(Opcode::memory_capacity)
            // resize - increase
            .append_opcode_i32(Opcode::imm_i32, 2)
            .append_opcode(Opcode::memory_resize)
            // resize - increase
            .append_opcode_i32(Opcode::imm_i32, 4)
            .append_opcode(Opcode::memory_resize)
            // resize - decrease
            .append_opcode_i32(Opcode::imm_i32, 1)
            .append_opcode(Opcode::memory_resize)
            // get the capcity
            .append_opcode(Opcode::memory_capacity)
            .append_opcode(Opcode::end)
            .to_bytes();

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
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);

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
    fn test_handler_memory_load_and_store() {
        //       |low address                                                              high address|
        //       |                                                                                     |
        // index |0x100                              0x200  0x300  0x400                     0x500     |
        //  type |bytes-------------------|         |f32|  |f64|  |i64------------------|   |i32-------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 13 17 19
        //       |imm        |imm     |  |          |      |      ^                          ^
        //       |store32    |store16 |  |          |sf32  |sf64  |                          |
        //        step0       step1   |  |          |step5 |step4 |                          |
        //                         imm|  |imm       |      |      |                          |
        //       |              store8|  |store8    |      |      |store64                   |store32
        //       |               step2    step3     |      |      |                          |
        //       \----->--load64-->---------------------------->--/-->-------------------->--/
        //
        //       11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 13 17 19
        //       |           |        |  |load8u    |      |      |                          |
        //       |           |        |  |load8s  loadf32  |      |                          |
        //       |           |        |                  loadf64  |                          |
        //       |           |        |load16u                    |                          |
        //       |           |        |load16s                 load64                      load32u
        //       |           |                                                             load32s
        //       |load64     |load32u
        //                   |load32s
        //
        // (f32, f64) -> (i64,i32,i32,i32,i32,i32,i32,  f32,f64,  i64,i32,i32)

        let code0 = BytecodeWriterHelper::new()
            // note that the init size of heap is 0
            // change the capacity of heap before test
            // init heap size
            .append_opcode_i32(Opcode::imm_i32, 1)
            .append_opcode(Opcode::memory_resize)
            //
            .append_opcode_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i32(Opcode::imm_i32, 0x19171311)
            // (param offset_bytes:i16) (operand memory_address:i64 value:i32) -> ()
            .append_opcode_i16(Opcode::memory_store_i32, 0)
            //
            .append_opcode_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i32(Opcode::imm_i32, 0xd0c0)
            .append_opcode_i16(Opcode::memory_store_i16, 4)
            //
            .append_opcode_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i32(Opcode::imm_i32, 0xe0)
            .append_opcode_i16(Opcode::memory_store_i8, 6)
            //
            .append_opcode_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i32(Opcode::imm_i32, 0xf0)
            .append_opcode_i16(Opcode::memory_store_i8, 7)
            //
            .append_opcode_i64(Opcode::imm_i64, 0x300) // addr for store f64
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode_i16(Opcode::memory_store_f64, 0) // store f64
            .append_opcode_i64(Opcode::imm_i64, 0x200) // addr for store f32
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode_i16(Opcode::memory_store_f32, 0) // store f32
            //
            .append_opcode_i64(Opcode::imm_i64, 0x400) // addr for store_i64
            .append_opcode_i64(Opcode::imm_i64, 0x100) // addr for load
            // (param offset_bytes:i16) (operand memory_address:i64) -> i64
            .append_opcode_i16(Opcode::memory_load_i64, 0)
            .append_opcode_i16(Opcode::memory_store_i64, 0)
            //
            .append_opcode_i64(Opcode::imm_i64, 0x500) // addr for store_i32
            .append_opcode_i64(Opcode::imm_i64, 0x100) // addr for load
            .append_opcode_i16(Opcode::memory_load_i64, 0)
            .append_opcode_i16(Opcode::memory_store_i32, 0)
            //
            .append_opcode_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::memory_load_i64, 0)
            .append_opcode_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::memory_load_i32_u, 4)
            .append_opcode_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::memory_load_i32_s, 4)
            .append_opcode_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::memory_load_i16_u, 6)
            .append_opcode_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::memory_load_i16_s, 6)
            .append_opcode_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::memory_load_i8_u, 7)
            .append_opcode_i64(Opcode::imm_i64, 0x100)
            .append_opcode_i16(Opcode::memory_load_i8_s, 7)
            //
            .append_opcode_i64(Opcode::imm_i64, 0x200)
            .append_opcode_i16(Opcode::memory_load_f32, 0)
            .append_opcode_i64(Opcode::imm_i64, 0x300)
            .append_opcode_i16(Opcode::memory_load_f64, 0)
            //
            .append_opcode_i64(Opcode::imm_i64, 0x400)
            .append_opcode_i16(Opcode::memory_load_i64, 0)
            .append_opcode_i64(Opcode::imm_i64, 0x500)
            .append_opcode_i16(Opcode::memory_load_i32_u, 0)
            .append_opcode_i64(Opcode::imm_i64, 0x500)
            .append_opcode_i16(Opcode::memory_load_i32_s, 0)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[OperandDataType::F32, OperandDataType::F64], // params
            &[
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                //
                OperandDataType::F32,
                OperandDataType::F64,
                //
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I32,
            ], // results
            &[],                                           // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &handler,
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
                ForeignValue::U32(0xf0e0d0c0u32), // i32u
                ForeignValue::U32(0xf0e0d0c0u32), // i32s
                ForeignValue::U32(0xf0e0u32),
                ForeignValue::U32(0xfffff0e0u32), // extend from i16 to i32
                ForeignValue::U32(0xf0u32),
                ForeignValue::U32(0xfffffff0u32), // extend from i8 to i32
                // group 1
                ForeignValue::F32(std::f32::consts::PI),
                ForeignValue::F64(std::f64::consts::E),
                // group 2
                ForeignValue::U64(0xf0e0d0c0_19171311u64),
                ForeignValue::U32(0x19171311u32), // i32u
                ForeignValue::U32(0x19171311u32), // i32s
            ]
        );
    }
}
