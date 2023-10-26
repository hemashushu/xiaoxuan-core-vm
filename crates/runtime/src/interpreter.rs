// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::sync::Once;

use ancvm_binary::utils::format_bytecodes;
use ancvm_program::thread_context::{ProgramCounter, ThreadContext};
use ancvm_types::{
    opcode::{Opcode, MAX_OPCODE_NUMBER},
    DataType, ForeignValue, OPERAND_SIZE_IN_BYTES,
};

use crate::{
    ecall::{self, init_ecall_handlers},
    InterpreterError, InterpreterErrorType,
};

type InterpretFunc = fn(&mut ThreadContext) -> InterpretResult;

mod arithmetic;
mod bitwise;
mod comparison;
mod control_flow;
mod conversion;
mod data;
mod function_call;
mod fundamental;
mod heap;
mod host;
mod local;
mod math;

pub enum InterpretResult {
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
    // param (original_pc: ProgramCounter)
    End(ProgramCounter),

    Panic,

    // pause the interpreter
    // for debug the program or the VM itself
    // param (code: u32)
    Debug(u32),
}

fn unreachable(thread_context: &mut ThreadContext) -> InterpretResult {
    let pc = &thread_context.pc;
    let func_item = &thread_context.program_context.program_modules[pc.module_index]
        .func_section
        .items[pc.function_internal_index];
    let codes = &thread_context.program_context.program_modules[pc.module_index]
        .func_section
        .codes_data
        [func_item.code_offset as usize..(func_item.code_offset + func_item.code_length) as usize];
    let code_text = format_bytecodes(codes);

    unreachable!(
        "Invalid opcode: 0x{:04x}
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

static INIT: Once = Once::new();
static mut INTERPRETERS: [InterpretFunc; MAX_OPCODE_NUMBER] = [unreachable; MAX_OPCODE_NUMBER];

// initilize the instruction interpreters
//
// note:
// ensure this initialization is only called once, to do that, the 3rd party crates
// such as 'lazy_static', 'once_cell' and 'rust-ctor' can be used.
// the same can be done with ''
pub fn init_interpreters() {
    INIT.call_once(|| {
        init_interpreters_internal();
    });
}

fn init_interpreters_internal() {
    // other initializations
    init_ecall_handlers();

    let interpreters = unsafe { &mut INTERPRETERS };

    // fundamental
    interpreters[Opcode::zero as usize] = fundamental::zero;
    interpreters[Opcode::drop as usize] = fundamental::drop_;
    interpreters[Opcode::duplicate as usize] = fundamental::duplicate;
    interpreters[Opcode::swap as usize] = fundamental::swap;
    interpreters[Opcode::select_nez as usize] = fundamental::select_nez;
    interpreters[Opcode::i32_imm as usize] = fundamental::i32_imm;
    interpreters[Opcode::i64_imm as usize] = fundamental::i64_imm;
    interpreters[Opcode::f32_imm as usize] = fundamental::f32_imm;
    interpreters[Opcode::f64_imm as usize] = fundamental::f64_imm;

    // local variables
    interpreters[Opcode::local_load as usize] = local::local_load;
    interpreters[Opcode::local_load32 as usize] = local::local_load32;
    interpreters[Opcode::local_load32_i16_s as usize] = local::local_load32_i16_s;
    interpreters[Opcode::local_load32_i16_u as usize] = local::local_load32_i16_u;
    interpreters[Opcode::local_load32_i8_s as usize] = local::local_load32_i8_s;
    interpreters[Opcode::local_load32_i8_u as usize] = local::local_load32_i8_u;
    interpreters[Opcode::local_load32_f32 as usize] = local::local_load32_f32;
    interpreters[Opcode::local_load_f64 as usize] = local::local_load_f64;
    interpreters[Opcode::local_store as usize] = local::local_store;
    interpreters[Opcode::local_store32 as usize] = local::local_store32;
    interpreters[Opcode::local_store16 as usize] = local::local_store16;
    interpreters[Opcode::local_store8 as usize] = local::local_store8;

    interpreters[Opcode::local_long_load as usize] = local::local_long_load;
    interpreters[Opcode::local_long_load32 as usize] = local::local_long_load32;
    interpreters[Opcode::local_long_load32_i16_s as usize] = local::local_long_load32_i16_s;
    interpreters[Opcode::local_long_load32_i16_u as usize] = local::local_long_load32_i16_u;
    interpreters[Opcode::local_long_load32_i8_s as usize] = local::local_long_load32_i8_s;
    interpreters[Opcode::local_long_load32_i8_u as usize] = local::local_long_load32_i8_u;
    interpreters[Opcode::local_long_load32_f32 as usize] = local::local_long_load32_f32;
    interpreters[Opcode::local_long_load_f64 as usize] = local::local_long_load_f64;
    interpreters[Opcode::local_long_store as usize] = local::local_long_store;
    interpreters[Opcode::local_long_store32 as usize] = local::local_long_store32;
    interpreters[Opcode::local_long_store16 as usize] = local::local_long_store16;
    interpreters[Opcode::local_long_store8 as usize] = local::local_long_store8;

    // data sections
    interpreters[Opcode::data_load as usize] = data::data_load;
    interpreters[Opcode::data_load32 as usize] = data::data_load32;
    interpreters[Opcode::data_load32_i16_s as usize] = data::data_load32_i16_s;
    interpreters[Opcode::data_load32_i16_u as usize] = data::data_load32_i16_u;
    interpreters[Opcode::data_load32_i8_s as usize] = data::data_load32_i8_s;
    interpreters[Opcode::data_load32_i8_u as usize] = data::data_load32_i8_u;
    interpreters[Opcode::data_load32_f32 as usize] = data::data_load32_f32;
    interpreters[Opcode::data_load_f64 as usize] = data::data_load_f64;
    interpreters[Opcode::data_store as usize] = data::data_store;
    interpreters[Opcode::data_store32 as usize] = data::data_store32;
    interpreters[Opcode::data_store16 as usize] = data::data_store16;
    interpreters[Opcode::data_store8 as usize] = data::data_store8;

    interpreters[Opcode::data_long_load as usize] = data::data_long_load;
    interpreters[Opcode::data_long_load32 as usize] = data::data_long_load32;
    interpreters[Opcode::data_long_load32_i16_s as usize] = data::data_long_load32_i16_s;
    interpreters[Opcode::data_long_load32_i16_u as usize] = data::data_long_load32_i16_u;
    interpreters[Opcode::data_long_load32_i8_s as usize] = data::data_long_load32_i8_s;
    interpreters[Opcode::data_long_load32_i8_u as usize] = data::data_long_load32_i8_u;
    interpreters[Opcode::data_long_load32_f32 as usize] = data::data_long_load32_f32;
    interpreters[Opcode::data_long_load_f64 as usize] = data::data_long_load_f64;
    interpreters[Opcode::data_long_store as usize] = data::data_long_store;
    interpreters[Opcode::data_long_store32 as usize] = data::data_long_store32;
    interpreters[Opcode::data_long_store16 as usize] = data::data_long_store16;
    interpreters[Opcode::data_long_store8 as usize] = data::data_long_store8;

    // heap
    interpreters[Opcode::heap_load as usize] = heap::heap_load;
    interpreters[Opcode::heap_load32 as usize] = heap::heap_load32;
    interpreters[Opcode::heap_load32_i16_s as usize] = heap::heap_load32_i16_s;
    interpreters[Opcode::heap_load32_i16_u as usize] = heap::heap_load32_i16_u;
    interpreters[Opcode::heap_load32_i8_s as usize] = heap::heap_load32_i8_s;
    interpreters[Opcode::heap_load32_i8_u as usize] = heap::heap_load32_i8_u;
    interpreters[Opcode::heap_load32_f32 as usize] = heap::heap_load32_f32;
    interpreters[Opcode::heap_load_f64 as usize] = heap::heap_load_f64;
    interpreters[Opcode::heap_store as usize] = heap::heap_store;
    interpreters[Opcode::heap_store32 as usize] = heap::heap_store32;
    interpreters[Opcode::heap_store16 as usize] = heap::heap_store16;
    interpreters[Opcode::heap_store8 as usize] = heap::heap_store8;

    // conversion
    interpreters[Opcode::i32_trunc_i64 as usize] = conversion::i32_trunc_i64;
    interpreters[Opcode::i64_extend_i32_s as usize] = conversion::i64_extend_i32_s;
    interpreters[Opcode::i64_extend_i32_u as usize] = conversion::i64_extend_i32_u;
    interpreters[Opcode::f32_demote_f64 as usize] = conversion::f32_demote_f64;
    interpreters[Opcode::f64_promote_f32 as usize] = conversion::f64_promote_f32;
    interpreters[Opcode::i32_convert_f32_s as usize] = conversion::i32_convert_f32_s;
    interpreters[Opcode::i32_convert_f32_u as usize] = conversion::i32_convert_f32_u;
    interpreters[Opcode::i32_convert_f64_s as usize] = conversion::i32_convert_f64_s;
    interpreters[Opcode::i32_convert_f64_u as usize] = conversion::i32_convert_f64_u;
    interpreters[Opcode::i64_convert_f32_s as usize] = conversion::i64_convert_f32_s;
    interpreters[Opcode::i64_convert_f32_u as usize] = conversion::i64_convert_f32_u;
    interpreters[Opcode::i64_convert_f64_s as usize] = conversion::i64_convert_f64_s;
    interpreters[Opcode::i64_convert_f64_u as usize] = conversion::i64_convert_f64_u;
    interpreters[Opcode::f32_convert_i32_s as usize] = conversion::f32_convert_i32_s;
    interpreters[Opcode::f32_convert_i32_u as usize] = conversion::f32_convert_i32_u;
    interpreters[Opcode::f32_convert_i64_s as usize] = conversion::f32_convert_i64_s;
    interpreters[Opcode::f32_convert_i64_u as usize] = conversion::f32_convert_i64_u;
    interpreters[Opcode::f64_convert_i32_s as usize] = conversion::f64_convert_i32_s;
    interpreters[Opcode::f64_convert_i32_u as usize] = conversion::f64_convert_i32_u;
    interpreters[Opcode::f64_convert_i64_s as usize] = conversion::f64_convert_i64_s;
    interpreters[Opcode::f64_convert_i64_u as usize] = conversion::f64_convert_i64_u;

    // comparison
    interpreters[Opcode::i32_eqz as usize] = comparison::i32_eqz;
    interpreters[Opcode::i32_eq as usize] = comparison::i32_eq;
    interpreters[Opcode::i32_nez as usize] = comparison::i32_nez;
    interpreters[Opcode::i32_ne as usize] = comparison::i32_ne;
    interpreters[Opcode::i32_lt_s as usize] = comparison::i32_lt_s;
    interpreters[Opcode::i32_lt_u as usize] = comparison::i32_lt_u;
    interpreters[Opcode::i32_gt_s as usize] = comparison::i32_gt_s;
    interpreters[Opcode::i32_gt_u as usize] = comparison::i32_gt_u;
    interpreters[Opcode::i32_le_s as usize] = comparison::i32_le_s;
    interpreters[Opcode::i32_le_u as usize] = comparison::i32_le_u;
    interpreters[Opcode::i32_ge_s as usize] = comparison::i32_ge_s;
    interpreters[Opcode::i32_ge_u as usize] = comparison::i32_ge_u;
    interpreters[Opcode::i64_eqz as usize] = comparison::i64_eqz;
    interpreters[Opcode::i64_eq as usize] = comparison::i64_eq;
    interpreters[Opcode::i64_nez as usize] = comparison::i64_nez;
    interpreters[Opcode::i64_ne as usize] = comparison::i64_ne;
    interpreters[Opcode::i64_lt_s as usize] = comparison::i64_lt_s;
    interpreters[Opcode::i64_lt_u as usize] = comparison::i64_lt_u;
    interpreters[Opcode::i64_gt_s as usize] = comparison::i64_gt_s;
    interpreters[Opcode::i64_gt_u as usize] = comparison::i64_gt_u;
    interpreters[Opcode::i64_le_s as usize] = comparison::i64_le_s;
    interpreters[Opcode::i64_le_u as usize] = comparison::i64_le_u;
    interpreters[Opcode::i64_ge_s as usize] = comparison::i64_ge_s;
    interpreters[Opcode::i64_ge_u as usize] = comparison::i64_ge_u;
    interpreters[Opcode::f32_eq as usize] = comparison::f32_eq;
    interpreters[Opcode::f32_ne as usize] = comparison::f32_ne;
    interpreters[Opcode::f32_lt as usize] = comparison::f32_lt;
    interpreters[Opcode::f32_gt as usize] = comparison::f32_gt;
    interpreters[Opcode::f32_le as usize] = comparison::f32_le;
    interpreters[Opcode::f32_ge as usize] = comparison::f32_ge;
    interpreters[Opcode::f64_eq as usize] = comparison::f64_eq;
    interpreters[Opcode::f64_ne as usize] = comparison::f64_ne;
    interpreters[Opcode::f64_lt as usize] = comparison::f64_lt;
    interpreters[Opcode::f64_gt as usize] = comparison::f64_gt;
    interpreters[Opcode::f64_le as usize] = comparison::f64_le;
    interpreters[Opcode::f64_ge as usize] = comparison::f64_ge;

    // arithmetic
    interpreters[Opcode::i32_add as usize] = arithmetic::i32_add;
    interpreters[Opcode::i32_sub as usize] = arithmetic::i32_sub;
    interpreters[Opcode::i32_mul as usize] = arithmetic::i32_mul;
    interpreters[Opcode::i32_div_s as usize] = arithmetic::i32_div_s;
    interpreters[Opcode::i32_div_u as usize] = arithmetic::i32_div_u;
    interpreters[Opcode::i32_rem_s as usize] = arithmetic::i32_rem_s;
    interpreters[Opcode::i32_rem_u as usize] = arithmetic::i32_rem_u;
    interpreters[Opcode::i32_inc as usize] = arithmetic::i32_inc;
    interpreters[Opcode::i32_dec as usize] = arithmetic::i32_dec;
    interpreters[Opcode::i64_add as usize] = arithmetic::i64_add;
    interpreters[Opcode::i64_sub as usize] = arithmetic::i64_sub;
    interpreters[Opcode::i64_mul as usize] = arithmetic::i64_mul;
    interpreters[Opcode::i64_div_s as usize] = arithmetic::i64_div_s;
    interpreters[Opcode::i64_div_u as usize] = arithmetic::i64_div_u;
    interpreters[Opcode::i64_rem_s as usize] = arithmetic::i64_rem_s;
    interpreters[Opcode::i64_rem_u as usize] = arithmetic::i64_rem_u;
    interpreters[Opcode::i64_inc as usize] = arithmetic::i64_inc;
    interpreters[Opcode::i64_dec as usize] = arithmetic::i64_dec;
    interpreters[Opcode::f32_add as usize] = arithmetic::f32_add;
    interpreters[Opcode::f32_sub as usize] = arithmetic::f32_sub;
    interpreters[Opcode::f32_mul as usize] = arithmetic::f32_mul;
    interpreters[Opcode::f32_div as usize] = arithmetic::f32_div;
    interpreters[Opcode::f64_add as usize] = arithmetic::f64_add;
    interpreters[Opcode::f64_sub as usize] = arithmetic::f64_sub;
    interpreters[Opcode::f64_mul as usize] = arithmetic::f64_mul;
    interpreters[Opcode::f64_div as usize] = arithmetic::f64_div;

    // bitwise
    interpreters[Opcode::i32_and as usize] = bitwise::i32_and;
    interpreters[Opcode::i32_or as usize] = bitwise::i32_or;
    interpreters[Opcode::i32_xor as usize] = bitwise::i32_xor;
    interpreters[Opcode::i32_not as usize] = bitwise::i32_not;
    interpreters[Opcode::i32_leading_zeros as usize] = bitwise::i32_leading_zeros;
    interpreters[Opcode::i32_trailing_zeros as usize] = bitwise::i32_trailing_zeros;
    interpreters[Opcode::i32_count_ones as usize] = bitwise::i32_count_ones;
    interpreters[Opcode::i32_shift_left as usize] = bitwise::i32_shift_left;
    interpreters[Opcode::i32_shift_right_s as usize] = bitwise::i32_shift_right_s;
    interpreters[Opcode::i32_shift_right_u as usize] = bitwise::i32_shift_right_u;
    interpreters[Opcode::i32_rotate_left as usize] = bitwise::i32_rotate_left;
    interpreters[Opcode::i32_rotate_right as usize] = bitwise::i32_rotate_right;
    interpreters[Opcode::i64_and as usize] = bitwise::i64_and;
    interpreters[Opcode::i64_or as usize] = bitwise::i64_or;
    interpreters[Opcode::i64_xor as usize] = bitwise::i64_xor;
    interpreters[Opcode::i64_not as usize] = bitwise::i64_not;
    interpreters[Opcode::i64_leading_zeros as usize] = bitwise::i64_leading_zeros;
    interpreters[Opcode::i64_trailing_zeros as usize] = bitwise::i64_trailing_zeros;
    interpreters[Opcode::i64_count_ones as usize] = bitwise::i64_count_ones;
    interpreters[Opcode::i64_shift_left as usize] = bitwise::i64_shift_left;
    interpreters[Opcode::i64_shift_right_s as usize] = bitwise::i64_shift_right_s;
    interpreters[Opcode::i64_shift_right_u as usize] = bitwise::i64_shift_right_u;
    interpreters[Opcode::i64_rotate_left as usize] = bitwise::i64_rotate_left;
    interpreters[Opcode::i64_rotate_right as usize] = bitwise::i64_rotate_right;

    // math
    interpreters[Opcode::f32_abs as usize] = math::f32_abs;
    interpreters[Opcode::f32_neg as usize] = math::f32_neg;
    interpreters[Opcode::f32_ceil as usize] = math::f32_ceil;
    interpreters[Opcode::f32_floor as usize] = math::f32_floor;
    interpreters[Opcode::f32_round_half_away_from_zero as usize] =
        math::f32_round_half_away_from_zero;
    interpreters[Opcode::f32_trunc as usize] = math::f32_trunc;
    interpreters[Opcode::f32_fract as usize] = math::f32_fract;
    interpreters[Opcode::f32_sqrt as usize] = math::f32_sqrt;
    interpreters[Opcode::f32_cbrt as usize] = math::f32_cbrt;
    interpreters[Opcode::f32_pow as usize] = math::f32_pow;
    interpreters[Opcode::f32_exp as usize] = math::f32_exp;
    interpreters[Opcode::f32_exp2 as usize] = math::f32_exp2;
    interpreters[Opcode::f32_ln as usize] = math::f32_ln;
    interpreters[Opcode::f32_log as usize] = math::f32_log;
    interpreters[Opcode::f32_log2 as usize] = math::f32_log2;
    interpreters[Opcode::f32_log10 as usize] = math::f32_log10;
    interpreters[Opcode::f32_sin as usize] = math::f32_sin;
    interpreters[Opcode::f32_cos as usize] = math::f32_cos;
    interpreters[Opcode::f32_tan as usize] = math::f32_tan;
    interpreters[Opcode::f32_asin as usize] = math::f32_asin;
    interpreters[Opcode::f32_acos as usize] = math::f32_acos;
    interpreters[Opcode::f32_atan as usize] = math::f32_atan;
    interpreters[Opcode::f64_abs as usize] = math::f64_abs;
    interpreters[Opcode::f64_neg as usize] = math::f64_neg;
    interpreters[Opcode::f64_ceil as usize] = math::f64_ceil;
    interpreters[Opcode::f64_floor as usize] = math::f64_floor;
    interpreters[Opcode::f64_round_half_away_from_zero as usize] =
        math::f64_round_half_away_from_zero;
    interpreters[Opcode::f64_trunc as usize] = math::f64_trunc;
    interpreters[Opcode::f64_fract as usize] = math::f64_fract;
    interpreters[Opcode::f64_sqrt as usize] = math::f64_sqrt;
    interpreters[Opcode::f64_cbrt as usize] = math::f64_cbrt;
    interpreters[Opcode::f64_pow as usize] = math::f64_pow;
    interpreters[Opcode::f64_exp as usize] = math::f64_exp;
    interpreters[Opcode::f64_exp2 as usize] = math::f64_exp2;
    interpreters[Opcode::f64_ln as usize] = math::f64_ln;
    interpreters[Opcode::f64_log as usize] = math::f64_log;
    interpreters[Opcode::f64_log2 as usize] = math::f64_log2;
    interpreters[Opcode::f64_log10 as usize] = math::f64_log10;
    interpreters[Opcode::f64_sin as usize] = math::f64_sin;
    interpreters[Opcode::f64_cos as usize] = math::f64_cos;
    interpreters[Opcode::f64_tan as usize] = math::f64_tan;
    interpreters[Opcode::f64_asin as usize] = math::f64_asin;
    interpreters[Opcode::f64_acos as usize] = math::f64_acos;
    interpreters[Opcode::f64_atan as usize] = math::f64_atan;

    // control flow
    interpreters[Opcode::end as usize] = control_flow::end;
    interpreters[Opcode::block as usize] = control_flow::block;
    interpreters[Opcode::break_ as usize] = control_flow::break_;
    interpreters[Opcode::recur as usize] = control_flow::recur;
    interpreters[Opcode::block_alt as usize] = control_flow::block_alt;
    interpreters[Opcode::block_nez as usize] = control_flow::block_nez;
    interpreters[Opcode::break_nez as usize] = control_flow::break_nez;
    interpreters[Opcode::recur_nez as usize] = control_flow::recur_nez;
    interpreters[Opcode::call as usize] = function_call::call;
    interpreters[Opcode::dcall as usize] = function_call::dcall;

    interpreters[Opcode::ecall as usize] = ecall::ecall;

    // machine
    interpreters[Opcode::nop as usize] = host::nop;
    interpreters[Opcode::debug as usize] = host::debug;
    interpreters[Opcode::host_addr_local as usize] = host::host_addr_local;
    interpreters[Opcode::host_addr_local_long as usize] = host::host_addr_local_long;
    interpreters[Opcode::host_addr_data as usize] = host::host_addr_data;
    interpreters[Opcode::host_addr_data_long as usize] = host::host_addr_data_long;
    interpreters[Opcode::host_addr_heap as usize] = host::host_addr_heap;
}

pub fn process_next_instruction(thread_context: &mut ThreadContext) -> InterpretResult {
    let opcode_num = thread_context.get_opcode_num();
    let func = unsafe { &INTERPRETERS[opcode_num as usize] };
    func(thread_context)
}

pub fn process_continuous_instructions(thread_context: &mut ThreadContext) {
    loop {
        let result = process_next_instruction(thread_context);
        match result {
            InterpretResult::Move(relate_offset_in_bytes) => {
                let next_instruction_offset =
                    thread_context.pc.instruction_address as isize + relate_offset_in_bytes;
                thread_context.pc.instruction_address = next_instruction_offset as usize;
            }
            InterpretResult::Jump(return_pc) => {
                thread_context.pc.module_index = return_pc.module_index;
                thread_context.pc.function_internal_index = return_pc.function_internal_index;
                thread_context.pc.instruction_address = return_pc.instruction_address;
            }
            InterpretResult::End(original_pc) => {
                thread_context.pc.module_index = original_pc.module_index;
                thread_context.pc.function_internal_index = original_pc.function_internal_index;
                thread_context.pc.instruction_address = original_pc.instruction_address;

                // break the instruction processing loop
                break;
            }
            InterpretResult::Panic => {
                panic!("VM was terminated by instruction panic.");
            }
            InterpretResult::Debug(code) => {
                panic!("VM was terminated by with code: {}", code);
            }
        }
    }
}

// note:
// 'function public index' includes the imported functions, it equals to
// 'amount of imported functions' + 'function internal index'
pub fn process_function(
    thread_context: &mut ThreadContext,
    module_index: usize,
    func_public_index: usize,
    arguments: &[ForeignValue],
) -> Result<Vec<ForeignValue>, InterpreterError> {
    // reset the statck
    thread_context.stack.reset();

    // find the code start address
    let (target_module_index, function_internal_index) = thread_context
        .get_function_target_module_index_and_internal_index(module_index, func_public_index);
    let (type_index, local_list_index, code_offset, local_variables_allocate_bytes) =
        thread_context
            .get_function_type_and_local_list_index_and_code_offset_and_local_variables_allocate_bytes(
                target_module_index,
                function_internal_index,
            );

    let (params, results) = {
        let pars = thread_context.program_context.program_modules[target_module_index]
            .type_section
            .get_item_params_and_results(type_index);
        (pars.0.to_vec(), pars.1.to_vec())
    };

    // the number of arguments does not match the specified funcion.
    if arguments.len() != params.len() {
        return Err(InterpreterError::new(
            InterpreterErrorType::InvalidFunctionCall,
        ));
    }

    // for simplicity, does not check the data type of arguments for now.

    // push arguments
    // the first value will be first inserted, and placed at the stack bottom:
    //
    // array [0, 1, 2] -> |  2  |
    //                    |  1  |
    //                    |  0  |
    //                    \-----/
    for value in arguments {
        match value {
            ForeignValue::UInt32(value) => thread_context.stack.push_i32_u(*value),
            ForeignValue::UInt64(value) => thread_context.stack.push_i64_u(*value),
            ForeignValue::Float32(value) => thread_context.stack.push_f32(*value),
            ForeignValue::Float64(value) => thread_context.stack.push_f64(*value),
        }
    }

    // create function statck frame
    thread_context.stack.create_frame(
        params.len() as u16,
        results.len() as u16,
        local_list_index as u32,
        local_variables_allocate_bytes,
        Some(ProgramCounter {
            instruction_address: 0,
            function_internal_index: 0,

            // set MSB of 'return module index' to '1' to indicate that it's the END of the
            // current function call.
            module_index: 0x8000_0000,
        }),
    );

    // set new PC
    thread_context.pc.module_index = target_module_index;
    thread_context.pc.function_internal_index = function_internal_index;
    thread_context.pc.instruction_address = code_offset;

    // start processing instructions
    process_continuous_instructions(thread_context);

    // pop the results from the stack
    //
    // the values on the stack top will be poped first and became
    // the LAST element of the array
    //
    // |  2  | -> array [0, 1, 2]
    // |  1  |
    // |  0  |
    // \-----/
    let result_operands = thread_context
        .stack
        .pop_operands_without_bound_check(results.len());
    let result_values = results
        .iter()
        .enumerate()
        .map(|(idx, dt)| match dt {
            DataType::I32 => ForeignValue::UInt32(u32::from_le_bytes(
                result_operands[(idx * OPERAND_SIZE_IN_BYTES)..(idx * OPERAND_SIZE_IN_BYTES + 4)]
                    .try_into()
                    .unwrap(),
            )),
            DataType::I64 => ForeignValue::UInt64(u64::from_le_bytes(
                result_operands[(idx * OPERAND_SIZE_IN_BYTES)..((idx + 1) * OPERAND_SIZE_IN_BYTES)]
                    .try_into()
                    .unwrap(),
            )),
            DataType::F32 => ForeignValue::Float32(f32::from_le_bytes(
                result_operands[(idx * OPERAND_SIZE_IN_BYTES)..(idx * OPERAND_SIZE_IN_BYTES + 4)]
                    .try_into()
                    .unwrap(),
            )),
            DataType::F64 => ForeignValue::Float64(f64::from_le_bytes(
                result_operands[(idx * OPERAND_SIZE_IN_BYTES)..((idx + 1) * OPERAND_SIZE_IN_BYTES)]
                    .try_into()
                    .unwrap(),
            )),
        })
        .collect::<Vec<_>>();

    Ok(result_values)
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn process_bridge_function_call(
    thread_context_ptr: *mut u8,
    target_module_index: usize,
    function_internal_index: usize,
    params_ptr: *const u8,
    results_ptr: *mut u8,
) {
    // params:
    // | 8 bytes | 8 bytes | ... |
    //
    // results:
    // | 8 bytes |

    let thread_context = unsafe { &mut *(thread_context_ptr as *mut ThreadContext) };

    let (type_index, local_list_index, code_offset, local_variables_allocate_bytes) =
        thread_context
            .get_function_type_and_local_list_index_and_code_offset_and_local_variables_allocate_bytes(
                target_module_index,
                function_internal_index,
            );
    let type_item = &thread_context.program_context.program_modules[target_module_index]
        .type_section
        .items[type_index];

    let params_count = type_item.params_count as usize;
    let results_count = type_item.results_count as usize;

    // reset the statck
    thread_context.stack.reset();

    // push arguments
    let stack_push_ptr = thread_context.stack.push_operands_from_memory(params_count);
    unsafe {
        std::ptr::copy(
            params_ptr,
            stack_push_ptr,
            OPERAND_SIZE_IN_BYTES * params_count,
        )
    };

    // create function statck frame
    thread_context.stack.create_frame(
        type_item.params_count,
        type_item.results_count,
        local_list_index as u32,
        local_variables_allocate_bytes,
        Some(ProgramCounter {
            instruction_address: 0,
            function_internal_index: 0,

            // set MSB of 'return module index' to '1' to indicate that it's the END of the
            // current function call.
            module_index: 0x8000_0000,
        }),
    );

    // set new PC
    thread_context.pc.module_index = target_module_index;
    thread_context.pc.function_internal_index = function_internal_index;
    thread_context.pc.instruction_address = code_offset;

    // start processing instructions
    process_continuous_instructions(thread_context);

    // pop the results from the stack
    // note:
    //
    // only 0 or 1 return value is allowed for C function.
    if results_count > 0 {
        let result_operands = thread_context.stack.pop_operands_without_bound_check(1);
        unsafe { std::ptr::copy(result_operands.as_ptr(), results_ptr, OPERAND_SIZE_IN_BYTES) };
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn process_callback_function_call(
    thread_context_ptr: *mut u8,
    target_module_index: usize,
    function_internal_index: usize,
    params_ptr: *const u8,
    results_ptr: *mut u8,
) {
    // params:
    // | 8 bytes | 8 bytes | ... |
    //
    // results:
    // | 8 bytes |

    let thread_context = unsafe { &mut *(thread_context_ptr as *mut ThreadContext) };

    let (type_index, local_list_index, code_offset, local_variables_allocate_bytes) =
        thread_context
            .get_function_type_and_local_list_index_and_code_offset_and_local_variables_allocate_bytes(
                target_module_index,
                function_internal_index,
            );
    let type_item = &thread_context.program_context.program_modules[target_module_index]
        .type_section
        .items[type_index];

    let params_count = type_item.params_count as usize;
    let results_count = type_item.results_count as usize;

    // push arguments
    let stack_push_ptr = thread_context.stack.push_operands_from_memory(params_count);
    unsafe {
        std::ptr::copy(
            params_ptr,
            stack_push_ptr,
            OPERAND_SIZE_IN_BYTES * params_count,
        )
    };

    // store the current PC as return PC
    let ProgramCounter {
        instruction_address: return_instruction_address,
        function_internal_index: return_function_internal_index,
        module_index: return_module_index,
    } = thread_context.pc;

    let return_pc = ProgramCounter {
        instruction_address: return_instruction_address,
        function_internal_index: return_function_internal_index,

        // set MSB of 'return module index' to '1' to indicate that it's the END of the
        // current function call.
        module_index: return_module_index | 0x8000_0000,
    };

    // module M, function A
    //
    // 0x0000 inst_0     callback function     module N, function B
    // 0x0004 inst_1     interrupt
    // 0x0008 inst_2   ----------------------> 0x0000 inst_0
    //     \------------<----------------\     0x0004 inst_1
    //                                   |     0x0008 inst_2
    // InterpretResult::Move(X) --\      ^     0x000c inst_3
    //                            |      |     0x0010 end
    //                            |      |       |
    // 0x000c inst_3   <----------/      \---<---/
    // 0x0010 inst_4
    // 0x0014 inst_5
    // 0x0018 inst_6
    // 0x001c end

    // create function statck frame
    thread_context.stack.create_frame(
        type_item.params_count,
        type_item.results_count,
        local_list_index as u32,
        local_variables_allocate_bytes,
        Some(return_pc),
    );

    // set new PC
    thread_context.pc.module_index = target_module_index;
    thread_context.pc.function_internal_index = function_internal_index;
    thread_context.pc.instruction_address = code_offset;

    // start processing instructions
    process_continuous_instructions(thread_context);

    // pop the results from the stack
    // note:
    //
    // only 0 or 1 return value is allowed for C function.
    if results_count > 0 {
        let result_operands = thread_context.stack.pop_operands_without_bound_check(1);
        unsafe { std::ptr::copy(result_operands.as_ptr(), results_ptr, OPERAND_SIZE_IN_BYTES) };
    }
}
