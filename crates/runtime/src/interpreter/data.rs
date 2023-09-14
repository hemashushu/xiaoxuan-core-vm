// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::{
    indexed_memory::IndexedMemory,
    thread::{ProgramCounter, Thread},
};

use super::InterpretResult;

pub fn data_load(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 data_index:i32)
    let (offset_bytes, data_index) = thread.get_param_i16_i32();
    do_data_load(thread, data_index as usize, offset_bytes as usize)
}

pub fn data_long_load(thread: &mut Thread) -> InterpretResult {
    // (param data_index:i32) (operand offset_bytes:i32)
    let data_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_data_load(thread, data_index as usize, offset_bytes as usize)
}

fn do_data_load(thread: &mut Thread, data_index: usize, offset_bytes: usize) -> InterpretResult {
    let dst_ptr = thread.stack.push_from_memory();
    let (datas, internal_data_idx) = get_internal_datas_and_index(thread, data_index);
    datas.load_idx_64(internal_data_idx, offset_bytes, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn data_load32(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 data_index:i32)
    let (offset_bytes, data_index) = thread.get_param_i16_i32();
    do_data_load32(thread, data_index as usize, offset_bytes as usize)
}

pub fn data_long_load32(thread: &mut Thread) -> InterpretResult {
    // (param data_index:i32) (operand offset_bytes:i32)
    let data_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_data_load32(thread, data_index as usize, offset_bytes as usize)
}

fn do_data_load32(thread: &mut Thread, data_index: usize, offset_bytes: usize) -> InterpretResult {
    let dst_ptr = thread.stack.push_from_memory();
    let (datas, internal_data_idx) = get_internal_datas_and_index(thread, data_index);
    datas.load_idx_32(internal_data_idx, offset_bytes, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn data_load32_i16_s(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 data_index:i32)
    let (offset_bytes, data_index) = thread.get_param_i16_i32();
    do_data_load32_i16_s(thread, data_index as usize, offset_bytes as usize)
}

pub fn data_long_load32_i16_s(thread: &mut Thread) -> InterpretResult {
    // (param data_index:i32) (operand offset_bytes:i32)
    let data_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_data_load32_i16_s(thread, data_index as usize, offset_bytes as usize)
}

fn do_data_load32_i16_s(
    thread: &mut Thread,
    data_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread.stack.push_from_memory();
    let (datas, internal_data_idx) = get_internal_datas_and_index(thread, data_index);
    datas.load_idx_32_extend_from_i16_s(internal_data_idx, offset_bytes, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn data_load32_i16_u(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 data_index:i32)
    let (offset_bytes, data_index) = thread.get_param_i16_i32();
    do_data_load32_i16_u(thread, data_index as usize, offset_bytes as usize)
}

pub fn data_long_load32_i16_u(thread: &mut Thread) -> InterpretResult {
    // (param data_index:i32) (operand offset_bytes:i32)
    let data_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_data_load32_i16_u(thread, data_index as usize, offset_bytes as usize)
}

fn do_data_load32_i16_u(
    thread: &mut Thread,
    data_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread.stack.push_from_memory();
    let (datas, internal_data_idx) = get_internal_datas_and_index(thread, data_index);
    datas.load_idx_32_extend_from_i16_u(internal_data_idx, offset_bytes, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn data_load32_i8_s(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 data_index:i32)
    let (offset_bytes, data_index) = thread.get_param_i16_i32();
    do_data_load32_i8_s(thread, data_index as usize, offset_bytes as usize)
}

pub fn data_long_load32_i8_s(thread: &mut Thread) -> InterpretResult {
    // (param data_index:i32) (operand offset_bytes:i32)
    let data_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_data_load32_i8_s(thread, data_index as usize, offset_bytes as usize)
}

fn do_data_load32_i8_s(
    thread: &mut Thread,
    data_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread.stack.push_from_memory();
    let (datas, internal_data_idx) = get_internal_datas_and_index(thread, data_index);
    datas.load_idx_32_extend_from_i8_s(internal_data_idx, offset_bytes, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn data_load32_i8_u(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 data_index:i32)
    let (offset_bytes, data_index) = thread.get_param_i16_i32();
    do_data_load32_i8_u(thread, data_index as usize, offset_bytes as usize)
}

pub fn data_long_load32_i8_u(thread: &mut Thread) -> InterpretResult {
    // (param data_index:i32) (operand offset_bytes:i32)
    let data_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_data_load32_i8_u(thread, data_index as usize, offset_bytes as usize)
}

fn do_data_load32_i8_u(
    thread: &mut Thread,
    data_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread.stack.push_from_memory();
    let (datas, internal_data_idx) = get_internal_datas_and_index(thread, data_index);
    datas.load_idx_32_extend_from_i8_u(internal_data_idx, offset_bytes, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn data_load32_f32(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 data_index:i32)
    let (offset_bytes, data_index) = thread.get_param_i16_i32();
    do_data_load32_f32(thread, data_index as usize, offset_bytes as usize)
}

pub fn data_long_load32_f32(thread: &mut Thread) -> InterpretResult {
    // (param data_index:i32) (operand offset_bytes:i32)
    let data_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_data_load32_f32(thread, data_index as usize, offset_bytes as usize)
}

fn do_data_load32_f32(
    thread: &mut Thread,
    data_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread.stack.push_from_memory();
    let (datas, internal_data_idx) = get_internal_datas_and_index(thread, data_index);
    datas.load_idx_32_with_float_check(internal_data_idx, offset_bytes, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn data_load_f64(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 data_index:i32)
    let (offset_bytes, data_index) = thread.get_param_i16_i32();
    do_data_load_f64(thread, data_index as usize, offset_bytes as usize)
}

pub fn data_long_load_f64(thread: &mut Thread) -> InterpretResult {
    // (param data_index:i32) (operand offset_bytes:i32)
    let data_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_data_load_f64(thread, data_index as usize, offset_bytes as usize)
}

fn do_data_load_f64(
    thread: &mut Thread,
    data_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let dst_ptr = thread.stack.push_from_memory();
    let (datas, internal_data_idx) = get_internal_datas_and_index(thread, data_index);
    datas.load_idx_64_with_float_check(internal_data_idx, offset_bytes, dst_ptr);

    InterpretResult::MoveOn(8)
}

pub fn data_store(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 data_index:i32)
    let (offset_bytes, data_index) = thread.get_param_i16_i32();
    do_data_store(thread, data_index as usize, offset_bytes as usize)
}

pub fn data_long_store(thread: &mut Thread) -> InterpretResult {
    // (param data_index:i32) (operand offset_bytes:i32)
    let data_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_data_store(thread, data_index as usize, offset_bytes as usize)
}

fn do_data_store(thread: &mut Thread, data_index: usize, offset_bytes: usize) -> InterpretResult {
    let src_ptr = thread.stack.pop_to_memory();
    let (datas, internal_data_idx) = get_internal_datas_and_index(thread, data_index);
    datas.store_idx_64(src_ptr, internal_data_idx, offset_bytes);

    InterpretResult::MoveOn(8)
}

pub fn data_store32(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 data_index:i32)
    let (offset_bytes, data_index) = thread.get_param_i16_i32();
    do_data_store32(thread, data_index as usize, offset_bytes as usize)
}

pub fn data_long_store32(thread: &mut Thread) -> InterpretResult {
    // (param data_index:i32) (operand offset_bytes:i32)
    let data_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_data_store32(thread, data_index as usize, offset_bytes as usize)
}

fn do_data_store32(thread: &mut Thread, data_index: usize, offset_bytes: usize) -> InterpretResult {
    let src_ptr = thread.stack.pop_to_memory();
    let (datas, internal_data_idx) = get_internal_datas_and_index(thread, data_index);
    datas.store_idx_32(src_ptr, internal_data_idx, offset_bytes);

    InterpretResult::MoveOn(8)
}

pub fn data_store16(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 data_index:i32)
    let (offset_bytes, data_index) = thread.get_param_i16_i32();
    do_data_store16(thread, data_index as usize, offset_bytes as usize)
}

pub fn data_long_store16(thread: &mut Thread) -> InterpretResult {
    // (param data_index:i32) (operand offset_bytes:i32)
    let data_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_data_store16(thread, data_index as usize, offset_bytes as usize)
}

fn do_data_store16(thread: &mut Thread, data_index: usize, offset_bytes: usize) -> InterpretResult {
    let src_ptr = thread.stack.pop_to_memory();
    let (datas, internal_data_idx) = get_internal_datas_and_index(thread, data_index);
    datas.store_idx_16(src_ptr, internal_data_idx, offset_bytes);

    InterpretResult::MoveOn(8)
}

pub fn data_store8(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 data_index:i32)
    let (offset_bytes, data_index) = thread.get_param_i16_i32();
    do_data_store8(thread, data_index as usize, offset_bytes as usize)
}

pub fn data_long_store8(thread: &mut Thread) -> InterpretResult {
    // (param data_index:i32) (operand offset_bytes:i32)
    let data_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_data_store8(thread, data_index as usize, offset_bytes as usize)
}

fn do_data_store8(thread: &mut Thread, data_index: usize, offset_bytes: usize) -> InterpretResult {
    let src_ptr = thread.stack.pop_to_memory();
    let (datas, internal_data_idx) = get_internal_datas_and_index(thread, data_index);
    datas.store_idx_8(src_ptr, internal_data_idx, offset_bytes);

    InterpretResult::MoveOn(8)
}

fn get_internal_datas_and_index<'a>(
    thread: &'a mut Thread,
    data_index: usize,
) -> (&'a mut dyn IndexedMemory, usize) {
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

    (datas, internal_data_index as usize)
}
