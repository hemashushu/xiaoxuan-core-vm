// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// note:
//
// the data types that VM supports:
//
// - i64
// - f64
// - i32
// - f32
//
// note that 'i32' above means a 32-bit integer, which is equivalent to
// the 'uint32_t' in C or 'u32' in Rust. do not confuse it with 'i32' in Rust.
// the same applies to the i8, i16 and i64.
//
// data layout:
//
// the default implement of XiaoXuan VM is stack-base, which its operands are
// 8-byte raw data, the data presentation is as the following:
//
//    MSB                             LSB
// 64 |---------------------------------| 0
//    |   16     16      16     8    8  | bits
//    |-------|-------|-------|----|----|
//    |               i64               | <-- native data type
//    |---------------------------------|
//    |???????????????|        i32      |
//    |---------------------------------|
//    |???????????????|-sign--|   i16   |
//    |---------------------------------|
//    |???????????????|-sign-extend| i8 |
//    |---------------------------------|
//    |               f64               | <-- native data type
//    |---------------------------------|
//    |???????????????|        f32      |
//    |---------------------------------|
//
// the value of the high end part of operand is undefined.

// note:
//
// the floating-point number:
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
// when load data from memory as floating-point number, there are some checking as the following:
// 1. exponent between (00000001) and (11111110): pass
// 2. exponent is zero, if the sign bit is zero: pass
// 3. failed.
//
// in other words, the +/-Infinity, -0, NaN, will cause the VM to throw exceptions.

// note:
//
// the boolean type:
//
// the boolean value is represented by a i32 number:
// - TRUE, the number is `1:i32`,
// - FALSE, the number is `0:i32`.
//
// on the other hand, 0:i32 and 0:i64 are both treated as FALSE,
// and all other i32/i64 non-zero are treated as TRUE.

// note:
//
// the instruction schemes:
//
// XiaoXuan VM instructions are not fixed-length code. there are
// 16 bits, 32 bits, 64 bits and 96 bits instructions, it is
// necessary to insert the `nop` instruction between the 16 (32) bits instruction and the 64 (96) bits
// instruction to achieve 32 bits (4-byte) alignment.
//
// - 16 bits:
//   instructions without parameters, such as `i32_eq`, `i32_add`.
// - 32 bits:
//   instructions with 1 parameter, such as `heap_load`, `heap_store`.
//   16 bits opcode + 16 bits parameter
// - 64 bits:
//   instructions with 1 parameter, such as `i32_imm`, `f32_imm`, `block`, `call`.
//   16 bits opcode + 16 bits padding + 32 bits parameter (4-byte alignment require)
// - 64 bits:
//   instructions with 2 parameters, such as `local_load`, `data_load`, `break`, `recur`.
//   16 bits opcode + 16 bits parameter 0 + 32 bits parameter 1 (4-byte alignment require)
// - 96 bits
//   instructions with 2 parameters, such as `i64_imm`, `f64_imm`, `block_nez`,
//   16 bits opcode + 16 bits padding + 32 bits parameter 0 + 32 bits parameter 1 (4-byte alignment require)
//
// the simplified schemes:
//
// - [opcode i16]
// - [opcode i16] - [param i16      ]
// - [opcode i16] - [param i16      ] + [param i32]
// - [opcode i16] - [padding 16 bits] + [param i32]
// - [opcode i16] - [padding 16 bits] + [param i32] + [param i32]
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
    //
    // fundamental
    //
    zero = 0x100,       // push 0 (i64) onto stack
    drop,               // drop one operand (the top most operand)
    duplicate,          // duplicate one operand (the top most operand)
    swap,               // swap the top two operands
    i32_imm,            // (param immediate_number:i32)
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
    // local variables loading and storing
    //
    // load the specified local variable and push onto to the stack, or
    // pop one operand off the stack and set the specified local variable.
    //
    // note that you CAN ALSO load/store function arguments using these
    // instructions. the index of arguments are follow the local variables, e.g.
    //
    //     local variable      arguments
    //     [i32 i32 i64 i64]  [i32 i32]
    // idx  0   1   2   3      4   5
    //
    //
    // note about the local variable (data, function) INDEX:
    //
    // using the 'index', rather than the 'address/pointer' to access local variables (including
    // data in the data section and functions talked about in the following sections) is the
    // security strategy of the XiaoXuan ISA and VM.
    // because the 'index' includes the type, data length (range), location information of the 'object',
    // when accessing the object, the VM can check whether the type of the object, and the range is legal
    // or not, so it can prevent a lot of errors.
    // for example, the traditional method of using pointers to access a array is very easy
    // to read/write data outside the range.

    // note:
    // in the default VM implementation, the arguments of a function are placed on the top
    // of the stack, so it is also possible to read the arguments directly in the function
    // using instructions with the POP function (e.g. the comparison instructions, the arithmetic
    // instructions).
    // this feature can be used as a trick to improve performance, but the XiaoXuan ISA doesn't
    // guarantee that this feature will always be available, so for general programs, use the
    // stable method of accessing the arguments, i.e. the index.

    local_load = 0x200,         // load local variable              (param offset_bytes:i16 local_variable_index:i32)
    local_load32,               //                                  (param offset_bytes:i16 local_variable_index:i32)
    local_load32_i16_s,         //                                  (param offset_bytes:i16 local_variable_index:i32)
    local_load32_i16_u,         //                                  (param offset_bytes:i16 local_variable_index:i32)
    local_load32_i8_s,          //                                  (param offset_bytes:i16 local_variable_index:i32)
    local_load32_i8_u,          //                                  (param offset_bytes:i16 local_variable_index:i32)
    local_load_f64,             // Load f64 with floating-point validity check.     (param offset_bytes:i16 local_variable_index:i32)
    local_load32_f32,           // Load f32 with floating-point validity check.     (param offset_bytes:i16 local_variable_index:i32)
    local_store,                // store local variable             (param offset_bytes:i16 local_variable_index:i32)
    local_store32,              //                                  (param offset_bytes:i16 local_variable_index:i32)
    local_store16,              //                                  (param offset_bytes:i16 local_variable_index:i32)
    local_store8,               //                                  (param offset_bytes:i16 local_variable_index:i32)

    local_long_load = 0x280,    //                                  (param local_variable_index:i32) (operand offset_bytes:i32)
    local_long_load32,          //                                  (param local_variable_index:i32) (operand offset_bytes:i32)
    local_long_load32_i16_s,    //                                  (param local_variable_index:i32) (operand offset_bytes:i32)
    local_long_load32_i16_u,    //                                  (param local_variable_index:i32) (operand offset_bytes:i32)
    local_long_load32_i8_s,     //                                  (param local_variable_index:i32) (operand offset_bytes:i32)
    local_long_load32_i8_u,     //                                  (param local_variable_index:i32) (operand offset_bytes:i32)
    local_long_load_f64,        //                                  (param local_variable_index:i32) (operand offset_bytes:i32)
    local_long_load32_f32,      //                                  (param local_variable_index:i32) (operand offset_bytes:i32)
    local_long_store,           //                                  (param local_variable_index:i32) (operand offset_bytes:i32)
    local_long_store32,         //                                  (param local_variable_index:i32) (operand offset_bytes:i32)
    local_long_store16,         //                                  (param local_variable_index:i32) (operand offset_bytes:i32)
    local_long_store8,          //                                  (param local_variable_index:i32) (operand offset_bytes:i32)

    // note:
    // - there are 2 sets of local load/store instructions, one set is the
    //   local_load.../local_store.., they are designed to access primitive type data
    //   and struct data, the other set is the local_long_load.../local_long_store..., they
    //   are designed to access long byte-type data.
    // - the local_(long_)load32* instructions will leave the value of the high part of
    //   operand (on the stack) undefined/unpredictable.

    //
    // data (thread-local variables) loading and storing
    //
    // load the specified data and push onto to the stack, or
    // pop one operand off the stack and set the specified data
    //

    data_load = 0x300,          // load data                        (param offset_bytes:i16 data_public_index:i32)
    data_load32,                //                                  (param offset_bytes:i16 data_public_index:i32)
    data_load32_i16_s,          //                                  (param offset_bytes:i16 data_public_index:i32)
    data_load32_i16_u,          //                                  (param offset_bytes:i16 data_public_index:i32)
    data_load32_i8_s,           //                                  (param offset_bytes:i16 data_public_index:i32)
    data_load32_i8_u,           //                                  (param offset_bytes:i16 data_public_index:i32)
    data_load_f64,              // Load f64 with floating-point validity check.     (param offset_bytes:i16 data_public_index:i32)
    data_load32_f32,            // Load f32 with floating-point validity check.     (param offset_bytes:i16 data_public_index:i32)
    data_store,                 // store data                       (param offset_bytes:i16 data_public_index:i32)
    data_store32,               //                                  (param offset_bytes:i16 data_public_index:i32)
    data_store16,               //                                  (param offset_bytes:i16 data_public_index:i32)
    data_store8,                //                                  (param offset_bytes:i16 data_public_index:i32)

    // there are also 2 sets of data load/store instructions, one set is the
    // data_load.../data_store.., they are designed to access primitive type data
    // and struct data, the other set is the data_long_load.../data_long_store..., they
    // are designed to access long byte-type data.

    data_long_load = 0x380,     //                                  (param data_public_index:i32) (operand offset_bytes:i32)
    data_long_load32,           //                                  (param data_public_index:i32) (operand offset_bytes:i32)
    data_long_load32_i16_s,     //                                  (param data_public_index:i32) (operand offset_bytes:i32)
    data_long_load32_i16_u,     //                                  (param data_public_index:i32) (operand offset_bytes:i32)
    data_long_load32_i8_s,      //                                  (param data_public_index:i32) (operand offset_bytes:i32)
    data_long_load32_i8_u,      //                                  (param data_public_index:i32) (operand offset_bytes:i32)
    data_long_load_f64,         //                                  (param data_public_index:i32) (operand offset_bytes:i32)
    data_long_load32_f32,       //                                  (param data_public_index:i32) (operand offset_bytes:i32)
    data_long_store,            //                                  (param data_public_index:i32) (operand offset_bytes:i32)
    data_long_store32,          //                                  (param data_public_index:i32) (operand offset_bytes:i32)
    data_long_store16,          //                                  (param data_public_index:i32) (operand offset_bytes:i32)
    data_long_store8,           //                                  (param data_public_index:i32) (operand offset_bytes:i32)

    //
    // heap (thread-local memory) loading and storing
    //

    // note that the address of heap is a 64-bit integer number, which means that you
    // must write the target address (to stack) using the
    // 'i64_imm', 'local_(long_)load' or 'data_(long_)load' instructions.
    // do NOT use the 'i32_imm', 'local_(long_)load32' or 'data_(long_)load32', because
    // the latter instructions leave the value of the high part of
    // operand (on the stack) undefined/unpredictable.

    heap_load = 0x400,      // load heap                        (param offset_bytes:i16) (operand heap_addr:i64)
    heap_load32,            //                                  (param offset_bytes:i16) (operand heap_addr:i64)
    heap_load32_i16_s,      //                                  (param offset_bytes:i16) (operand heap_addr:i64)
    heap_load32_i16_u,      //                                  (param offset_bytes:i16) (operand heap_addr:i64)
    heap_load32_i8_s,       //                                  (param offset_bytes:i16) (operand heap_addr:i64)
    heap_load32_i8_u,       //                                  (param offset_bytes:i16) (operand heap_addr:i64)
    heap_load_f64,          // Load f64 with floating-point validity check.     (param offset_bytes:i16) (operand heap_addr:i64)
    heap_load32_f32,        // Load f32 with floating-point validity check.     (param offset_bytes:i16) (operand heap_addr:i64)
    heap_store,             // store heap                       (param offset_bytes:i16) (operand heap_addr:i64)
    heap_store32,           //                                  (param offset_bytes:i16) (operand heap_addr:i64)
    heap_store16,           //                                  (param offset_bytes:i16) (operand heap_addr:i64)
    heap_store8,            //                                  (param offset_bytes:i16) (operand heap_addr:i64)

    //
    // shared-memory loading and storing
    // (UNDECIDED)
    //
    // share_load = 0x600,     // load
    // ...
    // share_store,
    // ...

    //
    // conversion
    //

    // demote i64 to i32
    // discard the high 32 bits of an i64 number directly
    i32_demote_i64 = 0x500,

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
    i32_trunc_f32_u,        // note -x.xx(float) -> 0(int)
    i32_trunc_f64_s,
    i32_trunc_f64_u,        // note -x.xx(float) -> 0(int)
    i64_trunc_f32_s,
    i64_trunc_f32_u,        // note -x.xx(float) -> 0(int)
    i64_trunc_f64_s,
    i64_trunc_f64_u,        // note -x.xx(float) -> 0(int)

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

    // the result of the comparison is a logical TRUE or FALSE (i.e., the data type 'boolean'),
    // when the result is TRUE, the number `1:i32` is pushed onto the stack,
    // and vice versa the number is `0:i32`.

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

    i32_eqz = 0x600,
    i32_nez,
    i32_eq,
    i32_ne,
    i32_lt_s,
    i32_lt_u,
    i32_gt_s,
    i32_gt_u,
    i32_le_s,   // redundant
    i32_le_u,   // redundant
    i32_ge_s,   // redundant
    i32_ge_u,   // redundant

    i64_eqz,
    i64_nez,
    i64_eq,
    i64_ne,
    i64_lt_s,
    i64_lt_u,
    i64_gt_s,
    i64_gt_u,
    i64_le_s,   // redundant
    i64_le_u,   // redundant
    i64_ge_s,   // redundant
    i64_ge_u,   // redundant

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

    i32_add = 0x700,
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

    i32_and = 0x800,    // bitwise AND
    i32_or,             // bitwise OR
    i32_xor,            // bitwise XOR
    i32_not,            // bitwise NOT
    i32_leading_zeros,  // count leading zeros          (number:i64) -> i32
    i32_trailing_zeros, // count trailing zeros         (number:i64) -> i32
    i32_count_ones,     // count the number of ones in the binary representation     (number:i64) -> i32
    i32_shift_left,     // left shift                   (operand number:i32 move_bits:i32) -> i32,  move_bits = [0,32)
    i32_shift_right_s,  // arithmetic right shift       (operand number:i32 move_bits:i32) -> i32,  move_bits = [0,32)
    i32_shift_right_u,  // logical right shift          (operand number:i32 move_bits:i32) -> i32,  move_bits = [0,32)
    i32_rotate_left,    // left rotate                  (operand number:i32 move_bits:i32) -> i32,  move_bits = [0,32)
    i32_rotate_right,   // right rotate                 (operand number:i32 move_bits:i32) -> i32,  move_bits = [0,32)


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
    i64_leading_zeros,  // (number:i64) -> i32
    i64_trailing_zeros, // (number:i64) -> i32
    i64_count_ones,     // (number:i64) -> i32
    i64_shift_left,     // left shift                   (operand number:i64 move_bits:i32) -> i64,  move_bits = [0,64)
    i64_shift_right_s,  // arithmetic right shift       (operand number:i64 move_bits:i32) -> i64,  move_bits = [0,64)
    i64_shift_right_u,  // logical right shift          (operand number:i64 move_bits:i32) -> i64,  move_bits = [0,64)
    i64_rotate_left,    // left rotate                  (operand number:i64 move_bits:i32) -> i64,  move_bits = [0,64)
    i64_rotate_right,   // right rotate                 (operand number:i64 move_bits:i32) -> i64,  move_bits = [0,64)


    //
    // math
    //

    f32_abs = 0x900,
    f32_neg,
    f32_ceil,
    f32_floor,
    f32_round_half_away_from_zero,
    // f32_round_half_to_even,
    f32_trunc,          // the integer part of x
    f32_fract,          // the fractional part of  x
    f32_sqrt,           // sqrt(x)
    f32_cbrt,           // cbrt(x), the cube root of x
    f32_pow,            // left^right
    f32_exp,            // e^x
    f32_exp2,           // 2^x
    f32_ln,             // log_e(x)
    f32_log,            // log_right(left)
    f32_log2,           // log_2(x)
    f32_log10,          // log_10(x)
    f32_sin,
    f32_cos,
    f32_tan,
    f32_asin,
    f32_acos,
    f32_atan,

    // examples of 'round_half_away_from_zero':
    // round(2.4) = 2.0
    // round(2.6) = 3.0
    // round(2.5) = 3.0
    // round(-2.5) = -3.0
    //
    // ref:
    // https://en.wikipedia.org/wiki/Rounding#Rounding_half_away_from_zero

    f64_abs,
    f64_neg,
    f64_ceil,
    f64_floor,
    f64_round_half_away_from_zero,
    // f64_round_half_to_even,
    f64_trunc,
    f64_fract,
    f64_sqrt,
    f64_cbrt,
    f64_pow,
    f64_exp,
    f64_exp2,
    f64_ln,
    f64_log,
    f64_log2,
    f64_log10,
    f64_sin,
    f64_cos,
    f64_tan,
    f64_asin,
    f64_acos,
    f64_atan,

    //
    // control flow
    //

    end = 0xa00,        // finish a block or a function.
    // when the 'end' instruction is executed, a stack frame will be removed and
    // the results of the current block or function will be placed on the top of stack.

    block,              // (param type_index:i32)
                        //
                        // create a block scope. a block is similar to a function, it also has
                        // parameters and results, it shares the type with function, so the 'block'
                        // instruction has a parameter called 'type_index'.
                        // this instruction leads VM to create a stack frame which is called 'block frame',
                        // block frame is similar to 'function frame' except it has no local variables.

    return_,            // (param skip_depth:i16, next_inst_offset:i32)

    // the 'return' instruction is similar to the 'end' instruction, it is
    // used for finishing a block or a function.
    // - for a block:
    //   a block stack frame will be removed and jump to the instruction
    //   that NEXT TO THE 'end' instruction.
    //   the value of the parameter 'next_inst_offset' should be (`addr of next inst to 'end'` - `addr of return`)
    // - for a function:
    //   a function stack frame will be removed and return to the the
    //   instruction next to the 'call' instruction.
    //   the value of the parameter 'next_inst_offset' is ignored.
    //
    // e.g.
    //
    // ```bytecode
    // 0d0000 block 0         ;; the size of 'block' instruction is 8 bytes
    // 0d0008   nop
    // 0d0010   return 0 14   ;; the size of 'return' instruction is 8 bytes, (14 = 24 - 10) --\
    // 0d0018   nop           ;;                                                               |
    // 0d0020   nop           ;;                                                               |
    // 0d0022 end             ;;                                                               |
    // 0d0024 nop             ;; <-- jump to here (the instruction that next to the 'end')-----/
    // ```
    //
    // instruction 'return' not only just finish a block or a function, but also
    // brings the operands out of the block or function, e.g.
    //
    // 0d0000 block 0         ;; assumes the block type is '()->(i32,i32)'
    // 0d0008   i32.imm 11
    // 0d0016   i32.imm 13                       | 17                 | -----\ operands '17' and '13' were
    // 0d0024   i32.imm 17    ;; --------------> | 13                 | -----\ taken out of the block frame
    // 0d0032   return 0 14   ;; ---\            | 11                 |      |
    // 0d0040   nop           ;;    |            | [block frame info] |      v
    // 0d0042   nop           ;;    | jump       | ..                 |    | 17                 |
    // 0d0044   nop           ;;    |            | [func frame info]  |    | 13                 |
    // 0d0046   nop           ;;    |            \____________________/    | ..                 |
    // 0d0048 end             ;;    |               the stack layout       | [func frame info]  |
    // 0d0050 nop             ;; <--/ -----------------------------------> \____________________/
    //                                                                        the stack layout
    //
    // the 'return' instruction can cross over multiple block nested.
    // when the parameter 'skip_depth' is 0, it simply finish the current block.
    // when the value is greater than 0, multiple block stack frame will be removed,
    // as well as the operands will be taken out of the block.
    // (the amount of the operands is determined by the 'target block type'.
    //
    // ```bytecode
    // 0d0000 block 0
    // 0d0008   block 0
    // 0d0016     block 0
    // 0d0024       nop
    // 0d0026       return 1 14   ;; (18 = 44 - 26) --------\
    // 0d0034       nop           ;;                        |
    // 0d0036     end             ;;                        |
    // 0d0038     nop             ;;                        |
    // 0d0040   end               ;;                        |
    // 0d0042   nop               ;; <----------------------/ jump to here
    // 0d0044 end
    // ```

    recur,              // (param skip_depth:i16, start_inst_offset:i32)

    // the 'recur' instruction lets VM to jump to the instruction next to the instruction 'block', 'block_nez'
    // or the first instruction of the current function,
    // as well as all the operands in the current stack frame will be removed except
    // the operands for the 'target block/function params' are reserved and placed on the top of stack.
    // it is commonly used to construct the 'while/for' structures in general programming languages,
    // it is also used to implement the TCO (tail call optimization, see below section).
    //
    // when the target frame is the function frame itself, the param 'start_inst_offset' is ignore and
    // all local variables will be reset to 0.
    //
    // note that the value of 'start_inst_offset' is a positive number.
    //
    // 0d0000 block 0         ;; assumes the block type is '()->(i32,i32)'
    // 0d0008   i32.imm 11    ;; <------\
    // 0d0016   i32.imm 13    ;;        |         | 17                 | -----\ operands '17' and '13' were
    // 0d0024   i32.imm 17    ;; ---------------> | 13                 | -----\ taken out of the block frame
    // 0d0032   nop           ;;        |         | 11                 |      |
    // 0d0034   nop           ;;        |         | [block frame info] |      v
    // 0d0036   nop           ;;        |         | ..                 |    | 17                 |
    // 0d0038   nop           ;;  jump  |         | [func frame info]  |    | 13                 |
    // 0d0040   recur 0 14    ;; -------/         \____________________/    | [block frame info] |
    // 0d0048 end             ;;        |            the stack layout       | ..                 |
    // 0d0050 nop             ;;        \---------------------------------> | [func frame info]  |
    //                                                                      \____________________/
    //                                                                           the stack layout

    // the 'recur' instruction can cross over multiple block nested also.
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

    // the 'block_nez' instruction is similar to the 'block', it also creates a new block scope
    // as well as a block stack frame.
    // but it jumps to the 'alternative instruction' if the operand on the top of stack is
    // equals to ZERO.
    //
    // note:
    // 0:i32 and 0:i64 are both treated as logic FALSE and
    // all other i32/i64 non-zero are treated as logic TRUE.
    //
    // so the 'block_nez' instruction means only executes the instructions that follow the instruction 'block_nez'
    // when the logic is TRUE, otherwise, jump to the 'alternative instruction'.
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
    //                          ;; the TRUE path    |                          ;;       the FALSE path
    //                          ;; |                |                          ;;       |
    // 0d0000 block_nez 0 158   ;; V                | 0d0000 block_nez 0 158   ;;       V jump to 0d0158 when FALSE
    // 0d0008 ...               ;; |+               | 0d0008 ...               ;;       |-
    // ;; the 'then' part       ;; |+               | ;; the 'then' part       ;;       |-
    // 0d0150 return 0 200      ;; \-->--\+         | 0d0150 return 0 200      ;;       |-
    // 0d0158 ...               ;;       |-         | 0d0158 ...               ;; /--<--/+
    // ;; the 'else' part       ;;       |-         | ;; the 'else' part       ;; |+
    // 0d0350 end               ;;       |-         | 0d0350 end               ;; |+
    // 0d0352 nop               ;; <-----/          | 0d0352 nop               ;; |
    // ```
    //
    // (+ => execute, - => pass)

    return_nez,         // (param skip_depth:i16, next_inst_offset:i32)

    // a complete 'for' structure is actually combined with instructions 'block', 'block_nez', 'recur', 'return'
    // and 'return_nez', e.g.
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
    // the code above can be optimized by instruction 'return_nez', e.g.
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

    recur_nez,          // (param skip_depth:i16, start_inst_offset:i32)

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

    // table of control flow structures and control flow instructions:
    //
    // | structure         | instruction(s)  |
    // |-------------------|-----------------|
    // |                   | ..a..           |
    // | if ..a.. {        | block_nez       |
    // |    ..b..          | ..b..           |
    // | }                 | end             |
    // |-------------------|-----------------|
    // |                   | ..a..           |
    // | if ..a.. {        | block_nez -\    |
    // |    ..b..          | ..b..      |    |
    // | } else {          | return     |    |
    // |    ..c..          | ..c..  <---/    |
    // | }                 | end             |
    // |-------------------|-----------------|
    // |                   | ..a..           |
    // | if ..a.. {        | block_nez -\    |
    // |    ..b..          | ..b..      |    |
    // | } else if ..c.. { | return     |    |
    // |    ..d..          | ..c..  <---/    |
    // | } else {          | block_nez -\    |
    // |    ..e..          | ..d..      |    |
    // | }                 | return     |    |
    // |                   | ..e..  <---/    |
    // | (or switch/case)  | end             |
    // |-------------------|-----------------|

    // | structure         | instructions(s) |
    // |-------------------|-----------------|
    // | loop {            | block <--\      |
    // |    ..a..          | ..a..    |      |
    // | }                 | recur ---/      |
    // |                   | end             |
    // |-------------------|-----------------|
    // | while ..a.. {     | block           |
    // |    ..b..          | ..a..   <--\    |
    // | }                 | return_nez |    |
    // |                   | ..b..       |   |
    // | (or for...)       | recur -----/    |
    // |                   | end             |
    // |-------------------|-----------------|
    // | do {              | block           |
    // |    ..a..          | ..a..    <--\   |
    // | }while(..b..)     | ..b..       |   |
    // |                   | recur_nez --/   |
    // | (or TCO)          | end             |
    // |-------------------|-----------------|

    //
    // function
    //

    call,                   // general function call            (param func_pub_index:i32)
    dcall,                  // closure/dynamic function call    (param VOID) (operand func_pub_index:i64)

    // note:
    // 'function public index' includes the imported functions, it equals to
    // 'amount of imported functions' + 'function internal index'

    // call a function which is specified at runtime.
    //
    // when a general or anonymous function is passed to another function as a parameter,
    // it passs a pointer of a struct 'closure_function_item' actually:
    //
    // closure_function_item {
    //     [ref_count],
    //     target_mopdule_index:i32,
    //     func_internal_index:i32,
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
    // an additional parameter is appended to the ananymous function automatically when it is compiled to assembly, e.g.
    //
    // `let a = fn (i32 a, i32 b) {...}`
    //
    // will be transformed into:
    //
    // `let a = fn (i32 a, i32 b, pointer captured_items) {...}`
    //
    // ```text
    //                              |--> func_internal_index --> fn (a, b, captured_items) {...}
    //                         /----|--> module_index
    //                         |    |--> captured_items
    //                         |
    // let a = filter(list, predicate)
    // ```
    //
    // for a general function, the compiler generates an anonymous function for wrapping the
    // general function, e.g.
    //
    // ```text
    //                              |--> func_internal_index --> | fn (a, b, captured_items)
    //                         /----|--> module_index            | {
    //                         |    |--> 0                       |     the_general_function(a, b)
    //                         |                                 | }
    // let a = filter(list, predicate)
    // ```
    //
    // an example of "dcall" instruction:
    //
    // ```xiaoxuan
    // type F = signature (i32, i32) result boolean
    // function filter(List data, F predicate) {
    //    ...
    //    let a = predicate(...)
    //    ...
    // }
    // ```
    //
    // the equivalent assembly:
    //
    // ```clojure
    // (func (param $data i64) (param $predicate i64)
    //    ...
    //    (local.get $predicate 0)  ;; get the target function index
    //    dcall
    //    ...
    // )
    // ```

    ecall,                  // environment call                 (param env_func_num:i32)

    scall,                  // syscall                          (param sys_call_num:i32)
                            // https://chromium.googlesource.com/chromiumos/docs/+/master/constants/syscalls.md
                            //
    ccall,                  // external C function call         (param c_func_index:i32)
                            //
                            // note that both 'scall' and 'ccall' are optional instructions, they may be
                            // unavailable in some environment.
                            // the supported feature list can be obtained through the 'ecall' instruction with code 'features'.

    //
    // machine
    //

    nop = 0xb00,        // instruction to do nothing,
                        // it's usually used for padding instructions to archieve 32/64 bits (4/8-byte) alignment.
    break_,             // for VM debug

    // MAYBE USELESS
    //
    // sp = 0x0e00,            // get stack pointer
    // fp,                     // get function stack frame pointer
    // pc,                     // get program counter (the position of instruction and the current module index)
                               //
                               // |            |
                               // | module idx |
                               // | inst addr  |
                               // \------------/
                               //

    // get the host address of memory
    //
    // it is currently assumed that the target architecture is 64-bit.

    host_addr_local,            // (param offset_bytes:i16 local_variable_index:i32)
                                // note that the host address only valid in the current function and
                                // in its sub-functions. when a function exited, the function stack frame
                                // will be destroied (or modified), as well as the local variables.
    host_addr_local_long,       // (param local_variable_index:i32) (operand offset_bytes:i32)
    host_addr_data,             // (param offset_bytes:i16 data_public_index:i32)
    host_addr_data_long,        // (param data_public_index:i32) (operand offset_bytes:i32)
    host_addr_heap,             // (param offset_bytes:i16) (operand heap_addr:i64)


    //
    // atomic
    //
    // only available in shared-memory
    // (UNDECIDED)
    //
    //
    // i32_cas = 0xf00,        // compare and swap     (operand addr:i64, old_for_compare:i32, new_for_set:i32) result 0/1:i32
                             //
                             //  |                 |
                             //  | new_for_set     |
                             //  | old_for_compare |
                             //  | addr            |
                             //  \-----------------/
                             //
    // i64_cas,                // compare and swap     (operand addr:i64, old_for_compare:i64, new_for_set:i64) result 0/1:i32


    //
    // SIMD
    //
    // ref:
    // - https://github.com/rust-lang/portable-simd
    // - https://doc.rust-lang.org/std/simd/index.html
    // - https://github.com/rust-lang/packed_simd
    // - https://github.com/rust-lang/rfcs/blob/master/text/2325-stable-simd.md
}

pub const MAX_OPCODE_NUMBER:usize = 0x1000;
