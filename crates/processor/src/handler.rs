// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use anc_context::thread_context::{ProgramCounter, ThreadContext};
use anc_image::bytecode_reader::format_bytecode_as_text;
use anc_isa::opcode::{Opcode, MAX_OPCODE_NUMBER};

use crate::{envcall_num::MAX_ENVCALL_CODE_NUMBER, envcall_handler::{generate_envcall_handlers, EnvCallHandlerFunc}, syscall_handler::{
    generate_syscall_handlers, SysCallHandlerFunc, MAX_SYSCALL_TYPE_NUMBER,
}};

pub type HandleFunc = fn(&Handler, &mut ThreadContext) -> HandleResult;

mod arithmetic;
mod bitwise;
mod comparison;
mod control_flow;
mod conversion;
mod data;
mod calling;
mod fundamental;
mod heap;
mod host;
mod local;
mod math;

// mod envcall;
// mod extcall;

pub enum HandleResult {
    // move to another address within a function
    // param (relate_offset_in_bytes:isize)
    Move(isize),

    // jump to another function (call), or
    // return from another function (return)
    // param (return_pc: ProgramCounter)
    Jump(ProgramCounter),

    // the current function call end
    // because the function call could be nested, so there is a
    // original PC need to be restore.
    //
    // if the function is the entry function,
    // then the program should end when encounter this result.
    //
    // if the function call is a nested (in a callback function loop),
    // then the current handler-loop should be ended.
    //
    // param (original_pc: ProgramCounter)
    End(ProgramCounter),

    // param (code: u32)
    Panic(u32),
    // param (code: u32)
    // Unreachable(u32),

    // pause the interpreter
    // for debug the program or the VM itself
    // param (code: u32)
    // Debug(u32),
}

fn unreachable_handler(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    let pc = &thread_context.pc;
    let function_item = &thread_context.module_common_instances[pc.module_index]
        .function_section
        .items[pc.function_internal_index];
    let codes = &thread_context.module_common_instances[pc.module_index]
        .function_section
        .codes_data[function_item.code_offset as usize
        ..(function_item.code_offset + function_item.code_length) as usize];
    let code_text = format_bytecode_as_text(codes);

    unreachable!(
        "\
Invalid opcode: 0x{:04x}
Module index: {}
Function index: {}
Instruction address: 0x{:04x}
Bytecode:
{}",
        thread_context.get_opcode_num(),
        pc.module_index,
        pc.function_internal_index,
        pc.instruction_address,
        code_text
    );
}

// static INIT: Once = Once::new();
// static mut INTERPRETERS: [InterpretFunc; MAX_OPCODE_NUMBER] = [unreachable; MAX_OPCODE_NUMBER];
//
// // initilize the instruction interpreters
// //
// // note:
// // ensure this initialization is only called once,
// // to do that, the 3rd party crates such as 'lazy_static', 'once_cell' and 'rust-ctor' can be used.
// // the same can be done with 'std::sync::Once'.
// #[inline]
// pub fn init_interpreters() {
//     INIT.call_once(|| {
//         init_interpreters_internal();
//     });
// }

pub struct Handler {
    pub handlers: [HandleFunc; MAX_OPCODE_NUMBER],
    pub syscall_handlers: [SysCallHandlerFunc; MAX_SYSCALL_TYPE_NUMBER],
    pub envcall_handlers: [EnvCallHandlerFunc; MAX_ENVCALL_CODE_NUMBER]
}

impl Handler {
    pub fn new() -> Self {
        let mut handlers: [HandleFunc; MAX_OPCODE_NUMBER] =
            [unreachable_handler; MAX_OPCODE_NUMBER];

        // fundamental
        handlers[Opcode::nop as usize] = fundamental::nop;
        // handlers[Opcode::zero as usize] = fundamental::zero;
        // handlers[Opcode::drop as usize] = fundamental::drop_;
        // handlers[Opcode::duplicate as usize] = fundamental::duplicate;
        // handlers[Opcode::swap as usize] = fundamental::swap;
        // handlers[Opcode::select_nez as usize] = fundamental::select_nez;
        handlers[Opcode::imm_i32 as usize] = fundamental::imm_i32;
        handlers[Opcode::imm_i64 as usize] = fundamental::imm_i64;
        handlers[Opcode::imm_f32 as usize] = fundamental::imm_f32;
        handlers[Opcode::imm_f64 as usize] = fundamental::imm_f64;

        // local variable
        handlers[Opcode::local_load_i64 as usize] = local::local_load_i64;
        handlers[Opcode::local_load_i32_s as usize] = local::local_load_i32_s;
        handlers[Opcode::local_load_i32_u as usize] = local::local_load_i32_u;
        handlers[Opcode::local_load_i16_s as usize] = local::local_load_i16_s;
        handlers[Opcode::local_load_i16_u as usize] = local::local_load_i16_u;
        handlers[Opcode::local_load_i8_s as usize] = local::local_load_i8_s;
        handlers[Opcode::local_load_i8_u as usize] = local::local_load_i8_u;
        handlers[Opcode::local_load_f32 as usize] = local::local_load_f32;
        handlers[Opcode::local_load_f64 as usize] = local::local_load_f64;
        handlers[Opcode::local_store_i64 as usize] = local::local_store_i64;
        handlers[Opcode::local_store_i32 as usize] = local::local_store_i32;
        handlers[Opcode::local_store_i16 as usize] = local::local_store_i16;
        handlers[Opcode::local_store_i8 as usize] = local::local_store_i8;
        handlers[Opcode::local_store_f64 as usize] = local::local_store_i64; // store_f64 == store_i64
        handlers[Opcode::local_store_f32 as usize] = local::local_store_i32; // store_f32 == store_i32

        // local variable extend
        handlers[Opcode::local_load_extend_i64 as usize] = local::local_load_extend_i64;
        handlers[Opcode::local_load_extend_i32_s as usize] = local::local_load_extend_i32_s;
        handlers[Opcode::local_load_extend_i32_u as usize] = local::local_load_extend_i32_u;
        handlers[Opcode::local_load_extend_i16_s as usize] = local::local_load_extend_i16_s;
        handlers[Opcode::local_load_extend_i16_u as usize] = local::local_load_extend_i16_u;
        handlers[Opcode::local_load_extend_i8_s as usize] = local::local_load_extend_i8_s;
        handlers[Opcode::local_load_extend_i8_u as usize] = local::local_load_extend_i8_u;
        handlers[Opcode::local_load_extend_f64 as usize] = local::local_load_extend_f64;
        handlers[Opcode::local_load_extend_f32 as usize] = local::local_load_extend_f32;
        handlers[Opcode::local_store_extend_i64 as usize] = local::local_store_extend_i64;
        handlers[Opcode::local_store_extend_i32 as usize] = local::local_store_extend_i32;
        handlers[Opcode::local_store_extend_i16 as usize] = local::local_store_extend_i16;
        handlers[Opcode::local_store_extend_i8 as usize] = local::local_store_extend_i8;
        handlers[Opcode::local_store_extend_f64 as usize] = local::local_store_extend_i64; // store_f64 == store_i64
        handlers[Opcode::local_store_extend_f32 as usize] = local::local_store_extend_i32; // store_f32 == store_i32

        // data
        handlers[Opcode::data_load_i64 as usize] = data::data_load_i64;
        handlers[Opcode::data_load_i32_s as usize] = data::data_load_i32_s;
        handlers[Opcode::data_load_i32_u as usize] = data::data_load_i32_u;
        handlers[Opcode::data_load_i16_s as usize] = data::data_load_i16_s;
        handlers[Opcode::data_load_i16_u as usize] = data::data_load_i16_u;
        handlers[Opcode::data_load_i8_s as usize] = data::data_load_i8_s;
        handlers[Opcode::data_load_i8_u as usize] = data::data_load_i8_u;
        handlers[Opcode::data_load_f64 as usize] = data::data_load_f64;
        handlers[Opcode::data_load_f32 as usize] = data::data_load_f32;
        handlers[Opcode::data_store_i64 as usize] = data::data_store_i64;
        handlers[Opcode::data_store_i32 as usize] = data::data_store_i32;
        handlers[Opcode::data_store_i16 as usize] = data::data_store_i16;
        handlers[Opcode::data_store_i8 as usize] = data::data_store_i8;
        handlers[Opcode::data_store_f64 as usize] = data::data_store_i64; // store_f64 == store_i64
        handlers[Opcode::data_store_f32 as usize] = data::data_store_i32; // store_f32 == store_i32

        // data extend
        handlers[Opcode::data_load_extend_i64 as usize] = data::data_load_extend_i64;
        handlers[Opcode::data_load_extend_i32_s as usize] = data::data_load_extend_i32_s;
        handlers[Opcode::data_load_extend_i32_u as usize] = data::data_load_extend_i32_u;
        handlers[Opcode::data_load_extend_i16_s as usize] = data::data_load_extend_i16_s;
        handlers[Opcode::data_load_extend_i16_u as usize] = data::data_load_extend_i16_u;
        handlers[Opcode::data_load_extend_i8_s as usize] = data::data_load_extend_i8_s;
        handlers[Opcode::data_load_extend_i8_u as usize] = data::data_load_extend_i8_u;
        handlers[Opcode::data_load_extend_f64 as usize] = data::data_load_extend_f64;
        handlers[Opcode::data_load_extend_f32 as usize] = data::data_load_extend_f32;
        handlers[Opcode::data_store_extend_i64 as usize] = data::data_store_extend_i64;
        handlers[Opcode::data_store_extend_i32 as usize] = data::data_store_extend_i32;
        handlers[Opcode::data_store_extend_i16 as usize] = data::data_store_extend_i16;
        handlers[Opcode::data_store_extend_i8 as usize] = data::data_store_extend_i8;
        handlers[Opcode::data_store_extend_f64 as usize] = data::data_store_extend_i64; // store_f64 == store_i64
        handlers[Opcode::data_store_extend_f32 as usize] = data::data_store_extend_i32; // store_f32 == store_i32

        // heap
        handlers[Opcode::heap_load_i64 as usize] = heap::heap_load_i64;
        handlers[Opcode::heap_load_i32_s as usize] = heap::heap_load_i32_s;
        handlers[Opcode::heap_load_i32_u as usize] = heap::heap_load_i32_u;
        handlers[Opcode::heap_load_i16_s as usize] = heap::heap_load_i16_s;
        handlers[Opcode::heap_load_i16_u as usize] = heap::heap_load_i16_u;
        handlers[Opcode::heap_load_i8_s as usize] = heap::heap_load_i8_s;
        handlers[Opcode::heap_load_i8_u as usize] = heap::heap_load_i8_u;
        handlers[Opcode::heap_load_f64 as usize] = heap::heap_load_f64;
        handlers[Opcode::heap_load_f32 as usize] = heap::heap_load_f32;
        handlers[Opcode::heap_store_i64 as usize] = heap::heap_store_i64;
        handlers[Opcode::heap_store_i32 as usize] = heap::heap_store_i32;
        handlers[Opcode::heap_store_i16 as usize] = heap::heap_store_i16;
        handlers[Opcode::heap_store_i8 as usize] = heap::heap_store_i8;
        handlers[Opcode::heap_store_f64 as usize] = heap::heap_store_i64; // store_f64 == store_i64
        handlers[Opcode::heap_store_f32 as usize] = heap::heap_store_i32; // store_f32 == store_i32

        // heap memory operations
        handlers[Opcode::heap_fill as usize] = heap::heap_fill;
        handlers[Opcode::heap_copy as usize] = heap::heap_copy;
        handlers[Opcode::heap_capacity as usize] = heap::heap_capacity;
        handlers[Opcode::heap_resize as usize] = heap::heap_resize;

        // conversion
        handlers[Opcode::truncate_i64_to_i32 as usize] = conversion::truncate_i64_to_i32;
        handlers[Opcode::extend_i32_s_to_i64 as usize] = conversion::extend_i32_s_to_i64;
        handlers[Opcode::extend_i32_u_to_i64 as usize] = conversion::extend_i32_u_to_i64;
        handlers[Opcode::demote_f64_to_f32 as usize] = conversion::demote_f64_to_f32;
        handlers[Opcode::promote_f32_to_f64 as usize] = conversion::promote_f32_to_f64;
        handlers[Opcode::convert_f32_to_i32_s as usize] = conversion::convert_f32_to_i32_s;
        handlers[Opcode::convert_f32_to_i32_u as usize] = conversion::convert_f32_to_i32_u;
        handlers[Opcode::convert_f64_to_i32_s as usize] = conversion::convert_f64_to_i32_s;
        handlers[Opcode::convert_f64_to_i32_u as usize] = conversion::convert_f64_to_i32_u;
        handlers[Opcode::convert_f32_to_i64_s as usize] = conversion::convert_f32_to_i64_s;
        handlers[Opcode::convert_f32_to_i64_u as usize] = conversion::convert_f32_to_i64_u;
        handlers[Opcode::convert_f64_to_i64_s as usize] = conversion::convert_f64_to_i64_s;
        handlers[Opcode::convert_f64_to_i64_u as usize] = conversion::convert_f64_to_i64_u;
        handlers[Opcode::convert_i32_s_to_f32 as usize] = conversion::convert_i32_s_to_f32;
        handlers[Opcode::convert_i32_u_to_f32 as usize] = conversion::convert_i32_u_to_f32;
        handlers[Opcode::convert_i64_s_to_f32 as usize] = conversion::convert_i64_s_to_f32;
        handlers[Opcode::convert_i64_u_to_f32 as usize] = conversion::convert_i64_u_to_f32;
        handlers[Opcode::convert_i32_s_to_f64 as usize] = conversion::convert_i32_s_to_f64;
        handlers[Opcode::convert_i32_u_to_f64 as usize] = conversion::convert_i32_u_to_f64;
        handlers[Opcode::convert_i64_s_to_f64 as usize] = conversion::convert_i64_s_to_f64;
        handlers[Opcode::convert_i64_u_to_f64 as usize] = conversion::convert_i64_u_to_f64;

        // comparison
        handlers[Opcode::eqz_i32 as usize] = comparison::eqz_i32;
        handlers[Opcode::nez_i32 as usize] = comparison::nez_i32;
        handlers[Opcode::eq_i32 as usize] = comparison::eq_i32;
        handlers[Opcode::ne_i32 as usize] = comparison::ne_i32;
        handlers[Opcode::lt_i32_s as usize] = comparison::lt_i32_s;
        handlers[Opcode::lt_i32_u as usize] = comparison::lt_i32_u;
        handlers[Opcode::gt_i32_s as usize] = comparison::gt_i32_s;
        handlers[Opcode::gt_i32_u as usize] = comparison::gt_i32_u;
        handlers[Opcode::le_i32_s as usize] = comparison::le_i32_s;
        handlers[Opcode::le_i32_u as usize] = comparison::le_i32_u;
        handlers[Opcode::ge_i32_s as usize] = comparison::ge_i32_s;
        handlers[Opcode::ge_i32_u as usize] = comparison::ge_i32_u;
        handlers[Opcode::eqz_i64 as usize] = comparison::eqz_i64;
        handlers[Opcode::nez_i64 as usize] = comparison::nez_i64;
        handlers[Opcode::eq_i64 as usize] = comparison::eq_i64;
        handlers[Opcode::ne_i64 as usize] = comparison::ne_i64;
        handlers[Opcode::lt_i64_s as usize] = comparison::lt_i64_s;
        handlers[Opcode::lt_i64_u as usize] = comparison::lt_i64_u;
        handlers[Opcode::gt_i64_s as usize] = comparison::gt_i64_s;
        handlers[Opcode::gt_i64_u as usize] = comparison::gt_i64_u;
        handlers[Opcode::le_i64_s as usize] = comparison::le_i64_s;
        handlers[Opcode::le_i64_u as usize] = comparison::le_i64_u;
        handlers[Opcode::ge_i64_s as usize] = comparison::ge_i64_s;
        handlers[Opcode::ge_i64_u as usize] = comparison::ge_i64_u;
        handlers[Opcode::eq_f32 as usize] = comparison::eq_f32;
        handlers[Opcode::ne_f32 as usize] = comparison::ne_f32;
        handlers[Opcode::lt_f32 as usize] = comparison::lt_f32;
        handlers[Opcode::gt_f32 as usize] = comparison::gt_f32;
        handlers[Opcode::le_f32 as usize] = comparison::le_f32;
        handlers[Opcode::ge_f32 as usize] = comparison::ge_f32;
        handlers[Opcode::eq_f64 as usize] = comparison::eq_f64;
        handlers[Opcode::ne_f64 as usize] = comparison::ne_f64;
        handlers[Opcode::lt_f64 as usize] = comparison::lt_f64;
        handlers[Opcode::gt_f64 as usize] = comparison::gt_f64;
        handlers[Opcode::le_f64 as usize] = comparison::le_f64;
        handlers[Opcode::ge_f64 as usize] = comparison::ge_f64;

        // arithmetic i32
        handlers[Opcode::add_i32 as usize] = arithmetic::add_i32;
        handlers[Opcode::sub_i32 as usize] = arithmetic::sub_i32;
        handlers[Opcode::add_imm_i32 as usize] = arithmetic::add_imm_i32;
        handlers[Opcode::sub_imm_i32 as usize] = arithmetic::sub_imm_i32;
        handlers[Opcode::mul_i32 as usize] = arithmetic::mul_i32;
        handlers[Opcode::div_i32_s as usize] = arithmetic::div_i32_s;
        handlers[Opcode::div_i32_u as usize] = arithmetic::div_i32_u;
        handlers[Opcode::rem_i32_s as usize] = arithmetic::rem_i32_s;
        handlers[Opcode::rem_i32_u as usize] = arithmetic::rem_i32_u;

        // arithmetic i64
        handlers[Opcode::add_i64 as usize] = arithmetic::add_i64;
        handlers[Opcode::sub_i64 as usize] = arithmetic::sub_i64;
        handlers[Opcode::add_imm_i64 as usize] = arithmetic::add_imm_i64;
        handlers[Opcode::sub_imm_i64 as usize] = arithmetic::sub_imm_i64;
        handlers[Opcode::mul_i64 as usize] = arithmetic::mul_i64;
        handlers[Opcode::div_i64_s as usize] = arithmetic::div_i64_s;
        handlers[Opcode::div_i64_u as usize] = arithmetic::div_i64_u;
        handlers[Opcode::rem_i64_s as usize] = arithmetic::rem_i64_s;
        handlers[Opcode::rem_i64_u as usize] = arithmetic::rem_i64_u;

        // arithmetic f32 and f64
        handlers[Opcode::add_f32 as usize] = arithmetic::add_f32;
        handlers[Opcode::sub_f32 as usize] = arithmetic::sub_f32;
        handlers[Opcode::mul_f32 as usize] = arithmetic::mul_f32;
        handlers[Opcode::div_f32 as usize] = arithmetic::div_f32;
        handlers[Opcode::add_f64 as usize] = arithmetic::add_f64;
        handlers[Opcode::sub_f64 as usize] = arithmetic::sub_f64;
        handlers[Opcode::mul_f64 as usize] = arithmetic::mul_f64;
        handlers[Opcode::div_f64 as usize] = arithmetic::div_f64;

        // bitwise
        handlers[Opcode::and as usize] = bitwise::and;
        handlers[Opcode::or as usize] = bitwise::or;
        handlers[Opcode::xor as usize] = bitwise::xor;
        handlers[Opcode::not as usize] = bitwise::not;
        handlers[Opcode::count_leading_zeros_i32 as usize] = bitwise::count_leading_zeros_i32;
        handlers[Opcode::count_leading_ones_i32 as usize] = bitwise::count_leading_ones_i32;
        handlers[Opcode::count_trailing_zeros_i32 as usize] = bitwise::count_trailing_zeros_i32;
        handlers[Opcode::count_ones_i32 as usize] = bitwise::count_ones_i32;
        handlers[Opcode::shift_left_i32 as usize] = bitwise::shift_left_i32;
        handlers[Opcode::shift_right_i32_s as usize] = bitwise::shift_right_i32_s;
        handlers[Opcode::shift_right_i32_u as usize] = bitwise::shift_right_i32_u;
        handlers[Opcode::rotate_left_i32 as usize] = bitwise::rotate_left_i32;
        handlers[Opcode::rotate_right_i32 as usize] = bitwise::rotate_right_i32;
        handlers[Opcode::count_leading_zeros_i64 as usize] = bitwise::count_leading_zeros_i64;
        handlers[Opcode::count_leading_ones_i64 as usize] = bitwise::count_leading_ones_i64;
        handlers[Opcode::count_trailing_zeros_i64 as usize] = bitwise::count_trailing_zeros_i64;
        handlers[Opcode::count_ones_i64 as usize] = bitwise::count_ones_i64;
        handlers[Opcode::shift_left_i64 as usize] = bitwise::shift_left_i64;
        handlers[Opcode::shift_right_i64_s as usize] = bitwise::shift_right_i64_s;
        handlers[Opcode::shift_right_i64_u as usize] = bitwise::shift_right_i64_u;
        handlers[Opcode::rotate_left_i64 as usize] = bitwise::rotate_left_i64;
        handlers[Opcode::rotate_right_i64 as usize] = bitwise::rotate_right_i64;

        // math - int
        handlers[Opcode::abs_i32 as usize] = math::abs_i32;
        handlers[Opcode::neg_i32 as usize] = math::neg_i32;
        handlers[Opcode::abs_i64 as usize] = math::abs_i64;
        handlers[Opcode::neg_i64 as usize] = math::neg_i64;

        // math - f32
        handlers[Opcode::abs_f32 as usize] = math::abs_f32;
        handlers[Opcode::neg_f32 as usize] = math::neg_f32;
        handlers[Opcode::ceil_f32 as usize] = math::ceil_f32;
        handlers[Opcode::floor_f32 as usize] = math::floor_f32;
        handlers[Opcode::round_half_away_from_zero_f32 as usize] =
            math::round_half_away_from_zero_f32;
        handlers[Opcode::round_half_to_even_f32 as usize] = math::round_half_to_even_f32;
        handlers[Opcode::trunc_f32 as usize] = math::trunc_f32;
        handlers[Opcode::fract_f32 as usize] = math::fract_f32;
        handlers[Opcode::sqrt_f32 as usize] = math::sqrt_f32;
        handlers[Opcode::cbrt_f32 as usize] = math::cbrt_f32;
        handlers[Opcode::exp_f32 as usize] = math::exp_f32;
        handlers[Opcode::exp2_f32 as usize] = math::exp2_f32;
        handlers[Opcode::ln_f32 as usize] = math::ln_f32;
        handlers[Opcode::log2_f32 as usize] = math::log2_f32;
        handlers[Opcode::log10_f32 as usize] = math::log10_f32;
        handlers[Opcode::sin_f32 as usize] = math::sin_f32;
        handlers[Opcode::cos_f32 as usize] = math::cos_f32;
        handlers[Opcode::tan_f32 as usize] = math::tan_f32;
        handlers[Opcode::asin_f32 as usize] = math::asin_f32;
        handlers[Opcode::acos_f32 as usize] = math::acos_f32;
        handlers[Opcode::atan_f32 as usize] = math::atan_f32;
        handlers[Opcode::copysign_f32 as usize] = math::copysign_f32;
        handlers[Opcode::pow_f32 as usize] = math::pow_f32;
        handlers[Opcode::log_f32 as usize] = math::log_f32;
        handlers[Opcode::min_f32 as usize] = math::min_f32;
        handlers[Opcode::max_f32 as usize] = math::max_f32;

        // math - f64
        handlers[Opcode::abs_f64 as usize] = math::abs_f64;
        handlers[Opcode::neg_f64 as usize] = math::neg_f64;
        handlers[Opcode::ceil_f64 as usize] = math::ceil_f64;
        handlers[Opcode::floor_f64 as usize] = math::floor_f64;
        handlers[Opcode::round_half_away_from_zero_f64 as usize] =
            math::round_half_away_from_zero_f64;
        handlers[Opcode::round_half_to_even_f64 as usize] = math::round_half_to_even_f64;
        handlers[Opcode::trunc_f64 as usize] = math::trunc_f64;
        handlers[Opcode::fract_f64 as usize] = math::fract_f64;
        handlers[Opcode::sqrt_f64 as usize] = math::sqrt_f64;
        handlers[Opcode::cbrt_f64 as usize] = math::cbrt_f64;
        handlers[Opcode::exp_f64 as usize] = math::exp_f64;
        handlers[Opcode::exp2_f64 as usize] = math::exp2_f64;
        handlers[Opcode::ln_f64 as usize] = math::ln_f64;
        handlers[Opcode::log2_f64 as usize] = math::log2_f64;
        handlers[Opcode::log10_f64 as usize] = math::log10_f64;
        handlers[Opcode::sin_f64 as usize] = math::sin_f64;
        handlers[Opcode::cos_f64 as usize] = math::cos_f64;
        handlers[Opcode::tan_f64 as usize] = math::tan_f64;
        handlers[Opcode::asin_f64 as usize] = math::asin_f64;
        handlers[Opcode::acos_f64 as usize] = math::acos_f64;
        handlers[Opcode::atan_f64 as usize] = math::atan_f64;
        handlers[Opcode::copysign_f64 as usize] = math::copysign_f64;
        handlers[Opcode::pow_f64 as usize] = math::pow_f64;
        handlers[Opcode::log_f64 as usize] = math::log_f64;
        handlers[Opcode::min_f64 as usize] = math::min_f64;
        handlers[Opcode::max_f64 as usize] = math::max_f64;

        // control flow
        handlers[Opcode::end as usize] = control_flow::end;
        handlers[Opcode::block as usize] = control_flow::block;
        handlers[Opcode::break_ as usize] = control_flow::break_;
        handlers[Opcode::recur as usize] = control_flow::recur;
        handlers[Opcode::block_alt as usize] = control_flow::block_alt;
        handlers[Opcode::break_alt as usize] = control_flow::break_alt;
        handlers[Opcode::block_nez as usize] = control_flow::block_nez;
        handlers[Opcode::break_nez as usize] = control_flow::break_nez;
        handlers[Opcode::recur_nez as usize] = control_flow::recur_nez;

        // function call
        handlers[Opcode::call as usize] = calling::call;
        handlers[Opcode::dyncall as usize] = calling::dyncall;

        // other calling
        handlers[Opcode::syscall as usize] = calling::syscall;
        handlers[Opcode::envcall as usize] = calling::envcall;
        handlers[Opcode::extcall as usize] = calling::extcall;
        handlers[Opcode::pub_index_function as usize] = calling::pub_index_function;

        // host
        handlers[Opcode::panic as usize] = host::panic;
        // handlers[Opcode::unreachable as usize] = host::unreachable;
        // handlers[Opcode::debug as usize] = host::debug;
        handlers[Opcode::host_addr_local as usize] = host::host_addr_local;
        handlers[Opcode::host_addr_local_extend as usize] = host::host_addr_local_extend;
        handlers[Opcode::host_addr_data as usize] = host::host_addr_data;
        handlers[Opcode::host_addr_data_extend as usize] = host::host_addr_data_extend;
        handlers[Opcode::host_addr_heap as usize] = host::host_addr_heap;
        handlers[Opcode::host_addr_function as usize] = host::host_addr_function;
        handlers[Opcode::host_copy_heap_to_memory as usize] = host::host_copy_heap_to_memory;
        handlers[Opcode::host_copy_memory_to_heap as usize] = host::host_copy_memory_to_heap;
        handlers[Opcode::host_memory_copy as usize] = host::host_memory_copy;

        Handler {
            handlers,
            syscall_handlers: generate_syscall_handlers(),
            envcall_handlers: generate_envcall_handlers()
        }
    }
}

impl Default for Handler {
    fn default() -> Self {
        Self::new()
    }
}