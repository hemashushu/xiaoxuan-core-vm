// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::thread_context::{ProgramCounter, ThreadContext};
use ancvm_image::bytecode_reader::format_bytecode_as_text;
use ancvm_isa::opcode::{Opcode, MAX_OPCODE_NUMBER};

type HandleFunc = fn(&mut ThreadContext) -> HandleResult;

// mod arithmetic;
// mod bitwise;
// mod comparison;
mod control_flow;
// mod conversion;
mod data;
// mod funcall;
mod fundamental;
// mod heap;
// mod host;
// mod local;
// mod math;
// mod syscall;

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

fn unreachable_handler(thread_context: &mut ThreadContext) -> HandleResult {
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
}

impl Handler {
    pub fn new() -> Self {
        // todo
        // // other initializations
        // init_ecall_handlers();
        // init_syscall_handlers();

        // let interpreters = unsafe { &mut INTERPRETERS };
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
        //
        // local variables
        // handlers[Opcode::local_load_i64 as usize] = local::local_load_i64;
        // handlers[Opcode::local_load_i32 as usize] = local::local_load_i32;
        // handlers[Opcode::local_load_i32 as usize] = local::local_load_i32;
        // handlers[Opcode::local_load_i16_s as usize] = local::local_load32_i16_s;
        // handlers[Opcode::local_load_i16_u as usize] = local::local_load32_i16_u;
        // handlers[Opcode::local_load_i8_s as usize] = local::local_load32_i8_s;
        // handlers[Opcode::local_load_i8_u as usize] = local::local_load32_i8_u;
        // handlers[Opcode::local_load_f32 as usize] = local::local_load32_f32;
        // handlers[Opcode::local_load_f64 as usize] = local::local_load64_f64;
        // handlers[Opcode::local_store_i64 as usize] = local::local_store_i64;
        // handlers[Opcode::local_store_i32 as usize] = local::local_store32;
        // handlers[Opcode::local_store_i16 as usize] = local::local_store16;
        // handlers[Opcode::local_store_i8 as usize] = local::local_store8;
        //
        //     handlers[Opcode::local_offset_load64_i64 as usize] = local::local_offset_load64_i64;
        //     handlers[Opcode::local_offset_load64_f64 as usize] = local::local_offset_load64_f64;
        //     handlers[Opcode::local_offset_load32_i32 as usize] = local::local_offset_load32_i32;
        //     handlers[Opcode::local_offset_load32_i16_s as usize] = local::local_offset_load32_i16_s;
        //     handlers[Opcode::local_offset_load32_i16_u as usize] = local::local_offset_load32_i16_u;
        //     handlers[Opcode::local_offset_load32_i8_s as usize] = local::local_offset_load32_i8_s;
        //     handlers[Opcode::local_offset_load32_i8_u as usize] = local::local_offset_load32_i8_u;
        //     handlers[Opcode::local_offset_load32_f32 as usize] = local::local_offset_load32_f32;
        //     handlers[Opcode::local_offset_store64 as usize] = local::local_offset_store64;
        //     handlers[Opcode::local_offset_store32 as usize] = local::local_offset_store32;
        //     handlers[Opcode::local_offset_store16 as usize] = local::local_offset_store16;
        //     handlers[Opcode::local_offset_store8 as usize] = local::local_offset_store8;
        //
            // data sections
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

        //     // heap
        //     handlers[Opcode::heap_load_i64 as usize] = heap::heap_load_i64;
        //     handlers[Opcode::heap_load64_f64 as usize] = heap::heap_load64_f64;
        //     handlers[Opcode::heap_load32_i32 as usize] = heap::heap_load32_i32;
        //     handlers[Opcode::heap_load32_i16_s as usize] = heap::heap_load32_i16_s;
        //     handlers[Opcode::heap_load32_i16_u as usize] = heap::heap_load32_i16_u;
        //     handlers[Opcode::heap_load32_i8_s as usize] = heap::heap_load32_i8_s;
        //     handlers[Opcode::heap_load32_i8_u as usize] = heap::heap_load32_i8_u;
        //     handlers[Opcode::heap_load32_f32 as usize] = heap::heap_load32_f32;
        //     handlers[Opcode::heap_store_i64 as usize] = heap::heap_store_i64;
        //     handlers[Opcode::heap_store32 as usize] = heap::heap_store32;
        //     handlers[Opcode::heap_store16 as usize] = heap::heap_store16;
        //     handlers[Opcode::heap_store8 as usize] = heap::heap_store8;
        //
        //     // heap memory
        //     handlers[Opcode::heap_fill as usize] = heap::heap_fill;
        //     handlers[Opcode::heap_copy as usize] = heap::heap_copy;
        //     handlers[Opcode::heap_capacity as usize] = heap::heap_capacity;
        //     handlers[Opcode::heap_resize as usize] = heap::heap_resize;
        //
        //     // conversion
        //     handlers[Opcode::truncate_i64_to_i32 as usize] = conversion::truncate_i64_to_i32;
        //     handlers[Opcode::i64_extend_i32_s as usize] = conversion::i64_extend_i32_s;
        //     handlers[Opcode::i64_extend_i32_u as usize] = conversion::i64_extend_i32_u;
        //     handlers[Opcode::f32_demote_f64 as usize] = conversion::f32_demote_f64;
        //     handlers[Opcode::f64_promote_f32 as usize] = conversion::f64_promote_f32;
        //     handlers[Opcode::i32_convert_f32_s as usize] = conversion::i32_convert_f32_s;
        //     handlers[Opcode::i32_convert_f32_u as usize] = conversion::i32_convert_f32_u;
        //     handlers[Opcode::i32_convert_f64_s as usize] = conversion::i32_convert_f64_s;
        //     handlers[Opcode::i32_convert_f64_u as usize] = conversion::i32_convert_f64_u;
        //     handlers[Opcode::i64_convert_f32_s as usize] = conversion::i64_convert_f32_s;
        //     handlers[Opcode::i64_convert_f32_u as usize] = conversion::i64_convert_f32_u;
        //     handlers[Opcode::i64_convert_f64_s as usize] = conversion::i64_convert_f64_s;
        //     handlers[Opcode::i64_convert_f64_u as usize] = conversion::i64_convert_f64_u;
        //     handlers[Opcode::f32_convert_i32_s as usize] = conversion::f32_convert_i32_s;
        //     handlers[Opcode::f32_convert_i32_u as usize] = conversion::f32_convert_i32_u;
        //     handlers[Opcode::f32_convert_i64_s as usize] = conversion::f32_convert_i64_s;
        //     handlers[Opcode::f32_convert_i64_u as usize] = conversion::f32_convert_i64_u;
        //     handlers[Opcode::f64_convert_i32_s as usize] = conversion::f64_convert_i32_s;
        //     handlers[Opcode::f64_convert_i32_u as usize] = conversion::f64_convert_i32_u;
        //     handlers[Opcode::f64_convert_i64_s as usize] = conversion::f64_convert_i64_s;
        //     handlers[Opcode::f64_convert_i64_u as usize] = conversion::f64_convert_i64_u;
        //
        //     // comparison
        //     handlers[Opcode::eqz_i32 as usize] = comparison::eqz_i32;
        //     handlers[Opcode::eq_i32 as usize] = comparison::eq_i32;
        //     handlers[Opcode::nez_i32 as usize] = comparison::nez_i32;
        //     handlers[Opcode::i32_ne as usize] = comparison::i32_ne;
        //     handlers[Opcode::i32_lt_s as usize] = comparison::i32_lt_s;
        //     handlers[Opcode::i32_lt_u as usize] = comparison::i32_lt_u;
        //     handlers[Opcode::i32_gt_s as usize] = comparison::i32_gt_s;
        //     handlers[Opcode::i32_gt_u as usize] = comparison::i32_gt_u;
        //     handlers[Opcode::i32_le_s as usize] = comparison::i32_le_s;
        //     handlers[Opcode::i32_le_u as usize] = comparison::i32_le_u;
        //     handlers[Opcode::i32_ge_s as usize] = comparison::i32_ge_s;
        //     handlers[Opcode::i32_ge_u as usize] = comparison::i32_ge_u;
        //     handlers[Opcode::i64_eqz as usize] = comparison::i64_eqz;
        //     handlers[Opcode::i64_eq as usize] = comparison::i64_eq;
        //     handlers[Opcode::i64_nez as usize] = comparison::i64_nez;
        //     handlers[Opcode::i64_ne as usize] = comparison::i64_ne;
        //     handlers[Opcode::i64_lt_s as usize] = comparison::i64_lt_s;
        //     handlers[Opcode::i64_lt_u as usize] = comparison::i64_lt_u;
        //     handlers[Opcode::i64_gt_s as usize] = comparison::i64_gt_s;
        //     handlers[Opcode::i64_gt_u as usize] = comparison::i64_gt_u;
        //     handlers[Opcode::i64_le_s as usize] = comparison::i64_le_s;
        //     handlers[Opcode::i64_le_u as usize] = comparison::i64_le_u;
        //     handlers[Opcode::i64_ge_s as usize] = comparison::i64_ge_s;
        //     handlers[Opcode::i64_ge_u as usize] = comparison::i64_ge_u;
        //     handlers[Opcode::f32_eq as usize] = comparison::f32_eq;
        //     handlers[Opcode::f32_ne as usize] = comparison::f32_ne;
        //     handlers[Opcode::f32_lt as usize] = comparison::f32_lt;
        //     handlers[Opcode::f32_gt as usize] = comparison::f32_gt;
        //     handlers[Opcode::f32_le as usize] = comparison::f32_le;
        //     handlers[Opcode::f32_ge as usize] = comparison::f32_ge;
        //     handlers[Opcode::f64_eq as usize] = comparison::f64_eq;
        //     handlers[Opcode::f64_ne as usize] = comparison::f64_ne;
        //     handlers[Opcode::f64_lt as usize] = comparison::f64_lt;
        //     handlers[Opcode::f64_gt as usize] = comparison::f64_gt;
        //     handlers[Opcode::f64_le as usize] = comparison::f64_le;
        //     handlers[Opcode::f64_ge as usize] = comparison::f64_ge;
        //
        //     // arithmetic
        //     handlers[Opcode::add_i32 as usize] = arithmetic::add_i32;
        //     handlers[Opcode::sub_i32 as usize] = arithmetic::sub_i32;
        //     handlers[Opcode::i32_mul as usize] = arithmetic::i32_mul;
        //     handlers[Opcode::i32_div_s as usize] = arithmetic::i32_div_s;
        //     handlers[Opcode::i32_div_u as usize] = arithmetic::i32_div_u;
        //     handlers[Opcode::i32_rem_s as usize] = arithmetic::i32_rem_s;
        //     handlers[Opcode::i32_rem_u as usize] = arithmetic::i32_rem_u;
        //     handlers[Opcode::i32_inc as usize] = arithmetic::i32_inc;
        //     handlers[Opcode::i32_dec as usize] = arithmetic::i32_dec;
        //     handlers[Opcode::i64_add as usize] = arithmetic::i64_add;
        //     handlers[Opcode::i64_sub as usize] = arithmetic::i64_sub;
        //     handlers[Opcode::i64_mul as usize] = arithmetic::i64_mul;
        //     handlers[Opcode::i64_div_s as usize] = arithmetic::i64_div_s;
        //     handlers[Opcode::i64_div_u as usize] = arithmetic::i64_div_u;
        //     handlers[Opcode::i64_rem_s as usize] = arithmetic::i64_rem_s;
        //     handlers[Opcode::i64_rem_u as usize] = arithmetic::i64_rem_u;
        //     handlers[Opcode::i64_inc as usize] = arithmetic::i64_inc;
        //     handlers[Opcode::i64_dec as usize] = arithmetic::i64_dec;
        //     handlers[Opcode::f32_add as usize] = arithmetic::f32_add;
        //     handlers[Opcode::f32_sub as usize] = arithmetic::f32_sub;
        //     handlers[Opcode::f32_mul as usize] = arithmetic::f32_mul;
        //     handlers[Opcode::f32_div as usize] = arithmetic::f32_div;
        //     handlers[Opcode::f64_add as usize] = arithmetic::f64_add;
        //     handlers[Opcode::f64_sub as usize] = arithmetic::f64_sub;
        //     handlers[Opcode::f64_mul as usize] = arithmetic::f64_mul;
        //     handlers[Opcode::f64_div as usize] = arithmetic::f64_div;
        //
        //     // bitwise
        //     handlers[Opcode::i32_and as usize] = bitwise::i32_and;
        //     handlers[Opcode::i32_or as usize] = bitwise::i32_or;
        //     handlers[Opcode::i32_xor as usize] = bitwise::i32_xor;
        //     handlers[Opcode::i32_not as usize] = bitwise::i32_not;
        //     handlers[Opcode::i32_leading_zeros as usize] = bitwise::i32_leading_zeros;
        //     handlers[Opcode::i32_leading_ones as usize] = bitwise::i32_leading_ones;
        //     handlers[Opcode::i32_trailing_zeros as usize] = bitwise::i32_trailing_zeros;
        //     handlers[Opcode::i32_count_ones as usize] = bitwise::i32_count_ones;
        //     handlers[Opcode::i32_shift_left as usize] = bitwise::i32_shift_left;
        //     handlers[Opcode::i32_shift_right_s as usize] = bitwise::i32_shift_right_s;
        //     handlers[Opcode::i32_shift_right_u as usize] = bitwise::i32_shift_right_u;
        //     handlers[Opcode::i32_rotate_left as usize] = bitwise::i32_rotate_left;
        //     handlers[Opcode::i32_rotate_right as usize] = bitwise::i32_rotate_right;
        //     handlers[Opcode::i64_and as usize] = bitwise::i64_and;
        //     handlers[Opcode::i64_or as usize] = bitwise::i64_or;
        //     handlers[Opcode::i64_xor as usize] = bitwise::i64_xor;
        //     handlers[Opcode::i64_not as usize] = bitwise::i64_not;
        //     handlers[Opcode::i64_leading_zeros as usize] = bitwise::i64_leading_zeros;
        //     handlers[Opcode::i64_leading_ones as usize] = bitwise::i64_leading_ones;
        //     handlers[Opcode::i64_trailing_zeros as usize] = bitwise::i64_trailing_zeros;
        //     handlers[Opcode::i64_count_ones as usize] = bitwise::i64_count_ones;
        //     handlers[Opcode::i64_shift_left as usize] = bitwise::i64_shift_left;
        //     handlers[Opcode::i64_shift_right_s as usize] = bitwise::i64_shift_right_s;
        //     handlers[Opcode::i64_shift_right_u as usize] = bitwise::i64_shift_right_u;
        //     handlers[Opcode::i64_rotate_left as usize] = bitwise::i64_rotate_left;
        //     handlers[Opcode::i64_rotate_right as usize] = bitwise::i64_rotate_right;
        //
        //     // math
        //     handlers[Opcode::i32_abs as usize] = math::i32_abs;
        //     handlers[Opcode::i32_neg as usize] = math::i32_neg;
        //     handlers[Opcode::i64_abs as usize] = math::i64_abs;
        //     handlers[Opcode::i64_neg as usize] = math::i64_neg;
        //     //
        //     handlers[Opcode::f32_abs as usize] = math::f32_abs;
        //     handlers[Opcode::f32_neg as usize] = math::f32_neg;
        //     handlers[Opcode::f32_ceil as usize] = math::f32_ceil;
        //     handlers[Opcode::f32_floor as usize] = math::f32_floor;
        //     handlers[Opcode::f32_round_half_away_from_zero as usize] =
        //         math::f32_round_half_away_from_zero;
        //     handlers[Opcode::f32_round_half_to_even as usize] = math::f32_round_half_to_even;
        //     handlers[Opcode::f32_trunc as usize] = math::f32_trunc;
        //     handlers[Opcode::f32_fract as usize] = math::f32_fract;
        //     handlers[Opcode::f32_sqrt as usize] = math::f32_sqrt;
        //     handlers[Opcode::f32_cbrt as usize] = math::f32_cbrt;
        //     handlers[Opcode::f32_exp as usize] = math::f32_exp;
        //     handlers[Opcode::f32_exp2 as usize] = math::f32_exp2;
        //     handlers[Opcode::f32_ln as usize] = math::f32_ln;
        //     handlers[Opcode::f32_log2 as usize] = math::f32_log2;
        //     handlers[Opcode::f32_log10 as usize] = math::f32_log10;
        //     handlers[Opcode::f32_sin as usize] = math::f32_sin;
        //     handlers[Opcode::f32_cos as usize] = math::f32_cos;
        //     handlers[Opcode::f32_tan as usize] = math::f32_tan;
        //     handlers[Opcode::f32_asin as usize] = math::f32_asin;
        //     handlers[Opcode::f32_acos as usize] = math::f32_acos;
        //     handlers[Opcode::f32_atan as usize] = math::f32_atan;
        //     handlers[Opcode::f32_copysign as usize] = math::f32_copysign;
        //     handlers[Opcode::f32_pow as usize] = math::f32_pow;
        //     handlers[Opcode::f32_log as usize] = math::f32_log;
        //     handlers[Opcode::f32_min as usize] = math::f32_min;
        //     handlers[Opcode::f32_max as usize] = math::f32_max;
        //
        //     handlers[Opcode::f64_abs as usize] = math::f64_abs;
        //     handlers[Opcode::f64_neg as usize] = math::f64_neg;
        //     handlers[Opcode::f64_ceil as usize] = math::f64_ceil;
        //     handlers[Opcode::f64_floor as usize] = math::f64_floor;
        //     handlers[Opcode::f64_round_half_away_from_zero as usize] =
        //         math::f64_round_half_away_from_zero;
        //     handlers[Opcode::f64_round_half_to_even as usize] = math::f64_round_half_to_even;
        //     handlers[Opcode::f64_trunc as usize] = math::f64_trunc;
        //     handlers[Opcode::f64_fract as usize] = math::f64_fract;
        //     handlers[Opcode::f64_sqrt as usize] = math::f64_sqrt;
        //     handlers[Opcode::f64_cbrt as usize] = math::f64_cbrt;
        //     handlers[Opcode::f64_exp as usize] = math::f64_exp;
        //     handlers[Opcode::f64_exp2 as usize] = math::f64_exp2;
        //     handlers[Opcode::f64_ln as usize] = math::f64_ln;
        //     handlers[Opcode::f64_log2 as usize] = math::f64_log2;
        //     handlers[Opcode::f64_log10 as usize] = math::f64_log10;
        //     handlers[Opcode::f64_sin as usize] = math::f64_sin;
        //     handlers[Opcode::f64_cos as usize] = math::f64_cos;
        //     handlers[Opcode::f64_tan as usize] = math::f64_tan;
        //     handlers[Opcode::f64_asin as usize] = math::f64_asin;
        //     handlers[Opcode::f64_acos as usize] = math::f64_acos;
        //     handlers[Opcode::f64_atan as usize] = math::f64_atan;
        //     handlers[Opcode::f64_copysign as usize] = math::f64_copysign;
        //     handlers[Opcode::f64_pow as usize] = math::f64_pow;
        //     handlers[Opcode::f64_log as usize] = math::f64_log;
        //     handlers[Opcode::f64_min as usize] = math::f64_min;
        //     handlers[Opcode::f64_max as usize] = math::f64_max;
        //
        // control flow
        handlers[Opcode::end as usize] = control_flow::end;
        handlers[Opcode::block as usize] = control_flow::block;
        handlers[Opcode::break_ as usize] = control_flow::break_;
        handlers[Opcode::recur as usize] = control_flow::recur;
        handlers[Opcode::block_alt as usize] = control_flow::block_alt;
        handlers[Opcode::block_nez as usize] = control_flow::block_nez;
        handlers[Opcode::break_nez as usize] = control_flow::break_nez;
        handlers[Opcode::recur_nez as usize] = control_flow::recur_nez;
        //
        //     // function call
        //     handlers[Opcode::call as usize] = funcall::call;
        //     handlers[Opcode::dyncall as usize] = funcall::dyncall;
        //     handlers[Opcode::envcall as usize] = envcall::envcall;
        //     handlers[Opcode::syscall as usize] = syscall::syscall;
        //     handlers[Opcode::extcall as usize] = extcall::extcall;
        //
        //     // host
        //     handlers[Opcode::panic as usize] = host::panic;
        //     handlers[Opcode::unreachable as usize] = host::unreachable;
        //     handlers[Opcode::debug as usize] = host::debug;
        //     handlers[Opcode::host_addr_local as usize] = host::host_addr_local;
        //     handlers[Opcode::host_addr_local_offset as usize] = host::host_addr_local_offset;
        //     handlers[Opcode::host_addr_data as usize] = host::host_addr_data;
        //     handlers[Opcode::host_addr_data_offset as usize] = host::host_addr_data_offset;
        //     handlers[Opcode::host_addr_heap as usize] = host::host_addr_heap;
        //     handlers[Opcode::host_addr_function as usize] = host::host_addr_function;
        //     handlers[Opcode::host_copy_heap_to_memory as usize] = host::host_copy_heap_to_memory;
        //     handlers[Opcode::host_copy_memory_to_heap as usize] = host::host_copy_memory_to_heap;
        //     handlers[Opcode::host_memory_copy as usize] = host::host_memory_copy;

        Handler { handlers }
    }
}
