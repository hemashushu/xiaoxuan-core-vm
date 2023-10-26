// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::{collections::HashMap, sync::Once};

use ancvm_types::opcode::Opcode;

// ref:
// https://doc.rust-lang.org/stable/std/collections/struct.HashMap.html
// https://doc.rust-lang.org/std/sync/struct.Once.html
static INIT: Once = Once::new();
pub static mut INSTRUCTION_PROPERTY_TABLE: Option<HashMap<&'static str, InstructionProperty>> =
    None;

// the kind of assembly instruction
#[derive(Debug, PartialEq, Clone)]
pub enum InstructionKind {
    NoParams,
    ParamI32,        // 'i32.imm', 'call', 'ecall'
    ParamI16,        // 'i32.inc', 'i32.dec', 'i64.inc', 'i64.dec'
    ImmI64,          // pesudo instruction, overwrite the original instruction i64.imm
    ImmF32,          // pesudo instruction, overwrite the original instruction f32.imm
    ImmF64,          // pesudo instruction, overwrite the original instruction f64.imm
    LocalAccess,     // includes 'host_addr_local'
    LocalAccessLong, // includes 'host_addr_local_long'
    DataAccess,      // includes 'host_addr_data'
    DataAccessLong,  // includes 'host_addr_data_long'
    HeapAccess,      // includes 'host_addr_heap'
    UnaryOp,
    BinaryOp,
    If,       // pesudo instruction, overwrite the original control flow instructions
    Cond,     // pesudo instruction, overwrite the original control flow instructions
    Branch,   // pesudo instruction, overwrite the original control flow instructions
    Case,     // pesudo sub-instruction for instruction 'branch'
    Default,  // pesudo sub-instruction for instruction 'branch'
    For,      // pesudo instruction, overwrite the original control flow instructions
    Sequence, // for node 'code', 'break', 'recur', 'return', 'tailcall'
}

#[derive(Debug, PartialEq, Clone)]
pub struct InstructionProperty {
    pub kind: InstructionKind,
    pub opcode: Opcode,
    pub operand_count: u8,
}

impl InstructionProperty {
    pub fn new(inst_type: InstructionKind, opcode: Opcode, operand_count: u8) -> Self {
        Self {
            kind: inst_type,
            opcode,
            operand_count,
        }
    }
}

pub fn init_instruction_table() {
    INIT.call_once(|| {
        init_instruction_table_internal();
    });
}

fn init_instruction_table_internal() {
    let mut table: HashMap<&'static str, InstructionProperty> = HashMap::new();

    let mut im =
        |name: &'static str, inst_type: InstructionKind, opcode: Opcode, operand_count: u8| {
            table.insert(
                name,
                InstructionProperty::new(inst_type, opcode, operand_count),
            );
        };

    // fundamental
    im("zero", InstructionKind::NoParams, Opcode::zero, 0);
    im("drop", InstructionKind::NoParams, Opcode::drop, 1);
    im("duplicate", InstructionKind::NoParams, Opcode::duplicate, 1);
    im("swap", InstructionKind::NoParams, Opcode::swap, 2);
    im(
        "select_nez",
        InstructionKind::NoParams,
        Opcode::select_nez,
        3,
    );

    im("i32.imm", InstructionKind::ParamI32, Opcode::i32_imm, 0);

    // note:
    // 'i64.imm', 'f32.imm', 'f64.imm' are replaced with pesudo instructions

    // load variables
    im(
        "local.load",
        InstructionKind::LocalAccess,
        Opcode::local_load,
        0,
    );
    im(
        "local.load32",
        InstructionKind::LocalAccess,
        Opcode::local_load32,
        0,
    );
    im(
        "local.load32_i16_s",
        InstructionKind::LocalAccess,
        Opcode::local_load32_i16_s,
        0,
    );
    im(
        "local.load32_i16_u",
        InstructionKind::LocalAccess,
        Opcode::local_load32_i16_u,
        0,
    );
    im(
        "local.load32_i8_s",
        InstructionKind::LocalAccess,
        Opcode::local_load32_i8_s,
        0,
    );
    im(
        "local.load32_i8_u",
        InstructionKind::LocalAccess,
        Opcode::local_load32_i8_u,
        0,
    );
    im(
        "local.load_f64",
        InstructionKind::LocalAccess,
        Opcode::local_load_f64,
        0,
    );
    im(
        "local.load32_f32",
        InstructionKind::LocalAccess,
        Opcode::local_load32_f32,
        0,
    );
    im(
        "local.store",
        InstructionKind::LocalAccess,
        Opcode::local_store,
        1,
    );
    im(
        "local.store32",
        InstructionKind::LocalAccess,
        Opcode::local_store32,
        1,
    );
    im(
        "local.store16",
        InstructionKind::LocalAccess,
        Opcode::local_store16,
        1,
    );
    im(
        "local.store8    ",
        InstructionKind::LocalAccess,
        Opcode::local_store8,
        1,
    );

    im(
        "local.long_load",
        InstructionKind::LocalAccessLong,
        Opcode::local_long_load,
        1,
    );
    im(
        "local.long_load32",
        InstructionKind::LocalAccessLong,
        Opcode::local_long_load32,
        1,
    );
    im(
        "local.long_load32_i16_s",
        InstructionKind::LocalAccessLong,
        Opcode::local_long_load32_i16_s,
        1,
    );
    im(
        "local.long_load32_i16_u",
        InstructionKind::LocalAccessLong,
        Opcode::local_long_load32_i16_u,
        1,
    );
    im(
        "local.long_load32_i8_s",
        InstructionKind::LocalAccessLong,
        Opcode::local_long_load32_i8_s,
        1,
    );
    im(
        "local.long_load32_i8_u",
        InstructionKind::LocalAccessLong,
        Opcode::local_long_load32_i8_u,
        1,
    );
    im(
        "local.long_load_f64",
        InstructionKind::LocalAccessLong,
        Opcode::local_long_load_f64,
        1,
    );
    im(
        "local.long_load32_f32",
        InstructionKind::LocalAccessLong,
        Opcode::local_long_load32_f32,
        1,
    );
    im(
        "local.long_store",
        InstructionKind::LocalAccessLong,
        Opcode::local_long_store,
        2,
    );
    im(
        "local.long_store32",
        InstructionKind::LocalAccessLong,
        Opcode::local_long_store32,
        2,
    );
    im(
        "local.long_store16",
        InstructionKind::LocalAccessLong,
        Opcode::local_long_store16,
        2,
    );
    im(
        "local.long_store8    ",
        InstructionKind::LocalAccessLong,
        Opcode::local_long_store8,
        2,
    );

    // data
    im(
        "data.load",
        InstructionKind::DataAccess,
        Opcode::data_load,
        0,
    );
    im(
        "data.load32",
        InstructionKind::DataAccess,
        Opcode::data_load32,
        0,
    );
    im(
        "data.load32_i16_s",
        InstructionKind::DataAccess,
        Opcode::data_load32_i16_s,
        0,
    );
    im(
        "data.load32_i16_u",
        InstructionKind::DataAccess,
        Opcode::data_load32_i16_u,
        0,
    );
    im(
        "data.load32_i8_s",
        InstructionKind::DataAccess,
        Opcode::data_load32_i8_s,
        0,
    );
    im(
        "data.load32_i8_u",
        InstructionKind::DataAccess,
        Opcode::data_load32_i8_u,
        0,
    );
    im(
        "data.load_f64",
        InstructionKind::DataAccess,
        Opcode::data_load_f64,
        0,
    );
    im(
        "data.load32_f32",
        InstructionKind::DataAccess,
        Opcode::data_load32_f32,
        0,
    );
    im(
        "data.store",
        InstructionKind::DataAccess,
        Opcode::data_store,
        1,
    );
    im(
        "data.store32",
        InstructionKind::DataAccess,
        Opcode::data_store32,
        1,
    );
    im(
        "data.store16",
        InstructionKind::DataAccess,
        Opcode::data_store16,
        1,
    );
    im(
        "data.store8    ",
        InstructionKind::DataAccess,
        Opcode::data_store8,
        1,
    );

    im(
        "data.long_load",
        InstructionKind::DataAccessLong,
        Opcode::data_long_load,
        1,
    );
    im(
        "data.long_load32",
        InstructionKind::DataAccessLong,
        Opcode::data_long_load32,
        1,
    );
    im(
        "data.long_load32_i16_s",
        InstructionKind::DataAccessLong,
        Opcode::data_long_load32_i16_s,
        1,
    );
    im(
        "data.long_load32_i16_u",
        InstructionKind::DataAccessLong,
        Opcode::data_long_load32_i16_u,
        1,
    );
    im(
        "data.long_load32_i8_s",
        InstructionKind::DataAccessLong,
        Opcode::data_long_load32_i8_s,
        1,
    );
    im(
        "data.long_load32_i8_u",
        InstructionKind::DataAccessLong,
        Opcode::data_long_load32_i8_u,
        1,
    );
    im(
        "data.long_load_f64",
        InstructionKind::DataAccessLong,
        Opcode::data_long_load_f64,
        1,
    );
    im(
        "data.long_load32_f32",
        InstructionKind::DataAccessLong,
        Opcode::data_long_load32_f32,
        1,
    );
    im(
        "data.long_store",
        InstructionKind::DataAccessLong,
        Opcode::data_long_store,
        2,
    );
    im(
        "data.long_store32",
        InstructionKind::DataAccessLong,
        Opcode::data_long_store32,
        2,
    );
    im(
        "data.long_store16",
        InstructionKind::DataAccessLong,
        Opcode::data_long_store16,
        2,
    );
    im(
        "data.long_store8",
        InstructionKind::DataAccessLong,
        Opcode::data_long_store8,
        2,
    );

    // heap
    im(
        "heap.load",
        InstructionKind::HeapAccess,
        Opcode::heap_load,
        1,
    );
    im(
        "heap.load32",
        InstructionKind::HeapAccess,
        Opcode::heap_load32,
        1,
    );
    im(
        "heap.load32_i16_s",
        InstructionKind::HeapAccess,
        Opcode::heap_load32_i16_s,
        1,
    );
    im(
        "heap.load32_i16_u",
        InstructionKind::HeapAccess,
        Opcode::heap_load32_i16_u,
        1,
    );
    im(
        "heap.load32_i8_s",
        InstructionKind::HeapAccess,
        Opcode::heap_load32_i8_s,
        1,
    );
    im(
        "heap.load32_i8_u",
        InstructionKind::HeapAccess,
        Opcode::heap_load32_i8_u,
        1,
    );
    im(
        "heap.load_f64",
        InstructionKind::HeapAccess,
        Opcode::heap_load_f64,
        1,
    );
    im(
        "heap.load32_f32",
        InstructionKind::HeapAccess,
        Opcode::heap_load32_f32,
        1,
    );
    im(
        "heap.store",
        InstructionKind::HeapAccess,
        Opcode::heap_store,
        2,
    );
    im(
        "heap.store32",
        InstructionKind::HeapAccess,
        Opcode::heap_store32,
        2,
    );
    im(
        "heap.store16",
        InstructionKind::HeapAccess,
        Opcode::heap_store16,
        2,
    );
    im(
        "heap.store8    ",
        InstructionKind::HeapAccess,
        Opcode::heap_store8,
        2,
    );

    // conversion
    im(
        "i32.trunc_i64",
        InstructionKind::NoParams,
        Opcode::i32_trunc_i64,
        1,
    );
    im(
        "i64.extend_i32_s",
        InstructionKind::NoParams,
        Opcode::i64_extend_i32_s,
        1,
    );
    im(
        "i64.extend_i32_u",
        InstructionKind::NoParams,
        Opcode::i64_extend_i32_u,
        1,
    );
    im(
        "f32.demote_f64",
        InstructionKind::NoParams,
        Opcode::f32_demote_f64,
        1,
    );
    im(
        "f64.promote_f32",
        InstructionKind::NoParams,
        Opcode::f64_promote_f32,
        1,
    );

    im(
        "i32.convert_f32_s",
        InstructionKind::NoParams,
        Opcode::i32_convert_f32_s,
        1,
    );
    im(
        "i32.convert_f32_u",
        InstructionKind::NoParams,
        Opcode::i32_convert_f32_u,
        1,
    );
    im(
        "i32.convert_f64_s",
        InstructionKind::NoParams,
        Opcode::i32_convert_f64_s,
        1,
    );
    im(
        "i32.convert_f64_u",
        InstructionKind::NoParams,
        Opcode::i32_convert_f64_u,
        1,
    );
    im(
        "i64.convert_f32_s",
        InstructionKind::NoParams,
        Opcode::i64_convert_f32_s,
        1,
    );
    im(
        "i64.convert_f32_u",
        InstructionKind::NoParams,
        Opcode::i64_convert_f32_u,
        1,
    );
    im(
        "i64.convert_f64_s",
        InstructionKind::NoParams,
        Opcode::i64_convert_f64_s,
        1,
    );
    im(
        "i64.convert_f64_u",
        InstructionKind::NoParams,
        Opcode::i64_convert_f64_u,
        1,
    );

    im(
        "f32.convert_i32_s",
        InstructionKind::NoParams,
        Opcode::f32_convert_i32_s,
        1,
    );
    im(
        "f32.convert_i32_u",
        InstructionKind::NoParams,
        Opcode::f32_convert_i32_u,
        1,
    );
    im(
        "f32.convert_i64_s",
        InstructionKind::NoParams,
        Opcode::f32_convert_i64_s,
        1,
    );
    im(
        "f32.convert_i64_u",
        InstructionKind::NoParams,
        Opcode::f32_convert_i64_u,
        1,
    );
    im(
        "f64.convert_i32_s",
        InstructionKind::NoParams,
        Opcode::f64_convert_i32_s,
        1,
    );
    im(
        "f64.convert_i32_u",
        InstructionKind::NoParams,
        Opcode::f64_convert_i32_u,
        1,
    );
    im(
        "f64.convert_i64_s",
        InstructionKind::NoParams,
        Opcode::f64_convert_i64_s,
        1,
    );
    im(
        "f64.convert_i64_u",
        InstructionKind::NoParams,
        Opcode::f64_convert_i64_u,
        1,
    );

    // comparsion
    im("i32.eqz", InstructionKind::NoParams, Opcode::i32_eqz, 1);
    im("i32.nez", InstructionKind::NoParams, Opcode::i32_nez, 1);
    im("i32.eq", InstructionKind::NoParams, Opcode::i32_eq, 2);
    im("i32.ne", InstructionKind::NoParams, Opcode::i32_ne, 2);
    im("i32.lt_s", InstructionKind::NoParams, Opcode::i32_lt_s, 2);
    im("i32.lt_u", InstructionKind::NoParams, Opcode::i32_lt_u, 2);
    im("i32.gt_s", InstructionKind::NoParams, Opcode::i32_gt_s, 2);
    im("i32.gt_u", InstructionKind::NoParams, Opcode::i32_gt_u, 2);
    im("i32.le_s", InstructionKind::NoParams, Opcode::i32_le_s, 2);
    im("i32.le_u", InstructionKind::NoParams, Opcode::i32_le_u, 2);
    im("i32.ge_s", InstructionKind::NoParams, Opcode::i32_ge_s, 2);
    im("i32.ge_u", InstructionKind::NoParams, Opcode::i32_ge_u, 2);

    im("i64.eqz", InstructionKind::NoParams, Opcode::i64_eqz, 1);
    im("i64.nez", InstructionKind::NoParams, Opcode::i64_nez, 1);
    im("i64.eq", InstructionKind::NoParams, Opcode::i64_eq, 2);
    im("i64.ne", InstructionKind::NoParams, Opcode::i64_ne, 2);
    im("i64.lt_s", InstructionKind::NoParams, Opcode::i64_lt_s, 2);
    im("i64.lt_u", InstructionKind::NoParams, Opcode::i64_lt_u, 2);
    im("i64.gt_s", InstructionKind::NoParams, Opcode::i64_gt_s, 2);
    im("i64.gt_u", InstructionKind::NoParams, Opcode::i64_gt_u, 2);
    im("i64.le_s", InstructionKind::NoParams, Opcode::i64_le_s, 2);
    im("i64.le_u", InstructionKind::NoParams, Opcode::i64_le_u, 2);
    im("i64.ge_s", InstructionKind::NoParams, Opcode::i64_ge_s, 2);
    im("i64.ge_u", InstructionKind::NoParams, Opcode::i64_ge_u, 2);

    im("f32.eq", InstructionKind::NoParams, Opcode::f32_eq, 2);
    im("f32.ne", InstructionKind::NoParams, Opcode::f32_ne, 2);
    im("f32.lt", InstructionKind::NoParams, Opcode::f32_lt, 2);
    im("f32.gt", InstructionKind::NoParams, Opcode::f32_gt, 2);
    im("f32.le", InstructionKind::NoParams, Opcode::f32_le, 2);
    im("f32.ge", InstructionKind::NoParams, Opcode::f32_ge, 2);

    im("f64.eq", InstructionKind::NoParams, Opcode::f64_eq, 2);
    im("f64.ne", InstructionKind::NoParams, Opcode::f64_ne, 2);
    im("f64.lt", InstructionKind::NoParams, Opcode::f64_lt, 2);
    im("f64.gt", InstructionKind::NoParams, Opcode::f64_gt, 2);
    im("f64.le", InstructionKind::NoParams, Opcode::f64_le, 2);
    im("f64.ge", InstructionKind::NoParams, Opcode::f64_ge, 2);

    // arithmetic
    im("i32.add", InstructionKind::NoParams, Opcode::i32_add, 2);
    im("i32.sub", InstructionKind::NoParams, Opcode::i32_sub, 2);
    im("i32.mul", InstructionKind::NoParams, Opcode::i32_mul, 2);
    im("i32.div_s", InstructionKind::NoParams, Opcode::i32_div_s, 2);
    im("i32.div_u", InstructionKind::NoParams, Opcode::i32_div_u, 2);
    im("i32.rem_s", InstructionKind::NoParams, Opcode::i32_rem_s, 2);
    im("i32.rem_u", InstructionKind::NoParams, Opcode::i32_rem_u, 2);
    im("i32.inc", InstructionKind::ParamI16, Opcode::i32_inc, 1);
    im("i32.dec", InstructionKind::ParamI16, Opcode::i32_dec, 1);

    im("i64.add", InstructionKind::NoParams, Opcode::i64_add, 2);
    im("i64.sub", InstructionKind::NoParams, Opcode::i64_sub, 2);
    im("i64.mul", InstructionKind::NoParams, Opcode::i64_mul, 2);
    im("i64.div_s", InstructionKind::NoParams, Opcode::i64_div_s, 2);
    im("i64.div_u", InstructionKind::NoParams, Opcode::i64_div_u, 2);
    im("i64.rem_s", InstructionKind::NoParams, Opcode::i64_rem_s, 2);
    im("i64.rem_u", InstructionKind::NoParams, Opcode::i64_rem_u, 2);
    im("i64.inc", InstructionKind::ParamI16, Opcode::i64_inc, 1);
    im("i64.dec", InstructionKind::ParamI16, Opcode::i64_dec, 1);

    im("f32.add", InstructionKind::NoParams, Opcode::f32_add, 2);
    im("f32.sub", InstructionKind::NoParams, Opcode::f32_sub, 2);
    im("f32.mul", InstructionKind::NoParams, Opcode::f32_mul, 2);
    im("f32.div", InstructionKind::NoParams, Opcode::f32_div, 2);

    im("f64.add", InstructionKind::NoParams, Opcode::f64_add, 2);
    im("f64.sub", InstructionKind::NoParams, Opcode::f64_sub, 2);
    im("f64.mul", InstructionKind::NoParams, Opcode::f64_mul, 2);
    im("f64.div", InstructionKind::NoParams, Opcode::f64_div, 2);

    // bitwise
    im("i32.and", InstructionKind::NoParams, Opcode::i32_and, 2);
    im("i32.or", InstructionKind::NoParams, Opcode::i32_or, 2);
    im("i32.xor", InstructionKind::NoParams, Opcode::i32_xor, 2);
    im("i32.not", InstructionKind::NoParams, Opcode::i32_not, 1);
    im(
        "i32.leading_zeros",
        InstructionKind::NoParams,
        Opcode::i32_leading_zeros,
        1,
    );
    im(
        "i32.trailing_zeros",
        InstructionKind::NoParams,
        Opcode::i32_trailing_zeros,
        1,
    );
    im(
        "i32.count_ones",
        InstructionKind::NoParams,
        Opcode::i32_count_ones,
        1,
    );
    im(
        "i32.shift_left",
        InstructionKind::NoParams,
        Opcode::i32_shift_left,
        2,
    );
    im(
        "i32.shift_right_s",
        InstructionKind::NoParams,
        Opcode::i32_shift_right_s,
        2,
    );
    im(
        "i32.shift_right_u",
        InstructionKind::NoParams,
        Opcode::i32_shift_right_u,
        2,
    );
    im(
        "i32.rotate_left",
        InstructionKind::NoParams,
        Opcode::i32_rotate_left,
        2,
    );
    im(
        "i32.rotate_right",
        InstructionKind::NoParams,
        Opcode::i32_rotate_right,
        2,
    );

    im("i64.and", InstructionKind::NoParams, Opcode::i64_and, 2);
    im("i64.or", InstructionKind::NoParams, Opcode::i64_or, 2);
    im("i64.xor", InstructionKind::NoParams, Opcode::i64_xor, 2);
    im("i64.not", InstructionKind::NoParams, Opcode::i64_not, 1);
    im(
        "i64.leading_zeros",
        InstructionKind::NoParams,
        Opcode::i64_leading_zeros,
        1,
    );
    im(
        "i64.trailing_zeros",
        InstructionKind::NoParams,
        Opcode::i64_trailing_zeros,
        1,
    );
    im(
        "i64.count_ones",
        InstructionKind::NoParams,
        Opcode::i64_count_ones,
        1,
    );
    im(
        "i64.shift_left",
        InstructionKind::NoParams,
        Opcode::i64_shift_left,
        2,
    );
    im(
        "i64.shift_right_s",
        InstructionKind::NoParams,
        Opcode::i64_shift_right_s,
        2,
    );
    im(
        "i64.shift_right_u",
        InstructionKind::NoParams,
        Opcode::i64_shift_right_u,
        2,
    );
    im(
        "i64.rotate_left",
        InstructionKind::NoParams,
        Opcode::i64_rotate_left,
        2,
    );
    im(
        "i64.rotate_right",
        InstructionKind::NoParams,
        Opcode::i64_rotate_right,
        2,
    );

    // math
    im("f32.abs", InstructionKind::NoParams, Opcode::f32_abs, 1);
    im("f32.neg", InstructionKind::NoParams, Opcode::f32_neg, 1);
    im("f32.ceil", InstructionKind::NoParams, Opcode::f32_ceil, 1);
    im("f32.floor", InstructionKind::NoParams, Opcode::f32_floor, 1);
    im(
        "f32.round_half_away_from_zero",
        InstructionKind::NoParams,
        Opcode::f32_round_half_away_from_zero,
        1,
    );
    im("f32.trunc", InstructionKind::NoParams, Opcode::f32_trunc, 1);
    im("f32.fract", InstructionKind::NoParams, Opcode::f32_fract, 1);
    im("f32.sqrt", InstructionKind::NoParams, Opcode::f32_sqrt, 1);
    im("f32.cbrt", InstructionKind::NoParams, Opcode::f32_cbrt, 1);
    im("f32.pow", InstructionKind::NoParams, Opcode::f32_pow, 2); // 2 operands
    im("f32.exp", InstructionKind::NoParams, Opcode::f32_exp, 1);
    im("f32.exp2", InstructionKind::NoParams, Opcode::f32_exp2, 1);
    im("f32.ln", InstructionKind::NoParams, Opcode::f32_ln, 1);
    im("f32.log", InstructionKind::NoParams, Opcode::f32_log, 2); // 2 operands
    im("f32.log2", InstructionKind::NoParams, Opcode::f32_log2, 1);
    im("f32.log10", InstructionKind::NoParams, Opcode::f32_log10, 1);
    im("f32.sin", InstructionKind::NoParams, Opcode::f32_sin, 1);
    im("f32.cos", InstructionKind::NoParams, Opcode::f32_cos, 1);
    im("f32.tan", InstructionKind::NoParams, Opcode::f32_tan, 1);
    im("f32.asin", InstructionKind::NoParams, Opcode::f32_asin, 1);
    im("f32.acos", InstructionKind::NoParams, Opcode::f32_acos, 1);
    im("f32.atan", InstructionKind::NoParams, Opcode::f32_atan, 1);

    im("f64.abs", InstructionKind::NoParams, Opcode::f64_abs, 1);
    im("f64.neg", InstructionKind::NoParams, Opcode::f64_neg, 1);
    im("f64.ceil", InstructionKind::NoParams, Opcode::f64_ceil, 1);
    im("f64.floor", InstructionKind::NoParams, Opcode::f64_floor, 1);
    im(
        "f64.round_half_away_from_zero",
        InstructionKind::NoParams,
        Opcode::f64_round_half_away_from_zero,
        1,
    );
    im("f64.trunc", InstructionKind::NoParams, Opcode::f64_trunc, 1);
    im("f64.fract", InstructionKind::NoParams, Opcode::f64_fract, 1);
    im("f64.sqrt", InstructionKind::NoParams, Opcode::f64_sqrt, 1);
    im("f64.cbrt", InstructionKind::NoParams, Opcode::f64_cbrt, 1);
    im("f64.pow", InstructionKind::NoParams, Opcode::f64_pow, 2); // 2 operands
    im("f64.exp", InstructionKind::NoParams, Opcode::f64_exp, 1);
    im("f64.exp2", InstructionKind::NoParams, Opcode::f64_exp2, 1);
    im("f64.ln", InstructionKind::NoParams, Opcode::f64_ln, 1);
    im("f64.log", InstructionKind::NoParams, Opcode::f64_log, 2); // 2 operands
    im("f64.log2", InstructionKind::NoParams, Opcode::f64_log2, 1);
    im("f64.log10", InstructionKind::NoParams, Opcode::f64_log10, 1);
    im("f64.sin", InstructionKind::NoParams, Opcode::f64_sin, 1);
    im("f64.cos", InstructionKind::NoParams, Opcode::f64_cos, 1);
    im("f64.tan", InstructionKind::NoParams, Opcode::f64_tan, 1);
    im("f64.asin", InstructionKind::NoParams, Opcode::f64_asin, 1);
    im("f64.acos", InstructionKind::NoParams, Opcode::f64_acos, 1);
    im("f64.atan", InstructionKind::NoParams, Opcode::f64_atan, 1);

    // control flow
    // note: all instructions in the 'control flow' catalog are replaced with pesudo instructions

    // function call
    im("call", InstructionKind::ParamI32, Opcode::dcall, 0);
    im("dcall", InstructionKind::NoParams, Opcode::dcall, 1);
    im("eall", InstructionKind::ParamI32, Opcode::dcall, 0);

    // host
    im("nop", InstructionKind::NoParams, Opcode::nop, 0);
    im("panic", InstructionKind::NoParams, Opcode::debug, 0);
    im("debug", InstructionKind::NoParams, Opcode::debug, 0);

    im(
        "host_addr_local",
        InstructionKind::LocalAccess,
        Opcode::host_addr_local,
        0,
    );
    im(
        "host_addr_local_long",
        InstructionKind::LocalAccessLong,
        Opcode::host_addr_local_long,
        1,
    );
    im(
        "host_addr_data",
        InstructionKind::DataAccess,
        Opcode::host_addr_data,
        0,
    );
    im(
        "host_addr_data_long",
        InstructionKind::DataAccessLong,
        Opcode::host_addr_data_long,
        1,
    );
    im(
        "host_addr_heap",
        InstructionKind::HeapAccess,
        Opcode::host_addr_heap,
        1,
    );

    // pesudo instructions
    im("if", InstructionKind::If, Opcode::nop, 0);
    im("cond", InstructionKind::Cond, Opcode::nop, 0);
    im("branch", InstructionKind::Branch, Opcode::nop, 0);
    im("case", InstructionKind::Case, Opcode::nop, 0);
    im("default", InstructionKind::Default, Opcode::nop, 0);
    im("for", InstructionKind::For, Opcode::nop, 0);
    im("i64.imm", InstructionKind::ImmI64, Opcode::i64_imm, 0);
    im("f32.imm", InstructionKind::ImmF32, Opcode::f32_imm, 0);
    im("f64.imm", InstructionKind::ImmF64, Opcode::f64_imm, 0);
    im("code", InstructionKind::Sequence, Opcode::nop, 0);
    im("break", InstructionKind::Sequence, Opcode::nop, 0);
    im("return", InstructionKind::Sequence, Opcode::nop, 0);
    im("recur", InstructionKind::Sequence, Opcode::nop, 0);
    im("tailcall", InstructionKind::Sequence, Opcode::nop, 0);

    unsafe { INSTRUCTION_PROPERTY_TABLE = Some(table) };
}
