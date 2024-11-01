// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::{memory::Memory, thread_context::ThreadContext};

use super::HandleResult;

const DATA_LENGTH_IN_BYTES_64_BIT: usize = 8;
const DATA_LENGTH_IN_BYTES_32_BIT: usize = 4;
const DATA_LENGTH_IN_BYTES_16_BIT: usize = 2;
const DATA_LENGTH_IN_BYTES_8_BIT: usize = 1;

pub fn local_load_i64(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_local_load64_i64(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn local_offset_load64_i64(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_load64_i64(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_local_load64_i64(
    thread_context: &mut ThreadContext,
    reversed_index: u16,
    local_variable_index: usize,
    offset_bytes: usize,
) -> HandleResult {
    // there are two approachs to transfer data from memory to stack, one
    // is to read data from memory to a (integer) variable and then
    // push the variable onto the stack, e.g.
    //
    // ```rust
    // let data = stack.read_u64(data_address);
    // stack.push_u64(data);
    // ```
    //
    // the another approach is the 'memcpy'.
    // the latter has a higher efficiency because it eliminates data conversion,
    // so the second method is adopted.

    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let data_address = thread_context
        .get_local_variable_address_by_index_and_offset_with_bounds_check(
            reversed_index,
            local_variable_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_64_BIT,
        );
    thread_context.stack.load_64(data_address, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load32_i32(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_local_load32_i32(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn local_offset_load32_i32(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_load32_i32(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_local_load32_i32(
    thread_context: &mut ThreadContext,
    reversed_index: u16,
    local_variable_index: usize,
    offset_bytes: usize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let data_address = thread_context
        .get_local_variable_address_by_index_and_offset_with_bounds_check(
            reversed_index,
            local_variable_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_32_BIT,
        );
    thread_context.stack.load_32(data_address, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load32_i16_s(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_local_load32_i16_s(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn local_offset_load32_i16_s(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_load32_i16_s(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_local_load32_i16_s(
    thread_context: &mut ThreadContext,
    reversed_index: u16,
    local_variable_index: usize,
    offset_bytes: usize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let data_address = thread_context
        .get_local_variable_address_by_index_and_offset_with_bounds_check(
            reversed_index,
            local_variable_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_16_BIT,
        );
    thread_context
        .stack
        .load_32_extend_from_i16_s(data_address, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load32_i16_u(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_local_load32_i16_u(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn local_offset_load32_i16_u(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_load32_i16_u(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_local_load32_i16_u(
    thread_context: &mut ThreadContext,
    reversed_index: u16,
    local_variable_index: usize,
    offset_bytes: usize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let data_address = thread_context
        .get_local_variable_address_by_index_and_offset_with_bounds_check(
            reversed_index,
            local_variable_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_16_BIT,
        );
    thread_context
        .stack
        .load_32_extend_from_i16_u(data_address, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load32_i8_s(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_local_load32_i8_s(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn local_offset_load32_i8_s(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_load32_i8_s(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_local_load32_i8_s(
    thread_context: &mut ThreadContext,
    reversed_index: u16,
    local_variable_index: usize,
    offset_bytes: usize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let data_address = thread_context
        .get_local_variable_address_by_index_and_offset_with_bounds_check(
            reversed_index,
            local_variable_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_8_BIT,
        );
    thread_context
        .stack
        .load_32_extend_from_i8_s(data_address, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load32_i8_u(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_local_load32_i8_u(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn local_offset_load32_i8_u(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_load32_i8_u(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_local_load32_i8_u(
    thread_context: &mut ThreadContext,
    reversed_index: u16,
    local_variable_index: usize,
    offset_bytes: usize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let data_address = thread_context
        .get_local_variable_address_by_index_and_offset_with_bounds_check(
            reversed_index,
            local_variable_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_8_BIT,
        );
    thread_context
        .stack
        .load_32_extend_from_i8_u(data_address, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load32_f32(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_local_load32_f32(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn local_offset_load32_f32(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_load32_f32(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_local_load32_f32(
    thread_context: &mut ThreadContext,
    reversed_index: u16,
    local_variable_index: usize,
    offset_bytes: usize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let data_address = thread_context
        .get_local_variable_address_by_index_and_offset_with_bounds_check(
            reversed_index,
            local_variable_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_32_BIT,
        );
    thread_context
        .stack
        .load_32_with_float_check(data_address, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load64_f64(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_local_load64_f64(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn local_offset_load64_f64(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_load64_f64(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_local_load64_f64(
    thread_context: &mut ThreadContext,
    reversed_index: u16,
    local_variable_index: usize,
    offset_bytes: usize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let data_address = thread_context
        .get_local_variable_address_by_index_and_offset_with_bounds_check(
            reversed_index,
            local_variable_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_64_BIT,
        );
    thread_context
        .stack
        .load_64_with_float_check(data_address, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_store_i64(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_local_store64(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

pub fn local_offset_store64(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_store64(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

fn do_local_store64(
    thread_context: &mut ThreadContext,
    reversed_index: u16,
    local_variable_index: usize,
    offset_bytes: usize,
    src_ptr: *const u8,
) -> HandleResult {
    let data_address = thread_context
        .get_local_variable_address_by_index_and_offset_with_bounds_check(
            reversed_index,
            local_variable_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_64_BIT,
        );
    thread_context.stack.store_64(src_ptr, data_address);

    HandleResult::Move(8)
}

pub fn local_store32(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_local_store32(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

pub fn local_offset_store32(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_store32(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

fn do_local_store32(
    thread_context: &mut ThreadContext,
    reversed_index: u16,
    local_variable_index: usize,
    offset_bytes: usize,
    src_ptr: *const u8,
) -> HandleResult {
    let data_address = thread_context
        .get_local_variable_address_by_index_and_offset_with_bounds_check(
            reversed_index,
            local_variable_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_32_BIT,
        );
    thread_context.stack.store_32(src_ptr, data_address);

    HandleResult::Move(8)
}

pub fn local_store16(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_local_store16(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

pub fn local_offset_store16(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_store16(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

fn do_local_store16(
    thread_context: &mut ThreadContext,
    reversed_index: u16,
    local_variable_index: usize,
    offset_bytes: usize,
    src_ptr: *const u8,
) -> HandleResult {
    let data_address = thread_context
        .get_local_variable_address_by_index_and_offset_with_bounds_check(
            reversed_index,
            local_variable_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_16_BIT,
        );
    thread_context.stack.store_16(src_ptr, data_address);

    HandleResult::Move(8)
}

pub fn local_store8(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_local_store8(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

pub fn local_offset_store8(thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_store8(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

fn do_local_store8(
    thread_context: &mut ThreadContext,
    reversed_index: u16,
    local_variable_index: usize,
    offset_bytes: usize,
    src_ptr: *const u8,
) -> HandleResult {
    let data_address = thread_context
        .get_local_variable_address_by_index_and_offset_with_bounds_check(
            reversed_index,
            local_variable_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_8_BIT,
        );
    thread_context.stack.store_8(src_ptr, data_address);

    HandleResult::Move(8)
}

// note::
//
// all testing here are ignore the 'reversed_index' because it relies on
// the instruction 'block'.
// the 'reversed_index' will be tested on the module 'interpreter/control_flow'.

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        bytecode_writer::BytecodeWriter, utils::helper_build_module_binary_with_single_function,
    };

    use crate::{in_memory_program_resource::InMemoryProgramResource, interpreter::process_function};
    use ancvm_context::program_resource::ProgramResource;
    use ancvm_isa::{entry::LocalVariableEntry, opcode::Opcode, OperandDataType, ForeignValue};

    #[test]
    fn test_interpreter_local_load_and_store() {
        // args index (also local var):     0       1
        // data type:                       f32     f64
        //
        //       |low address                                                              high address|
        // local |                                                                                     |
        // index |2                                  3      4      5                         6         |
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

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 0x19171311)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 0, 2)
            .append_opcode_i32(Opcode::imm_i32, 0xd0c0)
            .append_opcode_i16_i16_i16(Opcode::local_store16, 0, 4, 2)
            .append_opcode_i32(Opcode::imm_i32, 0xe0)
            .append_opcode_i16_i16_i16(Opcode::local_store8, 0, 6, 2)
            .append_opcode_i32(Opcode::imm_i32, 0xf0)
            .append_opcode_i16_i16_i16(Opcode::local_store8, 0, 7, 2)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_store_i64, 0, 0, 4) // store f64
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 0, 3) // store f32
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_store_i64, 0, 0, 5)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 0, 6)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 4, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i16_u, 0, 6, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i16_s, 0, 6, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i8_u, 0, 7, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i8_s, 0, 7, 2)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 3)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 4)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 5)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 6)
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
            vec![
                LocalVariableEntry::from_bytes(8, 8),
                LocalVariableEntry::from_f32(),
                LocalVariableEntry::from_f64(),
                LocalVariableEntry::from_i64(),
                LocalVariableEntry::from_i32(),
            ], // local vars
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[
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

    #[test]
    fn test_interpreter_local_offset_load_and_store() {
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

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i32(Opcode::imm_i32, 0x19171311)
            .append_opcode_i16_i32(Opcode::local_offset_store32, 0, 0) // store 32
            //
            .append_opcode_i32(Opcode::imm_i32, 4)
            .append_opcode_i32(Opcode::imm_i32, 0xd0c0)
            .append_opcode_i16_i32(Opcode::local_offset_store16, 0, 0) // store 16
            //
            .append_opcode_i32(Opcode::imm_i32, 6)
            .append_opcode_i32(Opcode::imm_i32, 0xe0)
            .append_opcode_i16_i32(Opcode::local_offset_store8, 0, 0) // store 8
            //
            .append_opcode_i32(Opcode::imm_i32, 7)
            .append_opcode_i32(Opcode::imm_i32, 0xf0)
            .append_opcode_i16_i32(Opcode::local_offset_store8, 0, 0) // store 8
            //
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i16_i32(Opcode::local_offset_load64_i64, 0, 0) // load 64
            .append_opcode_i16_i32(Opcode::local_offset_store64, 0, 1) // store 64
            //
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i16_i32(Opcode::local_offset_load64_i64, 0, 0)
            .append_opcode_i32(Opcode::imm_i32, 4)
            .append_opcode_i16_i32(Opcode::local_offset_load32_i32, 0, 0)
            .append_opcode_i32(Opcode::imm_i32, 6)
            .append_opcode_i16_i32(Opcode::local_offset_load32_i16_u, 0, 0)
            .append_opcode_i32(Opcode::imm_i32, 6)
            .append_opcode_i16_i32(Opcode::local_offset_load32_i16_s, 0, 0)
            .append_opcode_i32(Opcode::imm_i32, 7)
            .append_opcode_i16_i32(Opcode::local_offset_load32_i8_u, 0, 0)
            .append_opcode_i32(Opcode::imm_i32, 7)
            .append_opcode_i16_i32(Opcode::local_offset_load32_i8_s, 0, 0)
            //
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i16_i32(Opcode::local_offset_load64_i64, 0, 1)
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i16_i32(Opcode::local_offset_load32_i32, 0, 1)
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i16_i32(Opcode::local_offset_load32_i16_u, 0, 1)
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i16_i32(Opcode::local_offset_load32_i8_u, 0, 1)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![], // params
            vec![
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
            ], // results
            vec![
                LocalVariableEntry::from_bytes(8, 8),
                LocalVariableEntry::from_bytes(8, 8),
            ], // local vars
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
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
                ForeignValue::U64(0xf0e0d0c0_19171311u64),
                ForeignValue::U32(0x19171311u32),
                ForeignValue::U32(0x00001311u32), // extend from i16 to i32
                ForeignValue::U32(0x00000011u32), // extend from i8 to i32
            ]
        );
    }

    #[test]
    fn test_interpreter_local_bounds_check0() {
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 2, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                               // params
            vec![],                               // results
            vec![LocalVariableEntry::from_i32()], // local vars
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();

        let prev_hook = std::panic::take_hook(); // let panic silent
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let mut thread_context0 = process_context0.create_thread_context();
            let _ = process_function(&mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_interpreter_local_bounds_check1() {
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                               // params
            vec![],                               // results
            vec![LocalVariableEntry::from_i32()], // local vars
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();

        let prev_hook = std::panic::take_hook(); // let panic silent
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let mut thread_context0 = process_context0.create_thread_context();
            let _ = process_function(&mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_interpreter_local_bounds_check2() {
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 11)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 2, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                               // params
            vec![],                               // results
            vec![LocalVariableEntry::from_i32()], // local vars
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();

        let prev_hook = std::panic::take_hook(); // let panic silent
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let mut thread_context0 = process_context0.create_thread_context();
            let _ = process_function(&mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_interpreter_local_bounds_check3() {
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 2) // offset
            .append_opcode_i16_i32(Opcode::local_offset_load32_i32, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                               // params
            vec![],                               // results
            vec![LocalVariableEntry::from_i32()], // local vars
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();

        let prev_hook = std::panic::take_hook(); // let panic silent
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let mut thread_context0 = process_context0.create_thread_context();
            let _ = process_function(&mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }
}
