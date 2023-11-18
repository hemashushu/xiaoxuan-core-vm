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
            Opcode::zero | Opcode::drop | Opcode::duplicate | Opcode::swap | Opcode::select_nez => {
                (offset_param, String::new())
            }
            Opcode::i32_imm | Opcode::f32_imm => {
                let (o, v) = read_param_i32(codes, offset_param);
                (o, format!("0x{:08x}", v))
            }
            Opcode::i64_imm | Opcode::f64_imm => {
                let (o, v_low, v_high) = read_param_i32_i32(codes, offset_param);
                (o, format!("low:0x{:08x}  high:0x{:08x}", v_low, v_high))
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
                let (o, reversed_index, offset, index) =
                    read_param_i16_i16_i16(codes, offset_param);
                (
                    o,
                    format!(
                        "rev:{:<2}  off:0x{:02x}  idx:{}",
                        reversed_index, offset, index
                    ),
                )
            }
            //
            Opcode::local_long_load64_i64
            | Opcode::local_long_load32_i32
            | Opcode::local_long_load32_i16_s
            | Opcode::local_long_load32_i16_u
            | Opcode::local_long_load32_i8_s
            | Opcode::local_long_load32_i8_u
            | Opcode::local_long_load64_f64
            | Opcode::local_long_load32_f32
            | Opcode::local_long_store64
            | Opcode::local_long_store32
            | Opcode::local_long_store16
            | Opcode::local_long_store8 => {
                let (o, reversed_index, index) = read_param_i16_i32(codes, offset_param);
                (o, format!("rev:{:<2}  idx:{}", reversed_index, index))
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
                let (o, offset, index) = read_param_i16_i32(codes, offset_param);
                (o, format!("off:0x{:02x}  idx:{}", offset, index))
            }
            //
            Opcode::data_long_load64_i64
            | Opcode::data_long_load32_i32
            | Opcode::data_long_load32_i16_s
            | Opcode::data_long_load32_i16_u
            | Opcode::data_long_load32_i8_s
            | Opcode::data_long_load32_i8_u
            | Opcode::data_long_load64_f64
            | Opcode::data_long_load32_f32
            | Opcode::data_long_store64
            | Opcode::data_long_store32
            | Opcode::data_long_store16
            | Opcode::data_long_store8 => {
                let (o, index) = read_param_i32(codes, offset_param);
                (o, format!("idx:{}", index))
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
                let (o, offset) = read_param_i16(codes, offset_param);
                (o, format!("off:0x{:02x}", offset))
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
                let (o, amount) = read_param_i16(codes, offset_param);
                (o, format!("{}", amount))
            }
            Opcode::i64_add
            | Opcode::i64_sub
            | Opcode::i64_mul
            | Opcode::i64_div_s
            | Opcode::i64_div_u
            | Opcode::i64_rem_s
            | Opcode::i64_rem_u => (offset_param, String::new()),
            Opcode::i64_inc | Opcode::i64_dec => {
                let (o, amount) = read_param_i16(codes, offset_param);
                (o, format!("{}", amount))
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
            | Opcode::i64_trailing_zeros
            | Opcode::i64_count_ones
            | Opcode::i64_shift_left
            | Opcode::i64_shift_right_s
            | Opcode::i64_shift_right_u
            | Opcode::i64_rotate_left
            | Opcode::i64_rotate_right => (offset_param, String::new()),
            // math
            Opcode::f32_abs
            | Opcode::f32_neg
            | Opcode::f32_ceil
            | Opcode::f32_floor
            | Opcode::f32_round_half_away_from_zero
            | Opcode::f32_trunc
            | Opcode::f32_fract
            | Opcode::f32_sqrt
            | Opcode::f32_cbrt
            | Opcode::f32_pow
            | Opcode::f32_exp
            | Opcode::f32_exp2
            | Opcode::f32_ln
            | Opcode::f32_log
            | Opcode::f32_log2
            | Opcode::f32_log10
            | Opcode::f32_sin
            | Opcode::f32_cos
            | Opcode::f32_tan
            | Opcode::f32_asin
            | Opcode::f32_acos
            | Opcode::f32_atan
            | Opcode::f64_abs
            | Opcode::f64_neg
            | Opcode::f64_ceil
            | Opcode::f64_floor
            | Opcode::f64_round_half_away_from_zero
            | Opcode::f64_trunc
            | Opcode::f64_fract
            | Opcode::f64_sqrt
            | Opcode::f64_cbrt
            | Opcode::f64_pow
            | Opcode::f64_exp
            | Opcode::f64_exp2
            | Opcode::f64_ln
            | Opcode::f64_log
            | Opcode::f64_log2
            | Opcode::f64_log10
            | Opcode::f64_sin
            | Opcode::f64_cos
            | Opcode::f64_tan
            | Opcode::f64_asin
            | Opcode::f64_acos
            | Opcode::f64_atan => (offset_param, String::new()),
            // control flow
            Opcode::end => (offset_param, String::new()),
            Opcode::block => {
                let (o, type_idx, local_list_index) = read_param_i32_i32(codes, offset_param);
                (
                    o,
                    format!("type:{:<2}  local:{}", type_idx, local_list_index),
                )
            }
            Opcode::block_nez => {
                let (o, local_idx, offset) = read_param_i32_i32(codes, offset_param);
                (
                    o,
                    format!("local:{:<2}  off:0x{:02x}", local_idx, offset),
                )
            }
            Opcode::block_alt => {
                let (o, type_idx, local_idx, offset) = read_param_i32_i32_i32(codes, offset_param);
                (
                    o,
                    format!(
                        "type:{:<2}  local:{:<2}  off:0x{:02x}",
                        type_idx, local_idx, offset
                    ),
                )
            }
            Opcode::break_ | Opcode::break_nez | Opcode::recur | Opcode::recur_nez => {
                let (o, reversed_index, offset) = read_param_i16_i32(codes, offset_param);
                (
                    o,
                    format!("rev:{:<2}  off:0x{:02x}", reversed_index, offset),
                )
            }
            Opcode::call | Opcode::envcall => {
                let (o, idx) = read_param_i32(codes, offset_param);
                (o, format!("idx:{}", idx))
            }
            Opcode::dyncall | Opcode::syscall | Opcode::extcall => (offset_param, String::new()),
            // machine
            Opcode::nop | Opcode::panic => (offset_param, String::new()),
            Opcode::unreachable | Opcode::debug => {
                let (o, code) = read_param_i32(codes, offset_param);
                (o, format!("code:{}", code))
            }
            Opcode::host_addr_local => {
                let (o, reversed_idx, offset, idx) = read_param_i16_i16_i16(codes, offset_param);
                (
                    o,
                    format!(
                        "rev:{:<2}  off:0x{:02x}  idx:{}",
                        reversed_idx, offset, idx
                    ),
                )
            }
            Opcode::host_addr_local_long => {
                let (o, reversed_idx, idx) = read_param_i16_i32(codes, offset_param);
                (o, format!("rev:{:<2}  idx:{}", reversed_idx, idx))
            }
            Opcode::host_addr_data => {
                let (o, offset, idx) = read_param_i16_i32(codes, offset_param);
                (o, format!("off:0x{:02x}  idx:{}", offset, idx))
            }
            Opcode::host_addr_data_long => {
                let (o, idx) = read_param_i32(codes, offset_param);
                (o, format!("idx:{}", idx))
            }
            Opcode::host_addr_heap => {
                let (o, offset) = read_param_i16(codes, offset_param);
                (o, format!("off:0x{:02x}", offset))
            }
            Opcode::host_copy_from_heap | Opcode::host_copy_to_heap | Opcode::host_addr_func => {
                (offset_param, String::new())
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
