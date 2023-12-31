// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use crate::opcode::Opcode;

impl Opcode {
    pub fn get_name(&self) -> &'static str {
        match self {
            Opcode::nop => "nop",
            Opcode::zero => "zero",
            Opcode::drop => "drop",
            // Opcode::duplicate => "duplicate",
            // Opcode::swap => "swap",
            Opcode::select_nez => "select_nez",
            //
            Opcode::i32_imm => "i32.imm",
            Opcode::i64_imm => "i64.imm",
            Opcode::f32_imm => "f32.imm",
            Opcode::f64_imm => "f64.imm",
            //
            Opcode::local_load64_i64 => "local.load64_i64",
            Opcode::local_load32_i32 => "local.load32_i32",
            Opcode::local_load32_i16_s => "local.load32_i16_s",
            Opcode::local_load32_i16_u => "local.load32_i16_u",
            Opcode::local_load32_i8_s => "local.load32_i8_s",
            Opcode::local_load32_i8_u => "local.load32_i8_u",
            Opcode::local_load64_f64 => "local.load64_f64",
            Opcode::local_load32_f32 => "local.load32_f32",
            Opcode::local_store64 => "local.store64",
            Opcode::local_store32 => "local.store32",
            Opcode::local_store16 => "local.store16",
            Opcode::local_store8 => "local.store8",
            //
            Opcode::local_offset_load64_i64 => "local.offset_load64_i64",
            Opcode::local_offset_load32_i32 => "local.offset_load32_i32",
            Opcode::local_offset_load32_i16_s => "local.offset_load32_i16_s",
            Opcode::local_offset_load32_i16_u => "local.offset_load32_i16_u",
            Opcode::local_offset_load32_i8_s => "local.offset_load32_i8_s",
            Opcode::local_offset_load32_i8_u => "local.offset_load32_i8_u",
            Opcode::local_offset_load64_f64 => "local.offset_load64_f64",
            Opcode::local_offset_load32_f32 => "local.offset_load32_f32",
            Opcode::local_offset_store64 => "local.offset_store64",
            Opcode::local_offset_store32 => "local.offset_store32",
            Opcode::local_offset_store16 => "local.offset_store16",
            Opcode::local_offset_store8 => "local.offset_store8",
            //
            Opcode::data_load64_i64 => "data.load64_i64",
            Opcode::data_load32_i32 => "data.load32_i32",
            Opcode::data_load32_i16_s => "data.load32_i16_s",
            Opcode::data_load32_i16_u => "data.load32_i16_u",
            Opcode::data_load32_i8_s => "data.load32_i8_s",
            Opcode::data_load32_i8_u => "data.load32_i8_u",
            Opcode::data_load64_f64 => "data.load64_f64",
            Opcode::data_load32_f32 => "data.load32_f32",
            Opcode::data_store64 => "data.store64",
            Opcode::data_store32 => "data.store32",
            Opcode::data_store16 => "data.store16",
            Opcode::data_store8 => "data.store8",
            //
            Opcode::data_offset_load64_i64 => "data.offset_load64_i64",
            Opcode::data_offset_load32_i32 => "data.offset_load32_i32",
            Opcode::data_offset_load32_i16_s => "data.offset_load32_i16_s",
            Opcode::data_offset_load32_i16_u => "data.offset_load32_i16_u",
            Opcode::data_offset_load32_i8_s => "data.offset_load32_i8_s",
            Opcode::data_offset_load32_i8_u => "data.offset_load32_i8_u",
            Opcode::data_offset_load64_f64 => "data.offset_load64_f64",
            Opcode::data_offset_load32_f32 => "data.offset_load32_f32",
            Opcode::data_offset_store64 => "data.offset_store64",
            Opcode::data_offset_store32 => "data.offset_store32",
            Opcode::data_offset_store16 => "data.offset_store16",
            Opcode::data_offset_store8 => "data.offset_store8",
            //
            Opcode::heap_load64_i64 => "heap.load64_i64",
            Opcode::heap_load32_i32 => "heap.load32_i32",
            Opcode::heap_load32_i16_s => "heap.load32_i16_s",
            Opcode::heap_load32_i16_u => "heap.load32_i16_u",
            Opcode::heap_load32_i8_s => "heap.load32_i8_s",
            Opcode::heap_load32_i8_u => "heap.load32_i8_u",
            Opcode::heap_load64_f64 => "heap.load64_f64",
            Opcode::heap_load32_f32 => "heap.load32_f32",
            Opcode::heap_store64 => "heap.store64",
            Opcode::heap_store32 => "heap.store32",
            Opcode::heap_store16 => "heap.store16",
            Opcode::heap_store8 => "heap.store8",
            //
            Opcode::heap_fill => "heap.fill",
            Opcode::heap_copy => "heap.copy",
            Opcode::heap_capacity => "heap.capacity",
            Opcode::heap_resize => "heap.resize",
            //
            Opcode::i32_truncate_i64 => "i32.truncate_i64",
            Opcode::i64_extend_i32_s => "i64.extend_i32_s",
            Opcode::i64_extend_i32_u => "i64.extend_i32_u",
            Opcode::f32_demote_f64 => "f32.demote_f64",
            Opcode::f64_promote_f32 => "f64.promote_f32",
            //
            Opcode::i32_convert_f32_s => "i32.convert_f32_s",
            Opcode::i32_convert_f32_u => "i32.convert_f32_u",
            Opcode::i32_convert_f64_s => "i32.convert_f64_s",
            Opcode::i32_convert_f64_u => "i32.convert_f64_u",
            Opcode::i64_convert_f32_s => "i64.convert_f32_s",
            Opcode::i64_convert_f32_u => "i64.convert_f32_u",
            Opcode::i64_convert_f64_s => "i64.convert_f64_s",
            Opcode::i64_convert_f64_u => "i64.convert_f64_u",
            //
            Opcode::f32_convert_i32_s => "f32.convert_i32_s",
            Opcode::f32_convert_i32_u => "f32.convert_i32_u",
            Opcode::f32_convert_i64_s => "f32.convert_i64_s",
            Opcode::f32_convert_i64_u => "f32.convert_i64_u",
            Opcode::f64_convert_i32_s => "f64.convert_i32_s",
            Opcode::f64_convert_i32_u => "f64.convert_i32_u",
            Opcode::f64_convert_i64_s => "f64.convert_i64_s",
            Opcode::f64_convert_i64_u => "f64.convert_i64_u",
            //
            Opcode::i32_eqz => "i32.eqz",
            Opcode::i32_nez => "i32.nez",
            Opcode::i32_eq => "i32.eq",
            Opcode::i32_ne => "i32.ne",
            Opcode::i32_lt_s => "i32.lt_s",
            Opcode::i32_lt_u => "i32.lt_u",
            Opcode::i32_gt_s => "i32.gt_s",
            Opcode::i32_gt_u => "i32.gt_u",
            Opcode::i32_le_s => "i32.le_s",
            Opcode::i32_le_u => "i32.le_u",
            Opcode::i32_ge_s => "i32.ge_s",
            Opcode::i32_ge_u => "i32.ge_u",
            Opcode::i64_eqz => "i64.eqz",
            Opcode::i64_nez => "i64.nez",
            Opcode::i64_eq => "i64.eq",
            Opcode::i64_ne => "i64.ne",
            Opcode::i64_lt_s => "i64.lt_s",
            Opcode::i64_lt_u => "i64.lt_u",
            Opcode::i64_gt_s => "i64.gt_s",
            Opcode::i64_gt_u => "i64.gt_u",
            Opcode::i64_le_s => "i64.le_s",
            Opcode::i64_le_u => "i64.le_u",
            Opcode::i64_ge_s => "i64.ge_s",
            Opcode::i64_ge_u => "i64.ge_u",
            Opcode::f32_eq => "f32.eq",
            Opcode::f32_ne => "f32.ne",
            Opcode::f32_lt => "f32.lt",
            Opcode::f32_gt => "f32.gt",
            Opcode::f32_le => "f32.le",
            Opcode::f32_ge => "f32.ge",
            Opcode::f64_eq => "f64.eq",
            Opcode::f64_ne => "f64.ne",
            Opcode::f64_lt => "f64.lt",
            Opcode::f64_gt => "f64.gt",
            Opcode::f64_le => "f64.le",
            Opcode::f64_ge => "f64.ge",
            //
            Opcode::i32_add => "i32.add",
            Opcode::i32_sub => "i32.sub",
            Opcode::i32_mul => "i32.mul",
            Opcode::i32_div_s => "i32.div_s",
            Opcode::i32_div_u => "i32.div_u",
            Opcode::i32_rem_s => "i32.rem_s",
            Opcode::i32_rem_u => "i32.rem_u",
            Opcode::i32_inc => "i32.inc",
            Opcode::i32_dec => "i32.dec",
            Opcode::i64_add => "i64.add",
            Opcode::i64_sub => "i64.sub",
            Opcode::i64_mul => "i64.mul",
            Opcode::i64_div_s => "i64.div_s",
            Opcode::i64_div_u => "i64.div_u",
            Opcode::i64_rem_s => "i64.rem_s",
            Opcode::i64_rem_u => "i64.rem_u",
            Opcode::i64_inc => "i64.inc",
            Opcode::i64_dec => "i64.dec",
            Opcode::f32_add => "f32.add",
            Opcode::f32_sub => "f32.sub",
            Opcode::f32_mul => "f32.mul",
            Opcode::f32_div => "f32.div",
            Opcode::f64_add => "f64.add",
            Opcode::f64_sub => "f64.sub",
            Opcode::f64_mul => "f64.mul",
            Opcode::f64_div => "f64.div",
            //
            Opcode::i32_and => "i32.and",
            Opcode::i32_or => "i32.or",
            Opcode::i32_xor => "i32.xor",
            Opcode::i32_not => "i32.not",
            Opcode::i32_leading_zeros => "i32.leading_zeros",
            Opcode::i32_leading_ones => "i32.leading_ones",
            Opcode::i32_trailing_zeros => "i32.trailing_zeros",
            Opcode::i32_count_ones => "i32.count_ones",
            Opcode::i32_shift_left => "i32.shift_left",
            Opcode::i32_shift_right_s => "i32.shift_right_s",
            Opcode::i32_shift_right_u => "i32.shift_right_u",
            Opcode::i32_rotate_left => "i32.rotate_left",
            Opcode::i32_rotate_right => "i32.rotate_right",
            Opcode::i64_and => "i64.and",
            Opcode::i64_or => "i64.or",
            Opcode::i64_xor => "i64.xor",
            Opcode::i64_not => "i64.not",
            Opcode::i64_leading_zeros => "i64.leading_zeros",
            Opcode::i64_leading_ones => "i64.leading_ones",
            Opcode::i64_trailing_zeros => "i64.trailing_zeros",
            Opcode::i64_count_ones => "i64.count_ones",
            Opcode::i64_shift_left => "i64.shift_left",
            Opcode::i64_shift_right_s => "i64.shift_right_s",
            Opcode::i64_shift_right_u => "i64.shift_right_u",
            Opcode::i64_rotate_left => "i64.rotate_left",
            Opcode::i64_rotate_right => "i64.rotate_right",
            //
            Opcode::i32_abs => "i32.abs",
            Opcode::i32_neg => "i32.neg",
            Opcode::i64_abs => "i64.abs",
            Opcode::i64_neg => "i64.neg",
            //
            Opcode::f32_abs => "f32.abs",
            Opcode::f32_neg => "f32.neg",
            Opcode::f32_ceil => "f32.ceil",
            Opcode::f32_floor => "f32.floor",
            Opcode::f32_round_half_away_from_zero => "f32.round_half_away_from_zero",
            Opcode::f32_round_half_to_even => "f32.round_half_to_even",
            Opcode::f32_trunc => "f32.trunc",
            Opcode::f32_fract => "f32.fract",
            Opcode::f32_sqrt => "f32.sqrt",
            Opcode::f32_cbrt => "f32.cbrt",
            Opcode::f32_exp => "f32.exp",
            Opcode::f32_exp2 => "f32.exp2",
            Opcode::f32_ln => "f32.ln",
            Opcode::f32_log2 => "f32.log2",
            Opcode::f32_log10 => "f32.log10",
            Opcode::f32_sin => "f32.sin",
            Opcode::f32_cos => "f32.cos",
            Opcode::f32_tan => "f32.tan",
            Opcode::f32_asin => "f32.asin",
            Opcode::f32_acos => "f32.acos",
            Opcode::f32_atan => "f32.atan",
            Opcode::f32_copysign => "f32.copysign",
            Opcode::f32_pow => "f32.pow",
            Opcode::f32_log => "f32.log",
            Opcode::f32_min => "f32.min",
            Opcode::f32_max => "f32.max",
            //
            Opcode::f64_abs => "f64.abs",
            Opcode::f64_neg => "f64.neg",
            Opcode::f64_ceil => "f64.ceil",
            Opcode::f64_floor => "f64.floor",
            Opcode::f64_round_half_away_from_zero => "f64.round_half_away_from_zero",
            Opcode::f64_round_half_to_even => "f64.round_half_to_even",
            Opcode::f64_trunc => "f64.trunc",
            Opcode::f64_fract => "f64.fract",
            Opcode::f64_sqrt => "f64.sqrt",
            Opcode::f64_cbrt => "f64.cbrt",
            Opcode::f64_exp => "f64.exp",
            Opcode::f64_exp2 => "f64.exp2",
            Opcode::f64_ln => "f64.ln",
            Opcode::f64_log2 => "f64.log2",
            Opcode::f64_log10 => "f64.log10",
            Opcode::f64_sin => "f64.sin",
            Opcode::f64_cos => "f64.cos",
            Opcode::f64_tan => "f64.tan",
            Opcode::f64_asin => "f64.asin",
            Opcode::f64_acos => "f64.acos",
            Opcode::f64_atan => "f64.atan",
            Opcode::f64_copysign => "f64.copysign",
            Opcode::f64_pow => "f64.pow",
            Opcode::f64_log => "f64.log",
            Opcode::f64_min => "f64.min",
            Opcode::f64_max => "f64.max",
            //
            Opcode::end => "end",
            Opcode::block => "block",
            Opcode::break_ => "break",
            Opcode::recur => "recur",
            Opcode::block_alt => "block_alt",
            Opcode::block_nez => "block_nez",
            Opcode::break_nez => "break_nez",
            Opcode::recur_nez => "recur_nez",
            //
            Opcode::call => "call",
            Opcode::dyncall => "dyncall",
            Opcode::envcall => "envcall",
            Opcode::syscall => "syscall",
            Opcode::extcall => "extcall",
            //
            Opcode::panic => "panic",
            Opcode::unreachable => "unreachable",
            Opcode::debug => "debug",
            Opcode::host_addr_local => "host.addr_local",
            Opcode::host_addr_local_offset => "host.addr_local_offset",
            Opcode::host_addr_data => "host.addr_data",
            Opcode::host_addr_data_offset => "host.addr_data_offset",
            Opcode::host_addr_heap => "host.addr_heap",
            Opcode::host_copy_heap_to_memory => "host.copy_heap_to_memory",
            Opcode::host_copy_memory_to_heap => "host.copy_memory_to_heap",
            Opcode::host_memory_copy => "host.memory_copy",
            Opcode::host_addr_function => "host.addr_function",
        }
    }
}
