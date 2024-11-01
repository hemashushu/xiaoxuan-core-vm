// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// the data types that VM internal supports
// ----------------------------------------
//
// - i64
// - f64
// - i32
// - f32

// data memory layout
// ------------------
//
// the default implement of XiaoXuan Core VM is stack-base, which its operands are
// 8-bytes raw data, the data presentation in memory:
//
//    MSB                             LSB
// 64 |---------------------------------| 0
//    |   16     16      16     8    8  | bits
//    |-------|-------|-------|----|----|
//    |               i64               | <-- native data type
//    |---------------------------------|
//    | sign-extend   |        i32      |
//    |---------------------------------|
//    | sign-extend           |   i16   |
//    |---------------------------------|
//    | sign-extend                | i8 |
//    |---------------------------------|
//    |               f64               | <-- native data type
//    |---------------------------------|
//    | undefined     |        f32      |
//    |---------------------------------|
//
// all i8/i16/i32 integers loaded from memory will be signed-extended to i64

// the floating-point number
// -------------------------
//
// like most processors and VM, f32/f64 is stored with
// IEEE 754-2008 format.
// e.g. the f32 encoding:
//
//           MSB                                  LSB
//           sign    exponent (8 bits)   fraction (23 bits)                                 implicit leading number 1
//           ----    -----------------   ------------------                                 |
//           |       |                   |                                                  v
// format    0       00000000            0000000000 0000000000 000     value = (-1)^sign * (1 + fraction) * 2^(exponent-offset), offset = 127 for f32, 1023 for f64
// example   1       10000001            0100000000 0000000000 000     value = (-1)^1 * (1 + 0*2^(-1) + 1*2^(-2)) * 2^(129-127) = -1 * 1.25 * 4 = -5.0
// example   0       01111100            0100000000 0000000000 000     value = (-1)^0 * 1.25 * 2^(-3) = 0.15625
//
// support?
//  Y        -       00000001--\
//                   11111110--/         ---------- ---------- ---     normal number
//  Y        0       00000000            0000000000 0000000000 000     value = +0
//  N        1       00000000            0000000000 0000000000 000     value = -0
//  Y        -       00000000            ---------- ---------- ---     subnormal number (i.e., numbers between 0 and MIN)
//  N        0       11111111            0000000000 0000000000 000     value = +Infinity
//  N        1       11111111            0000000000 0000000000 000     value = -Infinity
//  N        -       11111111            0-not-all- -zero----- ---     NaN (SNaN)
//  N        -       11111111            1-------- ----------- ---     NaN (QNaN)
//
// ref:
// convert floating-point to decimal/hexadecimal
// https://www.h-schmidt.net/FloatConverter/IEEE754.html

// unsupported floating-point variants
// -----------------------------------
//
// in addition to the normal floating-point numbers,
// there are some special values (variants):
//
// - NaN
// - +Infinity, -Infinity
// - +0, -0
//
// these variants make the programming language become complex and
// sometimes make problems unpredictable, for example:
//
// - NaN is not comparable, everything includes itself compare with NaN
//   result false, e.g.
//   - NaN != NaN
//   - (NaN < 0) == false
//   - (NaN > 0) == false
//   - (a != b) cannot assert that !(a == b)
//
// - the 0.0, -0.0, +Inf and -Inf (Inf means infinity) are complex also, e.g.
//   - 1.0 ÷ 0.0 = +Inf
//   - -1.0 ÷ 0.0 = 1.0 ÷ -0.0 = -Inf
//   - 0.0 ÷ 0.0 = NaN
//   - 0.0 == -0.0
//   - +Inf != -Inf
//
// ref:
//
// - https://en.wikipedia.org/wiki/Floating-point_arithmetic
// - https://en.wikipedia.org/wiki/IEEE_754
// - https://en.wikipedia.org/wiki/IEEE_754-2008_revision
// - https://en.wikipedia.org/wiki/Signed_zero
// - https://en.wikipedia.org/wiki/Subnormal_number
// - https://en.wikipedia.org/wiki/Single-precision_floating-point_format
// - https://en.wikipedia.org/wiki/Half-precision_floating-point_format
// - https://doc.rust-lang.org/std/primitive.f32.html
//
// to simplify the XiaoXuan Core programming language, the f32/f64 in XiaoXuan Core VM
// only support the normal (includes subnormal) floating-point number and +0,
// the VM will simply throw an exception when
// other variants (such as NaN, -0.0, +inf, -Inf) are encountered.
//
// -  0.0 = 0x0000_0000   (Y)
// - -0.0 = 0x8000_0000   (N)
// -  nan = 0xffc0_0000   (N)
// - +inf = 0x7f80_0000   (N)
// - -inf = 0xff80_0000   (N)
//
// when load data from memory as floating-point number, there are some checking:
// 1. exponent between (00000001) and (11111110): pass
// 2. exponent is zero, if the sign bit is zero: pass
// 3. failed.
//
// in other words, the +/-Infinity, -0, NaN, will cause the VM to throw exceptions.

// the boolean type
// ----------------
//
// the boolean value is represented by a i64 number:
// - TRUE, the number is `1:i64`,
// - FALSE, the number is `0:i64`.
//
// when converts integers into boolean, 0:i32 and 0:i64 are both treated as FALSE,
// and all other i32/i64 non-zero are treated as TRUE.

pub const MAX_OPCODE_NUMBER: usize = 0x480;

// the instruction schemes
// -----------------------
//
// XiaoXuan Core VM instructions are NOT fixed-length code.
// there are 16, 32, 64, 96 and 128 bits length instructions,
//
// - 16 bits:
//   instructions without parameters, such as `eqz_i32`.
// - 32 bits:
//   instructions with 1 parameter, such as `add_imm_i32`.
//   16 bits opcode + 16 bits parameter
// - 64 bits:
//   instructions with 1 parameter, such as `imm_i32`.
//   16 bits opcode + (16 bits padding) + 32 bits parameter (ALIGN 4-byte alignment require)
// - 64 bits:
//   instructions with 2 parameters, such as `data_load_i64`.
//   16 bits opcode + 16 bits parameter 0 + 32 bits parameter 1 (ALIGN 4-byte alignment require)
// - 64 bits:
//   instructions with 3 parameter, such as `local_load_i64`.
//   16 bits opcode + 16 bits parameter 1 + 16 bits parameter 2 + 16 bits parameter 3
// - 96 bits
//   instructions with 2 parameters, such as `block`.
//   16 bits opcode + (16 bits padding) + 32 bits parameter 0 + 32 bits parameter 1 (ALIGN 4-byte alignment require)
// - 128 bits
//   instructions with 3 parameters, such as `block_alt`
//   16 bits opcode + (16 bits padding) + 32 bits parameter 0 + 32 bits parameter 1 + 32 bits parameter 2 (ALIGN 4-byte alignment require)
//
// note that a `nop` instruction will be inserted automatically before
// an instruction which contains `i32` parameters to achieve 32 bits (4-byte) alignment.

// the simplified schemes:
//
// - [opcode i16]                                                                               ;; 16-bit
// - [opcode i16] - [param i16    ]                                                             ;; 32-bit
// - [opcode i16] - [pading 16-bit] + [param i32              ]                                 ;; 64-bit
// - [opcode i16] - [param i16    ] + [param i32              ]                                 ;; 64-bit
// - [opcode i16] - [param i16    ] + [param i16] + [param i16]                                 ;; 64-bit
// - [opcode i16] - [pading 16-bit] + [param i32              ] + [param i32]                   ;; 96-bit
// - [opcode i16] - [pading 16-bit] + [param i32              ] + [param i32] + [param i32]     ;; 128-bit
//
// the opcode scheme:
//
// MSB           LSB
// 00000000 00000000
// -------- --------
// |        |
// |        \ items
// |
// \ catalogs
//
#[repr(u16)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum Opcode {
    // instruction to do nothing,
    // it's usually used for padding instructions to archieve 32 bits (4-byte) alignment.
    // () -> ()
    nop = 0x100,

    // the immediately number will be sign extend to i64
    // (param immediate_number:i32) -> i64
    imm_i32 = 0x140,

    // imm_i64, imm_f32 and imm_f64 are actually pesudo instructions,
    // because there is no i64/f32/f64 parameters in this ISA.
    // some ISA (VM or real machine) place the immediate numbers in a list of constants
    // in the program image, and then load the constants by address to archieve the
    // purpose of loading the immediate numbers, the ARM ISA has a similar scheme, it
    // place large immediate numbers in the instruction area outside of the current function
    // (or inside the function and using instruction 'jump' to skip these area so that they
    // are not parsed as instructions).
    // however, the XixoaXuan Core VM ISA are designed as variable-length and don't necessarily
    // require the program to have a data section or heap,so the immediate numbers are
    // placed directly in the 'imm_xxx' instructions.
    imm_i64, // (param number_low:i32, number_high:i32) -> i64
    imm_f32, // (param number:i32) -> f32
    imm_f64, // (param number_low:i32, number_high:i32) -> f64

    // data (thread-local variables) loading and storing
    // -------------------------------------------------
    //
    // i32/i64/f32/f64 load/store instructions require the address and offset alignment:
    //
    // | data type | align (bytes) |
    // |-----------|---------------|
    // | i8        | 1             |
    // | i16       | 2             |
    // | i32       | 4             |
    // | i64       | 8             |
    // | f32       | 4             |
    // | f64       | 8             |
    //
    // there are 2 sets of local load/store instructions, one set is the
    // local_load.../local_store.., they are designed to access primitive type data
    // and struct data, the other set is the local_load_extend.../local_store_extend..., they
    // are designed to access long byte-type data.
    //
    // data loading
    // note: all loaded data except the i64 will be signed-extended to i64
    data_load_i64 = 0x180, // (param offset_bytes:i16 data_public_index:i32) -> i64
    data_load_i32_s,       // (param offset_bytes:i16 data_public_index:i32) -> i64
    data_load_i32_u,       // (param offset_bytes:i16 data_public_index:i32) -> i64
    data_load_i16_s,       // (param offset_bytes:i16 data_public_index:i32) -> i64
    data_load_i16_u,       // (param offset_bytes:i16 data_public_index:i32) -> i64
    data_load_i8_s,        // (param offset_bytes:i16 data_public_index:i32) -> i64
    data_load_i8_u,        // (param offset_bytes:i16 data_public_index:i32) -> i64

    // Load f64 with floating-point validity check.
    // (param offset_bytes:i16 data_public_index:i32) -> f64
    data_load_f64,

    // Load f32 with floating-point validity check.
    // note: the high part of operand (on the stack) is undefined
    // (param offset_bytes:i16 data_public_index:i32) -> f32
    data_load_f32,

    data_store_i64, // (param offset_bytes:i16 data_public_index:i32) (operand value:i64) -> ()
    data_store_i32, // (param offset_bytes:i16 data_public_index:i32) (operand value:i32) -> ()
    data_store_i16, // (param offset_bytes:i16 data_public_index:i32) (operand value:i32) -> ()
    data_store_i8,  // (param offset_bytes:i16 data_public_index:i32) (operand value:i32) -> ()
    data_store_f64, // (param offset_bytes:i16 data_public_index:i32) (operand value:f64) -> ()
    data_store_f32, // (param offset_bytes:i16 data_public_index:i32) (operand value:f32) -> ()

    data_load_extend_i64, // (param data_public_index:i32) (operand offset_bytes:i32) -> i64
    data_load_extend_i32_s, // (param data_public_index:i32) (operand offset_bytes:i32) -> i64
    data_load_extend_i32_u, // (param data_public_index:i32) (operand offset_bytes:i32) -> i64
    data_load_extend_i16_s, // (param data_public_index:i32) (operand offset_bytes:i32) -> i64
    data_load_extend_i16_u, // (param data_public_index:i32) (operand offset_bytes:i32) -> i64
    data_load_extend_i8_s, // (param data_public_index:i32) (operand offset_bytes:i32) -> i64
    data_load_extend_i8_u, // (param data_public_index:i32) (operand offset_bytes:i32) -> i64
    data_load_extend_f64, // (param data_public_index:i32) (operand offset_bytes:i32) -> f64
    data_load_extend_f32, // (param data_public_index:i32) (operand offset_bytes:i32) -> f32
    data_store_extend_i64, // (param data_public_index:i32) (operand offset_bytes:i32 value:i64) -> ()
    data_store_extend_i32, // (param data_public_index:i32) (operand offset_bytes:i32 value:i32) -> ()
    data_store_extend_i16, // (param data_public_index:i32) (operand offset_bytes:i32 value:i32) -> ()
    data_store_extend_i8, // (param data_public_index:i32) (operand offset_bytes:i32 value:i32) -> ()
    data_store_extend_f64, // (param data_public_index:i32) (operand offset_bytes:i32 value:f64) -> ()
    data_store_extend_f32, // (param data_public_index:i32) (operand offset_bytes:i32 value:f32) -> ()

    // heap (thread-local memory) loading and storing
    // ----------------------------------------------
    //
    // note that the address of heap is a 64-bit integer number.
    heap_load_i64 = 0x1c0, // (param offset_bytes:i32) (operand heap_addr:i64) -> i64
    heap_load_i32_s,       // (param offset_bytes:i32) (operand heap_addr:i64) -> i64
    heap_load_i32_u,       // (param offset_bytes:i32) (operand heap_addr:i64) -> i64
    heap_load_i16_s,       // (param offset_bytes:i32) (operand heap_addr:i64) -> i64
    heap_load_i16_u,       // (param offset_bytes:i32) (operand heap_addr:i64) -> i64
    heap_load_i8_s,        // (param offset_bytes:i32) (operand heap_addr:i64) -> i64
    heap_load_i8_u,        // (param offset_bytes:i32) (operand heap_addr:i64) -> i64

    // Load f64 with floating-point validity check.
    // (param offset_bytes:i32) (operand heap_addr:i64) -> f64
    heap_load_f64,

    // Load f32 with floating-point validity check.
    // note: the high part of operand (on the stack) is undefined
    // (param offset_bytes:i32) (operand heap_addr:i64) -> f32
    heap_load_f32,

    heap_store_i64, // (param offset_bytes:i32) (operand heap_addr:i64 value:i64) -> ()
    heap_store_i32, // (param offset_bytes:i32) (operand heap_addr:i64 value:i32) -> ()
    heap_store_i16, // (param offset_bytes:i32) (operand heap_addr:i64 value:i32) -> ()
    heap_store_i8,  // (param offset_bytes:i32) (operand heap_addr:i64 value:i32) -> ()
    heap_store_f64, // (param offset_bytes:i32) (operand heap_addr:i64 value:f64) -> ()
    heap_store_f32, // (param offset_bytes:i32) (operand heap_addr:i64 value:f32) -> ()

    //     heap_load_bound_i64,              // load heap          (operand start_addr:i64 length_bytes:i32 offset_bytes:i32) -> i64
    //     heap_load_bound_i32_s,            //                    (operand start_addr:i64 length_bytes:i32 offset_bytes:i32) -> i64
    //     heap_load_bound_i32_u,            //                    (operand start_addr:i64 length_bytes:i32 offset_bytes:i32) -> i64
    //     heap_load_bound_i16_s,            //                    (operand start_addr:i64 length_bytes:i32 offset_bytes:i32) -> i64
    //     heap_load_bound_i16_u,            //                    (operand start_addr:i64 length_bytes:i32 offset_bytes:i32) -> i64
    //     heap_load_bound_i8_s,             //                    (operand start_addr:i64 length_bytes:i32 offset_bytes:i32) -> i64
    //     heap_load_bound_i8_u,             //                    (operand start_addr:i64 length_bytes:i32 offset_bytes:i32) -> i64
    //     heap_load_bound_f64,              //                    (operand start_addr:i64 length_bytes:i32 offset_bytes:i32) -> f64
    //     heap_load_bound_f32,              //                    (operand start_addr:i64 length_bytes:i32 offset_bytes:i32) -> f32
    //
    //     heap_store_bound_i64,             // store heap         (operand start_addr:i64 length_bytes:i32 offset_bytes:i32 value:i64) -> ()
    //     heap_store_bound_i32,             //                    (operand start_addr:i64 length_bytes:i32 offset_bytes:i32 value:i32) -> ()
    //     heap_store_bound_i16,             //                    (operand start_addr:i64 length_bytes:i32 offset_bytes:i32 value:i32) -> ()
    //     heap_store_bound_i8,              //                    (operand start_addr:i64 length_bytes:i32 offset_bytes:i32 value:i32) -> ()
    //     heap_store_bound_f64,             // store heap         (operand start_addr:i64 length_bytes:i32 offset_bytes:i32 value:f64) -> ()
    //     heap_store_bound_f32,             //                    (operand start_addr:i64 length_bytes:i32 offset_bytes:i32 value:f32) -> ()

    // fill the specified memory region with the specified (i8) value
    // () (operand offset:i64 value:i8 count:i64) -> ()
    heap_fill = 0x200,

    // copy the specified memory region to the specified location
    // () (operand dst_offset:i64 src_offset:i64 length_in_bytes:i64) -> ()
    heap_copy,

    // return the amount of pages of the thread-local
    // memory (i.e. heap), each page is MEMORY_PAGE_SIZE_IN_BYTES (64 KiB) by default
    // () -> pages:i64
    heap_capacity,

    // increase or decrease the heap size return the new capacity (in pages)
    // (operand pages:i64) -> new_pages:i64
    heap_resize,

    // local variables loading and storing
    // -----------------------------------
    //
    // load the specified local variable and push to the stack, or
    // pop one operand off the stack and set the specified local variable.
    //
    // note that arguments of function or block are also local variables, the index of local variables are
    // follow the arguments, e.g. consider there are 4 local variables in a function which has
    // 2 parameters, the indices of them are as the following:
    //
    //     arguments local variable
    //     [i32 i32] [i32 i32 i64 i64]
    // idx  0   1     2   3   4   5
    //
    // the INDEX of local variable (data, function):
    //
    // using the 'index', rather than the 'address/pointer' to access local variables (including
    // data in the data section and functions talked about in the following sections) is the
    // security strategy of the XiaoXuan Core ISA.
    // because the 'index' includes the type, length and location (the safe access range) of the 'data',
    // when accessing the data, the VM can check whether the type, and the range is legal
    // or not, so it can prevent a lot of unsafe accessing.
    // for example, the traditional method of using pointers to access a array is very easy
    // to read/write data outside the range.

    // note:
    // in some stack base VM, the arguments of a function are placed on the top
    // of the stack, so it is also possible to read the arguments directly in the function
    // using instructions which implies the `POP` ability (e.g. the comparison instructions, the arithmetic
    // instructions etc.).
    // this feature can be used as a trick to improve performance, but the XiaoXuan Core ISA doesn't
    // provide this feature. please note that you should always using the index to access arguments.
    local_load_i64 = 0x240, // (param reversed_index:i16 local_variable_index:i16 offset_bytes:i16) -> i64
    local_load_i32_s, // (param reversed_index:i16 local_variable_index:i16 offset_bytes:i16) -> i64
    local_load_i32_u, // (param reversed_index:i16 local_variable_index:i16 offset_bytes:i16) -> i64
    local_load_i16_s, // (param reversed_index:i16 local_variable_index:i16 offset_bytes:i16) -> i64
    local_load_i16_u, // (param reversed_index:i16 local_variable_index:i16 offset_bytes:i16) -> i64
    local_load_i8_s,  // (param reversed_index:i16 local_variable_index:i16 offset_bytes:i16) -> i64
    local_load_i8_u,  // (param reversed_index:i16 local_variable_index:i16 offset_bytes:i16) -> i64

    // Load f64 with floating-point validity check.
    // (param reversed_index:i16 local_variable_index:i16 offset_bytes:i16) -> f64
    local_load_f64,

    // Load f32 with floating-point validity check.
    // note: the high part of operand (on the stack) is undefined
    // (param reversed_index:i16 local_variable_index:i16 offset_bytes:i16) -> f32
    local_load_f32,

    local_store_i64, // (param reversed_index:i16 local_variable_index:i16 offset_bytes:i16) (operand value:i64) -> ()
    local_store_i32, // (param reversed_index:i16 local_variable_index:i16 offset_bytes:i16) (operand value:i32) -> ()
    local_store_i16, // (param reversed_index:i16 local_variable_index:i16 offset_bytes:i16) (operand value:i32) -> ()
    local_store_i8, // (param reversed_index:i16 local_variable_index:i16 offset_bytes:i16) (operand value:i32) -> ()
    local_store_f64, // (param reversed_index:i16 local_variable_index:i16 offset_bytes:i16) (operand value:f64) -> ()
    local_store_f32, // (param reversed_index:i16 local_variable_index:i16 offset_bytes:i16) (operand value:f32) -> ()

    local_load_extend_i64, // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32) -> i64
    local_load_extend_i32_s, // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32) -> i64
    local_load_extend_i32_u, // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32) -> i64
    local_load_extend_i16_s, // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32) -> i64
    local_load_extend_i16_u, // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32) -> i64
    local_load_extend_i8_s, // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32) -> i64
    local_load_extend_i8_u, // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32) -> i64
    local_load_extend_f64, // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32) -> f64
    local_load_extend_f32, // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32) -> f32

    local_store_extend_i64, // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32 value:i64) -> ()
    local_store_extend_i32, // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32 value:i32) -> ()
    local_store_extend_i16, // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32 value:i32) -> ()
    local_store_extend_i8, // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32 value:i32) -> ()
    local_store_extend_f64, // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32 value:f64) -> ()
    local_store_extend_f32, // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32 value:f32) -> ()

    // note: both local variables and data have NO data type internal at all,
    // they are both just bytes in the memory.
    // so you can call 'local_store_i8' and 'local_load_i8_u'
    // even if the local variable is defined as i64.

    // conversion
    // ----------
    //
    // truncate i64 to i32
    // discard the high 32 bits of an i64 number directly
    // (operand number:i64) -> i64
    truncate_i64_to_i32 = 0x280,

    // extend i32 to i64
    extend_i32_s_to_i64, // (operand number:i32) -> i64
    extend_i32_u_to_i64, // (operand number:i32) -> i64

    // demote f64 to f32
    demote_f64_to_f32, // (operand number:f64) -> f32

    // promote f32 to f64
    promote_f32_to_f64, // (operand number:i32) -> f64

    // convert float to int
    // truncate fractional part
    convert_f32_to_i32_s, //                                  (operand number:f32) -> i64
    convert_f32_to_i32_u, // note -x.xx(float) -> 0(int)      (operand number:f32) -> i64
    convert_f64_to_i32_s, //                                  (operand number:f64) -> i64
    convert_f64_to_i32_u, // note -x.xx(float) -> 0(int)      (operand number:f64) -> i64
    convert_f32_to_i64_s, //                                  (operand number:f32) -> i64
    convert_f32_to_i64_u, // note -x.xx(float) -> 0(int)      (operand number:f32) -> i64
    convert_f64_to_i64_s, //                                  (operand number:f64) -> i64
    convert_f64_to_i64_u, // note -x.xx(float) -> 0(int)      (operand number:f64) -> i64

    // convert from int
    convert_i32_s_to_f32, // (operand number:i32) -> f32
    convert_i32_u_to_f32, // (operand number:i32) -> f32
    convert_i64_s_to_f32, // (operand number:i64) -> f32
    convert_i64_u_to_f32, // (operand number:i64) -> f32
    convert_i32_s_to_f64, // (operand number:i32) -> f64
    convert_i32_u_to_f64, // (operand number:i32) -> f64
    convert_i64_s_to_f64, // (operand number:i64) -> f64
    convert_i64_u_to_f64, // (operand number:i64) -> f64

    // comparsion
    // ----------
    //
    // for the binary operations, the first one pops up from the
    // stack is the right-hand-side-value, e.g.
    //
    // |                 | --> stack end
    // | right hand side | --> 1st pop: RHS
    // | left hand side  | --> 2nd pop: LHS
    // \-----------------/ --> stack start
    //
    // it is the same order as the function parameter, e.g.
    // function `add (a, b)`
    // the parameters in the stack is:
    //
    //  |   |
    //  | b |
    //  | a |
    //  \---/
    //
    // note that two operands MUST be the same data type.
    //
    // the result of the comparison is a logical TRUE or FALSE (i.e., the data type 'boolean'),
    // when the result is TRUE, the number `1:i32` is pushed onto the stack,
    // and vice versa the number is `0:i32`.
    //
    // instruction `i32_lt_u` example:
    //
    // ```
    // ;; load 2 numbers on to the stack
    // (i32.imm 11)
    // (i32.imm 22)
    //
    // ;; now the stack layout is:
    // ;;
    // ;; |    |
    // ;; | 22 |
    // ;; | 11 |
    // ;; \----/
    //
    // ;; check if '11' is less then '22', i.e. `11 < 22 ?`
    // ;; `1` will be pushed on to the stack
    // i32.lt_u
    //
    // ;; now the stack layout is:
    // ;;
    // ;; |    |
    // ;; | 1  |
    // ;; \----/
    // ```
    eqz_i32 = 0x2c0, // (operand number:i32) -> i32
    nez_i32,         // (operand number:i32) -> i32
    eq_i32,          // (operand left:i32 right:i32) -> i32
    ne_i32,          // (operand left:i32 right:i32) -> i32
    lt_i32_s,        // (operand left:i32 right:i32) -> i32
    lt_i32_u,        // (operand left:i32 right:i32) -> i32
    gt_i32_s,        // (operand left:i32 right:i32) -> i32
    gt_i32_u,        // (operand left:i32 right:i32) -> i32
    le_i32_s,        // (operand left:i32 right:i32) -> i32, redundant
    le_i32_u,        // (operand left:i32 right:i32) -> i32, redundant
    ge_i32_s,        // (operand left:i32 right:i32) -> i32, redundant
    ge_i32_u,        // (operand left:i32 right:i32) -> i32, redundant

    eqz_i64,  // (operand number:i64) -> i64
    nez_i64,  // (operand number:i64) -> i64
    eq_i64,   // (operand left:i64 right:i64) -> i64
    ne_i64,   // (operand left:i64 right:i64) -> i64
    lt_i64_s, // (operand left:i64 right:i64) -> i64
    lt_i64_u, // (operand left:i64 right:i64) -> i64
    gt_i64_s, // (operand left:i64 right:i64) -> i64
    gt_i64_u, // (operand left:i64 right:i64) -> i64
    le_i64_s, // (operand left:i64 right:i64) -> i64, redundant
    le_i64_u, // (operand left:i64 right:i64) -> i64, redundant
    ge_i64_s, // (operand left:i64 right:i64) -> i64, redundant
    ge_i64_u, // (operand left:i64 right:i64) -> i64, redundant

    eq_f32, // (operand left:f32 right:f32) -> i64
    ne_f32, // (operand left:f32 right:f32) -> i64
    lt_f32, // (operand left:f32 right:f32) -> i64
    gt_f32, // (operand left:f32 right:f32) -> i64
    le_f32, // (operand left:f32 right:f32) -> i64
    ge_f32, // (operand left:f32 right:f32) -> i64

    eq_f64, // (operand left:f64 right:f64) -> i64
    ne_f64, // (operand left:f64 right:f64) -> i64
    lt_f64, // (operand left:f64 right:f64) -> i64
    gt_f64, // (operand left:f64 right:f64) -> i64
    le_f64, // (operand left:f64 right:f64) -> i64
    ge_f64, // (operand left:f64 right:f64) -> i64

    // arithmetic
    // ----------
    //
    // wrapping add, e.g. 0xffff_ffff + 2 = 1 (-1 + 2 = 1)
    // (operand left:i64 right:i64) -> i64
    add_i32 = 0x300,

    // wrapping sub, e.g. 11 - 211 = -200
    // (operand left:i64 right:i64) -> i64
    sub_i32,

    // wrapping inc, e.g. 0xffff_ffff inc 2 = 1
    // (param amount:i16) (operand number:i64) -> i64
    add_imm_i32,

    // wrapping dec, e.g. 0x1 dec 2 = 0xffff_ffff
    // (param amount:i16) (operand number:i64) -> i64
    sub_imm_i32,

    // wrapping mul, e.g. 0xf0e0d0c0 * 2 = 0xf0e0d0c0 << 1
    // (operand left:i64 right:i64) -> i64
    mul_i32,

    div_i32_s, // (operand left:i64 right:i64) -> i64
    div_i32_u, // (operand left:i64 right:i64) -> i64
    rem_i32_s, // (operand left:i64 right:i64) -> i64

    // calculate the remainder
    // (operand left:i64 right:i64) -> i64
    rem_i32_u,

    // remainder vs modulus
    // --------------------
    // The remainder (%) operator returns the remainder left over when one operand is
    // divided by a second operand. It always takes the sign of the dividend.
    // For the operation n % d, n is called the dividend and d is called the divisor.
    //
    // (13 % 5) = 3
    //  ^    ^
    //  |    |divisor
    //  |dividend <--------- the result always takes the sign of the dividend.
    //
    // (-13 % 5) = -3
    // (4 % 2) = 0
    // (-4 % 2) = -0
    //
    // ref: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Remainder
    //
    // Example of Modulus:
    //
    // 5 % 3 = 2 [here divisible is 5 which is positively signed so the remainder will also be
    // positively signed and the divisor is also positively signed. As both remainder and divisor
    // are of same sign the result will be same as remainder]
    //
    // -5 % 3 = 1 [here divisible is -5 which is negatively signed so the remainder will also be
    // negatively signed and the divisor is positively signed. As both remainder and divisor are
    // of opposite sign the result will be sum of remainder and divisor -2 + 3 = 1]
    //
    // 5 % -3 = -1 [here divisible is 5 which is positively signed so the remainder will also
    // be positively signed and the divisor is negatively signed. As both remainder and divisor
    // are of opposite sign the result will be sum of remainder and divisor 2 + -3 = -1]
    //
    // -5 % -3 = -2 [here divisible is -5 which is negatively signed so the remainder will also
    // be negatively signed and the divisor is also negatively signed. As both remainder and
    // divisor are of same sign the result will be same as remainder]
    //
    // ref: https://stackoverflow.com/questions/13683563/whats-the-difference-between-mod-and-remainder
    // ref: https://en.wikipedia.org/wiki/Euclidean_division
    // ref: https://en.wikipedia.org/wiki/Modulo

    // instruction `add_i32` example:
    //
    // ```
    // ;; load two numbers onto the stack
    // (i32.imm 10)
    // (i32.imm 3)
    // ;; subtract one number from the other
    // ;; the top item on the stack will be 7 (10 - 3 = 7)
    // i32.sub
    // ```

    // instruction `i32.rem_u` example:
    //
    // ```
    // ;; load two numbers onto the stack
    // (i32.imm 10)
    // (i32.imm 3)
    // ;; calculate the remainder of dividing one number by the other
    // ;; the top item on the stack will be 1 (10 % 3 = 1)
    // i32.rem_u
    // ```

    // wrapping add
    // (operand left:i64 right:i64) -> i64
    add_i64,

    // wrapping sub
    // (operand left:i64 right:i64) -> i64
    sub_i64,

    // wrapping inc
    // (param amount:i16) (operand number:i64) -> i64
    add_imm_i64,

    // wrapping dec
    // (param amount:i16) (operand number:i64) -> i64
    sub_imm_i64,

    // wrapping mul
    // (operand left:i64 right:i64) -> i64
    mul_i64,

    div_i64_s, // (operand left:i64 right:i64) -> i64
    div_i64_u, // (operand left:i64 right:i64) -> i64
    rem_i64_s, // (operand left:i64 right:i64) -> i64
    rem_i64_u, // (operand left:i64 right:i64) -> i64

    add_f32, // (operand left:f32 right:f32) -> f32
    sub_f32, // (operand left:f32 right:f32) -> f32
    mul_f32, // (operand left:f32 right:f32) -> f32
    div_f32, // (operand left:f32 right:f32) -> f32

    add_f64, // (operand left:f64 right:f64) -> f64
    sub_f64, // (operand left:f64 right:f64) -> f64
    mul_f64, // (operand left:f64 right:f64) -> f64
    div_f64, // (operand left:f64 right:f64) -> f64

    // bitwise
    // -------
    //
    // see also:
    // https://en.wikipedia.org/wiki/Bitwise_operation
    and = 0x340, // bitwise AND     (operand left:i64 right:i64) -> i64
    or,          // bitwise OR      (operand left:i64 right:i64) -> i64
    xor,         // bitwise XOR     (operand left:i64 right:i64) -> i64
    not,         // bitwise NOT     (operand number:i64) -> i64

    // instruction `shift_left_i32` example:
    //
    // ```
    // ;; load two numbers onto the stack
    // (i32.imm 7)              ;; 00000111
    // ;; perform a bitwise left-shift
    // ;; left shift one spot
    // ;; the top item on the stack will be 14 (00001110)
    // (shift_left_i32 1)
    // ```
    shift_left_i32, // left shift                   (operand number:i32 move_bits:i32) -> i32, move_bits = [0,32)
    shift_right_i32_s, // arithmetic right shift    (operand number:i32 move_bits:i32) -> i32, move_bits = [0,32)
    shift_right_i32_u, // logical right shift       (operand number:i32 move_bits:i32) -> i32, move_bits = [0,32)
    rotate_left_i32, // left rotate                 (operand number:i32 move_bits:i32) -> i32, move_bits = [0,32)
    rotate_right_i32, // right rotate               (operand number:i32 move_bits:i32) -> i32, move_bits = [0,32)

    // instructions `leading_zeros_i32`, `trailing_zeros_i32`
    //
    // ```
    // ;; load a number onto the stack
    // (i32.imm 8_388_608)      ;; 00000000_10000000_00000000_00000000
    // ;; count leading zeros
    // ;; the top item on the stack will be 8
    // leading_zeros_i32
    //
    // ;; load a number onto the stack
    // (i32.imm 8_388_608)      ;; 00000000_10000000_00000000_00000000
    // ;; count trailing zeros
    // ;; the top item on the stack will be 23
    // trailing_zeros_i32
    //
    leading_zeros_i32,  // count leading zeros       (operand number:i32) -> i32
    leading_ones_i32,   // count leading ones        (operand number:i32) -> i32
    trailing_zeros_i32, // count trailing zeros     (operand number:i32) -> i32

    // count the number of ones in the binary representation
    //
    // ;; load a number onto the stack
    // (i32.imm 130)            ;; 10000010
    // ;; count the 1s
    // ;; the top item on the stack will be 2
    // count_ones_i32
    // ```
    //
    // (operand number:i32) -> i32
    count_ones_i32,

    shift_left_i64, // left shift                   (operand number:i64 move_bits:i32) -> i64, move_bits = [0,64)
    shift_right_i64_s, // arithmetic right shift    (operand number:i64 move_bits:i32) -> i64, move_bits = [0,64)
    shift_right_i64_u, // logical right shift       (operand number:i64 move_bits:i32) -> i64, move_bits = [0,64)
    rotate_left_i64, // left rotate                 (operand number:i64 move_bits:i32) -> i64, move_bits = [0,64)
    rotate_right_i64, // right rotate               (operand number:i64 move_bits:i32) -> i64, move_bits = [0,64)
    leading_zeros_i64, // (operand number:i64) -> i32
    leading_ones_i64, // (operand number:i64) -> i32
    trailing_zeros_i64, // (operand number:i64) -> i32
    count_ones_i64,   // (operand number:i64) -> i32

    // math
    // ----
    //
    abs_i32 = 0x380,               // (operand number:i32) -> i32
    neg_i32,                       // (operand number:i32) -> i32
    abs_i64,                       // (operand number:i64) -> i64
    neg_i64,                       // (operand number:i64) -> i64
    abs_f32,                       // (operand number:f32) -> f32
    neg_f32,                       // (operand number:f32) -> f32
    copysign_f32,                  // (operand num:f32 sign:f32) -> f32
    sqrt_f32,                      // sqrt(x)                           (operand number:f32) -> f32
    min_f32,                       // (operand left:f32 right:f32) -> f32
    max_f32,                       // (operand left:f32 right:f32) -> f32
    ceil_f32,                      // (operand number:f32) -> f32
    floor_f32,                     // (operand number:f32) -> f32
    round_half_away_from_zero_f32, // (operand number:f32) -> f32
    round_half_to_even_f32,        // (operand number:f32) -> f32
    trunc_f32,                     // the integer part of x            (operand number:f32) -> f32
    fract_f32,                     // the fractional part of  x        (operand number:f32) -> f32
    cbrt_f32,                      // cbrt(x), the cube root of x      (operand number:f32) -> f32
    exp_f32,                       // e^x                              (operand number:f32) -> f32
    exp2_f32,                      // 2^x                              (operand number:f32) -> f32
    ln_f32,                        // log_e(x)                         (operand number:f32) -> f32
    log2_f32,                      // log_2(x)                         (operand number:f32) -> f32
    log10_f32,                     // log_10(x)                        (operand number:f32) -> f32
    sin_f32,                       // (operand number:f32) -> f32
    cos_f32,                       // (operand number:f32) -> f32
    tan_f32,                       // (operand number:f32) -> f32
    asin_f32,                      // (operand number:f32) -> f32
    acos_f32,                      // (operand number:f32) -> f32
    atan_f32,                      // (operand number:f32) -> f32
    pow_f32, // left^right                       (operand left:f32 right:f32) -> f32
    log_f32, // log_right(left)                  (operand left:f32 right:f32) -> f32

    // examples of 'round_half_away_from_zero':
    // round(2.4) = 2.0
    // round(2.6) = 3.0
    // round(2.5) = 3.0
    // round(-2.5) = -3.0
    //
    // ref:
    // https://en.wikipedia.org/wiki/Rounding#Rounding_half_away_from_zero
    abs_f64,                       // (operand number:f64) -> f64
    neg_f64,                       // (operand number:f64) -> f64
    copysign_f64,                  // (operand num:f32 sign:f32) -> f32
    sqrt_f64,                      // (operand number:f64) -> f64
    min_f64,                       // (operand left:f32 right:f32) -> f64
    max_f64,                       // (operand left:f32 right:f32) -> f64
    ceil_f64,                      // (operand number:f64) -> f64
    floor_f64,                     // (operand number:f64) -> f64
    round_half_away_from_zero_f64, // (operand number:f64) -> f64
    round_half_to_even_f64,        // (operand number:f64) -> f64
    trunc_f64,                     // (operand number:f64) -> f64
    fract_f64,                     // (operand number:f64) -> f64
    cbrt_f64,                      // (operand number:f64) -> f64
    exp_f64,                       // (operand number:f64) -> f64
    exp2_f64,                      // (operand number:f64) -> f64
    ln_f64,                        // (operand number:f64) -> f64
    log2_f64,                      // (operand number:f64) -> f64
    log10_f64,                     // (operand number:f64) -> f64
    sin_f64,                       // (operand number:f64) -> f64
    cos_f64,                       // (operand number:f64) -> f64
    tan_f64,                       // (operand number:f64) -> f64
    asin_f64,                      // (operand number:f64) -> f64
    acos_f64,                      // (operand number:f64) -> f64
    atan_f64,                      // (operand number:f64) -> f64
    pow_f64,                       // (operand left:f32 right:f32) -> f64
    log_f64,                       // (operand left:f32 right:f32) -> f64

    // control flow
    // ------------
    //
    // when the instruction 'end' is executed, a stack frame will be removed and
    // the results of the current block or function will be placed on the top of stack.
    //
    // ()->()
    end = 0x3c0,

    // create a block scope. a block is similar to a function, it also has
    // parameters and results, it shares the type with function, so the 'block'
    // instruction has a parameter called 'type_index'.
    // this instruction leads VM to create a stack frame which is called 'block frame',
    // block frame is similar to 'function frame' except it has no local variables.
    //
    // this instruction is different from the WebAssembly instruction 'block', which
    // its parameters are not 'local variables', and the values are placed on the
    // operands stack, they can not be accessed with 'local_load/store' instructions.
    //
    // (param type_index:i32, local_list_index:i32)
    block,

    // the instruction 'break' is similar to the instruction 'end', it is
    // used for finishing a block or a function.
    // - for a block:
    //   a block stack frame will be removed and jump to the next instruction
    //   that AFTER the instruction 'end'.
    //   the value of the parameter 'next_inst_offset' should be (`addr of next inst after 'end'` - `addr of break`)
    // - for a function:
    //   a function stack frame will be removed and return to the the
    //   instruction next to the instruction 'call'.
    //   the value of the parameter 'next_inst_offset' is ignored.
    //
    // note that this instruction implies the function of instruction 'end'.
    //
    // e.g.
    //
    // ```bytecode
    // 0d0000 block 0           ;; the size of instruction 'block' is 8 bytes
    // 0d0008   nop             ;;
    // 0d0010   break 0 14      ;; the size of instruction 'break' is 8 bytes, (14 = 24 - 10) ---\
    // 0d0018   nop             ;;                                                               |
    // 0d0020   nop             ;;                                                               |
    // 0d0022 end               ;;                                                               |
    // 0d0024 nop               ;; <-- jump to here (the instruction that next to the 'end')-----/
    // ```
    //
    // instruction 'break' not only just finish a block or a function, but also
    // brings the operands out of the block or function, e.g.
    //
    // 0d0000 block 0           ;; assumes the block type is '()->(i32,i32)'
    // 0d0008   i32.imm 11      ;;
    // 0d0016   i32.imm 13      ;;                 | 17                 | -----\ operands '17' and '13' were
    // 0d0024   i32.imm 17      ;; --------------> | 13                 | -----\ taken out of the block frame
    // 0d0032   break 0 14      ;; ---\            | 11                 |      |
    // 0d0040   nop             ;;    |            | [block frame info] |      v
    // 0d0042   nop             ;;    | jump       | ..                 |    | 17                 |
    // 0d0044   nop             ;;    |            | [func frame info]  |    | 13                 |
    // 0d0046   nop             ;;    |            \____________________/    | ..                 |
    // 0d0048 end               ;;    |               the stack layout       | [func frame info]  |
    // 0d0050 nop               ;; <--/ -----------------------------------> \____________________/
    //                                                                        the stack layout
    //
    // the instruction 'break' can cross over multiple block nested.
    // when the parameter 'reversed_index' is 0, it simply finish the current block.
    // when the value is greater than 0, multiple block stack frame will be removed,
    // as well as the operands will be taken out of the block.
    // (the amount of the operands is determined by the 'target block type'.
    //
    // ```bytecode
    // 0d0000 block 0           ;; assumes the block type is '()->(i32,i32,i32)'
    // 0d0008   block 0         ;; assumes the block type is '()->(i32,i32)'
    // 0d0016     block 0       ;; assumes the block type is '()->(i32)'
    // 0d0024       nop
    // 0d0026       break 1 14  ;; (18 = 44 - 26) --------\
    // 0d0034       nop         ;;                        |
    // 0d0036     end           ;;                        |
    // 0d0038     nop           ;;                        |
    // 0d0040   end             ;;                        | carries operands (i32, i32) and
    // 0d0042   nop             ;; <----------------------/ jumps to here
    // 0d0044 end
    // ```
    //
    // note of the block range
    //
    // the value of parameter 'next_inst_offset' should not exceed the block range.
    // the MAX value is the address of the instruction 'end' of the target frame.
    //
    // background:
    //
    // there is a similar instruction in WASM named 'br/break', it is used
    // to make the PC jump to the address of the instruction 'end'.
    // it is more elegant than XiaoXuan instruction 'break', but less efficient
    // because it need to carry the operand twice.
    // after balancing between performance and elegance, XiaoXuan instruction
    // 'break' implies 'end' as well as jumps directly to the next instruction
    // after the instruction 'end'.
    //
    // note that both instruction 'end' and 'break' can end
    // a function or a block, they are the same actually except
    // the 'break' instruction can specify the 'reversed_index'
    // and 'next_inst_offset'.
    // thus `end` == `break reversed_index=0 next_inst_offset=2`
    //
    // (param reversed_index:i16, next_inst_offset:i32)
    break_,

    // the instruction 'recur' lets VM to jump to the instruction next to the instruction 'block', 'block_alt'
    // or the first instruction of the current function,
    // as well as all the operands in the current stack frame will be removed except
    // the operands for the 'target block/function params' are reserved and placed on the top of stack.
    // it is commonly used to construct the 'while/for' structures in general programming languages,
    // it is also used to implement the TCO (tail call optimization, see below section).
    //
    // when the target frame is the function frame itself, the param 'start_inst_offset' is ignore and
    // all local variables will be reset to 0 (except the arguments).
    //
    // note that the value of 'start_inst_offset' is a positive number.
    //
    // 0d0000 block 0           ;; assumes the block type is '()->(i32,i32)'
    // 0d0008   i32.imm 11      ;; <------\
    // 0d0016   i32.imm 13      ;;        |         | 17                 | -----\ operands '17' and '13' were
    // 0d0024   i32.imm 17      ;; ---------------> | 13                 | -----\ taken out of the block frame
    // 0d0032   nop             ;;        |         | 11                 |      |
    // 0d0034   nop             ;;        |         | [block frame info] |      v
    // 0d0036   nop             ;;        |         | ..                 |    | 17                 |
    // 0d0038   nop             ;;  jump  |         | [func frame info]  |    | 13                 |
    // 0d0040   recur 0 14      ;; -------/         \____________________/    | [block frame info] |
    // 0d0048 end               ;;        |            the stack layout       | ..                 |
    // 0d0050 nop               ;;        \---------------------------------> | [func frame info]  |
    //                                                                        \____________________/
    //                                                                           the stack layout

    // the instruction 'recur' can cross over multiple block nested also.
    //
    // ```bytecode
    // 0d0000 block 0           ;; assumes the block type is '()->(i32,i32)'
    // 0d0008   nop             ;; <--------------------------------------------\ carries operands (i32,i32) and
    // 0d0010   block 0         ;; assumes the block type is '()->(i32)'        | jumps to here
    // 0d0018     nop           ;;                                              |
    // 0d0020     recur 1 12    ;; (12 = 20 - 8) -------------------------------/
    // 0d0028     nop
    // 0d0030   end
    // 0d0032 end
    // ```
    //
    // 'start_inst_offset' is the address of the next instruction follows 'block'
    //
    // (param reversed_index:i16, start_inst_offset:i32)
    recur,

    // create a block scope only if the operand on the top of stack is
    // NOT equals to ZERO (logic TRUE).
    //
    // the value of 'next_inst_offset' should be the address of the next instructions
    // AFTER the instruction 'end'.
    //
    // instruction 'block_nez' is commonly used to construct the 'if' structures
    // in general programming languages.
    //
    // e.g
    //
    // ```c
    // if (i != 0) {
    //     ...
    // }
    // ```
    //
    // ```bytecode
    // 0d0000 block_nez 0 100   ;; -----\
    // ....                     ;;      |
    // 0d0100 end               ;;      |
    // 0d0102 nop               ;; <----/ jump to here when FALSE
    // ```
    //
    // 'block_nez' has NO PARAMS and NO RESULTS, but it can owns local variables.
    //
    // (param local_list_index:i32, next_inst_offset:i32)
    block_nez,

    // the instruction 'block_alt' is similar to the 'block', it also creates a new block scope
    // as well as a block stack frame.
    // but it jumps to the 'alternative instruction' if the operand on the top of stack is
    // equals to ZERO (logic FALSE).
    //
    // note:
    // 0:i32 and 0:i64 are both treated as logic FALSE and
    // all other i32/i64 non-zero are treated as logic TRUE.
    //
    // so the instruction 'block_alt' means only executes the instructions that follow the instruction 'block_alt'
    // when the logic is TRUE, otherwise, jump to the 'alternative instruction'.
    //
    // e.g.
    //
    // ```c
    // if (i != 0) {
    //     /* the 'then' part */
    // } else {
    //     /* the 'else' part */
    // }
    // ```
    //
    // ```bytecode
    //                          ;; TRUE             | FALSE
    //                          ;; |                | |
    //                          ;; |                | \-->--\
    // 0d0000 block_alt 0 158   ;; V                |       | jump to 0d0158 when FALSE
    // 0d0008 ...               ;; |+               |       |-
    // ;; the 'then' part       ;; |+               |       |-
    // 0d0150 break 0 200       ;; \-->--\+         |       |-
    // 0d0158 ...               ;;       |-         | /--<--/+
    // ;; the 'else' part       ;;       |-         | |+
    // 0d0350 end               ;;       |-         | |+
    // 0d0352 nop               ;; <-----/          | |
    // ```
    //
    // (+ => execute, - => pass)
    //
    // 'block_alt' has NO PARAMS, but it can return values and own local variables.
    //
    // (param type_index:i32, local_list_index:i32, alt_inst_offset:i32)
    block_alt,

    // a complete 'for' structure is actually combined with instructions 'block', 'block_alt', 'recur', 'break'
    // and 'break_nez', e.g.
    //
    // ```rust
    // let i = loop {
    //   ...
    //   if ... break 100;
    //   ...
    // }
    // ```
    //
    // the equivalent bytecodes are:
    //
    // ```bytecode
    // 0d0000 block 0
    // 0d0008   ...             ;; <-------------\
    //          ...             ;;               |
    // 0d0100   block_alt 0 28  ;; ----\         |
    // 0d0112     i32.imm 100   ;;     |         |
    // 0d0120     break 1 88    ;; ----|----\    |
    // 0d0128   end             ;; <---/    |    |
    //          ...             ;;          |    |
    // 0d0200   recur 0 192     ;; ---------|----/
    // 0d0208 end               ;;          |
    // 0d0210 ...               ;; <--------/
    // ```
    //
    // the code above can be optimized by instruction 'break_nez', e.g.
    //
    // ```bytecode
    // 0d0000 block 0
    // 0d0008   ...             ;; <-------------\
    //          ...             ;;               |
    // 0d0100   i32.imm 100     ;;               |
    //          ...             ;;               |
    // 0d0120   break_nez 0 88  ;; ---------\    |
    // 0d0128   drop            ;;          |    |
    //          ...             ;;          |    |
    // 0d0200   recur 0 192     ;; ---------|----/
    // 0d0208 end               ;;          |
    // 0d0210 ...               ;; <--------/
    // ```
    //
    // (param reversed_index:i16, next_inst_offset:i32)
    break_nez,

    // when the target frame is the function frame itself, the param 'start_inst_offset' is ignore and
    // all local variables will be reset to 0.
    //
    // instruction 'recur_nez' is used to implement the TCO (tail call optimization).
    //
    // consider the following function:
    //
    // ```rust
    // /* calculate '3+2+1', the result should be '6' */
    //
    // let s = accumulate(0, 3);   //
    //
    // fn accumulate(sum: i32, number: i32) -> i32 {    // /-----\
    //     let new_sum = sum + number;                  // | +   |
    //     if number == 0 {                             // | --> | branch then
    //         new_sum                                  // | <-- | return 0
    //     } else {                                     // | --> | branch else
    //         accumulate(new_sum, number - 1)          // | <-- | return 1
    //     }                                            // |     |
    // }                                                // \-----/
    // ```
    //
    // when invoke function call 'accumulate(0, 3)', the calling path is as follows:
    //
    // (0,3)--\
    //        |    /-----\     (3,2)  /-----\     (5,1)  /-----\     (6,0)  /-----\
    //        \--> | +   |   /------> | +   |   /------> | +   |   /------> | +   |
    //             |=====|   |        |=====|   |        |=====|   |        |=====|
    //             | --> |   |        | --> |   |        | --> |   |        | --> | -----\
    //             | <-- |   |        | <-- |   |        | <-- |   |  /---- | <-- | <----/
    //             |=====|   |        |=====|   |        |=====|   |  |     |=====|
    //             | --> | --/        | --> | --/        | --> | --/  |     | --> |
    //        /--- | <-- | <--------- | <-- | <--------- | <-- | <----/     | <-- |
    //        |    \-----/            \-----/            \-----/            \-----/
    //   6 <--/
    //
    //
    // the function 'accumulate' is invoked 4 times and 4 stack frames are created.
    // since there is no other operation after statement 'accumulate(new_sum, number - 1)', and
    // only return a value afterwards, so the calling path can be simplified as follows:
    //
    // (0,3)--\
    //        |    /-----\     (3,2)  /-----\     (5,1)  /-----\     (6,0)  /-----\
    //        \--> | +   |   /------> | +   |   /------> | +   |   /------> | +   |
    //             |=====|   |        |=====|   |        |=====|   |        |=====|
    //             | --> |   |        | --> |   |        | --> |   |        | --> | ---\
    //             | <-- |   |        | <-- |   |        | <-- |   |    /-- | <-- | <--/
    //             |=====|   |        |=====|   |        |=====|   |    |   |=====|
    //             | --> | --/        | --> | --/        | --> | --/    |   | --> |
    //             | <-- |            | <-- |            | <-- |        |   | <-- |
    //             \-----/            \-----/            \-----/        |   \-----/
    //   6 <------------------------------------------------------------/
    //
    // now we can introduce the instruction 'recur/recur_nez', which lets the VM execute
    // (rather than 'call') the function again from beginning with new arguments,
    // so that we can optimize the calling path:
    //
    // (0,3)--\
    //        |    /-----\
    //        \--> | +   | <----\  <----\  <----\
    //             |=====|      |       |       |
    //             | --> | --\  |       |       |
    //        /--- | <-- | <-/  |       |       |
    //        |    |=====|      |       |       |
    //        |    | --> | -----/  -----/  -----/
    //        |    |     | (3,2)  (5,1)  (6,0)
    //        |    \-----/
    //   6 <--/
    //
    // shown above is TCO (tail call optimization), this optimization saves us from
    // creating and destroying calling stack frames multiple times, which saves resources
    // to improve program efficiency.
    //
    // an important prerequisite for TCO is that the 'recur call' statement
    // (in genernal programming language) must be the last operation of the function,
    // or the last operation in the function branch, otherwise the logical error will occur. e.g.
    //
    // ```rust
    // fn factorial(number: i32) -> i32 {
    //     if number == 0 {
    //         1
    //     } else {
    //         number * factorial(number - 1)
    //     }
    // }
    // ```
    //
    // the statement 'number * factorial(number - 1)' exands as follows:
    //
    // ```rust
    // let i = number;
    // let j = factorial(number - 1);
    // i * j
    // ```
    //
    // obviously the function call statement 'factorial(number - 1)' is neither the last operation
    // of the function nor the last operation of the function branch. the last operation
    // is 'i * j', so this code cann't apply TCO.
    //
    // of course we can modify this function so that it can apply TCO, e.g.
    //
    // ```rust
    // /* calculate '5*4*3*2*1', the result should be '120' */
    //
    // let s = factorial_tco(1, 5);
    //
    // fn factorial_tco(sum: i32, number: i32) -> i32 {
    //     if number == 0 {
    //         sum
    //     } else {
    //         let new_sum = sum * number;
    //         factorial_tco(new_sum, number - 1)
    //     }
    // }
    // ```
    //
    // (param reversed_index:i16, start_inst_offset:i32)
    recur_nez,

    // table of control flow structures and control flow instructions:
    //
    // ## branch
    //
    // | structure         | assembly          | instruction(s)     |
    // |-------------------|-------------------|--------------------|
    // |                   |                   | ..a..              |
    // | if ..a.. {        | (when (a)         | block_nez -\       |
    // |    ..b..          |       (b)         |   ..b..    |       |
    // | }                 | )                 | end        |       |
    // |                   |                   | ...    <---/       |
    // |-------------------|-------------------|--------------------|
    // |                   |                   | ..a..              |
    // | if ..a.. {        | (if (a)           | block_alt ---\     |
    // |    ..b..          |     (b)           |   ..b..      |     |
    // | } else {          |     (c)           |   break 0  --|-\   |
    // |    ..c..          | )                 |   ..c..  <---/ |   |
    // | }                 |                   | end            |   |
    // |                   |                   | ...      <-----/   |
    // |-------------------|-------------------|--------------------|
    // |                   |                   | ..a..              |
    // | if ..a.. {        | (if (a)           | block_alt ---\     |
    // |    ..b..          |     (b)           |   ..b..      |     |
    // | } else if ..c.. { |     (if (c)       |   break 0 ---|---\ |
    // |    ..d..          |         (d)       |   ..c..  <---/   | |
    // | } else {          |         (e)       |   block_alt --\  | |
    // |    ..e..          |     )             |     ..d..     |  | |
    // | }                 | )                 |     break 0 --|-\| |
    // |                   |                   |     ..e..  <--/ || |
    // |                   |                   |   end           || |
    // |                   |                   | end        <----/| |
    // |                   |                   | ...        <-----/ |
    // |                   |                   |                    |
    // |                   | ----------------- | ------------------ |
    // |                   |                   |                    |
    // |                   | (branch           | block              | // note:
    // |                   |   (case (a) (b))  |   ..a..            | //
    // |                   |   (case (c) (d))  |   block_nez -\     | // 'block_nez + x_imm/x_loadx + break N + end'
    // |                   |   (default (e))   |     ..b..    |     | // can be optimized as:
    // |                   | )                 |     break 1 -|--\  | // 'x_imm/x_loadx + swap + break_nez N-1 + drop'
    // |                   |                   |   end        |  |  |
    // |                   |                   |   ..c..  <---/  |  |
    // |                   |                   |   block_nez -\  |  |
    // |                   |                   |     ..d..    |  |  |
    // |                   |                   |     break 1 -|--|  |
    // |                   |                   |   end        |  |  |
    // |                   |                   |   ..e..  <---/  |  |
    // |                   |                   | end             |  |
    // |                   |                   | ...        <----/  |
    // |-------------------|-------------------|--------------------|
    //
    // ## loop
    //
    // | structure         | assembly          | instructions(s)    |
    // |-------------------|-------------------|--------------------|
    // | loop {            | (for (code        | block              |
    // |    ...            |   ...             |   ...   <--\       |
    // | }                 |   (recur ...)     |   recur 0 -/       |
    // |                   | ))                | end                |
    // |-------------------|-------------------|--------------------|
    // | while ..a.. {     |                   | block              |
    // |    ...            |                   |   ..a..   <----\   |
    // | }                 |                   |   break_nez 0 -|-\ |
    // |                   |                   |   ...          | | |
    // | (or for...)       |                   |   recur 0 -----/ | |
    // |                   |                   | end              | |
    // |                   |                   | ...        <-----/ |
    // |                   |                   |                    |
    // |                   | ----------------- | ------------------ |
    // |                   |                   |                    |
    // |                   | (for (code        | block              |
    // |                   |   (when (a)       |   ..a..    <---\   |
    // |                   |     (code ...     |   block_nez    |   |
    // |                   |       (recur ...) |     ...        |   |
    // |                   |     )             |     recur 1 ---/   |
    // |                   |   )               |   end              |
    // |                   | ))                | end                |
    // |                   |                   |                    |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    // | do {              |                   | block              |
    // |    ...            |                   |   ...      <---\   |
    // | }while(..a..)     |                   |   ..a..        |   |
    // |                   |                   |   recur_nez 0 -/   |
    // |                   |                   | end                |
    // |                   |                   |                    |
    // |                   | ----------------- | ------------------ |
    // |                   |                   |                    |
    // |                   | (for (code        | block              |
    // |                   |   ...             |   ...      <---\   |
    // |                   |   (when (a)       |   ..a..        |   |
    // |                   |     (recur ...)   |   block_nez    |   |
    // |                   |   )               |     recur 1 ---/   |
    // |                   | ))                |   end              |
    // |                   |                   | end                |
    // |                   |                   |                    |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    //
    //
    // ## TCO
    //
    // | structure         | assembly          | instructions(s)    |
    // |-------------------|-------------------|--------------------|
    // | func foo {        | (function         | -- func begin --   |
    // |    ...            |   (code ...       |   ...   <-------\  |
    // |    if ..a.. {     |     (when (a)     |   ..a..         |  |
    // |      foo()        |       (tcall ...) |   block_nez --\ |  |
    // |    }              |     )             |     recur 1 --|-/  |
    // | }                 |   )               |   end         |    |
    // |                   | )                 | end      <----/    |
    // |                   |                   |                    |
    // |                   | ----------------- | ------------------ |
    // |                   |                   |                    | // note:
    // |                   |                   | -- func begin --   | //
    // |                   |                   |   ...   <-------\  | // 'block_nez + recur 1/break 1 + end'
    // |                   |                   |   ..a..         |  | // can be optimized as:
    // |                   |                   |   recur_nez 0 --/  | // 'recur_nez 0' and 'break_nez 0'
    // |                   |                   | end                |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    // | func foo {        | (function (code   | -- func begin --   |
    // |    if ..a.. {     |   (if (a)         |   ..a.. <------\   |
    // |       ..b..       |     (b)           |   block_alt -\ |   |
    // |    } else {       |     (code         |     ..b..    | |   |
    // |       ..c..       |       ..c..       |     break 0 -|-|-\ |
    // |       foo()       |       (tcall ...) |     ..c.. <--/ | | |
    // |    }              |     )             |     recur 1 ---/ | |
    // | }                 |   )               |   end            | |
    // |                   | ))                | end         <----/ |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    //
    //
    // ## break
    //
    // | structure         | assembly          | instructions(s)    |
    // |-------------------|-------------------|--------------------|
    // |                   |                   |                    |
    // | loop {            | (for (code        | block              |
    // |    ...            |    ...            |   ...        <---\ |
    // |    if ..a.. {     |    (when (a)      |   ..a..          | |
    // |      break;       |                   |   block_nez      | |
    // |    }              |       (break ...) |     break 1 ---\ | |
    // |    ...            |    )              |   end          | | |
    // | }                 |    ...            |   ...          | | |
    // |                   |    (recur ...)    |   recur 0  ----|-/ |
    // |                   | ))                | end            |   |
    // |                   |                   | ...      <-----/   |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    // | func foo {        | (function (code   | -- func begin --   |
    // |    ...            |   ...             |   ...              |
    // |    if ..a.. {     |   (when (a)       |   ..a..            |
    // |      return ...;  |                   |   block_nez        |
    // |    }              |       (return ..) |     break 1  ---\  |
    // |    ...            |   )               |   end           |  |
    // | }                 |   ...             |   ...           |  |
    // |                   | ))                | end         <---/  |
    // |                   |                   |                    |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|

    // function call
    // ------------
    //
    // general function call
    // (param function_public_index:i32) -> (...)
    call = 0x400,

    // dynamic function call
    // () (operand function_public_index:i32) -> (...)
    dyncall,

    // call a function which is specified at runtime.
    //
    // when a general or anonymous function is passed to another function as a parameter,
    // it passs a pointer of a struct 'closure_function_item' actually:
    //
    // closure_function_item {
    //     [ref_count],
    //     function_public_index:i32,
    //     captured_data:i64_ref
    // }
    //
    // "captured_data" is a structure that contains the data that function captured,
    // for example, an anonymous function captured an i32 number and a string, then
    // the 'captured data struct' is:
    //
    // captured_data_001: {
    //     f0:i32,
    //     f1:i64_ref
    // }

    // the target of dynamic function includes 'anonymous function' and 'regular function',
    // when the target is anonymous function, an additional parameter is appended to
    // the anonymous function automatically when it is compiled to assembly, e.g.
    //
    // `let a = fn (i32 a, i32 b) {...}`
    //
    // will be compiled to:
    //
    // `let a = fn (i32 a, i32 b, addr captured_data) {...}`
    //
    // ```text
    //                              /--> function_public_index --> fn (a, b, captured_data) {...}
    //                         /--->|
    //                         |    \--> captured_data
    //                         |
    // let a = filter(list, predicate)
    // ```
    //
    // when the target is a regular function, the compiler generates an anonymous function
    // for wrapping the general function, e.g.
    //
    // ```text
    //                              /--> function_public_index --> fn wrapper (a, b, captured_data) --> fn regular (a, b)
    //                         /--->|
    //                         |    \--> captured_data = 0
    //                         |
    // let a = filter(list, predicate)
    // ```

    // note:
    // 'function public index' includes the imported functions, it equals to
    // 'amount of imported functions' + 'function internal index'
    //
    // environment call
    // (param env_func_num:i32) -> (...)
    envcall,

    // the syscall arguments should be pushed on the stack first, e.g.
    //
    // | params_count   |
    // | syscall_num    |
    // | arg6           |
    // | arg5           |
    // | arg4           |
    // | arg3           |
    // | arg2           |                  | error no       |
    // | arg1           |     return -->   | return value   |
    // | ...            |                  | ...            |
    // \----------------/ <-- stack start  \----------------/
    //
    // when a syscall complete, the return value is store into the 'rax' register,
    // if the operation fails, the value is a negative value (rax < 0).
    // there is no 'errno' if invoke syscall by assembly directly.
    //
    // () (operand args..., syscall_num:i32, params_count: i32) -> (return_value:i64, error_no:i32)
    syscall,

    // external function call
    //
    // note that both 'syscall' and 'extcall' are optional, they may be
    // unavailable in some environment.
    // the supported feature list can be obtained through the instruction 'envcall' with code 'features'.
    //
    // (param external_function_index:i32) -> void/i32/i64/f32/f64
    extcall,

    // terminate VM
    // (param reason_code:u32) -> ()
    panic = 0x440,

    // get the address of data to host
    //
    // it is not safe to access data using memory addresses,
    // but this is necessary to talk to external libraries (e.g. C library)
    //
    // |--------------------|--------------|------------------|-----------------|
    // |                    | by indice    | by mem allocator | by host address |
    // |--------------------|--------------|------------------|-----------------|
    // | local vars         | safe         | -                | unsafe          |
    // |--------------------|--------------|------------------|-----------------|
    // | read-only data     |              |                  |                 |
    // | read-write data    | safe         | -                | unsafe          |
    // | uninitilized data  |              |                  |                 |
    // |--------------------|--------------|------------------|-----------------|
    // | heap               | -            | controllable     | unsafe          |
    // |--------------------|--------------|------------------|-----------------|
    //
    // note that the host address only valid in the current function and
    // its sub-functions. when a function exited, the function stack frame
    // will be destroied (or modified), as well as the local variables.
    host_addr_local, // (param reversed_index:i16 local_variable_index:i16 offset_bytes:i16) -> i64/i32
    host_addr_local_extend, // (param reversed_index:i16 local_variable_index:i32) (operand offset_bytes:i32) -> i64/i32
    host_addr_data,         // (param offset_bytes:i16 data_public_index:i32) -> i64/i32
    host_addr_data_extend,  // (param data_public_index:i32) (operand offset_bytes:i32) -> i64/i32
    host_addr_heap,         // (param offset_bytes:i32) (operand heap_addr:i64) -> i64/i32

    // create a new host function and map it to a VM function.
    // this host function named 'bridge funcion'
    //
    // return the existing bridge function if the bridge function corresponding
    // to the specified VM function has already been created.

    // it's commonly used for creating a callback function pointer for external C function.
    //
    // note:
    // - a bridge function (host function) will be created when `create_host_function` is executed,
    //   as well as the specified VM function will be appended to the "host function bridge table" to
    //   prevent duplicate creation.
    // - a bridge function is refered to a (module idx, function idx) tuple.
    // - the bridge function is created via JIT codegen.
    // - when the external C function calls the bridge function, a new thread is created.
    //
    // when the XiaoXUan VM is embed into a C or Rust application as a library, the C or Rust application
    // can call the VM function through the bridge function as if it calls a native function.
    //
    // call bridge functon from Rust application example:
    //
    // ref:
    // https://doc.rust-lang.org/nomicon/ffi.html
    // https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html
    // https://doc.rust-lang.org/stable/reference/items/functions.html
    //
    // ```rust
    // fn main() {
    //     let func_ptr = ... (pointer of the bridge function)
    //     let func_addr = ... (virtual memory address of the bridge function)
    //
    //     /*
    //      * mock pointer and address
    //      * let func_ptr = cb_func as *const extern "C" fn(usize, usize);
    //      * let func_ptr = cb_func as *const u8;
    //      * let func_addr = func_ptr as usize;
    //      * */
    //
    //     println!("{:p}", func_ptr);
    //     println!("0x{:x}", func_addr);
    //
    //     let func_from_ptr: fn(usize, usize) = unsafe { std::mem::transmute(func_ptr) };
    //     (func_from_ptr)(11, 13);
    //
    //     let ptr_from_addr = func_addr as *const ();
    //     let func_from_addr: fn(usize, usize) = unsafe { std::mem::transmute(ptr_from_addr) };
    //     (func_from_addr)(17, 19);
    // }
    //
    // #[no_mangle]
    // pub extern "C" fn cb_func(a1: usize, a2: usize) {
    //     println!("numbers: {},{}", a1, a2);
    // }
    // ```
    //
    // call bridge functon from C application example:
    //
    // ```c
    // int main(void)
    // {
    //     void *func_ptr = ...
    //     int (*func_from_ptr)(int, int) = (int (*)(int, int))func_ptr;
    //     printf("1+2=%d\n", (*func_from_ptr)(1, 2));
    //     exit(EXIT_SUCCESS);
    // }
    // ```
    //
    // (param function_public_index:i32) -> i64/i32
    host_addr_function,

    // copy data from VM heap to host memory
    // () (operand dst_pointer:i64 src_offset:i64 length_in_bytes:i64) -> ()
    host_copy_heap_to_memory,

    // copy data from host memory to VM heap
    // () (operand dst_offset:i64 src_pointer:i64 length_in_bytes:i64) -> ()
    host_copy_memory_to_heap,

    // copy data between host memory
    // () (operand dst_pointer:i64 src_pointer:i64 length_in_bytes:i64) -> ()
    host_memory_copy,
    // atomic_rmw_add_i32 = 0xd00,      // fn (addr, value) -> old_value
    // atomic_rmw_sub_i32,      // ...
    // atomic_rmw_and_i32,      // ...
    // atomic_rmw_or_i32,       // ...
    // atomic_rmw_xor_i32,      // ...
    // atomic_rmw_exchange_i32, // ...
    // atomic_cas_i32,          // fn (addr, expect_value, new_value) -> old_value
    //
    // atomic_rmw_add_i64,      // fn (addr, value) -> old_value
    // atomic_rmw_sub_i64,      // ...
    // atomic_rmw_and_i64,      // ...
    // atomic_rmw_or_i64,       // ...
    // atomic_rmw_xor_i64,      // ...
    // atomic_rmw_exchange_i64, // ...
    // atomic_cas_i64,          // fn (addr, expect_value, new_value) -> old_value

    //
    // SIMD/Vectorization
    //

    // ref:
    // - https://github.com/rust-lang/portable-simd
    // - https://doc.rust-lang.org/std/simd/index.html
    // - https://github.com/rust-lang/packed_simd
    // - https://github.com/rust-lang/rfcs/blob/master/text/2325-stable-simd.md
}

impl Opcode {
    pub fn get_name(&self) -> &'static str {
        match self {
            Opcode::nop => "nop",
            //
            Opcode::imm_i32 => "imm_i32",
            Opcode::imm_i64 => "imm_i64",
            Opcode::imm_f32 => "imm_f32",
            Opcode::imm_f64 => "imm_f64",
            //
            Opcode::data_load_i64 => "data_load_i64",
            Opcode::data_load_i32_s => "data_load_i32_s",
            Opcode::data_load_i32_u => "data_load_i32_u",
            Opcode::data_load_i16_s => "data_load_i16_s",
            Opcode::data_load_i16_u => "data_load_i16_u",
            Opcode::data_load_i8_s => "data_load_i8_s",
            Opcode::data_load_i8_u => "data_load_i8_u",
            Opcode::data_load_f64 => "data_load_f64",
            Opcode::data_load_f32 => "data_load_f32",
            Opcode::data_store_i64 => "data_store_i64",
            Opcode::data_store_i32 => "data_store_i32",
            Opcode::data_store_i16 => "data_store_i16",
            Opcode::data_store_i8 => "data_store_i8",
            Opcode::data_store_f64 => "data_store_f64",
            Opcode::data_store_f32 => "data_store_f32",
            //
            Opcode::data_load_extend_i64 => "data_load_extend_i64",
            Opcode::data_load_extend_i32_s => "data_load_extend_i32_s",
            Opcode::data_load_extend_i32_u => "data_load_extend_i32_u",
            Opcode::data_load_extend_i16_s => "data_load_extend_i16_s",
            Opcode::data_load_extend_i16_u => "data_load_extend_i16_u",
            Opcode::data_load_extend_i8_s => "data_load_extend_i8_s",
            Opcode::data_load_extend_i8_u => "data_load_extend_i8_u",
            Opcode::data_load_extend_f64 => "data_load_extend_f64",
            Opcode::data_load_extend_f32 => "data_load_extend_f32",
            //
            Opcode::data_store_extend_i64 => "data_store_extend_i64",
            Opcode::data_store_extend_i32 => "data_store_extend_i32",
            Opcode::data_store_extend_i16 => "data_store_extend_i16",
            Opcode::data_store_extend_i8 => "data_store_extend_i8",
            Opcode::data_store_extend_f64 => "data_store_extend_f64",
            Opcode::data_store_extend_f32 => "data_store_extend_f32",
            //
            Opcode::local_load_i64 => "local_load_64",
            Opcode::local_load_i32_s => "local_load_i32_s",
            Opcode::local_load_i32_u => "local_load_i32_u",
            Opcode::local_load_i16_s => "local_load_i16_s",
            Opcode::local_load_i16_u => "local_load_i16_u",
            Opcode::local_load_i8_s => "local_load_i8_s",
            Opcode::local_load_i8_u => "local_load_i8_u",
            Opcode::local_load_f64 => "local_load_f64",
            Opcode::local_load_f32 => "local_load_f32",
            Opcode::local_store_i64 => "local_store_i64",
            Opcode::local_store_i32 => "local_store_i32",
            Opcode::local_store_i16 => "local_store_i16",
            Opcode::local_store_i8 => "local_store_i8",
            Opcode::local_store_f64 => "local_store_f64",
            Opcode::local_store_f32 => "local_store_f32",
            //
            Opcode::local_load_extend_i64 => "local_load_extend_i64",
            Opcode::local_load_extend_i32_s => "local_load_extend_i32_s",
            Opcode::local_load_extend_i32_u => "local_load_extend_i32_u",
            Opcode::local_load_extend_i16_s => "local_load_extend_i16_s",
            Opcode::local_load_extend_i16_u => "local_load_extend_i16_u",
            Opcode::local_load_extend_i8_s => "local_load_extend_i8_s",
            Opcode::local_load_extend_i8_u => "local_load_extend_i8_u",
            Opcode::local_load_extend_f64 => "local_load_extend_f64",
            Opcode::local_load_extend_f32 => "local_load_extend_f32",
            //
            Opcode::local_store_extend_i64 => "local_store_extend_i64",
            Opcode::local_store_extend_i32 => "local_store_extend_i32",
            Opcode::local_store_extend_i16 => "local_store_extend_i16",
            Opcode::local_store_extend_i8 => "local_store_extend_i8",
            Opcode::local_store_extend_f64 => "local_store_extend_f64",
            Opcode::local_store_extend_f32 => "local_store_extend_f32",
            //
            Opcode::heap_load_i64 => "heap_load_i64",
            Opcode::heap_load_i32_s => "heap_load_i32_s",
            Opcode::heap_load_i32_u => "heap_load_i32_u",
            Opcode::heap_load_i16_s => "heap_load_i16_s",
            Opcode::heap_load_i16_u => "heap_load_i16_u",
            Opcode::heap_load_i8_s => "heap_load_i8_s",
            Opcode::heap_load_i8_u => "heap_load_i8_u",
            Opcode::heap_load_f64 => "heap_load_f64",
            Opcode::heap_load_f32 => "heap_load_f32",
            //
            Opcode::heap_store_i64 => "heap_store_i64",
            Opcode::heap_store_i32 => "heap_store_i32",
            Opcode::heap_store_i16 => "heap_store_i16",
            Opcode::heap_store_i8 => "heap_store_i8",
            Opcode::heap_store_f64 => "heap_store_f64",
            Opcode::heap_store_f32 => "heap_store_f32",
            //
            Opcode::heap_fill => "heap_fill",
            Opcode::heap_copy => "heap_copy",
            Opcode::heap_capacity => "heap_capacity",
            Opcode::heap_resize => "heap_resize",
            //
            Opcode::truncate_i64_to_i32 => "truncate_i64_to_i32",
            Opcode::extend_i32_s_to_i64 => "extend_i32_s_to_i64",
            Opcode::extend_i32_u_to_i64 => "extend_i32_u_to_i64",
            Opcode::demote_f64_to_f32 => "demote_f64_to_f32",
            Opcode::promote_f32_to_f64 => "promote_f32_to_f64",
            //
            Opcode::convert_f32_to_i32_s => "convert_f32_to_i32_s",
            Opcode::convert_f32_to_i32_u => "convert_f32_to_i32_u",
            Opcode::convert_f64_to_i32_s => "convert_f64_to_i32_s",
            Opcode::convert_f64_to_i32_u => "convert_f64_to_i32_u",
            Opcode::convert_f32_to_i64_s => "convert_f32_to_i64_s",
            Opcode::convert_f32_to_i64_u => "convert_f32_to_i64_u",
            Opcode::convert_f64_to_i64_s => "convert_f64_to_i64_s",
            Opcode::convert_f64_to_i64_u => "convert_f64_to_i64_u",
            //
            Opcode::convert_i32_s_to_f32 => "convert_i32_s_to_f32",
            Opcode::convert_i32_u_to_f32 => "convert_i32_u_to_f32",
            Opcode::convert_i64_s_to_f32 => "convert_i64_s_to_f32",
            Opcode::convert_i64_u_to_f32 => "convert_i64_u_to_f32",
            Opcode::convert_i32_s_to_f64 => "convert_i32_s_to_f64",
            Opcode::convert_i32_u_to_f64 => "convert_i32_u_to_f64",
            Opcode::convert_i64_s_to_f64 => "convert_i64_s_to_f64",
            Opcode::convert_i64_u_to_f64 => "convert_i64_u_to_f64",
            //
            Opcode::eqz_i32 => "eqz_i32",
            Opcode::nez_i32 => "nez_i32",
            Opcode::eq_i32 => "eq_i32",
            Opcode::ne_i32 => "ne_i32",
            Opcode::lt_i32_s => "lt_i32_s",
            Opcode::lt_i32_u => "lt_i32_u",
            Opcode::gt_i32_s => "gt_i32_s",
            Opcode::gt_i32_u => "gt_i32_u",
            Opcode::le_i32_s => "le_i32_s",
            Opcode::le_i32_u => "le_i32_u",
            Opcode::ge_i32_s => "ge_i32_s",
            Opcode::ge_i32_u => "ge_i32_u",
            //
            Opcode::eqz_i64 => "eqz_i64",
            Opcode::nez_i64 => "nez_i64",
            Opcode::eq_i64 => "eq_i64",
            Opcode::ne_i64 => "ne_i64",
            Opcode::lt_i64_s => "lt_i64_s",
            Opcode::lt_i64_u => "lt_i64_u",
            Opcode::gt_i64_s => "gt_i64_s",
            Opcode::gt_i64_u => "gt_i64_u",
            Opcode::le_i64_s => "le_i64_s",
            Opcode::le_i64_u => "le_i64_u",
            Opcode::ge_i64_s => "ge_i64_s",
            Opcode::ge_i64_u => "ge_i64_u",
            //
            Opcode::eq_f32 => "eq_f32",
            Opcode::ne_f32 => "ne_f32",
            Opcode::lt_f32 => "lt_f32",
            Opcode::gt_f32 => "gt_f32",
            Opcode::le_f32 => "le_f32",
            Opcode::ge_f32 => "ge_f32",
            //
            Opcode::eq_f64 => "eq_f64",
            Opcode::ne_f64 => "ne_f64",
            Opcode::lt_f64 => "lt_f64",
            Opcode::gt_f64 => "gt_f64",
            Opcode::le_f64 => "le_f64",
            Opcode::ge_f64 => "ge_f64",
            //
            Opcode::add_i32 => "add_i32",
            Opcode::sub_i32 => "sub_i32",
            Opcode::add_imm_i32 => "add_imm_i32",
            Opcode::sub_imm_i32 => "sub_imm_i32",
            Opcode::mul_i32 => "mul_i32",
            Opcode::div_i32_s => "div_i32_s",
            Opcode::div_i32_u => "div_i32_u",
            Opcode::rem_i32_s => "rem_i32_s",
            Opcode::rem_i32_u => "rem_i32_u",
            //
            Opcode::add_i64 => "add_i64",
            Opcode::sub_i64 => "sub_i64",
            Opcode::add_imm_i64 => "add_imm_i64",
            Opcode::sub_imm_i64 => "sub_imm_i64",
            Opcode::mul_i64 => "mul_i64",
            Opcode::div_i64_s => "div_i64_s",
            Opcode::div_i64_u => "div_i64_u",
            Opcode::rem_i64_s => "rem_i64_s",
            Opcode::rem_i64_u => "rem_i64_u",
            //
            Opcode::add_f32 => "add_f32",
            Opcode::sub_f32 => "sub_f32",
            Opcode::mul_f32 => "mul_f32",
            Opcode::div_f32 => "div_f32",
            //
            Opcode::add_f64 => "add_f64",
            Opcode::sub_f64 => "sub_f64",
            Opcode::mul_f64 => "mul_f64",
            Opcode::div_f64 => "div_f64",
            //
            Opcode::and => "and",
            Opcode::or => "or",
            Opcode::xor => "xor",
            Opcode::not => "not",
            //
            Opcode::leading_zeros_i32 => "leading_zeros_i32",
            Opcode::leading_ones_i32 => "leading_ones_i32",
            Opcode::trailing_zeros_i32 => "trailing_zeros_i32",
            Opcode::count_ones_i32 => "count_ones_i32",
            Opcode::shift_left_i32 => "shift_left_i32",
            Opcode::shift_right_i32_s => "shift_right_i32_s",
            Opcode::shift_right_i32_u => "shift_right_i32_u",
            Opcode::rotate_left_i32 => "rotate_left_i32",
            Opcode::rotate_right_i32 => "rotate_right_i32",
            Opcode::leading_zeros_i64 => "leading_zeros_i64",
            Opcode::leading_ones_i64 => "leading_ones_i64",
            Opcode::trailing_zeros_i64 => "trailing_zeros_i64",
            Opcode::count_ones_i64 => "count_ones_i64",
            Opcode::shift_left_i64 => "shift_left_i64",
            Opcode::shift_right_i64_s => "shift_right_i64_s",
            Opcode::shift_right_i64_u => "shift_right_i64_u",
            Opcode::rotate_left_i64 => "rotate_left_i64",
            Opcode::rotate_right_i64 => "rotate_right_i64",
            //
            Opcode::abs_i32 => "abs_i32",
            Opcode::neg_i32 => "neg_i32",
            Opcode::abs_i64 => "abs_i64",
            Opcode::neg_i64 => "neg_i64",
            //
            Opcode::abs_f32 => "abs_f32",
            Opcode::neg_f32 => "neg_f32",
            Opcode::copysign_f32 => "copysign_f32",
            Opcode::sqrt_f32 => "sqrt_f32",
            Opcode::min_f32 => "min_f32",
            Opcode::max_f32 => "max_f32",
            Opcode::ceil_f32 => "ceil_f32",
            Opcode::floor_f32 => "floor_f32",
            Opcode::round_half_away_from_zero_f32 => "round_half_away_from_zero_f32",
            Opcode::round_half_to_even_f32 => "round_half_to_even_f32",
            Opcode::trunc_f32 => "trunc_f32",
            Opcode::fract_f32 => "fract_f32",
            Opcode::cbrt_f32 => "cbrt_f32",
            Opcode::exp_f32 => "exp_f32",
            Opcode::exp2_f32 => "exp2_f32",
            Opcode::ln_f32 => "ln_f32",
            Opcode::log2_f32 => "log2_f32",
            Opcode::log10_f32 => "log10_f32",
            Opcode::sin_f32 => "sin_f32",
            Opcode::cos_f32 => "cos_f32",
            Opcode::tan_f32 => "tan_f32",
            Opcode::asin_f32 => "asin_f32",
            Opcode::acos_f32 => "acos_f32",
            Opcode::atan_f32 => "atan_f32",
            Opcode::pow_f32 => "pow_f32",
            Opcode::log_f32 => "log_f32",
            //
            Opcode::abs_f64 => "abs_f64",
            Opcode::neg_f64 => "neg_f64",
            Opcode::copysign_f64 => "copysign_f64",
            Opcode::sqrt_f64 => "sqrt_f64",
            Opcode::min_f64 => "min_f64",
            Opcode::max_f64 => "max_f64",
            Opcode::ceil_f64 => "ceil_f64",
            Opcode::floor_f64 => "floor_f64",
            Opcode::round_half_away_from_zero_f64 => "round_half_away_from_zero_f64",
            Opcode::round_half_to_even_f64 => "round_half_to_even_f64",
            Opcode::trunc_f64 => "trunc_f64",
            Opcode::fract_f64 => "fract_f64",
            Opcode::cbrt_f64 => "cbrt_f64",
            Opcode::exp_f64 => "exp_f64",
            Opcode::exp2_f64 => "exp2_f64",
            Opcode::ln_f64 => "ln_f64",
            Opcode::log2_f64 => "log2_f64",
            Opcode::log10_f64 => "log10_f64",
            Opcode::sin_f64 => "sin_f64",
            Opcode::cos_f64 => "cos_f64",
            Opcode::tan_f64 => "tan_f64",
            Opcode::asin_f64 => "asin_f64",
            Opcode::acos_f64 => "acos_f64",
            Opcode::atan_f64 => "atan_f64",
            Opcode::pow_f64 => "pow_f64",
            Opcode::log_f64 => "log_f64",
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
            //
            Opcode::host_addr_local => "host_addr_local",
            Opcode::host_addr_local_extend => "host_addr_local_extend",
            Opcode::host_addr_data => "host_addr_data",
            Opcode::host_addr_data_extend => "host_addr_data_extend",
            Opcode::host_addr_heap => "host_addr_heap",
            Opcode::host_copy_heap_to_memory => "host_copy_heap_to_memory",
            Opcode::host_copy_memory_to_heap => "host_copy_memory_to_heap",
            Opcode::host_memory_copy => "host_memory_copy",
            Opcode::host_addr_function => "host_addr_function",
        }
    }
}
