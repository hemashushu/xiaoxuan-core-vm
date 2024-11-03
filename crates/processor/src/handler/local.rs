// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::{memory::Memory, thread_context::ThreadContext};

use super::{HandleResult, Handler};

const DATA_LENGTH_IN_BYTES_64_BIT: usize = 8;
const DATA_LENGTH_IN_BYTES_32_BIT: usize = 4;
const DATA_LENGTH_IN_BYTES_16_BIT: usize = 2;
const DATA_LENGTH_IN_BYTES_8_BIT: usize = 1;

pub fn local_load_i64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_local_load_i64(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn local_load_extend_i64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_load_i64(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_local_load_i64(
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
    thread_context.stack.load_i64(data_address, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load_i32_s(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_local_load_i32_s(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn local_load_extend_i32_s(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_load_i32_s(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_local_load_i32_s(
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
    thread_context.stack.load_i32_s(data_address, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load_i32_u(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_local_load_i32_u(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn local_load_extend_i32_u(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_load_i32_u(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_local_load_i32_u(
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
    thread_context.stack.load_i32_u(data_address, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load_i16_s(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_local_load_i16_s(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn local_load_extend_i16_s(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_load_i16_s(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_local_load_i16_s(
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
    thread_context.stack.load_i16_s(data_address, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load_i16_u(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_local_load_i16_u(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn local_load_extend_i16_u(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_load_i16_u(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_local_load_i16_u(
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
    thread_context.stack.load_i16_u(data_address, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load_i8_s(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_local_load_i8_s(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn local_load_extend_i8_s(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_load_i8_s(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_local_load_i8_s(
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
    thread_context.stack.load_i8_s(data_address, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load_i8_u(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_local_load_i8_u(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn local_load_extend_i8_u(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_load_i8_u(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_local_load_i8_u(
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
    thread_context.stack.load_i8_u(data_address, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load_f32(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_local_load_f32(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn local_load_extend_f32(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_load_f32(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_local_load_f32(
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
    thread_context.stack.load_f32(data_address, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_load_f64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    do_local_load_f64(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

pub fn local_load_extend_f64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_load_f64(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
    )
}

fn do_local_load_f64(
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
    thread_context.stack.load_f64(data_address, dst_ptr);

    HandleResult::Move(8)
}

pub fn local_store_i64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_local_store_i64(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

pub fn local_store_extend_i64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_store_i64(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

fn do_local_store_i64(
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
    thread_context.stack.store_i64(src_ptr, data_address);

    HandleResult::Move(8)
}

pub fn local_store_i32(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_local_store_i32(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

pub fn local_store_extend_i32(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_store_i32(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

fn do_local_store_i32(
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
    thread_context.stack.store_i32(src_ptr, data_address);

    HandleResult::Move(8)
}

pub fn local_store_i16(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_local_store_i16(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

pub fn local_store_extend_i16(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_store_i16(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

fn do_local_store_i16(
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
    thread_context.stack.store_i16(src_ptr, data_address);

    HandleResult::Move(8)
}

pub fn local_store_i8(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16)
    let (reversed_index, offset_bytes, local_variable_index) =
        thread_context.get_param_i16_i16_i16();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_local_store_i8(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

pub fn local_store_extend_i8(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32)
    let (reversed_index, local_variable_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i32_u();
    do_local_store_i8(
        thread_context,
        reversed_index,
        local_variable_index as usize,
        offset_bytes as usize,
        src_ptr,
    )
}

fn do_local_store_i8(
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
    thread_context.stack.store_i8(src_ptr, data_address);

    HandleResult::Move(8)
}

// note::
//
// all testing here are ignore the 'reversed_index' because it relies on
// the instruction 'block'.
// the 'reversed_index' will be tested on the module 'interpreter/control_flow'.

#[cfg(test)]
mod tests {
    use ancvm_context::resource::Resource;
    use ancvm_image::{
        bytecode_writer::BytecodeWriterHelper, entry::LocalVariableEntry,
        utils::helper_build_module_binary_with_single_function,
    };
    use ancvm_isa::{opcode::Opcode, ForeignValue, OperandDataType};

    use crate::{
        handler::Handler, in_memory_resource::InMemoryResource, process::process_function,
    };

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
        //       |imm        |imm     |  |          |      |      ^                          ^
        //       |store32    |store16 |  |          |sf32  |sf64  |                          |
        //        step0       step1   |  |          |step5 |step4 |                          |
        //                         imm|  |imm       |      |      |                          |
        //       |              store8|  |store8    |      |      |store64                   |store32
        //       |               step2     step3     |      |      |                          |
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
            .append_opcode_i32(Opcode::imm_i32, 0x19171311)
            .append_opcode_i16_i16_i16(Opcode::local_store_i32, 0, 0, 2)
            .append_opcode_i32(Opcode::imm_i32, 0xd0c0)
            .append_opcode_i16_i16_i16(Opcode::local_store_i16, 0, 4, 2)
            .append_opcode_i32(Opcode::imm_i32, 0xe0)
            .append_opcode_i16_i16_i16(Opcode::local_store_i8, 0, 6, 2)
            .append_opcode_i32(Opcode::imm_i32, 0xf0)
            .append_opcode_i16_i16_i16(Opcode::local_store_i8, 0, 7, 2)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_store_f64, 0, 0, 4) // store f64
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_store_f32, 0, 0, 3) // store f32
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_store_i64, 0, 0, 5)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_store_i32, 0, 0, 6)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 4, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 4, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i16_u, 0, 6, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i16_s, 0, 6, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i8_u, 0, 7, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load_i8_s, 0, 7, 2)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 3)
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 4)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 5)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 0, 6)
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_s, 0, 0, 6)
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
                OperandDataType::I32,
                //
                OperandDataType::F32,
                OperandDataType::F64,
                //
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I32,
            ], // results
            vec![
                LocalVariableEntry::from_raw(8, 8),
                LocalVariableEntry::from_f32(),
                LocalVariableEntry::from_f64(),
                LocalVariableEntry::from_i64(),
                LocalVariableEntry::from_i32(),
            ], // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &handler,
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

    #[test]
    fn test_interpreter_local_load_and_store_extend() {
        //       |low address                                 high address|
        //       |                                                        |
        // index |0                                  1                    |
        //  type |bytes-------------------|         |bytes----------------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0         11 13 17 19 c0 d0 e0 f0
        //       |           |        |  |          ^
        //       |store32    |store16 |  |          |
        //        step0       step1   |  |          |
        //                         imm|  |imm       |
        //       |              store8|  |store8    |store64
        //       |              step2     step3     |
        //       \----->--load64-->-----------------/
        //
        //       11 13 17 19 c0 d0    e0 f0         11 13 17 19 c0 d0 e0 f0
        //       |           |        |  |load8u    |
        //       |           |        |  |load8s    |load64
        //       |           |        |             |load32u
        //       |           |        |load16u      |load16u
        //       |           |        |load16s      |load8u
        //       |           |
        //       |load64     |load32u
        //                   |load32s
        //
        // () -> (i64,i32,i32,i32,i32,i32,i32,  i64,i32,i32,i32)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 0) // offset
            .append_opcode_i32(Opcode::imm_i32, 0x19171311) // value
            .append_opcode_i16_i32(Opcode::local_store_extend_i32, 0, 0) // store 32
            //
            .append_opcode_i32(Opcode::imm_i32, 4)
            .append_opcode_i32(Opcode::imm_i32, 0xd0c0)
            .append_opcode_i16_i32(Opcode::local_store_extend_i16, 0, 0) // store 16
            //
            .append_opcode_i32(Opcode::imm_i32, 6)
            .append_opcode_i32(Opcode::imm_i32, 0xe0)
            .append_opcode_i16_i32(Opcode::local_store_extend_i8, 0, 0) // store 8
            //
            .append_opcode_i32(Opcode::imm_i32, 7)
            .append_opcode_i32(Opcode::imm_i32, 0xf0)
            .append_opcode_i16_i32(Opcode::local_store_extend_i8, 0, 0) // store 8
            //
            .append_opcode_i32(Opcode::imm_i32, 0) // offset for store_extend
            .append_opcode_i32(Opcode::imm_i32, 0) // offset for load_extend
            .append_opcode_i16_i32(Opcode::local_load_extend_i64, 0, 0) // load 64
            .append_opcode_i16_i32(Opcode::local_store_extend_i64, 0, 1) // store 64
            //
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i16_i32(Opcode::local_load_extend_i64, 0, 0)
            .append_opcode_i32(Opcode::imm_i32, 4)
            .append_opcode_i16_i32(Opcode::local_load_extend_i32_u, 0, 0)
            .append_opcode_i32(Opcode::imm_i32, 4)
            .append_opcode_i16_i32(Opcode::local_load_extend_i32_s, 0, 0)
            .append_opcode_i32(Opcode::imm_i32, 6)
            .append_opcode_i16_i32(Opcode::local_load_extend_i16_u, 0, 0)
            .append_opcode_i32(Opcode::imm_i32, 6)
            .append_opcode_i16_i32(Opcode::local_load_extend_i16_s, 0, 0)
            .append_opcode_i32(Opcode::imm_i32, 7)
            .append_opcode_i16_i32(Opcode::local_load_extend_i8_u, 0, 0)
            .append_opcode_i32(Opcode::imm_i32, 7)
            .append_opcode_i16_i32(Opcode::local_load_extend_i8_s, 0, 0)
            //
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i16_i32(Opcode::local_load_extend_i64, 0, 1)
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i16_i32(Opcode::local_load_extend_i32_u, 0, 1)
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i16_i32(Opcode::local_load_extend_i16_u, 0, 1)
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i16_i32(Opcode::local_load_extend_i8_u, 0, 1)
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
                OperandDataType::I32,
                //
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
            ], // results
            vec![
                LocalVariableEntry::from_raw(8, 8),
                LocalVariableEntry::from_raw(8, 8),
            ], // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
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
                ForeignValue::U64(0xf0e0d0c0_19171311u64),
                ForeignValue::U32(0x19171311u32), // i32u
                ForeignValue::U32(0x00001311u32), // extend from i16 to i32
                ForeignValue::U32(0x00000011u32), // extend from i8 to i32
            ]
        );
    }

    #[test]
    fn test_interpreter_local_bounds_check_offset_out_of_range() {
        let code0 = BytecodeWriterHelper::new()
            // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> i64
            .append_opcode_i16_i16_i16(Opcode::local_load_i32_u, 0, 2, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                               // params
            vec![],                               // results
            vec![LocalVariableEntry::from_i32()], // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();

        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let mut thread_context0 = process_context0.create_thread_context();
            // err: offset(+length) is out of data area
            let _ = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_interpreter_local_bounds_check_type_length_overflow() {
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                               // params
            vec![],                               // results
            vec![LocalVariableEntry::from_i32()], // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();

        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let mut thread_context0 = process_context0.create_thread_context();
            // err: load i32 variable with load_i64 instruction
            let _ = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_interpreter_local_bounds_check_index_out_of_range() {
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 11)
            // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) (operand value:i32) -> ()
            .append_opcode_i16_i16_i16(Opcode::local_store_i32, 0, 0, 2)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                               // params
            vec![],                               // results
            vec![LocalVariableEntry::from_i32()], // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();

        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let mut thread_context0 = process_context0.create_thread_context();
            // err: access non-exist index local variable
            let _ = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_interpreter_local_bounds_check_extend_offset_out_of_range() {
        let code0 = BytecodeWriterHelper::new()
            // offset for load
            .append_opcode_i32(Opcode::imm_i32, 2)
            // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32) -> i64
            .append_opcode_i16_i32(Opcode::local_load_extend_i32_u, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                               // params
            vec![],                               // results
            vec![LocalVariableEntry::from_i32()], // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();

        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let mut thread_context0 = process_context0.create_thread_context();
            // err: offset(+length) is out of data area
            let _ = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }
}
