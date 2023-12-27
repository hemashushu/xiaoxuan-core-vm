// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_types::opcode::Opcode;

// format the bytecode as the following text:
//
// 0x0008  00 11 22 33  44 55 66 77
// 0x0000  88 99 aa bb  cc dd ee ff
//
pub fn print_bytecode_as_binary(codes: &[u8]) -> String {
    codes
        .chunks(8)
        .enumerate()
        .map(|(chunk_addr, chunk)| {
            let binary = chunk
                .iter()
                .enumerate()
                .map(|(idx, byte)| {
                    // format the bytes as the following text:
                    // 00 11 22 33  44 55 66 77
                    // 00 11 22 33
                    // 00 11
                    //
                    // Rust std format!()
                    // https://doc.rust-lang.org/std/fmt/
                    if idx == 4 {
                        format!("  {:02x}", byte)
                    } else if idx == 0 {
                        format!("{:02x}", byte)
                    } else {
                        format!(" {:02x}", byte)
                    }
                })
                .collect::<Vec<String>>()
                .join("");

            format!("0x{:04x}  {}", chunk_addr * 8, binary)
        })
        .collect::<Vec<String>>()
        .join("\n")
}

// format the bytecode as the following text:
//
// 0x0000  00 07                       i32.add
// 0x0002  00 04 02 00                 heap.load       off:0x02
// 0x0006  08 04 03 00                 heap.store      off:0x03
pub fn print_bytecode_as_text(codes: &[u8]) -> String {
    let mut lines: Vec<String> = Vec::new();

    let code_length = codes.len(); // in bytes
    let mut offset = 0; // in bytes

    loop {
        if offset == code_length {
            break;
        };

        let (offset_param, opcode) = read_opcode(codes, offset);

        let (offset_next, param_text) = match opcode {
            // fundemental
            Opcode::nop
            | Opcode::zero
            | Opcode::drop
            // | Opcode::duplicate
            // | Opcode::swap
            | Opcode::select_nez => (offset_param, String::new()),
            Opcode::i32_imm | Opcode::f32_imm => {
                let (offset_next, v) = read_param_i32(codes, offset_param);
                (offset_next, format!("0x{:08x}", v))
            }
            Opcode::i64_imm | Opcode::f64_imm => {
                let (offset_next, v_low, v_high) = read_param_i32_i32(codes, offset_param);
                (
                    offset_next,
                    format!("low:0x{:08x}  high:0x{:08x}", v_low, v_high),
                )
            }
            // local load/store
            Opcode::local_load64_i64
            | Opcode::local_load32_i32
            | Opcode::local_load32_i16_s
            | Opcode::local_load32_i16_u
            | Opcode::local_load32_i8_s
            | Opcode::local_load32_i8_u
            | Opcode::local_load64_f64
            | Opcode::local_load32_f32
            | Opcode::local_store64
            | Opcode::local_store32
            | Opcode::local_store16
            | Opcode::local_store8 => {
                let (offset_next, reversed_index, offset, index) =
                    read_param_i16_i16_i16(codes, offset_param);
                (
                    offset_next,
                    format!(
                        "rev:{:<2}  off:0x{:02x}  idx:{}",
                        reversed_index, offset, index
                    ),
                )
            }
            //
            Opcode::local_offset_load64_i64
            | Opcode::local_offset_load32_i32
            | Opcode::local_offset_load32_i16_s
            | Opcode::local_offset_load32_i16_u
            | Opcode::local_offset_load32_i8_s
            | Opcode::local_offset_load32_i8_u
            | Opcode::local_offset_load64_f64
            | Opcode::local_offset_load32_f32
            | Opcode::local_offset_store64
            | Opcode::local_offset_store32
            | Opcode::local_offset_store16
            | Opcode::local_offset_store8 => {
                let (offset_next, reversed_index, index) = read_param_i16_i32(codes, offset_param);
                (
                    offset_next,
                    format!("rev:{:<2}  idx:{}", reversed_index, index),
                )
            }
            // data load/store
            Opcode::data_load64_i64
            | Opcode::data_load32_i32
            | Opcode::data_load32_i16_s
            | Opcode::data_load32_i16_u
            | Opcode::data_load32_i8_s
            | Opcode::data_load32_i8_u
            | Opcode::data_load64_f64
            | Opcode::data_load32_f32
            | Opcode::data_store64
            | Opcode::data_store32
            | Opcode::data_store16
            | Opcode::data_store8 => {
                let (offset_next, offset, index) = read_param_i16_i32(codes, offset_param);
                (offset_next, format!("off:0x{:02x}  idx:{}", offset, index))
            }
            //
            Opcode::data_offset_load64_i64
            | Opcode::data_offset_load32_i32
            | Opcode::data_offset_load32_i16_s
            | Opcode::data_offset_load32_i16_u
            | Opcode::data_offset_load32_i8_s
            | Opcode::data_offset_load32_i8_u
            | Opcode::data_offset_load64_f64
            | Opcode::data_offset_load32_f32
            | Opcode::data_offset_store64
            | Opcode::data_offset_store32
            | Opcode::data_offset_store16
            | Opcode::data_offset_store8 => {
                let (offset_next, index) = read_param_i32(codes, offset_param);
                (offset_next, format!("idx:{}", index))
            }
            // heap load/store
            Opcode::heap_load64_i64
            | Opcode::heap_load32_i32
            | Opcode::heap_load32_i16_s
            | Opcode::heap_load32_i16_u
            | Opcode::heap_load32_i8_s
            | Opcode::heap_load32_i8_u
            | Opcode::heap_load64_f64
            | Opcode::heap_load32_f32
            | Opcode::heap_store64
            | Opcode::heap_store32
            | Opcode::heap_store16
            | Opcode::heap_store8 => {
                let (offset_next, offset) = read_param_i16(codes, offset_param);
                (offset_next, format!("off:0x{:02x}", offset))
            }
            // heap memory
            Opcode::heap_fill | Opcode::heap_copy | Opcode::heap_capacity | Opcode::heap_resize => {
                (offset_param, String::new())
            }
            // conversion
            Opcode::i32_truncate_i64
            | Opcode::i64_extend_i32_s
            | Opcode::i64_extend_i32_u
            | Opcode::f32_demote_f64
            | Opcode::f64_promote_f32
            | Opcode::i32_convert_f32_s
            | Opcode::i32_convert_f32_u
            | Opcode::i32_convert_f64_s
            | Opcode::i32_convert_f64_u
            | Opcode::i64_convert_f32_s
            | Opcode::i64_convert_f32_u
            | Opcode::i64_convert_f64_s
            | Opcode::i64_convert_f64_u
            | Opcode::f32_convert_i32_s
            | Opcode::f32_convert_i32_u
            | Opcode::f32_convert_i64_s
            | Opcode::f32_convert_i64_u
            | Opcode::f64_convert_i32_s
            | Opcode::f64_convert_i32_u
            | Opcode::f64_convert_i64_s
            | Opcode::f64_convert_i64_u => (offset_param, String::new()),
            // comparsion
            Opcode::i32_eqz
            | Opcode::i32_nez
            | Opcode::i32_eq
            | Opcode::i32_ne
            | Opcode::i32_lt_s
            | Opcode::i32_lt_u
            | Opcode::i32_gt_s
            | Opcode::i32_gt_u
            | Opcode::i32_le_s
            | Opcode::i32_le_u
            | Opcode::i32_ge_s
            | Opcode::i32_ge_u
            | Opcode::i64_eqz
            | Opcode::i64_nez
            | Opcode::i64_eq
            | Opcode::i64_ne
            | Opcode::i64_lt_s
            | Opcode::i64_lt_u
            | Opcode::i64_gt_s
            | Opcode::i64_gt_u
            | Opcode::i64_le_s
            | Opcode::i64_le_u
            | Opcode::i64_ge_s
            | Opcode::i64_ge_u
            | Opcode::f32_eq
            | Opcode::f32_ne
            | Opcode::f32_lt
            | Opcode::f32_gt
            | Opcode::f32_le
            | Opcode::f32_ge
            | Opcode::f64_eq
            | Opcode::f64_ne
            | Opcode::f64_lt
            | Opcode::f64_gt
            | Opcode::f64_le
            | Opcode::f64_ge => (offset_param, String::new()),
            // arithmetic
            Opcode::i32_add
            | Opcode::i32_sub
            | Opcode::i32_mul
            | Opcode::i32_div_s
            | Opcode::i32_div_u
            | Opcode::i32_rem_s
            | Opcode::i32_rem_u => (offset_param, String::new()),
            Opcode::i32_inc | Opcode::i32_dec => {
                let (offset_next, amount) = read_param_i16(codes, offset_param);
                (offset_next, format!("{}", amount))
            }
            Opcode::i64_add
            | Opcode::i64_sub
            | Opcode::i64_mul
            | Opcode::i64_div_s
            | Opcode::i64_div_u
            | Opcode::i64_rem_s
            | Opcode::i64_rem_u => (offset_param, String::new()),
            Opcode::i64_inc | Opcode::i64_dec => {
                let (offset_next, amount) = read_param_i16(codes, offset_param);
                (offset_next, format!("{}", amount))
            }
            Opcode::f32_add
            | Opcode::f32_sub
            | Opcode::f32_mul
            | Opcode::f32_div
            | Opcode::f64_add
            | Opcode::f64_sub
            | Opcode::f64_mul
            | Opcode::f64_div => (offset_param, String::new()),
            // bitwise
            Opcode::i32_and
            | Opcode::i32_or
            | Opcode::i32_xor
            | Opcode::i32_not
            | Opcode::i32_leading_zeros
            | Opcode::i32_leading_ones
            | Opcode::i32_trailing_zeros
            | Opcode::i32_count_ones
            | Opcode::i32_shift_left
            | Opcode::i32_shift_right_s
            | Opcode::i32_shift_right_u
            | Opcode::i32_rotate_left
            | Opcode::i32_rotate_right
            | Opcode::i64_and
            | Opcode::i64_or
            | Opcode::i64_xor
            | Opcode::i64_not
            | Opcode::i64_leading_zeros
            | Opcode::i64_leading_ones
            | Opcode::i64_trailing_zeros
            | Opcode::i64_count_ones
            | Opcode::i64_shift_left
            | Opcode::i64_shift_right_s
            | Opcode::i64_shift_right_u
            | Opcode::i64_rotate_left
            | Opcode::i64_rotate_right => (offset_param, String::new()),
            // math
            Opcode::i32_abs
            | Opcode::i32_neg
            | Opcode::i64_abs
            | Opcode::i64_neg
            | Opcode::f32_abs
            | Opcode::f32_neg
            | Opcode::f32_ceil
            | Opcode::f32_floor
            | Opcode::f32_round_half_away_from_zero
            | Opcode::f32_round_half_to_even
            | Opcode::f32_trunc
            | Opcode::f32_fract
            | Opcode::f32_sqrt
            | Opcode::f32_cbrt
            | Opcode::f32_exp
            | Opcode::f32_exp2
            | Opcode::f32_ln
            | Opcode::f32_log2
            | Opcode::f32_log10
            | Opcode::f32_sin
            | Opcode::f32_cos
            | Opcode::f32_tan
            | Opcode::f32_asin
            | Opcode::f32_acos
            | Opcode::f32_atan
            | Opcode::f32_copysign
            | Opcode::f32_pow
            | Opcode::f32_log
            | Opcode::f32_min
            | Opcode::f32_max
            | Opcode::f64_abs
            | Opcode::f64_neg
            | Opcode::f64_ceil
            | Opcode::f64_floor
            | Opcode::f64_round_half_away_from_zero
            | Opcode::f64_round_half_to_even
            | Opcode::f64_trunc
            | Opcode::f64_fract
            | Opcode::f64_sqrt
            | Opcode::f64_cbrt
            | Opcode::f64_exp
            | Opcode::f64_exp2
            | Opcode::f64_ln
            | Opcode::f64_log2
            | Opcode::f64_log10
            | Opcode::f64_sin
            | Opcode::f64_cos
            | Opcode::f64_tan
            | Opcode::f64_asin
            | Opcode::f64_acos
            | Opcode::f64_atan
            | Opcode::f64_copysign
            | Opcode::f64_pow
            | Opcode::f64_log
            | Opcode::f64_min
            | Opcode::f64_max => (offset_param, String::new()),
            // control flow
            Opcode::end => (offset_param, String::new()),
            Opcode::block => {
                let (offset_next, type_idx, local_list_index) =
                    read_param_i32_i32(codes, offset_param);
                (
                    offset_next,
                    format!("type:{:<2}  local:{}", type_idx, local_list_index),
                )
            }
            Opcode::block_nez => {
                let (offset_next, local_idx, offset) = read_param_i32_i32(codes, offset_param);
                (
                    offset_next,
                    format!("local:{:<2}  off:0x{:02x}", local_idx, offset),
                )
            }
            Opcode::block_alt => {
                let (offset_next, type_idx, local_idx, offset) =
                    read_param_i32_i32_i32(codes, offset_param);
                (
                    offset_next,
                    format!(
                        "type:{:<2}  local:{:<2}  off:0x{:02x}",
                        type_idx, local_idx, offset
                    ),
                )
            }
            Opcode::break_ | Opcode::break_nez | Opcode::recur | Opcode::recur_nez => {
                let (offset_next, reversed_index, offset) = read_param_i16_i32(codes, offset_param);
                (
                    offset_next,
                    format!("rev:{:<2}  off:0x{:02x}", reversed_index, offset),
                )
            }
            Opcode::call | Opcode::envcall | Opcode::extcall => {
                let (offset_next, idx) = read_param_i32(codes, offset_param);
                (offset_next, format!("idx:{}", idx))
            }
            Opcode::dyncall | Opcode::syscall => (offset_param, String::new()),
            // host
            Opcode::panic => (offset_param, String::new()),
            Opcode::unreachable | Opcode::debug => {
                let (offset_next, code) = read_param_i32(codes, offset_param);
                (offset_next, format!("code:{}", code))
            }
            Opcode::host_addr_local => {
                let (offset_next, reversed_idx, offset, idx) =
                    read_param_i16_i16_i16(codes, offset_param);
                (
                    offset_next,
                    format!("rev:{:<2}  off:0x{:02x}  idx:{}", reversed_idx, offset, idx),
                )
            }
            Opcode::host_addr_local_offset => {
                let (offset_next, reversed_idx, idx) = read_param_i16_i32(codes, offset_param);
                (offset_next, format!("rev:{:<2}  idx:{}", reversed_idx, idx))
            }
            Opcode::host_addr_data => {
                let (offset_next, offset, idx) = read_param_i16_i32(codes, offset_param);
                (offset_next, format!("off:0x{:02x}  idx:{}", offset, idx))
            }
            Opcode::host_addr_data_offset => {
                let (offset_next, idx) = read_param_i32(codes, offset_param);
                (offset_next, format!("idx:{}", idx))
            }
            Opcode::host_addr_heap => {
                let (offset_next, offset) = read_param_i16(codes, offset_param);
                (offset_next, format!("off:0x{:02x}", offset))
            }
            Opcode::host_copy_heap_to_memory
            | Opcode::host_copy_memory_to_heap
            | Opcode::host_memory_copy => (offset_param, String::new()),
            Opcode::host_addr_function => {
                let (offset_next, idx) = read_param_i32(codes, offset_param);
                (offset_next, format!("idx:{}", idx))
            }
        };

        // format!(...)
        // https://doc.rust-lang.org/std/fmt/

        let mut line = format!("0x{:04x}  ", offset);
        let addr_width = line.len();

        let inst_data = &codes[offset..offset_next];
        let mut chunks = inst_data.chunks(8);

        // format the bytes as the following text:
        //
        // 0x0006  08 04 03 00
        // 0x000a  00 02 05 00  07 00 11 00
        let print_binary = |data: &[u8]| {
            data.iter()
                .enumerate()
                .map(|(idx, byte)| {
                    if idx == 4 {
                        format!("  {:02x}", byte)
                    } else if idx == 0 {
                        format!("{:02x}", byte)
                    } else {
                        format!(" {:02x}", byte)
                    }
                })
                .collect::<Vec<String>>()
                .join("")
        };

        if param_text.is_empty() {
            line.push_str(&format!(
                "{:28}{}",
                print_binary(chunks.next().unwrap()),
                opcode.get_name()
            ));
        } else {
            line.push_str(&format!(
                "{:28}{:16}  {}",
                print_binary(chunks.next().unwrap()),
                opcode.get_name(),
                param_text
            ));
        }

        lines.push(line);

        let indent_text = " ".repeat(addr_width);
        for chunk in chunks {
            lines.push(format!("{}{}", indent_text, print_binary(chunk)));
        }

        // move on
        offset = offset_next;
    }

    lines.join("\n")
}

// opcode, or
// 16 bits instruction
// [opcode]
fn read_opcode(codes: &[u8], offset: usize) -> (usize, Opcode) {
    let opcode_data = &codes[offset..offset + 2];
    let opcode_u16 = u16::from_le_bytes(opcode_data.try_into().unwrap());

    (offset + 2, unsafe {
        std::mem::transmute::<u16, Opcode>(opcode_u16)
    })
}

// 32 bits instruction
// [opcode + i16]
fn read_param_i16(codes: &[u8], offset: usize) -> (usize, u16) {
    let param_data0 = &codes[offset..offset + 2];
    (
        offset + 2,
        u16::from_le_bytes(param_data0.try_into().unwrap()),
    )
}

// 64 bits instruction
// [opcode + padding + i32]
//
// note that 'i32' in function name means a 32-bit integer, which is equivalent to
// the 'uint32_t' in C or 'u32' in Rust. do not confuse it with 'i32' in Rust.
// the same applies to the i8, i16 and i64.
fn read_param_i32(codes: &[u8], offset: usize) -> (usize, u32) {
    let param_data0 = &codes[offset + 2..offset + 6];

    (
        offset + 6,
        u32::from_le_bytes(param_data0.try_into().unwrap()),
    )
}

// 64 bits instruction
// [opcode + i16 + i32]
fn read_param_i16_i32(codes: &[u8], offset: usize) -> (usize, u16, u32) {
    let param_data0 = &codes[offset..offset + 2];
    let param_data1 = &codes[offset + 2..offset + 6];

    (
        offset + 6,
        u16::from_le_bytes(param_data0.try_into().unwrap()),
        u32::from_le_bytes(param_data1.try_into().unwrap()),
    )
}

// 64 bits instruction
// [opcode + i16 + i16 + i16]
fn read_param_i16_i16_i16(codes: &[u8], offset: usize) -> (usize, u16, u16, u16) {
    let param_data0 = &codes[offset..offset + 2];
    let param_data1 = &codes[offset + 2..offset + 4];
    let param_data2 = &codes[offset + 4..offset + 6];

    (
        offset + 6,
        u16::from_le_bytes(param_data0.try_into().unwrap()),
        u16::from_le_bytes(param_data1.try_into().unwrap()),
        u16::from_le_bytes(param_data2.try_into().unwrap()),
    )
}

// 96 bits instruction
// [opcode + padding + i32 + i32]
fn read_param_i32_i32(codes: &[u8], offset: usize) -> (usize, u32, u32) {
    let param_data0 = &codes[offset + 2..offset + 6];
    let param_data1 = &codes[offset + 6..offset + 10];

    (
        offset + 10,
        u32::from_le_bytes(param_data0.try_into().unwrap()),
        u32::from_le_bytes(param_data1.try_into().unwrap()),
    )
}

// 128 bits instruction
// [opcode + padding + i32 + i32 + i32]
fn read_param_i32_i32_i32(codes: &[u8], offset: usize) -> (usize, u32, u32, u32) {
    let param_data0 = &codes[offset + 2..offset + 6];
    let param_data1 = &codes[offset + 6..offset + 10];
    let param_data2 = &codes[offset + 10..offset + 14];

    (
        offset + 14,
        u32::from_le_bytes(param_data0.try_into().unwrap()),
        u32::from_le_bytes(param_data1.try_into().unwrap()),
        u32::from_le_bytes(param_data2.try_into().unwrap()),
    )
}

#[cfg(test)]
mod tests {
    use ancvm_types::opcode::Opcode;

    use crate::{
        bytecode_reader::{print_bytecode_as_binary, print_bytecode_as_text},
        bytecode_writer::BytecodeWriter,
    };

    #[test]
    fn test_print_bytecodes_as_binary() {
        let code0 = BytecodeWriter::new()
            .append_opcode(Opcode::i32_add)
            .append_opcode_i16(Opcode::heap_load64_i64, 0x2)
            .append_opcode_i16(Opcode::heap_store64, 0x3)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0x5, 0x7, 0x11)
            .append_opcode_i16_i16_i16(Opcode::local_store64, 0x13, 0x17, 0x19)
            // padding
            .append_opcode_i16_i32(Opcode::data_load64_i64, 0x23, 0x29)
            .append_opcode_i16_i32(Opcode::data_store64, 0x31, 0x37)
            .append_opcode(Opcode::i32_sub)
            .append_opcode(Opcode::i32_eqz)
            .append_opcode_i16_i32(Opcode::data_load64_i64, 0x41, 0x43)
            .append_opcode_i16_i32(Opcode::data_store64, 0x47, 0x53)
            .append_opcode(Opcode::i32_nez)
            // padding
            .append_opcode_i32(Opcode::i32_imm, 0x59)
            .append_opcode_i32(Opcode::call, 0x61)
            .append_opcode(Opcode::i32_eq)
            // padding
            .append_opcode_i32_i32(Opcode::i64_imm, 0x67, 0x71)
            .append_opcode_i32_i32(Opcode::block, 0x73, 0x79)
            .append_opcode(Opcode::zero)
            // padding
            .append_opcode_i32_i32_i32(Opcode::block_alt, 0x11, 0x13, 0x17)
            .append_opcode_i32_i32(Opcode::block_nez, 0x19, 0x23)
            // end
            .append_opcode(Opcode::end)
            .to_bytes();

        let text = print_bytecode_as_binary(&code0);

        assert_eq!(
            text,
            "\
0x0000  00 07 00 04  02 00 08 04
0x0008  03 00 00 02  05 00 07 00
0x0010  11 00 08 02  13 00 17 00
0x0018  19 00 00 01  00 03 23 00
0x0020  29 00 00 00  08 03 31 00
0x0028  37 00 00 00  01 07 00 06
0x0030  00 03 41 00  43 00 00 00
0x0038  08 03 47 00  53 00 00 00
0x0040  01 06 00 01  80 01 00 00
0x0048  59 00 00 00  00 0b 00 00
0x0050  61 00 00 00  02 06 00 01
0x0058  81 01 00 00  67 00 00 00
0x0060  71 00 00 00  01 0a 00 00
0x0068  73 00 00 00  79 00 00 00
0x0070  01 01 00 01  05 0a 00 00
0x0078  11 00 00 00  13 00 00 00
0x0080  17 00 00 00  04 0a 00 00
0x0088  19 00 00 00  23 00 00 00
0x0090  00 0a"
        );
    }

    #[test]
    fn test_print_bytecodes_as_text() {
        let code0 = BytecodeWriter::new()
            .append_opcode(Opcode::i32_add)
            .append_opcode_i16(Opcode::heap_load64_i64, 0x2)
            .append_opcode_i16(Opcode::heap_store64, 0x3)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0x5, 0x7, 0x11)
            .append_opcode_i16_i16_i16(Opcode::local_store64, 0x13, 0x17, 0x19)
            // padding
            .append_opcode_i16_i32(Opcode::data_load64_i64, 0x23, 0x29)
            .append_opcode_i16_i32(Opcode::data_store64, 0x31, 0x37)
            .append_opcode(Opcode::i32_sub)
            .append_opcode(Opcode::i32_eqz)
            .append_opcode_i16_i32(Opcode::data_load64_i64, 0x41, 0x43)
            .append_opcode_i16_i32(Opcode::data_store64, 0x47, 0x53)
            .append_opcode(Opcode::i32_nez)
            // padding
            .append_opcode_i32(Opcode::i32_imm, 0x59)
            .append_opcode_i32(Opcode::call, 0x61)
            .append_opcode(Opcode::i32_eq)
            // padding
            .append_opcode_i32_i32(Opcode::i64_imm, 0x67, 0x71)
            .append_opcode_i32_i32(Opcode::block, 0x73, 0x79)
            .append_opcode(Opcode::zero)
            // padding
            .append_opcode_i32_i32_i32(Opcode::block_alt, 0x11, 0x13, 0x17)
            .append_opcode_i32_i32(Opcode::block_nez, 0x19, 0x23)
            // end
            .append_opcode(Opcode::end)
            .to_bytes();

        let text = print_bytecode_as_text(&code0);

        assert_eq!(
            text,
            "\
0x0000  00 07                       i32.add
0x0002  00 04 02 00                 heap.load64_i64   off:0x02
0x0006  08 04 03 00                 heap.store64      off:0x03
0x000a  00 02 05 00  07 00 11 00    local.load64_i64  rev:5   off:0x07  idx:17
0x0012  08 02 13 00  17 00 19 00    local.store64     rev:19  off:0x17  idx:25
0x001a  00 01                       nop
0x001c  00 03 23 00  29 00 00 00    data.load64_i64   off:0x23  idx:41
0x0024  08 03 31 00  37 00 00 00    data.store64      off:0x31  idx:55
0x002c  01 07                       i32.sub
0x002e  00 06                       i32.eqz
0x0030  00 03 41 00  43 00 00 00    data.load64_i64   off:0x41  idx:67
0x0038  08 03 47 00  53 00 00 00    data.store64      off:0x47  idx:83
0x0040  01 06                       i32.nez
0x0042  00 01                       nop
0x0044  80 01 00 00  59 00 00 00    i32.imm           0x00000059
0x004c  00 0b 00 00  61 00 00 00    call              idx:97
0x0054  02 06                       i32.eq
0x0056  00 01                       nop
0x0058  81 01 00 00  67 00 00 00    i64.imm           low:0x00000067  high:0x00000071
        71 00 00 00
0x0064  01 0a 00 00  73 00 00 00    block             type:115  local:121
        79 00 00 00
0x0070  01 01                       zero
0x0072  00 01                       nop
0x0074  05 0a 00 00  11 00 00 00    block_alt         type:17  local:19  off:0x17
        13 00 00 00  17 00 00 00
0x0084  04 0a 00 00  19 00 00 00    block_nez         local:25  off:0x23
        23 00 00 00
0x0090  00 0a                       end"
        )
    }
}
