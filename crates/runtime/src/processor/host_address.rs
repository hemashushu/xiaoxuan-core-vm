// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

//! it is currently assumed that the target architecture is 64-bit.

use crate::{
    memory::Memory,
    thread::{ProgramCounter, Thread},
};

use super::InterpretResult;

pub fn host_addr_local(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 local_variable_index:i32)
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    do_host_addr_local(thread, local_variable_index as usize, offset_bytes as usize)
}

pub fn host_addr_local_long(thread: &mut Thread) -> InterpretResult {
    // (param local_variable_index:i32) (operand offset_bytes:i32)
    let local_variable_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_u32();
    do_host_addr_local(thread, local_variable_index as usize, offset_bytes as usize)
}

fn do_host_addr_local(
    thread: &mut Thread,
    local_variable_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let local_start_address = thread.stack.get_local_variables_start_address();

    // get the local variable info
    let ProgramCounter {
        instruction_address: _instruction_address,
        module_index,
    } = thread.pc;

    let internal_function_index = thread
        .stack
        .get_function_frame()
        .frame
        .internal_function_index;

    let variable_item = &thread.context.modules[module_index]
        .local_variable_section
        .get_variable_list(internal_function_index)[local_variable_index];

    let total_offset = local_start_address + variable_item.var_offset as usize + offset_bytes;
    let ptr = thread.stack.get_ptr(total_offset);
    let address = ptr as u64;

    thread.stack.push_u64(address);

    InterpretResult::MoveOn(8)
}

pub fn host_addr_data(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 data_index:i32)
    let (offset_bytes, data_index) = thread.get_param_i16_i32();
    do_host_addr_data(thread, data_index as usize, offset_bytes as usize)
}

pub fn host_addr_data_long(thread: &mut Thread) -> InterpretResult {
    // (param data_index:i32) (operand offset_bytes:i32)
    let data_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_u32();
    do_host_addr_data(thread, data_index as usize, offset_bytes as usize)
}

fn do_host_addr_data(
    thread: &mut Thread,
    data_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    // get the target data item
    let ProgramCounter {
        instruction_address: _instruction_address,
        module_index,
    } = thread.pc;

    let range = &thread.context.data_index_section.ranges[module_index];
    let data_index_item =
        &thread.context.data_index_section.items[range.offset as usize + data_index];
    let target_module = &mut thread.context.modules[data_index_item.target_module_index as usize];
    let datas = target_module.datas[data_index_item.target_data_section_type as usize].as_mut();
    let internal_data_index = data_index_item.target_data_internal_index;

    let total_offset = datas.get_idx_address(internal_data_index as usize, offset_bytes);
    let ptr = datas.get_ptr(total_offset);
    let address = ptr as u64;

    thread.stack.push_u64(address);

    InterpretResult::MoveOn(8)
}

pub fn host_addr_heap(_thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16)
    // InterpretResult::MoveOn(4);
    unimplemented!()
}
