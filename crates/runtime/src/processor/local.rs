// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::{
    memory::Memory,
    thread::{ProgramCounter, Thread},
};

use super::InterpretResult;

pub fn local_load(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 local_variable_index:i32)

    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    let data_address = get_data_address_by_index(thread, local_variable_index, offset_bytes);

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
    thread.stack.load_64(data_address, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn local_load32(thread: &mut Thread) -> InterpretResult {
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    let data_address = get_data_address_by_index(thread, local_variable_index, offset_bytes);

    let dst_ptr = thread.stack.push_from_memory();
    thread.stack.load_32(data_address, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn local_load32_i16_s(thread: &mut Thread) -> InterpretResult {
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    let data_address = get_data_address_by_index(thread, local_variable_index, offset_bytes);

    let dst_ptr = thread.stack.push_from_memory();
    thread.stack.load_32_extend_from_i16(data_address, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn local_load32_i16_u(thread: &mut Thread) -> InterpretResult {
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    let data_address = get_data_address_by_index(thread, local_variable_index, offset_bytes);

    let dst_ptr = thread.stack.push_from_memory();
    thread.stack.load_32_extend_from_u16(data_address, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn local_load32_i8_s(thread: &mut Thread) -> InterpretResult {
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    let data_address = get_data_address_by_index(thread, local_variable_index, offset_bytes);

    let dst_ptr = thread.stack.push_from_memory();
    thread.stack.load_32_extend_from_i8(data_address, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn local_load32_i8_u(thread: &mut Thread) -> InterpretResult {
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    let data_address = get_data_address_by_index(thread, local_variable_index, offset_bytes);

    let dst_ptr = thread.stack.push_from_memory();
    thread.stack.load_32_extend_from_u8(data_address, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn local_load32_f32(thread: &mut Thread) -> InterpretResult {
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    let data_address = get_data_address_by_index(thread, local_variable_index, offset_bytes);

    let dst_ptr = thread.stack.push_from_memory();
    thread.stack.load_32_with_float_check(data_address, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn local_load_f64(thread: &mut Thread) -> InterpretResult {
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    let data_address = get_data_address_by_index(thread, local_variable_index, offset_bytes);

    let dst_ptr = thread.stack.push_from_memory();
    thread.stack.load_64_with_float_check(data_address, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn local_store(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 local_variable_index:i32)

    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    let data_address = get_data_address_by_index(thread, local_variable_index, offset_bytes);

    let src_ptr = thread.stack.pop_to_memory();
    thread.stack.store_64(src_ptr, data_address);

    InterpretResult::MoveOn(8)
}

pub fn local_store32(thread: &mut Thread) -> InterpretResult {
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    let data_address = get_data_address_by_index(thread, local_variable_index, offset_bytes);

    let src_ptr = thread.stack.pop_to_memory();
    thread.stack.store_32(src_ptr, data_address);

    InterpretResult::MoveOn(8)
}

pub fn local_store16(thread: &mut Thread) -> InterpretResult {
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    let data_address = get_data_address_by_index(thread, local_variable_index, offset_bytes);

    let src_ptr = thread.stack.pop_to_memory();
    thread.stack.store_16(src_ptr, data_address);

    InterpretResult::MoveOn(8)
}

pub fn local_store8(thread: &mut Thread) -> InterpretResult {
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    let data_address = get_data_address_by_index(thread, local_variable_index, offset_bytes);

    let src_ptr = thread.stack.pop_to_memory();
    thread.stack.store_8(src_ptr, data_address);

    InterpretResult::MoveOn(8)
}

fn get_data_address_by_index(
    thread: &Thread,
    local_variable_index: u32,
    offset_bytes: u16,
) -> usize {
    let local_start_address = thread.stack.get_local_variables_start_address();

    // get the local variable info
    let ProgramCounter {
        instruction_address: _,
        module_index,
    } = thread.pc;

    let internal_function_index = thread
        .stack
        .get_function_frame()
        .frame
        .internal_function_index;

    let variable_item = &thread.context.modules[module_index]
        .local_variable_section
        .get_variable_list(internal_function_index)[local_variable_index as usize];

    local_start_address + variable_item.var_offset as usize + offset_bytes as usize
}
