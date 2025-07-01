// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::sync::Mutex;

use anc_context::thread_context::ThreadContext;
use anc_image::bytecode_reader::format_bytecode_as_text;
use anc_isa::opcode::Opcode;
use anc_stack::ProgramCounter;
// use cranelift_jit::JITModule;

pub type HandleFunc = fn(/* &Handler, */ &mut ThreadContext) -> HandleResult;

mod arithmetic;
mod bitwise;
mod calling;
mod comparison;
mod control_flow;
mod conversion;
mod data;
mod fundamental;
mod local;
mod machine;
mod math;
mod memory;

/// The result of a instruction is executed.
pub enum HandleResult {
    // move to next program counter (PC) within a function.
    // the parameter `isize` is the offset in bytes.
    Move(/* relate_offset_in_bytes */ isize),

    // Move between functions.
    //
    // there are two cases:
    // - Calling: jump to another function.
    // - Returning: return from another function.
    //
    // the parameter `ProgramCounter` is the address of the next instruction.
    // This result is similar to `Move`, but the `Jump` use an absolute address
    // instead of a relative offset, and `Jump` can change the module index and
    // function index.
    Jump(/* next PC */ ProgramCounter),

    // End the current "function calling path".
    //
    // there are two cases:
    //
    // 1. End of the entry function, the program running is finished.
    //    The parameter `ProgramCounter` is 0 in this case.
    // 2. End of a callback function, the execution will return to the external function, and then
    //    return to the next instruction of the `extcall` instruction.
    //    The parameter `ProgramCounter` is the address of next instruction.
    //    The follow diagram illustrates the flow:
    //
    // ```diagram
    // | module M, function A |             | external func |       | module N, function B |
    // |----------------------|             |---------------|       |----------------------|
    // |                      |    wrapper  |               | callback delegate            |
    // | 0x0000 inst_0        |    function |               | function                     |
    // | 0x0004 inst_1   extcall   |        |               |   |   |                      |
    // | 0x0008 inst_2   ----------O---------> 0x0000       | /-O----> 0x0000 inst_0       |
    // |     \------------<---------------\ |  0x0004       | |     |  0x0004 inst_1       |
    // |                      |           | |  0x0008  -------/     |  0x0008 inst_2       |
    // | HandleResult::Jump(X)            ^ |               |       |  0x000c inst_3       |
    // |                   |  |           | |               | /------- 0x0010 end          |
    // |                   |  |           | |  0x000c  <------/     |                      |
    // | 0x000c inst_3   <-/  |           \--- 0x0010       |       |----------------------|
    // | 0x0010 inst_4        |             |               |
    // | 0x0014 inst_5        |             |---------------|
    // | 0x0018 inst_6        |
    // | ...                  |
    // ```
    End(ProgramCounter),

    // Program terminated.
    Terminate(/* terminate_code */ i32),
}

fn unreachable_handler(
    /* _handler: &Handler, */ thread_context: &mut ThreadContext,
) -> HandleResult {
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
Invalid Opcode: 0x{:04x}
Module index: {}
Function internal index: {}
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

// #[non_exhaustive]
// pub struct Handler {
//     // pub syscall_handlers: [SysCallHandlerFunc; MAX_SYSCALL_TYPE_NUMBER],
//     // pub envcall_handlers: [EnvCallHandlerFunc; MAX_ENVCALL_CODE_NUMBER],
//     pub jit_generator: Mutex<Generator<JITModule>>,
// }
//
// impl Handler {
//     pub fn new() -> Self {
//         Handler {
//             // handlers,
//             // syscall_handlers: generate_syscall_handlers(),
//             // envcall_handlers: generate_envcall_handlers(),
//             jit_generator: get_jit_generator_without_imported_symbols(),
//         }
//     }
// }
//
// impl Default for Handler {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl Handler {
#[inline]
pub fn get_instruction_handler(opcode_num: u16) -> HandleFunc {
    let opcode = unsafe { std::mem::transmute::<u16, Opcode>(opcode_num) };
    let category = opcode_num >> 8;
    match category {
        0x01 => {
            // fundamental
            match opcode {
                Opcode::nop => fundamental::nop,
                Opcode::imm_i32 => fundamental::imm_i32,
                Opcode::imm_i64 => fundamental::imm_i64,
                Opcode::imm_f32 => fundamental::imm_f32,
                Opcode::imm_f64 => fundamental::imm_f64,
                _ => unreachable_handler,
            }
        }
        0x02 => {
            // local
            match opcode {
                Opcode::local_load_i64 => local::local_load_i64,
                Opcode::local_load_i32_s => local::local_load_i32_s,
                Opcode::local_load_i32_u => local::local_load_i32_u,
                Opcode::local_load_i16_s => local::local_load_i16_s,
                Opcode::local_load_i16_u => local::local_load_i16_u,
                Opcode::local_load_i8_s => local::local_load_i8_s,
                Opcode::local_load_i8_u => local::local_load_i8_u,
                Opcode::local_load_f32 => local::local_load_f32,
                Opcode::local_load_f64 => local::local_load_f64,
                Opcode::local_store_i64 => local::local_store_i64,
                Opcode::local_store_i32 => local::local_store_i32,
                Opcode::local_store_i16 => local::local_store_i16,
                Opcode::local_store_i8 => local::local_store_i8,
                Opcode::local_store_f64 => local::local_store_i64, // reuse store i64
                Opcode::local_store_f32 => local::local_store_i32, // reuse store i32
                _ => unreachable_handler,
            }
        }
        0x03 => {
            // data
            match opcode {
                Opcode::data_load_i64 => data::data_load_i64,
                Opcode::data_load_i32_s => data::data_load_i32_s,
                Opcode::data_load_i32_u => data::data_load_i32_u,
                Opcode::data_load_i16_s => data::data_load_i16_s,
                Opcode::data_load_i16_u => data::data_load_i16_u,
                Opcode::data_load_i8_s => data::data_load_i8_s,
                Opcode::data_load_i8_u => data::data_load_i8_u,
                Opcode::data_load_f64 => data::data_load_f64,
                Opcode::data_load_f32 => data::data_load_f32,
                Opcode::data_store_i64 => data::data_store_i64,
                Opcode::data_store_i32 => data::data_store_i32,
                Opcode::data_store_i16 => data::data_store_i16,
                Opcode::data_store_i8 => data::data_store_i8,
                Opcode::data_store_f64 => data::data_store_i64, // reuse store i64
                Opcode::data_store_f32 => data::data_store_i32, // reuse store i32
                Opcode::data_load_extend_i64 => data::data_load_extend_i64,
                Opcode::data_load_extend_i32_s => data::data_load_extend_i32_s,
                Opcode::data_load_extend_i32_u => data::data_load_extend_i32_u,
                Opcode::data_load_extend_i16_s => data::data_load_extend_i16_s,
                Opcode::data_load_extend_i16_u => data::data_load_extend_i16_u,
                Opcode::data_load_extend_i8_s => data::data_load_extend_i8_s,
                Opcode::data_load_extend_i8_u => data::data_load_extend_i8_u,
                Opcode::data_load_extend_f64 => data::data_load_extend_f64,
                Opcode::data_load_extend_f32 => data::data_load_extend_f32,
                Opcode::data_store_extend_i64 => data::data_store_extend_i64,
                Opcode::data_store_extend_i32 => data::data_store_extend_i32,
                Opcode::data_store_extend_i16 => data::data_store_extend_i16,
                Opcode::data_store_extend_i8 => data::data_store_extend_i8,
                Opcode::data_store_extend_f64 => data::data_store_extend_i64, // reuse store_i64
                Opcode::data_store_extend_f32 => data::data_store_extend_i32, // reuse store_i32
                Opcode::data_load_dynamic_i64 => data::data_load_dynamic_i64,
                Opcode::data_load_dynamic_i32_s => data::data_load_dynamic_i32_s,
                Opcode::data_load_dynamic_i32_u => data::data_load_dynamic_i32_u,
                Opcode::data_load_dynamic_i16_s => data::data_load_dynamic_i16_s,
                Opcode::data_load_dynamic_i16_u => data::data_load_dynamic_i16_u,
                Opcode::data_load_dynamic_i8_s => data::data_load_dynamic_i8_s,
                Opcode::data_load_dynamic_i8_u => data::data_load_dynamic_i8_u,
                Opcode::data_load_dynamic_f64 => data::data_load_dynamic_f64,
                Opcode::data_load_dynamic_f32 => data::data_load_dynamic_f32,
                Opcode::data_store_dynamic_i64 => data::data_store_dynamic_i64,
                Opcode::data_store_dynamic_i32 => data::data_store_dynamic_i32,
                Opcode::data_store_dynamic_i16 => data::data_store_dynamic_i16,
                Opcode::data_store_dynamic_i8 => data::data_store_dynamic_i8,
                Opcode::data_store_dynamic_f64 => data::data_store_dynamic_i64, // reuse store_i64
                Opcode::data_store_dynamic_f32 => data::data_store_dynamic_i32, // reuse store_i32

                _ => unreachable_handler,
            }
        }
        0x04 => {
            // arithmetic
            match opcode {
                Opcode::add_i32 => arithmetic::add_i32,
                Opcode::sub_i32 => arithmetic::sub_i32,
                Opcode::add_imm_i32 => arithmetic::add_imm_i32,
                Opcode::sub_imm_i32 => arithmetic::sub_imm_i32,
                Opcode::mul_i32 => arithmetic::mul_i32,
                Opcode::div_i32_s => arithmetic::div_i32_s,
                Opcode::div_i32_u => arithmetic::div_i32_u,
                Opcode::rem_i32_s => arithmetic::rem_i32_s,
                Opcode::rem_i32_u => arithmetic::rem_i32_u,
                Opcode::add_i64 => arithmetic::add_i64,
                Opcode::sub_i64 => arithmetic::sub_i64,
                Opcode::add_imm_i64 => arithmetic::add_imm_i64,
                Opcode::sub_imm_i64 => arithmetic::sub_imm_i64,
                Opcode::mul_i64 => arithmetic::mul_i64,
                Opcode::div_i64_s => arithmetic::div_i64_s,
                Opcode::div_i64_u => arithmetic::div_i64_u,
                Opcode::rem_i64_s => arithmetic::rem_i64_s,
                Opcode::rem_i64_u => arithmetic::rem_i64_u,
                Opcode::add_f32 => arithmetic::add_f32,
                Opcode::sub_f32 => arithmetic::sub_f32,
                Opcode::mul_f32 => arithmetic::mul_f32,
                Opcode::div_f32 => arithmetic::div_f32,
                Opcode::add_f64 => arithmetic::add_f64,
                Opcode::sub_f64 => arithmetic::sub_f64,
                Opcode::mul_f64 => arithmetic::mul_f64,
                Opcode::div_f64 => arithmetic::div_f64,

                _ => unreachable_handler,
            }
        }
        0x05 => {
            // bitwise
            match opcode {
                Opcode::and => bitwise::and,
                Opcode::or => bitwise::or,
                Opcode::xor => bitwise::xor,
                Opcode::not => bitwise::not,
                Opcode::count_leading_zeros_i32 => bitwise::count_leading_zeros_i32,
                Opcode::count_leading_ones_i32 => bitwise::count_leading_ones_i32,
                Opcode::count_trailing_zeros_i32 => bitwise::count_trailing_zeros_i32,
                Opcode::count_ones_i32 => bitwise::count_ones_i32,
                Opcode::shift_left_i32 => bitwise::shift_left_i32,
                Opcode::shift_right_i32_s => bitwise::shift_right_i32_s,
                Opcode::shift_right_i32_u => bitwise::shift_right_i32_u,
                Opcode::rotate_left_i32 => bitwise::rotate_left_i32,
                Opcode::rotate_right_i32 => bitwise::rotate_right_i32,
                Opcode::count_leading_zeros_i64 => bitwise::count_leading_zeros_i64,
                Opcode::count_leading_ones_i64 => bitwise::count_leading_ones_i64,
                Opcode::count_trailing_zeros_i64 => bitwise::count_trailing_zeros_i64,
                Opcode::count_ones_i64 => bitwise::count_ones_i64,
                Opcode::shift_left_i64 => bitwise::shift_left_i64,
                Opcode::shift_right_i64_s => bitwise::shift_right_i64_s,
                Opcode::shift_right_i64_u => bitwise::shift_right_i64_u,
                Opcode::rotate_left_i64 => bitwise::rotate_left_i64,
                Opcode::rotate_right_i64 => bitwise::rotate_right_i64,

                _ => unreachable_handler,
            }
        }
        0x06 => {
            // math
            match opcode {
                Opcode::abs_i32 => math::abs_i32,
                Opcode::neg_i32 => math::neg_i32,
                Opcode::abs_i64 => math::abs_i64,
                Opcode::neg_i64 => math::neg_i64,
                // f32
                Opcode::abs_f32 => math::abs_f32,
                Opcode::neg_f32 => math::neg_f32,
                Opcode::copysign_f32 => math::copysign_f32,
                Opcode::sqrt_f32 => math::sqrt_f32,
                Opcode::min_f32 => math::min_f32,
                Opcode::max_f32 => math::max_f32,
                Opcode::ceil_f32 => math::ceil_f32,
                Opcode::floor_f32 => math::floor_f32,
                Opcode::round_half_away_from_zero_f32 => math::round_half_away_from_zero_f32,
                Opcode::round_half_to_even_f32 => math::round_half_to_even_f32,
                Opcode::trunc_f32 => math::trunc_f32,
                Opcode::fract_f32 => math::fract_f32,
                Opcode::cbrt_f32 => math::cbrt_f32,
                Opcode::exp_f32 => math::exp_f32,
                Opcode::exp2_f32 => math::exp2_f32,
                Opcode::ln_f32 => math::ln_f32,
                Opcode::log2_f32 => math::log2_f32,
                Opcode::log10_f32 => math::log10_f32,
                Opcode::sin_f32 => math::sin_f32,
                Opcode::cos_f32 => math::cos_f32,
                Opcode::tan_f32 => math::tan_f32,
                Opcode::asin_f32 => math::asin_f32,
                Opcode::acos_f32 => math::acos_f32,
                Opcode::atan_f32 => math::atan_f32,
                Opcode::pow_f32 => math::pow_f32,
                Opcode::log_f32 => math::log_f32,
                // f64
                Opcode::abs_f64 => math::abs_f64,
                Opcode::neg_f64 => math::neg_f64,
                Opcode::copysign_f64 => math::copysign_f64,
                Opcode::sqrt_f64 => math::sqrt_f64,
                Opcode::min_f64 => math::min_f64,
                Opcode::max_f64 => math::max_f64,
                Opcode::ceil_f64 => math::ceil_f64,
                Opcode::floor_f64 => math::floor_f64,
                Opcode::round_half_away_from_zero_f64 => math::round_half_away_from_zero_f64,
                Opcode::round_half_to_even_f64 => math::round_half_to_even_f64,
                Opcode::trunc_f64 => math::trunc_f64,
                Opcode::fract_f64 => math::fract_f64,
                Opcode::cbrt_f64 => math::cbrt_f64,
                Opcode::exp_f64 => math::exp_f64,
                Opcode::exp2_f64 => math::exp2_f64,
                Opcode::ln_f64 => math::ln_f64,
                Opcode::log2_f64 => math::log2_f64,
                Opcode::log10_f64 => math::log10_f64,
                Opcode::sin_f64 => math::sin_f64,
                Opcode::cos_f64 => math::cos_f64,
                Opcode::tan_f64 => math::tan_f64,
                Opcode::asin_f64 => math::asin_f64,
                Opcode::acos_f64 => math::acos_f64,
                Opcode::atan_f64 => math::atan_f64,
                Opcode::pow_f64 => math::pow_f64,
                Opcode::log_f64 => math::log_f64,
                _ => unreachable_handler,
            }
        }
        0x07 => {
            // conversion
            match opcode {
                Opcode::truncate_i64_to_i32 => conversion::truncate_i64_to_i32,
                Opcode::extend_i32_s_to_i64 => conversion::extend_i32_s_to_i64,
                Opcode::extend_i32_u_to_i64 => conversion::extend_i32_u_to_i64,
                Opcode::demote_f64_to_f32 => conversion::demote_f64_to_f32,
                Opcode::promote_f32_to_f64 => conversion::promote_f32_to_f64,
                Opcode::convert_f32_to_i32_s => conversion::convert_f32_to_i32_s,
                Opcode::convert_f32_to_i32_u => conversion::convert_f32_to_i32_u,
                Opcode::convert_f64_to_i32_s => conversion::convert_f64_to_i32_s,
                Opcode::convert_f64_to_i32_u => conversion::convert_f64_to_i32_u,
                Opcode::convert_f32_to_i64_s => conversion::convert_f32_to_i64_s,
                Opcode::convert_f32_to_i64_u => conversion::convert_f32_to_i64_u,
                Opcode::convert_f64_to_i64_s => conversion::convert_f64_to_i64_s,
                Opcode::convert_f64_to_i64_u => conversion::convert_f64_to_i64_u,
                Opcode::convert_i32_s_to_f32 => conversion::convert_i32_s_to_f32,
                Opcode::convert_i32_u_to_f32 => conversion::convert_i32_u_to_f32,
                Opcode::convert_i64_s_to_f32 => conversion::convert_i64_s_to_f32,
                Opcode::convert_i64_u_to_f32 => conversion::convert_i64_u_to_f32,
                Opcode::convert_i32_s_to_f64 => conversion::convert_i32_s_to_f64,
                Opcode::convert_i32_u_to_f64 => conversion::convert_i32_u_to_f64,
                Opcode::convert_i64_s_to_f64 => conversion::convert_i64_s_to_f64,
                Opcode::convert_i64_u_to_f64 => conversion::convert_i64_u_to_f64,

                _ => unreachable_handler,
            }
        }
        0x08 => {
            // comparison
            match opcode {
                Opcode::eqz_i32 => comparison::eqz_i32,
                Opcode::nez_i32 => comparison::nez_i32,
                Opcode::eq_i32 => comparison::eq_i32,
                Opcode::ne_i32 => comparison::ne_i32,
                Opcode::lt_i32_s => comparison::lt_i32_s,
                Opcode::lt_i32_u => comparison::lt_i32_u,
                Opcode::gt_i32_s => comparison::gt_i32_s,
                Opcode::gt_i32_u => comparison::gt_i32_u,
                Opcode::le_i32_s => comparison::le_i32_s,
                Opcode::le_i32_u => comparison::le_i32_u,
                Opcode::ge_i32_s => comparison::ge_i32_s,
                Opcode::ge_i32_u => comparison::ge_i32_u,
                Opcode::eqz_i64 => comparison::eqz_i64,
                Opcode::nez_i64 => comparison::nez_i64,
                Opcode::eq_i64 => comparison::eq_i64,
                Opcode::ne_i64 => comparison::ne_i64,
                Opcode::lt_i64_s => comparison::lt_i64_s,
                Opcode::lt_i64_u => comparison::lt_i64_u,
                Opcode::gt_i64_s => comparison::gt_i64_s,
                Opcode::gt_i64_u => comparison::gt_i64_u,
                Opcode::le_i64_s => comparison::le_i64_s,
                Opcode::le_i64_u => comparison::le_i64_u,
                Opcode::ge_i64_s => comparison::ge_i64_s,
                Opcode::ge_i64_u => comparison::ge_i64_u,
                Opcode::eq_f32 => comparison::eq_f32,
                Opcode::ne_f32 => comparison::ne_f32,
                Opcode::lt_f32 => comparison::lt_f32,
                Opcode::gt_f32 => comparison::gt_f32,
                Opcode::le_f32 => comparison::le_f32,
                Opcode::ge_f32 => comparison::ge_f32,
                Opcode::eq_f64 => comparison::eq_f64,
                Opcode::ne_f64 => comparison::ne_f64,
                Opcode::lt_f64 => comparison::lt_f64,
                Opcode::gt_f64 => comparison::gt_f64,
                Opcode::le_f64 => comparison::le_f64,
                Opcode::ge_f64 => comparison::ge_f64,
                _ => unreachable_handler,
            }
        }
        0x09 => {
            // control_flow
            match opcode {
                Opcode::end => control_flow::end,
                Opcode::block => control_flow::block,
                Opcode::break_ => control_flow::break_,
                Opcode::recur => control_flow::recur,
                Opcode::block_alt => control_flow::block_alt,
                Opcode::break_alt => control_flow::break_alt,
                Opcode::block_nez => control_flow::block_nez,

                _ => unreachable_handler,
            }
        }
        0x0A => {
            // calling
            match opcode {
                Opcode::call => calling::call,
                Opcode::call_dynamic => calling::call_dynamic,
                Opcode::syscall => calling::syscall,
                Opcode::envcall => calling::envcall,
                Opcode::extcall => calling::extcall,

                _ => unreachable_handler,
            }
        }
        0x0B => {
            // memory
            match opcode {
                Opcode::memory_allocate => memory::memory_allocate,
                Opcode::memory_resize => memory::memory_resize,
                Opcode::memory_free => memory::memory_free,
                Opcode::memory_fill => memory::memory_fill,
                Opcode::memory_copy => memory::memory_copy,

                _ => unreachable_handler,
            }
        }
        0x0C => {
            // machine
            match opcode {
                Opcode::terminate => machine::terminate,
                Opcode::get_function => machine::get_function,
                Opcode::get_data => machine::get_data,
                Opcode::host_addr_function => machine::host_addr_function,
                Opcode::host_addr_function_dynamic => machine::host_addr_function_dynamic,
                Opcode::host_addr_data => machine::host_addr_data,
                Opcode::host_addr_data_extend => machine::host_addr_data_extend,
                Opcode::host_addr_data_dynamic => machine::host_addr_data_dynamic,

                _ => unreachable_handler,
            }
        }
        _ => unreachable_handler,
    }
}
// }
