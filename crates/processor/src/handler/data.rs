// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;

use super::{HandleResult, Handler};

const DATA_LENGTH_IN_BYTES_64_BIT: usize = 8;
const DATA_LENGTH_IN_BYTES_32_BIT: usize = 4;
const DATA_LENGTH_IN_BYTES_16_BIT: usize = 2;
const DATA_LENGTH_IN_BYTES_8_BIT: usize = 1;

pub fn data_load_i64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32) -> i64
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_i64(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_extend_i64(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i32) -> i64
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i64_u();
    do_data_load_i64(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_dynamic_i64(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param) (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> i64
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_public_index = thread_context.stack.pop_i32_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_i64(
        thread_context,
        module_index as usize,
        data_public_index as usize,
        offset_bytes as usize,
        4,
    )
}

fn do_data_load_i64(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_public_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_target_data_object(
            module_index,
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_64_BIT,
        );
    data_object.load_idx_i64(data_internal_index, offset_bytes, dst_ptr);

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_load_i32_s(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_i32_s(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_extend_i32_s(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i64_u();
    do_data_load_i32_s(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_dynamic_i32_s(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param) (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> i64
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_public_index = thread_context.stack.pop_i32_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_i32_s(
        thread_context,
        module_index as usize,
        data_public_index as usize,
        offset_bytes as usize,
        4,
    )
}

fn do_data_load_i32_s(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_public_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_target_data_object(
            module_index,
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_32_BIT,
        );
    data_object.load_idx_i32_s(data_internal_index, offset_bytes, dst_ptr);

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_load_i32_u(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_i32_u(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_extend_i32_u(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i64_u();
    do_data_load_i32_u(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_dynamic_i32_u(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param) (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> i64
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_public_index = thread_context.stack.pop_i32_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_i32_u(
        thread_context,
        module_index as usize,
        data_public_index as usize,
        offset_bytes as usize,
        4,
    )
}

fn do_data_load_i32_u(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_public_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_target_data_object(
            module_index,
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_32_BIT,
        );
    data_object.load_idx_i32_u(data_internal_index, offset_bytes, dst_ptr);

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_load_i16_s(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_i16_s(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_extend_i16_s(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i64_u();
    do_data_load_i16_s(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_dynamic_i16_s(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param) (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> i64
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_public_index = thread_context.stack.pop_i32_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_i16_s(
        thread_context,
        module_index as usize,
        data_public_index as usize,
        offset_bytes as usize,
        4,
    )
}

fn do_data_load_i16_s(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_public_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_target_data_object(
            module_index,
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_16_BIT,
        );
    data_object.load_idx_i16_s(data_internal_index, offset_bytes, dst_ptr);

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_load_i16_u(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_i16_u(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_extend_i16_u(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i64_u();
    do_data_load_i16_u(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_dynamic_i16_u(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param) (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> i64
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_public_index = thread_context.stack.pop_i32_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_i16_u(
        thread_context,
        module_index as usize,
        data_public_index as usize,
        offset_bytes as usize,
        4,
    )
}

fn do_data_load_i16_u(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_public_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_target_data_object(
            module_index,
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_16_BIT,
        );
    data_object.load_idx_i16_u(data_internal_index, offset_bytes, dst_ptr);

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_load_i8_s(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_i8_s(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_extend_i8_s(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i64_u();
    do_data_load_i8_s(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_dynamic_i8_s(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param) (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> i64
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_public_index = thread_context.stack.pop_i32_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_i8_s(
        thread_context,
        module_index as usize,
        data_public_index as usize,
        offset_bytes as usize,
        4,
    )
}

fn do_data_load_i8_s(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_public_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_target_data_object(
            module_index,
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_8_BIT,
        );
    data_object.load_idx_i8_s(data_internal_index, offset_bytes, dst_ptr);

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_load_i8_u(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_i8_u(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_extend_i8_u(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i64_u();
    do_data_load_i8_u(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_dynamic_i8_u(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param) (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> i64
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_public_index = thread_context.stack.pop_i32_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_i8_u(
        thread_context,
        module_index as usize,
        data_public_index as usize,
        offset_bytes as usize,
        4,
    )
}

fn do_data_load_i8_u(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_public_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_target_data_object(
            module_index,
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_8_BIT,
        );
    data_object.load_idx_i8_u(data_internal_index, offset_bytes, dst_ptr);

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_load_f32(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_f32(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_extend_f32(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i64_u();
    do_data_load_f32(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_dynamic_f32(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param) (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> i64
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_public_index = thread_context.stack.pop_i32_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_f32(
        thread_context,
        module_index as usize,
        data_public_index as usize,
        offset_bytes as usize,
        4,
    )
}

fn do_data_load_f32(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_public_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_target_data_object(
            module_index,
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_32_BIT,
        );
    data_object.load_idx_f32(data_internal_index, offset_bytes, dst_ptr);

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_load_f64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_f64(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_extend_f64(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i32)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i64_u();
    do_data_load_f64(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_dynamic_f64(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param) (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> i64
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_public_index = thread_context.stack.pop_i32_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_f64(
        thread_context,
        module_index as usize,
        data_public_index as usize,
        offset_bytes as usize,
        4,
    )
}

fn do_data_load_f64(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_public_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.prepare_pushing_operand_from_memory();
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_target_data_object(
            module_index,
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_64_BIT,
        );
    data_object.load_idx_f64(data_internal_index, offset_bytes, dst_ptr);

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_store_i64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32) (operand value:i64) -> ()
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    do_data_store_i64(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        8,
    )
}

pub fn data_store_extend_i64(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i32 value:i64)
    let data_public_index = thread_context.get_param_i32();
    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i64_u();
    do_data_store_i64(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        8,
    )
}

pub fn data_store_dynamic_i64(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param) (operand module_index:i32 data_public_index:i32 offset_bytes:i64 value:i64) -> (remain_values)
    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_public_index = thread_context.stack.pop_i32_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_store_i64(
        thread_context,
        module_index as usize,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        4,
    )
}

fn do_data_store_i64(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_public_index: usize,
    offset_bytes: usize,
    src_ptr: *const u8,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_target_data_object(
            module_index,
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_64_BIT,
        );
    data_object.store_idx_i64(src_ptr, data_internal_index, offset_bytes);

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_store_i32(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    do_data_store_i32(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        8,
    )
}

pub fn data_store_extend_i32(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i32 value:i32)
    let data_public_index = thread_context.get_param_i32();
    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i64_u();
    do_data_store_i32(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        8,
    )
}

pub fn data_store_dynamic_i32(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param) (operand module_index:i32 data_public_index:i32 offset_bytes:i64 value:i64) -> (remain_values)
    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_public_index = thread_context.stack.pop_i32_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_store_i32(
        thread_context,
        module_index as usize,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        4,
    )
}

fn do_data_store_i32(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_public_index: usize,
    offset_bytes: usize,
    src_ptr: *const u8,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_target_data_object(
            module_index,
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_32_BIT,
        );
    data_object.store_idx_i32(src_ptr, data_internal_index, offset_bytes);

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_store_i16(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    do_data_store_i16(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        8,
    )
}

pub fn data_store_extend_i16(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i32 value:i32)
    let data_public_index = thread_context.get_param_i32();
    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i64_u();
    do_data_store_i16(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        8,
    )
}

pub fn data_store_dynamic_i16(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param) (operand module_index:i32 data_public_index:i32 offset_bytes:i64 value:i64) -> (remain_values)
    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_public_index = thread_context.stack.pop_i32_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_store_i16(
        thread_context,
        module_index as usize,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        4,
    )
}

fn do_data_store_i16(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_public_index: usize,
    offset_bytes: usize,
    src_ptr: *const u8,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_target_data_object(
            module_index,
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_16_BIT,
        );
    data_object.store_idx_i16(src_ptr, data_internal_index, offset_bytes);

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_store_i8(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    do_data_store_i8(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        8,
    )
}

pub fn data_store_extend_i8(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i32 value:i32)
    let data_public_index = thread_context.get_param_i32();
    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i64_u();
    do_data_store_i8(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        8,
    )
}

pub fn data_store_dynamic_i8(
    _handler: &Handler,
    thread_context: &mut ThreadContext,
) -> HandleResult {
    // (param) (operand module_index:i32 data_public_index:i32 offset_bytes:i64 value:i64) -> (remain_values)
    let src_ptr = thread_context.stack.prepare_popping_operand_to_memory();
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_public_index = thread_context.stack.pop_i32_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_store_i8(
        thread_context,
        module_index as usize,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        4,
    )
}

fn do_data_store_i8(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_public_index: usize,
    offset_bytes: usize,
    src_ptr: *const u8,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let (_target_module_index, data_internal_index, data_object) = thread_context
        .get_target_data_object(
            module_index,
            data_public_index,
            offset_bytes,
            DATA_LENGTH_IN_BYTES_8_BIT,
        );
    data_object.store_idx_i8(src_ptr, data_internal_index, offset_bytes);

    HandleResult::Move(instruction_length_in_bytes)
}

#[cfg(test)]
mod tests {
    use anc_context::process_resource::ProgramSource;
    use anc_image::{
        bytecode_writer::BytecodeWriterHelper,
        entry::{InitedDataEntry, UninitDataEntry},
        utils::helper_build_module_binary_with_single_function_and_data,
    };
    use anc_isa::{opcode::Opcode, ForeignValue, OperandDataType};

    use crate::{
        handler::Handler, in_memory_program_source::InMemoryProgramSource,
        process::process_function,
    };

    #[test]
    fn test_handler_data_load_and_store_initialized() {
        //        read-only data section
        //        ======================
        //
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
        //        =======================
        //
        //       |low address                                                              high address|
        //       |                                                                                     |
        // index |2                                  3      4      5                         6         |
        //  type |bytes-------------------|         |f32|  |f64|  |i64------------------|   |i32-------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 13 17 19
        //       |           |        |  |          |      |      ^                          ^
        //       |store32    |store16 |  |          |      |      |                          |
        //                            |  |          |      |      |                          |
        //                      store8|  |          |      |      |                          |
        //       |                       |store8    |      |      |store64                   |store32
        //       |                                  |      |      |                          |
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
        // () -> (i64,i32,i32,i32,i32,i32,i32,  f32,f64,  i64,i32,i32)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::data_load_i8_u, 3, 1)
            .append_opcode_i16_i32(Opcode::data_load_i8_u, 2, 1)
            .append_opcode_i16_i32(Opcode::data_load_i16_u, 0, 1)
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 0, 0)
            //
            .append_opcode_i16_i32(Opcode::data_store_i32, 0, 2)
            .append_opcode_i16_i32(Opcode::data_store_i16, 4, 2)
            .append_opcode_i16_i32(Opcode::data_store_i8, 6, 2)
            .append_opcode_i16_i32(Opcode::data_store_i8, 7, 2)
            //
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 5)
            //
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::data_store_i32, 0, 6)
            //
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 4, 2)
            .append_opcode_i16_i32(Opcode::data_load_i32_s, 4, 2)
            .append_opcode_i16_i32(Opcode::data_load_i16_u, 6, 2)
            .append_opcode_i16_i32(Opcode::data_load_i16_s, 6, 2)
            .append_opcode_i16_i32(Opcode::data_load_i8_u, 7, 2)
            .append_opcode_i16_i32(Opcode::data_load_i8_s, 7, 2)
            //
            .append_opcode_i16_i32(Opcode::data_load_f32, 0, 3)
            .append_opcode_i16_i32(Opcode::data_load_f64, 0, 4)
            //
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 5)
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 0, 6)
            .append_opcode_i16_i32(Opcode::data_load_i32_s, 0, 6)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
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
            &[], // local variables
            code0,
            &[
                InitedDataEntry::from_i32(0x19171311),
                InitedDataEntry::from_i32(0xf0e0d0c0),
            ],
            &[
                InitedDataEntry::from_bytes(vec![0u8, 11, 22, 33, 44, 55, 66, 77], 8), // random init data
                InitedDataEntry::from_f32(std::f32::consts::PI),
                InitedDataEntry::from_f64(std::f64::consts::E),
                InitedDataEntry::from_i64(0),
                InitedDataEntry::from_i32(0),
            ],
            &[],
        );

        let handler = Handler::new();
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::U64(0xf0e0d0c0_19171311_u64),
                ForeignValue::U32(0xf0e0d0c0_u32), // i32_u
                ForeignValue::U32(0xF0E0D0C0_u32), // i32_s
                ForeignValue::U32(0xf0e0_u32),
                ForeignValue::U32(0xfffff0e0_u32), // extend from i16 to i32
                ForeignValue::U32(0xf0_u32),
                ForeignValue::U32(0xfffffff0_u32), // extend from i8 to i32
                // group 1
                ForeignValue::F32(std::f32::consts::PI),
                ForeignValue::F64(std::f64::consts::E),
                // group 2
                ForeignValue::U64(0xf0e0d0c0_19171311_u64),
                ForeignValue::U32(0x19171311_u32), // i32_u
                ForeignValue::U32(0x19171311_u32), // i32_s
            ]
        );
    }

    #[test]
    fn test_handler_data_load_and_store_uninitialized() {
        //        read-only data section
        //        ======================
        //
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
        //        ==========================
        //
        //       |low address                                                              high address|
        //       |                                                                                     |
        // index |2                                  3      4      5                         6         |
        //  type |bytes-------------------|         |f32|  |f64|  |i64------------------|   |i32-------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 13 17 19
        //       |           |        |  |          |      |      ^                          ^
        //       |store32    |store16 |  |          |sf32  |sf64  |                          |
        //                            |  |          |stepN'|stepN |                          |
        //                      store8|  |          |      |      |                          |
        //       |                       |store8    |      |      |store64                   |store32
        //       |                                  |      |      |                          |
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
            .append_opcode_i16_i32(Opcode::data_load_i8_u, 3, 1)
            .append_opcode_i16_i32(Opcode::data_load_i8_u, 2, 1)
            .append_opcode_i16_i32(Opcode::data_load_i16_u, 0, 1)
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 0, 0)
            //
            .append_opcode_i16_i32(Opcode::data_store_i32, 0, 2)
            .append_opcode_i16_i32(Opcode::data_store_i16, 4, 2)
            .append_opcode_i16_i32(Opcode::data_store_i8, 6, 2)
            .append_opcode_i16_i32(Opcode::data_store_i8, 7, 2)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .append_opcode_i16_i32(Opcode::data_store_f64, 0, 4) // store f64
            .append_opcode_i16_i16_i16(Opcode::local_load_f32, 0, 0, 0)
            .append_opcode_i16_i32(Opcode::data_store_f32, 0, 3) // store f32
            //
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 5)
            //
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::data_store_i32, 0, 6)
            //
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 2)
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 4, 2)
            .append_opcode_i16_i32(Opcode::data_load_i32_s, 4, 2)
            .append_opcode_i16_i32(Opcode::data_load_i16_u, 6, 2)
            .append_opcode_i16_i32(Opcode::data_load_i16_s, 6, 2)
            .append_opcode_i16_i32(Opcode::data_load_i8_u, 7, 2)
            .append_opcode_i16_i32(Opcode::data_load_i8_s, 7, 2)
            //
            .append_opcode_i16_i32(Opcode::data_load_f32, 0, 3)
            .append_opcode_i16_i32(Opcode::data_load_f64, 0, 4)
            //
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 5)
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 0, 6)
            .append_opcode_i16_i32(Opcode::data_load_i32_s, 0, 6)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
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
            &[
                InitedDataEntry::from_i32(0x19171311),
                InitedDataEntry::from_i32(0xf0e0d0c0),
            ],
            &[],
            &[
                UninitDataEntry::from_bytes(8, 8),
                UninitDataEntry::from_f32(),
                UninitDataEntry::from_f64(),
                UninitDataEntry::from_i64(),
                UninitDataEntry::from_i32(),
            ],
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
    fn test_handler_data_load_and_store_extend() {
        //        uninitialized data section
        //        ==========================
        //
        //       |low address                                 high address|
        //       |                                                        |
        // index |0                                  1                    |
        //  type |bytes-------------------|         |bytes----------------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0         11 13 17 19 c0 d0 e0 f0
        //       |imm        |imm     |  |          ^
        //       |store32    |store16 |  |          |
        //        step0       step1   |  |          |
        //                         imm|  |imm       |
        //       |              store8|  |store8    |store64
        //       |               step2    step3     |
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
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i32(Opcode::imm_i32, 0x19171311)
            .append_opcode_i32(Opcode::data_store_extend_i32, 0) // store 32
            //
            .append_opcode_i32(Opcode::imm_i32, 4)
            .append_opcode_i32(Opcode::imm_i32, 0xd0c0)
            .append_opcode_i32(Opcode::data_store_extend_i16, 0) // store 16
            //
            .append_opcode_i32(Opcode::imm_i32, 6)
            .append_opcode_i32(Opcode::imm_i32, 0xe0)
            .append_opcode_i32(Opcode::data_store_extend_i8, 0) // store 8
            //
            .append_opcode_i32(Opcode::imm_i32, 7)
            .append_opcode_i32(Opcode::imm_i32, 0xf0)
            .append_opcode_i32(Opcode::data_store_extend_i8, 0) // store 8
            //
            .append_opcode_i32(Opcode::imm_i32, 0) // offset for store
            .append_opcode_i32(Opcode::imm_i32, 0) // offset for load
            .append_opcode_i32(Opcode::data_load_extend_i64, 0) // load 64
            .append_opcode_i32(Opcode::data_store_extend_i64, 1) // store 64
            //
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i32(Opcode::data_load_extend_i64, 0)
            .append_opcode_i32(Opcode::imm_i32, 4)
            .append_opcode_i32(Opcode::data_load_extend_i32_u, 0)
            .append_opcode_i32(Opcode::imm_i32, 4)
            .append_opcode_i32(Opcode::data_load_extend_i32_s, 0)
            .append_opcode_i32(Opcode::imm_i32, 6)
            .append_opcode_i32(Opcode::data_load_extend_i16_u, 0)
            .append_opcode_i32(Opcode::imm_i32, 6)
            .append_opcode_i32(Opcode::data_load_extend_i16_s, 0)
            .append_opcode_i32(Opcode::imm_i32, 7)
            .append_opcode_i32(Opcode::data_load_extend_i8_u, 0)
            .append_opcode_i32(Opcode::imm_i32, 7)
            .append_opcode_i32(Opcode::data_load_extend_i8_s, 0)
            //
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i32(Opcode::data_load_extend_i64, 1)
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i32(Opcode::data_load_extend_i32_u, 1)
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i32(Opcode::data_load_extend_i16_u, 1)
            .append_opcode_i32(Opcode::imm_i32, 0)
            .append_opcode_i32(Opcode::data_load_extend_i8_u, 1)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[
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
            &[], // local variables
            code0,
            &[],
            &[],
            &[
                UninitDataEntry::from_bytes(8, 8),
                UninitDataEntry::from_bytes(8, 8),
            ],
        );

        let handler = Handler::new();
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
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
    fn test_handler_data_bounds_check_offset_out_of_range() {
        let code0 = BytecodeWriterHelper::new()
            // (param offset_bytes:i16 data_public_index:i32) -> i64
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 2, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[], // results
            &[], // local variables
            code0,
            &[],
            &[InitedDataEntry::from_i32(11)],
            &[],
        );

        // capture the panic and keep silent
        // it is also possible to check the panic by
        // adding `#[should_panic]` attribute to the function.
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let handler = Handler::new();
            let resource0 = InMemoryProgramSource::new(vec![binary0]);
            let process_context0 = resource0.create_process_context().unwrap();
            let mut thread_context0 = process_context0.create_thread_context();
            // err: offset(+length) is out of data area
            let _ = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_handler_data_bounds_check_type_length_overflow() {
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[], // results
            &[], // local variables
            code0,
            &[],
            &[InitedDataEntry::from_i32(11)],
            &[],
        );

        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let handler = Handler::new();
            let resource0 = InMemoryProgramSource::new(vec![binary0]);
            let process_context0 = resource0.create_process_context().unwrap();
            let mut thread_context0 = process_context0.create_thread_context();
            // err: load i32 variable with load_i64 instruction
            let _ = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_handler_data_bounds_check_index_out_of_range() {
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 11)
            // (param offset_bytes:i16 data_public_index:i32) (operand value:i32) -> ()
            .append_opcode_i16_i32(Opcode::data_store_i32, 2, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[], // results
            &[], // local variables
            code0,
            &[],
            &[InitedDataEntry::from_i32(11)],
            &[],
        );

        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let handler = Handler::new();
            let resource0 = InMemoryProgramSource::new(vec![binary0]);
            let process_context0 = resource0.create_process_context().unwrap();
            let mut thread_context0 = process_context0.create_thread_context();
            // err: access non-exist index local variable
            let _ = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_handler_data_bounds_check_extend_offset_out_of_range() {
        let code0 = BytecodeWriterHelper::new()
            // offset for load
            .append_opcode_i32(Opcode::imm_i32, 2)
            // (param data_public_index:i32) (operand offset_bytes:i32) -> i64
            .append_opcode_i32(Opcode::data_load_extend_i32_u, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[], // results
            &[], // local variables
            code0,
            &[],
            &[InitedDataEntry::from_i32(11)],
            &[],
        );

        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let handler = Handler::new();
            let resource0 = InMemoryProgramSource::new(vec![binary0]);
            let process_context0 = resource0.create_process_context().unwrap();
            let mut thread_context0 = process_context0.create_thread_context();
            // err: offset(+length) is out of data area
            let _ = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }
}
