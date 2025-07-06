// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;

use crate::TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS;

use super::HandleResult;

const DATA_LENGTH_IN_BYTES_64_BIT: usize = 8;
const DATA_LENGTH_IN_BYTES_32_BIT: usize = 4;
const DATA_LENGTH_IN_BYTES_16_BIT: usize = 2;
const DATA_LENGTH_IN_BYTES_8_BIT: usize = 1;

pub fn data_load_i64(thread_context: &mut ThreadContext) -> HandleResult {
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

pub fn data_load_extend_i64(thread_context: &mut ThreadContext) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i64) -> i64
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

pub fn memory_load_i64(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand module_index:i32 data_access_index:i64 offset_bytes:i64) -> i64
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_i64(
        thread_context,
        module_index as usize,
        data_access_index as usize,
        offset_bytes as usize,
        2,
    )
}

fn do_data_load_i64(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_access_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let target_data_object = thread_context.get_target_data_object(
        module_index,
        data_access_index,
        offset_bytes,
        DATA_LENGTH_IN_BYTES_64_BIT,
    );
    target_data_object.accessor.read_idx_i64(
        target_data_object.data_internal_index_in_section,
        offset_bytes,
        dst_ptr as *mut u64,
    );

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_load_i32_s(thread_context: &mut ThreadContext) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i64) -> i32
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_i32_s(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_extend_i32_s(thread_context: &mut ThreadContext) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i64) -> i32
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

pub fn memory_load_i32_s(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand module_index:i32 data_access_index:i64 offset_bytes:i64) -> i32
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_i32_s(
        thread_context,
        module_index as usize,
        data_access_index as usize,
        offset_bytes as usize,
        2,
    )
}

fn do_data_load_i32_s(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_access_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let target_data_object = thread_context.get_target_data_object(
        module_index,
        data_access_index,
        offset_bytes,
        DATA_LENGTH_IN_BYTES_32_BIT,
    );
    target_data_object.accessor.read_idx_i32_s_to_i64(
        target_data_object.data_internal_index_in_section,
        offset_bytes,
        dst_ptr as *mut i64,
    );

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_load_i32_u(thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32) -> i32
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_i32_u(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_extend_i32_u(thread_context: &mut ThreadContext) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i64) -> i32
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

pub fn memory_load_i32_u(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand module_index:i32 data_access_index:i64 offset_bytes:i64) -> i32
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_i32_u(
        thread_context,
        module_index as usize,
        data_access_index as usize,
        offset_bytes as usize,
        2,
    )
}

fn do_data_load_i32_u(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_access_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let target_data_object = thread_context.get_target_data_object(
        module_index,
        data_access_index,
        offset_bytes,
        DATA_LENGTH_IN_BYTES_32_BIT,
    );
    target_data_object.accessor.read_idx_i32_u_to_u64(
        target_data_object.data_internal_index_in_section,
        offset_bytes,
        dst_ptr as *mut u64,
    );

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_load_i16_s(thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32) -> i16
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_i16_s(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_extend_i16_s(thread_context: &mut ThreadContext) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i64) -> i16
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

pub fn memory_dynamic_i16_s(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand module_index:i32 data_access_index:i64 offset_bytes:i64) -> i16
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_i16_s(
        thread_context,
        module_index as usize,
        data_access_index as usize,
        offset_bytes as usize,
        2,
    )
}

fn do_data_load_i16_s(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_access_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let target_data_object = thread_context.get_target_data_object(
        module_index,
        data_access_index,
        offset_bytes,
        DATA_LENGTH_IN_BYTES_16_BIT,
    );
    target_data_object.accessor.read_idx_i16_s_to_i64(
        target_data_object.data_internal_index_in_section,
        offset_bytes,
        dst_ptr as *mut i64,
    );

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_load_i16_u(thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32) -> i16
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_i16_u(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_extend_i16_u(thread_context: &mut ThreadContext) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i64) -> i16
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

pub fn memory_load_i16_u(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand module_index:i32 data_access_index:i64 offset_bytes:i64) -> i16
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_i16_u(
        thread_context,
        module_index as usize,
        data_access_index as usize,
        offset_bytes as usize,
        2,
    )
}

fn do_data_load_i16_u(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_access_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let target_data_object = thread_context.get_target_data_object(
        module_index,
        data_access_index,
        offset_bytes,
        DATA_LENGTH_IN_BYTES_16_BIT,
    );
    target_data_object.accessor.read_idx_i16_u_to_u64(
        target_data_object.data_internal_index_in_section,
        offset_bytes,
        dst_ptr as *mut u64,
    );

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_load_i8_s(thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32) -> i8
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_i8_s(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_extend_i8_s(thread_context: &mut ThreadContext) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i64) -> i8
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

pub fn memory_load_i8_s(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand module_index:i32 data_access_index:i64 offset_bytes:i64) -> i8
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_i8_s(
        thread_context,
        module_index as usize,
        data_access_index as usize,
        offset_bytes as usize,
        2,
    )
}

fn do_data_load_i8_s(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_access_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let target_data_object = thread_context.get_target_data_object(
        module_index,
        data_access_index,
        offset_bytes,
        DATA_LENGTH_IN_BYTES_8_BIT,
    );
    target_data_object.accessor.read_idx_i8_s_to_i64(
        target_data_object.data_internal_index_in_section,
        offset_bytes,
        dst_ptr as *mut i64,
    );

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_load_i8_u(thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32) -> i8
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_i8_u(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_extend_i8_u(thread_context: &mut ThreadContext) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i64) -> i8
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

pub fn memory_load_i8_u(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand module_index:i32 data_access_index:i64 offset_bytes:i64) -> i8
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_i8_u(
        thread_context,
        module_index as usize,
        data_access_index as usize,
        offset_bytes as usize,
        2,
    )
}

fn do_data_load_i8_u(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_access_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let target_data_object = thread_context.get_target_data_object(
        module_index,
        data_access_index,
        offset_bytes,
        DATA_LENGTH_IN_BYTES_8_BIT,
    );
    target_data_object.accessor.read_idx_i8_u_to_u64(
        target_data_object.data_internal_index_in_section,
        offset_bytes,
        dst_ptr as *mut u64,
    );

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_load_f32(thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32) -> f32
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_f32(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_extend_f32(thread_context: &mut ThreadContext) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i64) -> f32
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

pub fn memory_load_f32(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand module_index:i32 data_access_index:i64 offset_bytes:i64) -> f32
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_f32(
        thread_context,
        module_index as usize,
        data_access_index as usize,
        offset_bytes as usize,
        2,
    )
}

fn do_data_load_f32(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_access_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let target_data_object = thread_context.get_target_data_object(
        module_index,
        data_access_index,
        offset_bytes,
        DATA_LENGTH_IN_BYTES_32_BIT,
    );

    match target_data_object.accessor.read_idx_f32(
        target_data_object.data_internal_index_in_section,
        offset_bytes,
        dst_ptr as *mut f32,
    ) {
        Ok(_) => HandleResult::Move(instruction_length_in_bytes),
        Err(_) => HandleResult::Terminate(TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS),
    }
}

pub fn data_load_f64(thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32) -> f64
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    do_data_load_f64(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        8,
    )
}

pub fn data_load_extend_f64(thread_context: &mut ThreadContext) -> HandleResult {
    // (param data_public_index:i32) (operand offset_bytes:i64) -> f64
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

pub fn memory_load_f64(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand module_index:i32 data_access_index:i64 offset_bytes:i64) -> f64
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();
    do_data_load_f64(
        thread_context,
        module_index as usize,
        data_access_index as usize,
        offset_bytes as usize,
        2,
    )
}

fn do_data_load_f64(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_access_index: usize,
    offset_bytes: usize,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let dst_ptr = thread_context.stack.push_operand_from_memory();
    let target_data_object = thread_context.get_target_data_object(
        module_index,
        data_access_index,
        offset_bytes,
        DATA_LENGTH_IN_BYTES_64_BIT,
    );

    match target_data_object.accessor.read_idx_f64(
        target_data_object.data_internal_index_in_section,
        offset_bytes,
        dst_ptr as *mut f64,
    ) {
        Ok(_) => HandleResult::Move(instruction_length_in_bytes),
        Err(_) => HandleResult::Terminate(TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS),
    }
}

pub fn data_store_i64(thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32) (operand value:i64) -> (remain_values)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_data_store_i64(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        8,
    )
}

pub fn data_store_extend_i64(thread_context: &mut ThreadContext) -> HandleResult {
    // (param data_public_index:i32) (operand value:i64 offset_bytes:i64) -> (remain_values)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i64_u();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_data_store_i64(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        8,
    )
}

pub fn memory_store_i64(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand value:i64 module_index:i32 data_access_index:i64 offset_bytes:i64) -> (remain_values)
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_data_store_i64(
        thread_context,
        module_index as usize,
        data_access_index as usize,
        offset_bytes as usize,
        src_ptr,
        2,
    )
}

fn do_data_store_i64(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_access_index: usize,
    offset_bytes: usize,
    src_ptr: *const u8,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let target_data_object = thread_context.get_target_data_object(
        module_index,
        data_access_index,
        offset_bytes,
        DATA_LENGTH_IN_BYTES_64_BIT,
    );
    target_data_object.accessor.write_idx_i64(
        src_ptr,
        target_data_object.data_internal_index_in_section,
        offset_bytes,
    );

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_store_i32(thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32) (operand value:i32) -> (remain_values)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_data_store_i32(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        8,
    )
}

pub fn data_store_extend_i32(thread_context: &mut ThreadContext) -> HandleResult {
    // (param data_public_index:i32) (operand value:i32 offset_bytes:i64) -> (remain_values)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i64_u();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_data_store_i32(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        8,
    )
}

pub fn memory_store_i32(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand value:i32 module_index:i32 data_access_index:i64 offset_bytes:i64) -> (remain_values)
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_data_store_i32(
        thread_context,
        module_index as usize,
        data_access_index as usize,
        offset_bytes as usize,
        src_ptr,
        2,
    )
}

fn do_data_store_i32(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_access_index: usize,
    offset_bytes: usize,
    src_ptr: *const u8,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let target_data_object = thread_context.get_target_data_object(
        module_index,
        data_access_index,
        offset_bytes,
        DATA_LENGTH_IN_BYTES_32_BIT,
    );
    target_data_object.accessor.write_idx_i32(
        src_ptr,
        target_data_object.data_internal_index_in_section,
        offset_bytes,
    );

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_store_i16(thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32) (operand value:i32) -> (remain_values)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_data_store_i16(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        8,
    )
}

pub fn data_store_extend_i16(thread_context: &mut ThreadContext) -> HandleResult {
    // (param data_public_index:i32) (operand value:i32 offset_bytes:i64) -> (remain_values)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i64_u();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_data_store_i16(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        8,
    )
}

pub fn memory_store_i16(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand value:i32 module_index:i32 data_access_index:i64 offset_bytes:i64) -> (remain_values)
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_data_store_i16(
        thread_context,
        module_index as usize,
        data_access_index as usize,
        offset_bytes as usize,
        src_ptr,
        2,
    )
}

fn do_data_store_i16(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_access_index: usize,
    offset_bytes: usize,
    src_ptr: *const u8,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let target_data_object = thread_context.get_target_data_object(
        module_index,
        data_access_index,
        offset_bytes,
        DATA_LENGTH_IN_BYTES_16_BIT,
    );
    target_data_object.accessor.write_idx_i16(
        src_ptr,
        target_data_object.data_internal_index_in_section,
        offset_bytes,
    );

    HandleResult::Move(instruction_length_in_bytes)
}

pub fn data_store_i8(thread_context: &mut ThreadContext) -> HandleResult {
    // (param offset_bytes:i16 data_public_index:i32) (operand value:i32) -> (remain_values)
    let (offset_bytes, data_public_index) = thread_context.get_param_i16_i32();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_data_store_i8(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        8,
    )
}

pub fn data_store_extend_i8(thread_context: &mut ThreadContext) -> HandleResult {
    // (param data_public_index:i32) (operand value:i32 offset_bytes:i64) -> (remain_values)
    let data_public_index = thread_context.get_param_i32();
    let offset_bytes = thread_context.stack.pop_i64_u();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_data_store_i8(
        thread_context,
        thread_context.pc.module_index,
        data_public_index as usize,
        offset_bytes as usize,
        src_ptr,
        8,
    )
}

pub fn memory_store_i8(thread_context: &mut ThreadContext) -> HandleResult {
    // () (operand value:i32 module_index:i32 data_access_index:i64 offset_bytes:i64) -> (remain_values)
    let offset_bytes = thread_context.stack.pop_i64_u();
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();
    let src_ptr = thread_context.stack.pop_operand_to_memory();
    do_data_store_i8(
        thread_context,
        module_index as usize,
        data_access_index as usize,
        offset_bytes as usize,
        src_ptr,
        2,
    )
}

fn do_data_store_i8(
    thread_context: &mut ThreadContext,
    module_index: usize,
    data_access_index: usize,
    offset_bytes: usize,
    src_ptr: *const u8,
    instruction_length_in_bytes: isize,
) -> HandleResult {
    let target_data_object = thread_context.get_target_data_object(
        module_index,
        data_access_index,
        offset_bytes,
        DATA_LENGTH_IN_BYTES_8_BIT,
    );
    target_data_object.accessor.write_idx_i8(
        src_ptr,
        target_data_object.data_internal_index_in_section,
        offset_bytes,
    );

    HandleResult::Move(instruction_length_in_bytes)
}

#[cfg(test)]
mod tests {

    use anc_context::program_source::ProgramSource;
    use anc_image::{
        bytecode_writer::BytecodeWriterHelper,
        entry::{ReadOnlyDataEntry, ReadWriteDataEntry},
        utils::helper_build_module_binary_with_single_function_and_data,
    };
    use anc_isa::{opcode::Opcode, ForeignValue, OperandDataType};

    use crate::{
        in_memory_program_source::InMemoryProgramSource, process::process_function, ProcessorError,
        ProcessorErrorType, TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS,
    };

    #[test]
    fn test_handler_data_load_read_only() {
        //        read-only data section
        //        ======================
        //
        //       |low address                                                             high address|
        //       |                                                                                    |
        // index |0                                  1      2      3                         4     5  |
        //  type |bytes-------------------|         |f32|  |f64|  |i64------------------|   |i32| |i32|
        //
        //  data 00 22 44 66 88 aa    cc dd         f32    f64    11 13 17 19 23 29 31 37   0403  5060
        //                                                                                  0201  7080
        //
        //  read 00 22 44 66 88 aa    cc dd         f32    f64    11 13 17 19 23 29 31 37   i32   i32
        //       |           |        |  |load8u    |      |      |           |             |     |
        //       |load64     |load32u |  |load8s  loadf32  |      |           |             |     |
        //       |           |load32s |                  loadf64  |         load32u       load32s |
        //       |           |        |load16u                    |         load32s               |
        //       |           |        |load16s                 load64                           load32s
        //
        // () -> (i64,i32,i32,i32,i32,i32,i32,  f32,f64,  i64,i32,i32,i32,i32)

        let code0 = BytecodeWriterHelper::new()
            // load group 0
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 0)
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 4, 0)
            .append_opcode_i16_i32(Opcode::data_load_i32_s, 4, 0)
            .append_opcode_i16_i32(Opcode::data_load_i16_u, 6, 0)
            .append_opcode_i16_i32(Opcode::data_load_i16_s, 6, 0)
            .append_opcode_i16_i32(Opcode::data_load_i8_u, 7, 0)
            .append_opcode_i16_i32(Opcode::data_load_i8_s, 7, 0)
            // load group 1
            .append_opcode_i16_i32(Opcode::data_load_f32, 0, 1)
            .append_opcode_i16_i32(Opcode::data_load_f64, 0, 2)
            // load group 2
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 3)
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 4, 3)
            .append_opcode_i16_i32(Opcode::data_load_i32_s, 4, 3)
            .append_opcode_i16_i32(Opcode::data_load_i32_s, 0, 4)
            .append_opcode_i16_i32(Opcode::data_load_i32_s, 0, 5)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[
                // group 0
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                // group 1
                OperandDataType::F32,
                OperandDataType::F64,
                // group 2
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
            ], // results
            &[], // local variables
            code0,
            &[
                ReadOnlyDataEntry::from_bytes(
                    vec![0u8, 0x22, 0x44, 0x66, 0x88, 0xaa, 0xcc, 0xdd],
                    8,
                ),
                ReadOnlyDataEntry::from_f32(std::f32::consts::PI),
                ReadOnlyDataEntry::from_f64(std::f64::consts::E),
                ReadOnlyDataEntry::from_i64(0x37312923_19171311),
                ReadOnlyDataEntry::from_i32(0x01020304),
                ReadOnlyDataEntry::from_i32(0x80706050),
            ],
            &[],
            &[],
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::U64(0xddccaa88_66442200),
                ForeignValue::U32(0xddccaa88),
                ForeignValue::U32(0xddccaa88),
                ForeignValue::U32(0xddcc),
                ForeignValue::U32(0xffffddcc),
                ForeignValue::U32(0xdd),
                ForeignValue::U32(0xffffffdd),
                // group 1
                ForeignValue::F32(std::f32::consts::PI),
                ForeignValue::F64(std::f64::consts::E),
                // group 2
                ForeignValue::U64(0x37312923_19171311),
                ForeignValue::U32(0x37312923),
                ForeignValue::U32(0x37312923),
                ForeignValue::U32(0x01020304),
                ForeignValue::U32(0x80706050),
            ]
        );
    }

    #[test]
    fn test_handler_data_load_and_store_read_write() {
        //        read-write data section
        //        =======================
        //
        //       |low address                                                              high address|
        //       |                                                                                     |
        // index |0                                  1      2      3                         4      5  |
        //  type |bytes-------------------|         |f32|  |f64|  |i64------------------|   |i64|  |i64|
        //
        //  data 00 22 44 66 88 aa    cc dd         f32    f64    11 13 17 19 23 29 31 37   0000   0000
        // write 80 90 a0 b0 c0 d0    e0 f0         ---    ---    80 90 a0 b0 c0 d0 e0 f0   i64    i64
        //       |           |        |  |          |      |      ^                         ^      ^
        //       |store32    |store16 |  |          |      |      |                         |      |
        //       |step0      |step1   |  |          |      |      |                         |      |
        //       |           |  store8|  |          |      |      |                         |      |
        //       |           |   step2   |store8    |      |      |store64                  |st64  |st64
        //       |           |            step3     |      |      |                         |      |
        //       \--------------->--load64--------------------->--/                    ld32u| ld32s|
        //        step4      |                      |      |                           step5| step6|
        //                   \--->------------------------------------------------------->--/--->--/
        //
        //  read 80 90 a0 b0 c0 d0    e0 f0         f32    f64    80 90 a0 b0 c0 d0 e0 f0   i64   i64
        //       |           |        |  |load8u    |      |      |           |              |     |
        //       |load64     |load32u |  |load8s  loadf32  |      |           |              |     |
        //       |           |load32s |                  loadf64  |         load32u        load64  |
        //       |           |        |load16u                    |         load32s              load64
        //       |           |        |load16s                 load64
        //
        // () -> (i64,i32,i32,i32,i32,i32,i32,  f32,f64,  i64,i32,i32,i64,i64)

        let code0 = BytecodeWriterHelper::new()
            // step 0, store i32
            .append_opcode_i32(Opcode::imm_i32, 0xb0a09080)
            .append_opcode_i16_i32(Opcode::data_store_i32, 0, 0)
            // step 1, store i16
            .append_opcode_i32(Opcode::imm_i32, 0xd0c0)
            .append_opcode_i16_i32(Opcode::data_store_i16, 4, 0)
            // step 2, store i8
            .append_opcode_i32(Opcode::imm_i32, 0xe0)
            .append_opcode_i16_i32(Opcode::data_store_i8, 6, 0)
            // step 3, store i8
            .append_opcode_i32(Opcode::imm_i32, 0xf0)
            .append_opcode_i16_i32(Opcode::data_store_i8, 7, 0)
            // step 4, load i64 and store i64
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 0)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 3)
            // step 5, load i32u and store i64
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 4, 0)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 4)
            // step 6, load i32s and store i64
            .append_opcode_i16_i32(Opcode::data_load_i32_s, 4, 0)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 5)
            // load group 0
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 0)
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 4, 0)
            .append_opcode_i16_i32(Opcode::data_load_i32_s, 4, 0)
            .append_opcode_i16_i32(Opcode::data_load_i16_u, 6, 0)
            .append_opcode_i16_i32(Opcode::data_load_i16_s, 6, 0)
            .append_opcode_i16_i32(Opcode::data_load_i8_u, 7, 0)
            .append_opcode_i16_i32(Opcode::data_load_i8_s, 7, 0)
            // load group 1
            .append_opcode_i16_i32(Opcode::data_load_f32, 0, 1)
            .append_opcode_i16_i32(Opcode::data_load_f64, 0, 2)
            // load group 2
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 3)
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 4, 3)
            .append_opcode_i16_i32(Opcode::data_load_i32_s, 4, 3)
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 4)
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 5)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[
                // group 0
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                // group 1
                OperandDataType::F32,
                OperandDataType::F64,
                // group 2
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I64,
                OperandDataType::I64,
            ], // results
            &[], // local variables
            code0,
            &[],
            &[
                ReadWriteDataEntry::from_bytes(
                    vec![0u8, 0x22, 0x44, 0x66, 0x88, 0xaa, 0xcc, 0xdd],
                    8,
                ),
                ReadWriteDataEntry::from_f32(std::f32::consts::PI),
                ReadWriteDataEntry::from_f64(std::f64::consts::E),
                ReadWriteDataEntry::from_i64(0x37312923_19171311),
                ReadWriteDataEntry::from_i64(0),
                ReadWriteDataEntry::from_i64(0),
            ],
            &[],
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::U64(0xf0e0d0c0_b0a09080),
                ForeignValue::U32(0xf0e0d0c0),
                ForeignValue::U32(0xf0e0d0c0),
                ForeignValue::U32(0xf0e0),
                ForeignValue::U32(0xfffff0e0),
                ForeignValue::U32(0xf0),
                ForeignValue::U32(0xfffffff0),
                // group 1
                ForeignValue::F32(std::f32::consts::PI),
                ForeignValue::F64(std::f64::consts::E),
                // group 2
                ForeignValue::U64(0xf0e0d0c0_b0a09080),
                ForeignValue::U32(0xf0e0d0c0),
                ForeignValue::U32(0xf0e0d0c0),
                ForeignValue::U64(0xf0e0d0c0),
                ForeignValue::U64(0xffffffff_f0e0d0c0),
            ]
        );
    }

    #[test]
    fn test_handler_data_load_and_store_uninitialized() {
        //        uninitialized data section
        //        ==========================
        //
        //       |low address                                                              high address|
        //       |                                                                                     |
        // index |0                                  1      2      3                         4      5  |
        //  type |bytes-------------------|         |f32|  |f64|  |i64------------------|   |i64|  |i64|
        //
        // write 80 90 a0 b0 c0 d0    e0 f0         ---    ---    80 90 a0 b0 c0 d0 e0 f0   i64    i64
        //       |           |        |  |          |      |      ^                         ^      ^
        //       |store32    |store16 |  |          |      |      |                         |      |
        //       |step0      |step1   |  |          |      |      |                         |      |
        //       |           |  store8|  |          |      |      |                         |      |
        //       |           |   step2   |store8    |      |      |store64                  |st64  |st64
        //       |           |            step3     |      |      |                         |      |
        //       \--------------->--load64--------------------->--/                    ld32u| ld32s|
        //        step4      |                      |      |                           step5| step6|
        //                   \--->------------------------------------------------------->--/--->--/
        //                                          |      |
        //                                         Pi      E
        //                                     storef32  storef64
        //                                       step7   step8
        //
        //  read 80 90 a0 b0 c0 d0    e0 f0         f32    f64    80 90 a0 b0 c0 d0 e0 f0   i64   i64
        //       |           |        |  |load8u    |      |      |           |              |     |
        //       |load64     |load32u |  |load8s  loadf32  |      |           |              |     |
        //       |           |load32s |                  loadf64  |         load32u        load64  |
        //       |           |        |load16u                    |         load32s              load64
        //       |           |        |load16s                 load64
        //
        // () -> (i64,i32,i32,i32,i32,i32,i32,  f32,f64,  i64,i32,i32,i64,i64)

        let code0 = BytecodeWriterHelper::new()
            // step 0, store i32
            .append_opcode_i32(Opcode::imm_i32, 0xb0a09080)
            .append_opcode_i16_i32(Opcode::data_store_i32, 0, 0)
            // step 1, store i16
            .append_opcode_i32(Opcode::imm_i32, 0xd0c0)
            .append_opcode_i16_i32(Opcode::data_store_i16, 4, 0)
            // step 2, store i8
            .append_opcode_i32(Opcode::imm_i32, 0xe0)
            .append_opcode_i16_i32(Opcode::data_store_i8, 6, 0)
            // step 3, store i8
            .append_opcode_i32(Opcode::imm_i32, 0xf0)
            .append_opcode_i16_i32(Opcode::data_store_i8, 7, 0)
            // step 4, load i64 and store i64
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 0)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 3)
            // step 5, load i32u and store i64
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 4, 0)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 4)
            // step 6, load i32s and store i64
            .append_opcode_i16_i32(Opcode::data_load_i32_s, 4, 0)
            .append_opcode_i16_i32(Opcode::data_store_i64, 0, 5)
            // step 7, store f32
            .append_opcode_f32(Opcode::imm_f32, std::f32::consts::PI)
            .append_opcode_i16_i32(Opcode::data_store_f32, 0, 1)
            .append_opcode_f64(Opcode::imm_f64, std::f64::consts::E)
            .append_opcode_i16_i32(Opcode::data_store_f64, 0, 2)
            // load group 0
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 0)
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 4, 0)
            .append_opcode_i16_i32(Opcode::data_load_i32_s, 4, 0)
            .append_opcode_i16_i32(Opcode::data_load_i16_u, 6, 0)
            .append_opcode_i16_i32(Opcode::data_load_i16_s, 6, 0)
            .append_opcode_i16_i32(Opcode::data_load_i8_u, 7, 0)
            .append_opcode_i16_i32(Opcode::data_load_i8_s, 7, 0)
            // load group 1
            .append_opcode_i16_i32(Opcode::data_load_f32, 0, 1)
            .append_opcode_i16_i32(Opcode::data_load_f64, 0, 2)
            // load group 2
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 3)
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 4, 3)
            .append_opcode_i16_i32(Opcode::data_load_i32_s, 4, 3)
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 4)
            .append_opcode_i16_i32(Opcode::data_load_i64, 0, 5)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[
                // group 0
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                // group 1
                OperandDataType::F32,
                OperandDataType::F64,
                // group 2
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I64,
                OperandDataType::I64,
            ], // results
            &[], // local variables
            code0,
            &[],
            &[
                ReadWriteDataEntry::from_bytes(
                    vec![0u8, 0x22, 0x44, 0x66, 0x88, 0xaa, 0xcc, 0xdd],
                    8,
                ),
                ReadWriteDataEntry::from_f32(std::f32::consts::PI),
                ReadWriteDataEntry::from_f64(std::f64::consts::E),
                ReadWriteDataEntry::from_i64(0x37312923_19171311),
                ReadWriteDataEntry::from_i64(0),
                ReadWriteDataEntry::from_i64(0),
            ],
            &[],
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::U64(0xf0e0d0c0_b0a09080),
                ForeignValue::U32(0xf0e0d0c0),
                ForeignValue::U32(0xf0e0d0c0),
                ForeignValue::U32(0xf0e0),
                ForeignValue::U32(0xfffff0e0),
                ForeignValue::U32(0xf0),
                ForeignValue::U32(0xfffffff0),
                // group 1
                ForeignValue::F32(std::f32::consts::PI),
                ForeignValue::F64(std::f64::consts::E),
                // group 2
                ForeignValue::U64(0xf0e0d0c0_b0a09080),
                ForeignValue::U32(0xf0e0d0c0),
                ForeignValue::U32(0xf0e0d0c0),
                ForeignValue::U64(0xf0e0d0c0),
                ForeignValue::U64(0xffffffff_f0e0d0c0),
            ]
        );
    }

    #[test]
    fn test_handler_data_load_and_store_extend() {
        //        read-write data section
        //        =======================
        //
        //       |low address                                                              high address|
        //       |                                                                                     |
        // index |0                                  1      2      3                         4      5  |
        //  type |bytes-------------------|         |f32|  |f64|  |i64------------------|   |i64|  |i64|
        //
        //  data 00 22 44 66 88 aa    cc dd         f32    f64    11 13 17 19 23 29 31 37   0000   0000
        // write 80 90 a0 b0 c0 d0    e0 f0         ---    ---    80 90 a0 b0 c0 d0 e0 f0   i64    i64
        //       |           |        |  |          |      |      ^                         ^      ^
        //       |store32    |store16 |  |          |      |      |                         |      |
        //       |step0      |step1   |  |          |      |      |                         |      |
        //       |           |  store8|  |          |      |      |                         |      |
        //       |           |   step2   |store8    |      |      |store64                  |st64  |st64
        //       |           |            step3     |      |      |                         |      |
        //       \--------------->--load64--------------------->--/                    ld32u| ld32s|
        //        step4      |                      |      |                           step5| step6|
        //                   \--->------------------------------------------------------->--/--->--/
        //
        //  read 80 90 a0 b0 c0 d0    e0 f0         f32    f64    80 90 a0 b0 c0 d0 e0 f0   i64   i64
        //       |           |        |  |load8u    |      |      |           |              |     |
        //       |load64     |load32u |  |load8s  loadf32  |      |           |              |     |
        //       |           |load32s |                  loadf64  |         load32u        load64  |
        //       |           |        |load16u                    |         load32s              load64
        //       |           |        |load16s                 load64
        //
        // () -> (i64,i32,i32,i32,i32,i32,i32,  f32,f64,  i64,i32,i32,i64,i64)

        let code0 = BytecodeWriterHelper::new()
            // step 0, store i32
            .append_opcode_i32(Opcode::imm_i32, 0xb0a09080)
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i32(Opcode::data_store_extend_i32, 0)
            // step 1, store i16
            .append_opcode_i32(Opcode::imm_i32, 0xd0c0)
            .append_opcode_i64(Opcode::imm_i64, 4) // offset in bytes
            .append_opcode_i32(Opcode::data_store_extend_i16, 0)
            // step 2, store i8
            .append_opcode_i32(Opcode::imm_i32, 0xe0)
            .append_opcode_i64(Opcode::imm_i64, 6) // offset in bytes
            .append_opcode_i32(Opcode::data_store_extend_i8, 0)
            // step 3, store i8
            .append_opcode_i32(Opcode::imm_i32, 0xf0)
            .append_opcode_i64(Opcode::imm_i64, 7) // offset in bytes
            .append_opcode_i32(Opcode::data_store_extend_i8, 0)
            // step 4, load i64 and store i64
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i32(Opcode::data_load_extend_i64, 0)
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i32(Opcode::data_store_extend_i64, 3)
            // step 5, load i32u and store i64
            .append_opcode_i64(Opcode::imm_i64, 4) // offset in bytes
            .append_opcode_i32(Opcode::data_load_extend_i32_u, 0)
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i32(Opcode::data_store_extend_i64, 4)
            // step 6, load i32s and store i64
            .append_opcode_i64(Opcode::imm_i64, 4) // offset in bytes
            .append_opcode_i32(Opcode::data_load_extend_i32_s, 0)
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i32(Opcode::data_store_extend_i64, 5)
            // load group 0
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i32(Opcode::data_load_extend_i64, 0)
            .append_opcode_i64(Opcode::imm_i64, 4) // offset in bytes
            .append_opcode_i32(Opcode::data_load_extend_i32_u, 0)
            .append_opcode_i64(Opcode::imm_i64, 4) // offset in bytes
            .append_opcode_i32(Opcode::data_load_extend_i32_s, 0)
            .append_opcode_i64(Opcode::imm_i64, 6) // offset in bytes
            .append_opcode_i32(Opcode::data_load_extend_i16_u, 0)
            .append_opcode_i64(Opcode::imm_i64, 6) // offset in bytes
            .append_opcode_i32(Opcode::data_load_extend_i16_s, 0)
            .append_opcode_i64(Opcode::imm_i64, 7) // offset in bytes
            .append_opcode_i32(Opcode::data_load_extend_i8_u, 0)
            .append_opcode_i64(Opcode::imm_i64, 7) // offset in bytes
            .append_opcode_i32(Opcode::data_load_extend_i8_s, 0)
            // load group 1
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i32(Opcode::data_load_extend_f32, 1)
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i32(Opcode::data_load_extend_f64, 2)
            // load group 2
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i32(Opcode::data_load_extend_i64, 3)
            .append_opcode_i64(Opcode::imm_i64, 4) // offset in bytes
            .append_opcode_i32(Opcode::data_load_extend_i32_u, 3)
            .append_opcode_i64(Opcode::imm_i64, 4) // offset in bytes
            .append_opcode_i32(Opcode::data_load_extend_i32_s, 3)
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i32(Opcode::data_load_extend_i64, 4)
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode_i32(Opcode::data_load_extend_i64, 5)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[
                // group 0
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                // group 1
                OperandDataType::F32,
                OperandDataType::F64,
                // group 2
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I64,
                OperandDataType::I64,
            ], // results
            &[], // local variables
            code0,
            &[],
            &[
                ReadWriteDataEntry::from_bytes(
                    vec![0u8, 0x22, 0x44, 0x66, 0x88, 0xaa, 0xcc, 0xdd],
                    8,
                ),
                ReadWriteDataEntry::from_f32(std::f32::consts::PI),
                ReadWriteDataEntry::from_f64(std::f64::consts::E),
                ReadWriteDataEntry::from_i64(0x37312923_19171311),
                ReadWriteDataEntry::from_i64(0),
                ReadWriteDataEntry::from_i64(0),
            ],
            &[],
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::U64(0xf0e0d0c0_b0a09080),
                ForeignValue::U32(0xf0e0d0c0),
                ForeignValue::U32(0xf0e0d0c0),
                ForeignValue::U32(0xf0e0),
                ForeignValue::U32(0xfffff0e0),
                ForeignValue::U32(0xf0),
                ForeignValue::U32(0xfffffff0),
                // group 1
                ForeignValue::F32(std::f32::consts::PI),
                ForeignValue::F64(std::f64::consts::E),
                // group 2
                ForeignValue::U64(0xf0e0d0c0_b0a09080),
                ForeignValue::U32(0xf0e0d0c0),
                ForeignValue::U32(0xf0e0d0c0),
                ForeignValue::U64(0xf0e0d0c0),
                ForeignValue::U64(0xffffffff_f0e0d0c0),
            ]
        );
    }

    #[test]
    fn test_handler_memory_load_and_store() {
        //        read-write data section
        //        =======================
        //
        //       |low address                                                              high address|
        //       |                                                                                     |
        // index |0                                  1      2      3                         4      5  |
        //  type |bytes-------------------|         |f32|  |f64|  |i64------------------|   |i64|  |i64|
        //
        //  data 00 22 44 66 88 aa    cc dd         f32    f64    11 13 17 19 23 29 31 37   0000   0000
        // write 80 90 a0 b0 c0 d0    e0 f0         ---    ---    80 90 a0 b0 c0 d0 e0 f0   i64    i64
        //       |           |        |  |          |      |      ^                         ^      ^
        //       |store32    |store16 |  |          |      |      |                         |      |
        //       |step0      |step1   |  |          |      |      |                         |      |
        //       |           |  store8|  |          |      |      |                         |      |
        //       |           |   step2   |store8    |      |      |store64                  |st64  |st64
        //       |           |            step3     |      |      |                         |      |
        //       \--------------->--load64--------------------->--/                    ld32u| ld32s|
        //        step4      |                      |      |                           step5| step6|
        //                   \--->------------------------------------------------------->--/--->--/
        //
        //  read 80 90 a0 b0 c0 d0    e0 f0         f32    f64    80 90 a0 b0 c0 d0 e0 f0   i64   i64
        //       |           |        |  |load8u    |      |      |           |              |     |
        //       |load64     |load32u |  |load8s  loadf32  |      |           |              |     |
        //       |           |load32s |                  loadf64  |         load32u        load64  |
        //       |           |        |load16u                    |         load32s              load64
        //       |           |        |load16s                 load64
        //
        // () -> (i64,i32,i32,i32,i32,i32,i32,  f32,f64,  i64,i32,i32,i64,i64)

        let code0 = BytecodeWriterHelper::new()
            // step 0, store i32
            .append_opcode_i32(Opcode::imm_i32, 0xb0a09080)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 0) // data public index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::memory_store_i32)
            // step 1, store i16
            .append_opcode_i32(Opcode::imm_i32, 0xd0c0)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 0) // data public index
            .append_opcode_i64(Opcode::imm_i64, 4) // offset in bytes
            .append_opcode(Opcode::memory_store_i16)
            // step 2, store i8
            .append_opcode_i32(Opcode::imm_i32, 0xe0)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 0) // data public index
            .append_opcode_i64(Opcode::imm_i64, 6) // offset in bytes
            .append_opcode(Opcode::memory_store_i8)
            // step 3, store i8
            .append_opcode_i32(Opcode::imm_i32, 0xf0)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 0) // data public index
            .append_opcode_i64(Opcode::imm_i64, 7) // offset in bytes
            .append_opcode(Opcode::memory_store_i8)
            // step 4, load i64 and store i64
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 0) // data public index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::memory_load_i64)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 3) // data public index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::memory_store_i64)
            // step 5, load i32u and store i64
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 0) // data public index
            .append_opcode_i64(Opcode::imm_i64, 4) // offset in bytes
            .append_opcode(Opcode::memory_load_i32_u)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 4) // data public index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::memory_store_i64)
            // step 6, load i32s and store i64
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 0) // data public index
            .append_opcode_i64(Opcode::imm_i64, 4) // offset in bytes
            .append_opcode(Opcode::memory_load_i32_s)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 5) // data public index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::memory_store_i64)
            // load group 0
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 0) // data public index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::memory_load_i64)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 0) // data public index
            .append_opcode_i64(Opcode::imm_i64, 4) // offset in bytes
            .append_opcode(Opcode::memory_load_i32_u)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 0) // data public index
            .append_opcode_i64(Opcode::imm_i64, 4) // offset in bytes
            .append_opcode(Opcode::memory_load_i32_s)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 0) // data public index
            .append_opcode_i64(Opcode::imm_i64, 6) // offset in bytes
            .append_opcode(Opcode::memory_load_i16_u)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 0) // data public index
            .append_opcode_i64(Opcode::imm_i64, 6) // offset in bytes
            .append_opcode(Opcode::memory_load_i16_s)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 0) // data public index
            .append_opcode_i64(Opcode::imm_i64, 7) // offset in bytes
            .append_opcode(Opcode::memory_load_i8_u)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 0) // data public index
            .append_opcode_i64(Opcode::imm_i64, 7) // offset in bytes
            .append_opcode(Opcode::memory_load_i8_s)
            // load group 1
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 1) // data public index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::memory_load_f32)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 2) // data public index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::memory_load_f64)
            // load group 2
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 3) // data public index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::memory_load_i64)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 3) // data public index
            .append_opcode_i64(Opcode::imm_i64, 4) // offset in bytes
            .append_opcode(Opcode::memory_load_i32_u)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 3) // data public index
            .append_opcode_i64(Opcode::imm_i64, 4) // offset in bytes
            .append_opcode(Opcode::memory_load_i32_s)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 4) // data public index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::memory_load_i64)
            .append_opcode_i32(Opcode::imm_i32, 0) // module index
            .append_opcode_i32(Opcode::imm_i32, 5) // data public index
            .append_opcode_i64(Opcode::imm_i64, 0) // offset in bytes
            .append_opcode(Opcode::memory_load_i64)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[
                // group 0
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I32,
                // group 1
                OperandDataType::F32,
                OperandDataType::F64,
                // group 2
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I32,
                OperandDataType::I64,
                OperandDataType::I64,
            ], // results
            &[], // local variables
            code0,
            &[],
            &[
                ReadWriteDataEntry::from_bytes(
                    vec![0u8, 0x22, 0x44, 0x66, 0x88, 0xaa, 0xcc, 0xdd],
                    8,
                ),
                ReadWriteDataEntry::from_f32(std::f32::consts::PI),
                ReadWriteDataEntry::from_f64(std::f64::consts::E),
                ReadWriteDataEntry::from_i64(0x37312923_19171311),
                ReadWriteDataEntry::from_i64(0),
                ReadWriteDataEntry::from_i64(0),
            ],
            &[],
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::U64(0xf0e0d0c0_b0a09080),
                ForeignValue::U32(0xf0e0d0c0),
                ForeignValue::U32(0xf0e0d0c0),
                ForeignValue::U32(0xf0e0),
                ForeignValue::U32(0xfffff0e0),
                ForeignValue::U32(0xf0),
                ForeignValue::U32(0xfffffff0),
                // group 1
                ForeignValue::F32(std::f32::consts::PI),
                ForeignValue::F64(std::f64::consts::E),
                // group 2
                ForeignValue::U64(0xf0e0d0c0_b0a09080),
                ForeignValue::U32(0xf0e0d0c0),
                ForeignValue::U32(0xf0e0d0c0),
                ForeignValue::U64(0xf0e0d0c0),
                ForeignValue::U64(0xffffffff_f0e0d0c0),
            ]
        );
    }

    #[test]
    fn test_handler_data_bounds_check_offset_out_of_range() {
        // Testing: Attempt to load an `i32` data with offset 2.
        // This should fail because the data length exceeds the expected size.

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::data_load_i32_u, 2, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[], // results
            &[], // local variables
            code0,
            &[],
            &[ReadWriteDataEntry::from_i32(11)],
            &[],
        );

        // capture the panic and keep silent
        // it is also possible to check the panic by
        // adding `#[should_panic]` attribute to the function.
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let resource0 = InMemoryProgramSource::new(vec![binary0]);
            let process_context0 = resource0.create_process_context().unwrap();
            let mut thread_context0 = process_context0.create_thread_context();
            // Error: Attempting to load `i32` data with offset 2 (data length exceeded).
            let _ = process_function(&mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_handler_data_bounds_check_data_length_exceeded() {
        // Testing: Attempt to load an `i32` variable using the `local_load_i64` instruction.
        // This should fail because the data length exceeds the expected size.

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
            &[ReadWriteDataEntry::from_i32(11)],
            &[],
        );

        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let resource0 = InMemoryProgramSource::new(vec![binary0]);
            let process_context0 = resource0.create_process_context().unwrap();
            let mut thread_context0 = process_context0.create_thread_context();
            // Error: Attempting to load `i64` from an `i32` variable (data length exceeded).
            let _ = process_function(&mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_handler_data_bounds_check_index_out_of_range() {
        // Testing: Attempt to store an `i32` value into a non-existent data (index 2).
        // This should fail because the index is out of range.

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 11)
            .append_opcode_i16_i32(Opcode::data_store_i32, 2, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[], // results
            &[], // local variables
            code0,
            &[],
            &[ReadWriteDataEntry::from_i32(11)],
            &[],
        );

        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let resource0 = InMemoryProgramSource::new(vec![binary0]);
            let process_context0 = resource0.create_process_context().unwrap();
            let mut thread_context0 = process_context0.create_thread_context();
            // Error: Attempting to access a non-existent local variable (index out of range).
            let _ = process_function(&mut thread_context0, 0, 0, &[]);
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_handler_data_unsupported_floating_point_variant() {
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::data_load_f32, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[], // params
            &[OperandDataType::F32],
            &[], // local variables
            code0,
            &[],
            &[ReadWriteDataEntry::from_f32(std::f32::NAN)],
            &[],
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();

        let mut thread_context0 = process_context0.create_thread_context();
        // Error: Attempting to access an unsupported floating-point variant.
        let result = process_function(&mut thread_context0, 0, 0, &[]);

        assert!(matches!(
            result,
            Err(ProcessorError {
                error_type: ProcessorErrorType::Terminate(
                    TERMINATE_CODE_UNSUPPORTED_FLOATING_POINT_VARIANTS
                )
            })
        ));
    }
}
