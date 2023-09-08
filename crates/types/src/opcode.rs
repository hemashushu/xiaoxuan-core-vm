// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// note:
//
// the data types that VM supports:
//
// - i32
// - i64
// - f32
// - f64
//
// the note of floating-point number:
//
// like most processors and VM, f32/f64 is stored with
// IEEE 754-2008 format, in addition to the normal floating-point numbers,
// there are some special values (variants):
//
// - NaN
// - +Infinity, -Infinity
// - +0, -0
//
// these variants make the programming language become complex and
// sometimes make problems unpredictable, for example:
//
// - NaN != NaN
// - (NaN < 0) == false, (NaN > 0) == false
// - (a != b) cannot assert that !(a == b)
// - 1/-0 = -Infinity
//
// ref:
// - https://en.wikipedia.org/wiki/Floating-point_arithmetic
// - https://en.wikipedia.org/wiki/IEEE_754
// - https://en.wikipedia.org/wiki/IEEE_754-2008_revision
// - https://en.wikipedia.org/wiki/Signed_zero
// - https://en.wikipedia.org/wiki/Subnormal_number
// - https://en.wikipedia.org/wiki/Single-precision_floating-point_format
// - https://en.wikipedia.org/wiki/Half-precision_floating-point_format
// - https://doc.rust-lang.org/std/primitive.f32.html
//
// to simplify the upper-level programming language, the f32/f64 in XiaoXuan VM
// only support the normal (includes subnormal) floating-point number and +0,
// and the VM will simply throw an exception when other variants are encountered.
//
// e.g. for f32:
//
//                   MSB                                  LSB
//                   sign    exponent (8 bits)   fraction (23 bits)                                 implicit leading number 1
//                   ----    -----------------   ------------------                                 |
//                   |       |                   |                                                  v
//          format   0       00000000            0000000000 0000000000 000     value = (-1)^sign * (1 fraction) * 2^(exponent-offset), offset = 127 for f32, 1023 for f64
//          example  1       10000001            0100000000 0000000000 000     value = (-1)^1 * (1 + 0*2^(-1) + 1*2^(-2)) * 2^(129-127) = -1 * 1.25 * 4 = -5.0
//          example  0       01111100            0100000000 0000000000 000     value = (-1)^0 * 1.25 * 2^(-3) = 0.15625
// support?
//  Y                -       00000001--\
//                           11111110--/         ---------- ---------- ---     normal number
//  Y                0       00000000            0000000000 0000000000 000     value = +0
//  N                1       00000000            0000000000 0000000000 000     value = -0
//  Y                -       00000000            ---------- ---------- ---     subnormal number (i.e., numbers between 0 and MIN)
//  N                -       11111111            0000000000 0000000000 000     value = +/- Infinity
//  N                -       11111111            ---------- ---------- ---     NaN
//
// when load data from memory as floating-point number, there is the following check:
// 1. exponent between (00000001) and (11111110): pass
// 2. exponent is zero, if the sign bit is zero: pass
// 3. failed.
//
// in other words, the +/-Infinity, -0, NaN, will cause the VM to throw exceptions.

// the note of data layout:
//
// the default implement of XiaoXuan VM is stack-base, which its operands are
// 8-byte raw data, the data presentation is as the following:
//
//    MSB                             LSB
// 64 |---------------------------------| 0
//    |   16     16      16     8    8  | bits
//    |-------|-------|-------|----|----|
//    |000000000000000|-sign-extend| i8 |
//    |---------------------------------|
//    |000000000000000|-sign--|   i16   |
//    |---------------------------------|
//    |000000000000000|        i32      |
//    |---------------------------------|
//    |               i64               |
//    |---------------------------------|
//    |000000000000000|        f32      |
//    |---------------------------------|
//    |               f64               |
//    |---------------------------------|
//
// the i8 and i16 are sign-extend as i32, so there are actually two kinds of length for
// operands, 32-bit and 64-bit operands.

// the note of instruction format
//
// XiaoXuan VM instructions are not fixed-length code. there are
// 16 bits, 32 bits, 64 bits and 96 bits instructions, sometimes it is
// necessary to insert the `nop` instruction after the 16 bits instruction
// to form 32 bits (4-byte) alignment.
//
// - 16 bits:
//   instructions without parameters, such as `i32_eq`, `i32_add`.
// - 32 bits:
//   instructions with one parameter, such as `i32_shl`.
//   16 bits opcode + 16 bits parameter
// - 64 bits:
//   instructions with one parameter, such as `i32_imm`, `f32_imm`, `local_get`, `data_get`, `block`, `call`.
//   16 bits opcode + 16 bits padding + 32 bits parameter
// - 64 bits:
//   instructions with two parameters, there are `break`, `recur`.
//   16 bits opcode + 16 bits parameter + 32 bits parameter
// - 96 bits
//   instructions with two parameters, there are `i64_imm`, `f64_imm`, `block_nez`,
//   `host_addr_memory`, `host_addr_shared_memory`.
//   16 bits opcode + 16 bits padding + 32 bits parameter 1 + 32 bits parameter 2
//
// the instruction schemes:
//
// - [opcode i16]
// - [opcode i16] - [param i16      ]
// - [opcode i16] - [param i16      ] + [param i32]
// - [opcode i16] - [padding 16 bits] + [param i32]
// - [opcode i16] - [padding 16 bits] + [param i32] + [param i32]
//
// opcode scheme:
//
// MSB           LSB
// 00000000 00000000
// -------- --------
// |        |
// |        \ items
// |
// \ catalogs
#[repr(u16)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum Opcode {
    nop = 0x100,        // instruction to do nothing,
                        // it's usually used for padding instructions to archieve 32/64 bits (4/8-byte) alignment.
    drop,               // drop one operand (the top most operand)
    duplicate,          // duplicate one operand (the top most operand)

    //
    // immediate number
    //

    i32_imm = 0x200,    // (param immediate_number:i32)
    i64_imm,            // (param immediate_number_low:i32, immediate_number_high:i32)
    f32_imm,            // (param immediate_number:i32)
    f64_imm,            // (param immediate_number_low:i32, immediate_number_high:i32)

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
    // local data/variables loading and storing
    // load the specified local variable and push onto to the stack, or
    // pop one operand off the stack and set the specified local variable.
    //

    local_load = 0x300,     // load local variable              (param offset_bytes:i16 local_variable_index:i32)
    local_load_i8_s,        //                                  (param offset_bytes:i16 local_variable_index:i32)
    local_load_i8_u,        //                                  (param offset_bytes:i16 local_variable_index:i32)
    local_load_i16_s,       //                                  (param offset_bytes:i16 local_variable_index:i32)
    local_load_i16_u,       //                                  (param offset_bytes:i16 local_variable_index:i32)
    local_load_i32,         //                                  (param offset_bytes:i16 local_variable_index:i32)
    local_load_f32,         // Load f32 with floating-point validity check.     (param offset_bytes:i16 heap_index:i32)
    local_load_f64,         // Load f64 with floating-point validity check.     (param offset_bytes:i16 heap_index:i32)
    local_store,            // store local variable             (param offset_bytes:i16 local_variable_index:i32)
    local_store8,           //                                  (param offset_bytes:i16 local_variable_index:i32)
    local_store16,          //                                  (param offset_bytes:i16 local_variable_index:i32)
    local_store32,          //                                  (param offset_bytes:i16 local_variable_index:i32)

    //
    // data (thread-local variables) loading and storing
    // load the specified data and push onto to the stack, or
    // pop one operand off the stack and set the specified data
    //

    data_load = 0x400,      // load data                        (param offset_bytes:i16 data_index:i32)
    data_load_i8_s,         //                                  (param offset_bytes:i16 data_index:i32)
    data_load_i8_u,         //                                  (param offset_bytes:i16 data_index:i32)
    data_load_i16_s,        //                                  (param offset_bytes:i16 data_index:i32)
    data_load_i16_u,        //                                  (param offset_bytes:i16 data_index:i32)
    data_load_i32,          //                                  (param offset_bytes:i16 data_index:i32)
    data_load_f32,          // Load f32 with floating-point validity check.     (param offset_bytes:i16 heap_index:i32)
    data_load_f64,          // Load f64 with floating-point validity check.     (param offset_bytes:i16 heap_index:i32)
    data_store,             // store data                       (param offset_bytes:i16 data_index:i32)
    data_store8,            //                                  (param offset_bytes:i16 data_index:i32)
    data_store16,           //                                  (param offset_bytes:i16 data_index:i32)
    data_store32,           //                                  (param offset_bytes:i16 data_index:i32)

    //
    // heap (thread-local memory) loading and storing
    //

    heap_load = 0x500,      // load heap                        (param offset_bytes:i16 heap_index:i32)
    heap_load_i8_s,         //                                  (param offset_bytes:i16 heap_index:i32)
    heap_load_i8_u,         //                                  (param offset_bytes:i16 heap_index:i32)
    heap_load_i16_s,        //                                  (param offset_bytes:i16 heap_index:i32)
    heap_load_i16_u,        //                                  (param offset_bytes:i16 heap_index:i32)
    heap_load_i32,          //                                  (param offset_bytes:i16 heap_index:i32)
    heap_load_f32,          // Load f32 with floating-point validity check.     (param offset_bytes:i16 heap_index:i32)
    heap_load_f64,          // Load f64 with floating-point validity check.     (param offset_bytes:i16 heap_index:i32)
    heap_store,             // store heap                       (param offset_bytes:i16 heap_index:i32)
    heap_store8,            //                                  (param offset_bytes:i16 heap_index:i32)
    heap_store16,           //                                  (param offset_bytes:i16 heap_index:i32)
    heap_store32,           //                                  (param offset_bytes:i16 heap_index:i32)

    heap_fill,              // fill the specified memory region with specified value    (operand start_addr:i64, count:i64, value:i8)
    heap_copy,              // copy the specified memory region to specified address    (operand src_addr:i64, dst_addr:i64, length:i64)

    heap_size,              // the result is the amount of the thread-local memory (i.e. heap) pages, each page is 32 KiB
    heap_grow,              //                                  (operand pages:i64)

    //
    // conversion
    //

    // demote i64 to i32
    // discard the high 32 bits of an i64 number directly
    i32_demote_i64 = 0x600,

    // promote i32 to i64
    i64_promote_i32_s,
    i64_promote_i32_u,

    // demote f64 to f32
    f32_demote_f64,

    // promote f32 to f64
    f64_promote_f32,

    // convert float to int
    // truncate fractional part
    i32_trunc_f32_s,
    i32_trunc_f32_u,
    i32_trunc_f64_s,
    i32_trunc_f64_u,
    i64_trunc_f32_s,
    i64_trunc_f32_u,
    i64_trunc_f64_s,
    i64_trunc_f64_u,

    // convert int to float
    f32_convert_i32_s,
    f32_convert_i32_u,
    f32_convert_i64_s,
    f32_convert_i64_u,
    f64_convert_i32_s,
    f64_convert_i32_u,
    f64_convert_i64_s,
    f64_convert_i64_u,

    //
    // comparsion
    //

    // for the binary operations, the first one popped up from the
    // stack is the right-hand-side value, e.g.
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

    // the result of the comparison is a logical TRUE or FALSE, when
    // the result is TRUE, the number `1:i32` is pushed onto the stack,
    // and vice versa the number `0:i32` is pushed onto.

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

    i32_eqz = 0x700,
    i32_eq,
    i32_nez,
    i32_ne,
    i32_lt_s,
    i32_lt_u,
    i32_gt_s,
    i32_gt_u,
    i32_le_s,
    i32_le_u,
    i32_ge_s,
    i32_ge_u,

    i64_eqz,
    i64_eq,
    i64_nez,
    i64_ne,
    i64_lt_s,
    i64_lt_u,
    i64_gt_s,
    i64_gt_u,
    i64_le_s,
    i64_le_u,
    i64_ge_s,
    i64_ge_u,

    f32_eq,
    f32_ne,
    f32_lt,
    f32_gt,
    f32_le,
    f32_ge,

    f64_eq,
    f64_ne,
    f64_lt,
    f64_gt,
    f64_le,
    f64_ge,

    //
    // arithmetic
    //

    i32_add = 0x800,
    i32_sub,
    i32_mul,
    i32_div_s,
    i32_div_u,
    i32_rem_s, // calculate the remainder
    i32_rem_u,

    // remainder vs modulus
    // --------------------
    // The remainder (%) operator returns the remainder left over when one operand is
    // divided by a second operand. It always takes the sign of the dividend.
    // For the operation n % d, n is called the dividend and d is called the divisor.
    //
    // (13 % 5) = 3
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

    // instruction `i32_add` example:
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

    i64_add,
    i64_sub,
    i64_mul,
    i64_div_s,
    i64_div_u,
    i64_rem_s,
    i64_rem_u,

    f32_add,
    f32_sub,
    f32_mul,
    f32_div,

    f64_add,
    f64_sub,
    f64_mul,
    f64_div,

    //
    // bitwise
    //

    // see also:
    // https://en.wikipedia.org/wiki/Bitwise_operation

    i32_and = 0x900,    // bitwise AND
    i32_or,             // bitwise OR
    i32_xor,            // bitwise XOR
    i32_not,            // bitwise NOT
    i32_clz,            // count leading zeros
    i32_ctz,            // count trailing zeros
    i32_popcnt,         // count the total amount of value `1` bits
    i32_shl,            // shift left                   (param move_bits:i16)
    i32_shr_s,          // arithmetic right shift       (param move_bits:i16)
    i32_shr_u,          // logical right shift          (param move_bits:i16)
    i32_rotl,           // left rotate                  (param move_bits:i16)
    i32_rotr,           // right rotate                 (param move_bits:i16)


    // instruction `i32.shl` example:
    //
    // ```
    // ;; load two numbers onto the stack
    // (i32.imm 7)              ;; 00000111
    // ;; perform a bitwise left-shift
    // ;; left shift one spot
    // ;; the top item on the stack will be 14 (00001110)
    // (i32.shl 1)
    // ```

    // instructions `i32.clz`, `i32.ctz` and `i32.popcnt` examples:
    //
    // ```
    // ;; load a number onto the stack
    // (i32.imm 8_388_608)      ;; 00000000_10000000_00000000_00000000
    // ;; count leading zeros
    // ;; the top item on the stack will be 8
    // i32.clz
    //
    // ;; load a number onto the stack
    // (i32.imm 8_388_608)      ;; 00000000_10000000_00000000_00000000
    // ;; count trailing zeros
    // ;; the top item on the stack will be 23
    // i32.ctz
    //
    // ;; load a number onto the stack
    // (i32.imm 130)            ;; 10000010
    // ;; count the 1s
    // ;; the top item on the stack will be 2
    // i32.popcnt
    // ```

    i64_and,
    i64_or,
    i64_xor,
    i64_not,
    i64_clz,
    i64_ctz,
    i64_popcnt,
    i64_shl,            // shift left                   (param move_bits:i16)
    i64_shr_s,          // arithmetic right shift       (param move_bits:i16)
    i64_shr_u,          // logical right shift          (param move_bits:i16)
    i64_rotl,           // left rotate                  (param move_bits:i16)
    i64_rotr,           // right rotate                 (param move_bits:i16)


    //
    // math functions
    //

    f32_abs = 0xa00,
    f32_neg,
    f32_ceil,
    f32_floor,
    f32_trunc,
    f32_round_half_to_even,
    f32_sqrt,
    f32_pow,    // x^y
    f32_exp,    // e^x
    f32_sin,
    f32_cos,
    f32_tan,
    f32_asin,
    f32_acos,
    f32_atan,
    f32_copysign, // copy sign

    // instruction `f32_copy_sign` example:
    //
    // |           | --> stack end
    // | sign from |
    // | sign to   |
    // \-----------/ --> stack start
    //
    // ```
    // ;; load two numbers onto the stack
    // (f32.imm 10)
    // (f32.imm -1)
    //
    // ;; now the stack layout is:
    // ;;
    // ;; |    |
    // ;; | -1 |
    // ;; | 10 |
    // ;; \----/
    //
    // ;; copy the sign of the top most operand.
    // ;; the top item on the stack will be -10
    // f32.copysign
    //
    // ;; now the stack layout is:
    // ;;
    // ;; |     |
    // ;; | -10 |
    // ;; \-----/
    // ```

    f64_abs,
    f64_neg,
    f64_ceil,
    f64_floor,
    f64_trunc,
    f64_round_half_to_even,
    f64_sqrt,
    f64_pow,    // x^y
    f64_exp,    // e^x
    f64_sin,
    f64_cos,
    f64_tan,
    f64_asin,
    f64_acos,
    f64_atan,
    f64_copysign, // copy sign

    //
    // control flow
    //

    end = 0xb00,        // finish a block or a function.
    // when the 'end' instruction is executed, a stack frame will be removed and
    // the results of the current block or function will be placed on the top of stack.

    block,              // (param type_index:i32)
                        //
                        // create a block region. a block is similar to a function, it also has
                        // parameters and results, it shares the type with function, so the 'block'
                        // instruction has parameter 'type_index'.
                        // this instruction will make VM to create a stack frame which is called 'block frame'.

    return_,            // (param skip_depth:i16, end_inst_offset:i32)

    // the 'return' instruction is similar to the 'end' instruction, it is also
    // used for finishing a block or a function.
    // for a block, a block stack frame will be removed and jump to the instruction
    // that next to the 'end' instruction.
    // for a function, a function stack frame will be removed and return the the
    // instruction next to the 'call' instruction.
    // the operands for the amount of the block or function are placed
    // on the top of stack.

    // the value of the parameter 'end_inst_offset' should be (`addr of end` - `addr of return`)
    // e.g.
    //
    // ```bytecode
    // 0d0000 block 0           ;; the size of 'block' instruction is 8 bytes
    // 0d0008   nop
    // 0d0010   return 0 12   ;; the size of 'return' instruction is 8 bytes, (12 = 22 - 10) --\
    // 0d0018   nop           ;;                                                               |
    // 0d0020   nop           ;;                                                               |
    // 0d0022 end             ;;                                                               |
    // 0d0028 nop             ;; <-- jump to here ---------------------------------------------/
    // ```
    //
    // the 'return' instruction can cross over multiple block nested.
    // when the parameter 'skip_depth' is 0, it simply finish the current block.
    // when the value is greater than 0, multiple block stack frame will be removed and
    // the operands for the amount of the 'target block results' are placed
    // on the top of stack. e.g.
    //
    // ```bytecode
    // 0d0000 block 0
    // 0d0008   block 0
    // 0d0016     block 0
    // 0d0024       nop
    // 0d0026       return 1 14   ;; (14 = 40 - 26) --------\
    // 0d0034       nop           ;;                        |
    // 0d0036     end             ;;                        |
    // 0d0038     nop             ;;                        |
    // 0d0040   end               ;;                        |
    // 0d0042   nop               ;; <----------------------/ jump to here
    // 0d0044 end
    // ```

    recur,              // (param skip_depth:i16, start_inst_offset:i32)

    // the 'recur' instruction make VM to jump to the instruction next to the 'block' or 'block_nez',
    // as well as all the operands in the current stack frame will be removed, and the operands
    // for the amount of the 'target block/function params' are placed on the top of stack.
    // it is commonly used to construct the 'while/for' structures in general programming languages,
    // it is also used to implement the TCO (tail call optimization).
    //
    // ```bytecode
    // 0d0000 block 0
    // 0d0008   nop             ;; <-----------------\ jump to here
    // 0d0010   block 0         ;;                   |
    // 0d0018     nop           ;;                   |
    // 0d0020     recur 1 12    ;; (12 = 20 - 8) ----/
    // 0d0028     nop
    // 0d0030   end
    // 0d0032 end
    // ```

    block_nez,          // (param type_index:i32, alt_inst_offset:i32)

    // the 'block_nez' instruction is similar to the 'block', it also creates a new block region
    // as well as a block stack frame.
    // but it jumps to the 'alternative instruction' if the operand on the top of stack is
    // equals to ZERO.
    //
    // 0:i32 and 0:i64 are both treated as logic FALSE and
    // all other i32/i64 non-zero are treated as logic TRUE.
    // so the 'block_nez' instruction means only executes the instructions follows the 'block_nez'
    // when the logic is TRUE, otherwise, go to the 'alternative instruction'.
    //
    //
    // the 'block_nez' instruction is commonly used to construct the 'if' structures
    // in general programming languages.
    //
    // e.g. 1
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
    // 0d0100 end               ;; <----/ jump to here when FALSE
    // ```
    //
    // e.g. 2
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
    //                          ;; the TRUE path    the FALSE path
    //                          ;; |                |
    // 0d0000 block_nez 0 158   ;; V                V jump to 0d0158 when FALSE
    // 0d0008 ...               ;; |+               |-
    // ;; the 'then' part       ;; |+               |-
    // 0d0150 return 0 200      ;; \-->--\+         |-
    // 0d0158 ...               ;;       |-   /--<--/+
    // ;; the 'else' part       ;;       |-   |+
    // 0d0350 end               ;;       |-   |+
    // 0d0352 nop               ;; <-----/    |
    //                          ;;
    //                          ;; (+ => execute, - => pass)
    // ```

    // a complete 'loop' structure is actually combined with 'block', 'block_nez', 'recur', 'return'
    // and 'return_nez' instructions, e.g.
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
    // 0d0100   block_nez 0 28  ;; ----\         |
    // 0d0112     i32.imm 100   ;;     |         |
    // 0d0120     return 1 88   ;; ----|----\    |
    // 0d0128   end             ;; <---/    |    |
    //          ...             ;;          |    |
    // 0d0200   recur 0 192     ;; ---------|----/
    // 0d0208 end               ;;          |
    // 0d0210 ...               ;; <--------/
    // ```
    //
    // the 'block_nez' block above can be optimised as a 'return_nez' instruction, e.g.
    //
    // ```bytecode
    // 0d0000 block 0
    // 0d0008   ...             ;; <-------------\
    //          ...             ;;               |
    // 0d0100   i32.imm 100     ;;               |
    //          ...             ;;               |
    // 0d0120   return_nez 0 88 ;; ---------\    |
    // 0d0128   drop            ;;          |    |
    //          ...             ;;          |    |
    // 0d0200   recur 0 192     ;; ---------|----/
    // 0d0208 end               ;;          |
    // 0d0210 ...               ;; <--------/
    // ```

    // P.S.
    // there is a pesudo instruction 'break' in the text assembly, it is actually
    // translated to the 'return' instruction.

    recur_nez,          // (param skip_depth:i16, start_inst_offset:i32)

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
    // function
    //

    call = 0xc00,           // general function call            (param func_index:i32)
    dcall,                  // closure/dynamic function call    (operand func_index:i64)

    // the operand "func_index" is part of the "closure_function_item":
    //
    // closure_function_item {
    //     [ref_count],
    //     func_index:i32,
    //     captured_items:i64_ref
    // }
    //
    // "captured_items" is a singly linked list:
    //
    // closure_item_node: {
    //     [ref_count],
    //     previous_node_addr:i64_ref,
    //     data_type:i32,
    //     data_value:i32/i64/f32/f64,  // for primitive data
    //     data_value_ref:i64_ref       // for struct data
    // }
    //
    // an additional parameter is appended to the closure function automatically when it is compiled to assembly, e.g.
    //
    // `let a = fn (i32 a, i32 b) {...}`
    //
    // will be transformed into:
    //
    // `let a = fn (i32 a, i32 b, pointer captured_items) {...}`
    //
    // when a general function is passed to another function as a parameter, this function is also wrapped
    // as a closure function, except the value of 'captured_items' is 0.
    //
    // an example of "dcall" instruction:
    //
    // ```xiaoxuan
    // type F = signature (i32, i32) result boolean
    // function filter(List data, F predicate) {
    //    ...
    //    let pass = predicate(1, 2)
    //    ...
    // }
    // ```
    //
    // the equivalent assembly:
    //
    // ```clojure
    // (func (param $data i64) (param $predicate i64)
    //    ...
    //    (i32.imm 1)
    //    (i32.imm 2)
    //    (local.get $predicate)
    //    dcall
    //    ...
    // )
    // ```

    ecall,                  // environment call                 (param env_func_num:i32)

    scall,                  // syscall                          (param sys_call_num:i32)
                            // https://chromium.googlesource.com/chromiumos/docs/+/master/constants/syscalls.md

    ccall,                  // external C function call         (param c_func_index:i32)

    //
    // host memory address
    //

    host_addr_local = 0xd00,    // (param local_variable_index:i32)
    host_addr_data,             // (param data_index:i32)
    host_addr_heap,             // (param addr_low:i32, addr_high: i32)
    host_addr_function,         // (param func_index:i32)
                                // note:
                                // a host function will be created when `addr_function` is executed, as well as
                                // the specified VM function will be appended to the "function pointer table" to
                                // prevent duplicate creation.

    //
    // VM status
    //

    sp = 0x1200,            // get stack pointer
    fp,                     // get frame pointer (function stack frame only)
    pc,                     // get program counter (the position of instruction and the current module index)
                            //
                            // |            |
                            // | module idx |
                            // | inst addr  |
                            // \------------/
                            //
    tid,                    // get the current thread id

    //
    // atomic
    //

    i32_cas = 0xe00,        // compare and swap     (operand addr:i64, old_for_compare:i32, new_for_set:i32) result 0/1:i32
                            //
                            //  |                 |
                            //  | new_for_set     |
                            //  | old_for_compare |
                            //  | addr            |
                            //  \-----------------/

    i64_cas,                // compare and swap     (operand addr:i64, old_for_compare:i64, new_for_set:i64) result 0/1:i32

}
