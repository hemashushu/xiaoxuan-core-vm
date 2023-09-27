// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::sync::Mutex;

use ancvm_binary::utils::format_bytecodes;
use ancvm_thread::thread::{ProgramCounter, Thread};
use ancvm_types::{
    opcode::{Opcode, MAX_OPCODE_NUMBER},
    ForeignValue,
};

use crate::{ecall, InterpreterError};

type InterpretFunc = fn(&mut Thread) -> InterpretResult;

mod arithmetic;
mod bitwise;
mod comparison;
mod control_flow;
mod conversion;
mod data;
mod fundamental;
mod heap;
mod local;
mod machine;
mod math;

pub enum InterpretResult {
    // move to another address within a function
    // param (relate_offset_in_bytes:isize)
    Move(isize),

    // jump to another function (call), or
    // return from another function (return)
    // param (return_pc:ProgramCounter)
    Jump(ProgramCounter),

    // program end
    End,

    // pause the interpreter
    // for debug the program or the VM itself
    Debug,
}

fn unreachable(thread: &mut Thread) -> InterpretResult {
    let pc = &thread.pc;
    let func_item =
        &thread.context.modules[pc.module_index].func_section.items[pc.function_internal_index];
    let codes = &thread.context.modules[pc.module_index]
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
        thread.get_opcode_num(),
        pc.module_index,
        pc.function_internal_index,
        pc.instruction_address,
        code_text
    );
}

static INIT_LOCK: Mutex<i32> = Mutex::new(0);
static mut IS_INIT: bool = false;
static mut INTERPRETERS: [InterpretFunc; MAX_OPCODE_NUMBER] = [unreachable; MAX_OPCODE_NUMBER];

/// initilize the instruction interpreters
pub fn init_interpreters() {
    let _lock = INIT_LOCK.lock().unwrap();

    unsafe {
        if IS_INIT {
            return;
        }
        IS_INIT = true;
    }

    let interpreters = unsafe { &mut INTERPRETERS };

    // the initialization can only be called once
    // in the unit test environment (`$ cargo test`), the init procedure
    // runs in parallel.
    // if interpreters[Opcode::zero as usize] == fundamental::zero {
    //     return;
    // }

    // fundamental
    interpreters[Opcode::zero as usize] = fundamental::zero;
    interpreters[Opcode::drop as usize] = fundamental::drop_;
    interpreters[Opcode::duplicate as usize] = fundamental::duplicate;
    interpreters[Opcode::swap as usize] = fundamental::swap;
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
    interpreters[Opcode::i32_demote_i64 as usize] = conversion::i32_demote_i64;
    interpreters[Opcode::i64_promote_i32_s as usize] = conversion::i64_promote_i32_s;
    interpreters[Opcode::i64_promote_i32_u as usize] = conversion::i64_promote_i32_u;
    interpreters[Opcode::f32_demote_f64 as usize] = conversion::f32_demote_f64;
    interpreters[Opcode::f64_promote_f32 as usize] = conversion::f64_promote_f32;
    interpreters[Opcode::i32_trunc_f32_s as usize] = conversion::i32_trunc_f32_s;
    interpreters[Opcode::i32_trunc_f32_u as usize] = conversion::i32_trunc_f32_u;
    interpreters[Opcode::i32_trunc_f64_s as usize] = conversion::i32_trunc_f64_s;
    interpreters[Opcode::i32_trunc_f64_u as usize] = conversion::i32_trunc_f64_u;
    interpreters[Opcode::i64_trunc_f32_s as usize] = conversion::i64_trunc_f32_s;
    interpreters[Opcode::i64_trunc_f32_u as usize] = conversion::i64_trunc_f32_u;
    interpreters[Opcode::i64_trunc_f64_s as usize] = conversion::i64_trunc_f64_s;
    interpreters[Opcode::i64_trunc_f64_u as usize] = conversion::i64_trunc_f64_u;
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
    interpreters[Opcode::call as usize] = control_flow::call;
    interpreters[Opcode::dcall as usize] = control_flow::dcall;

    interpreters[Opcode::ecall as usize] = ecall::ecall;

    // machine
    interpreters[Opcode::nop as usize] = machine::nop;
    interpreters[Opcode::debug as usize] = machine::debug;
    interpreters[Opcode::host_addr_local as usize] = machine::host_addr_local;
    interpreters[Opcode::host_addr_local_long as usize] = machine::host_addr_local_long;
    interpreters[Opcode::host_addr_data as usize] = machine::host_addr_data;
    interpreters[Opcode::host_addr_data_long as usize] = machine::host_addr_data_long;
    interpreters[Opcode::host_addr_heap as usize] = machine::host_addr_heap;
}

pub fn process_next_instruction(thread: &mut Thread) -> InterpretResult {
    let opcode_num = thread.get_opcode_num();
    let func = unsafe { &INTERPRETERS[opcode_num as usize] };
    func(thread)
}

pub fn process_continuous_instructions(thread: &mut Thread) {
    loop {
        let result = //self.
                process_next_instruction(thread);
        match result {
            InterpretResult::Move(relate_offset_in_bytes) => {
                let next_instruction_offset =
                    thread.pc.instruction_address as isize + relate_offset_in_bytes;
                thread.pc.instruction_address = next_instruction_offset as usize;
            }
            InterpretResult::Jump(return_pc) => {
                thread.pc.module_index = return_pc.module_index;
                thread.pc.function_internal_index = return_pc.function_internal_index;
                thread.pc.instruction_address = return_pc.instruction_address;
            }
            InterpretResult::End => break,
            InterpretResult::Debug => {
                thread.pc.instruction_address += 2;
            }
        }
    }
}

// note:
// 'function public index' includes the imported functions, it equals to
// 'amount of imported functions' + 'function internal index'
pub fn process_function(
    thread: &mut Thread,
    module_index: usize,
    func_public_index: usize,
    arguments: &[ForeignValue],
) -> Result<Vec<ForeignValue>, InterpreterError> {
    thread.stack.reset();

    // find the code start address
    let (target_module_index, function_internal_index) =
        thread.get_function_internal_index_and_module_index(module_index, func_public_index);
    let (type_index, local_variables_list_index, code_offset, local_variables_allocate_bytes) =
        thread
            .get_function_type_and_local_index_and_code_offset_and_local_variables_allocate_bytes(
                target_module_index,
                function_internal_index,
            );

    let type_entry = thread.context.modules[target_module_index]
        .type_section
        .get_entry(type_index);

    if type_entry.params.len() != arguments.len() {
        return Err(InterpreterError::new(
            "The number of arguments does not match the specified funcion.",
        ));
    }

    // for simplicity, does not check the data type of arguments for now.

    // push arguments
    thread.push_values(arguments);

    // create function statck frame
    thread.stack.create_frame(
        type_entry.params.len() as u16,
        type_entry.results.len() as u16,
        local_variables_list_index as u32,
        local_variables_allocate_bytes,
        Some(ProgramCounter {
            // the '0' for 'return instruction address' is used to indicate that it's the END of the thread.
            //
            // the function stack frame is created only by 'call' instruction or
            // thread beginning, the 'call' instruction will set the 'return instruction address' to
            // the instruction next to 'call', which can't be '0'.
            // so when a stack frame exits and the 'return address' is zero, it can only
            // be the end of a thread.
            instruction_address: 0,
            function_internal_index: 0,
            module_index: 0,
        }),
    );

    // set new PC
    thread.pc.module_index = target_module_index;
    thread.pc.function_internal_index = function_internal_index;
    thread.pc.instruction_address = code_offset;

    // self.
    process_continuous_instructions(thread);

    // pop results off the stack
    let results = thread.pop_values(&type_entry.results);

    Ok(results)
}
