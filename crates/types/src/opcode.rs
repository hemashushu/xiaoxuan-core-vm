// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// note:
//
// - the host data types:
//   int8, int16, int32, int64, float32, float64
// - the vm data types:
//   i8, i16, i32, i64, f32, f64

// note
// i8: data type, data section type, module type
// i16: module index, function type index, data index, local variable index, block level/index, section id
// i32: function index, dynamic function index, c function index, syscall number, env call number

// XiaoXuan VM instructions are not fixed-length code.
//
// - instructions without parameters are 16 bits width
// - instructions with one parameter, such as `local_get`, `i32_load`, `i32_shl` are 32 bits width
//   16 bits opcode + 16 bits parameter
// - instructions with one parameter, such as `call`, `ccall` and `addr_function` are 64 bits width
//   16 bits opcode + 16 bits padding + 32 bits parameter
// - instructions with two parameters, such as `block_nez`, `break`, `recur` are 64 bits width
//   16 bits opcode + 16 bits parameter 1 + 32 bits parameter 2
// - instructions `i32_imm`, i64_imm_high` and `i64_imm_low` are 64 bits width,
//   16 bits opcode + 16 bits padding + 32 bits immediate number
//
// so there are 16 bits, 32 bits and 64 bits instructions, sometimes it is
// necessary to insert the `nop` instruction to form 32/64 bits (4/8-byte) alignment.

#[repr(u16)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum Opcode {
    nop = 0x0,          // instruction to do nothing,
                        // it's usually used for padding instructions to archieve 32/64 bits (4/8-byte) alignment.
    drop,               // drop one operand (the top most operand on the operand stack)

    //
    // immediate number
    //

    i32_imm = 0x100,    // (param: immediate_number:int32)
    i64_imm_high,       // (param: immediate_number_high_32bits:int32)
    i64_imm_low,        // (param: immediate_number_low_32bits:int32)

    // there are also pesudo instructions in the assembly:
    // - i64_imm (param: immediate_number:int64)
    // - f32_imm (param: immediate_number:float32)
    // - f64_imm (param: immediate_number:float64)

    //
    // local variable
    //

    // in the default XiaoXuan VM implement,
    // each local variable takes up 8-bytes, so the "variable index" can also be used as the
    // data offset.

    local_get = 0x200,  // get the specified local variable and push to the stack    (param: local_variable_index:int16)
    local_set,          // pop from the stack and set the specified local variable   (param: local_variable_index:int16)
    local_tee,          // peek from the stack and set the specified local variable  (param: local_variable_index:int16)

    //
    // thread local variable
    //

    // `thread_local_get`, `thread_local_set` and `thread_local_tee` are
    // valid only when the type of specified data is i32/i64/f32/f64,
    // to access the `byte` type thread local data, instructions `addr_data`,
    // `i32_load8_u` and `i32_store8` should be used.

    thread_local_get = 0x300,   // get the specified global variable and push to stack        (param: data_index:int16)
    thread_local_set,           // pop from the stack and set the specified global variable   (param: data_index:int16)
    thread_local_tee,           // peek from the stack and set the specified global variable  (param: data_index:int16)

    //
    // operand stack data, thread local data and global shared memory loading and storing
    //

    // note:
    // integer i8 and i16 can be stored as i32 or i64

    i32_load = 0x400,   // (param: offset_bytes:int16)
    i32_load8_s,        // (param: offset_bytes:int16)
    i32_load8_u,        // (param: offset_bytes:int16)
    i32_load16_s,       // (param: offset_bytes:int16)
    i32_load16_u,       // (param: offset_bytes:int16)
                        //
    i64_load,           // (param: offset_bytes:int16)
    i64_load8_s,        // (param: offset_bytes:int16)
    i64_load8_u,        // (param: offset_bytes:int16)
    i64_load16_s,       // (param: offset_bytes:int16)
    i64_load16_u,       // (param: offset_bytes:int16)
    i64_load32_s,       // (param: offset_bytes:int16)
    i64_load32_u,       // (param: offset_bytes:int16)
                        //
    i32_store,          // (param: offset_bytes:int16)
    i32_store8,         // (param: offset_bytes:int16)
    i32_store16,        // (param: offset_bytes:int16)
                        //
    i64_store,          // (param: offset_bytes:int16)
    i64_store8,         // (param: offset_bytes:int16)
    i64_store16,        // (param: offset_bytes:int16)
    i64_store32,        // (param: offset_bytes:int16)
                        //
    f32_load,           // (param: offset_bytes:int16)
    f64_load,           // (param: offset_bytes:int16)
    f32_store,          // (param: offset_bytes:int16)
    f64_store,          // (param: offset_bytes:int16)

    byte_fill,          // `fn byte_fill(start_addr:i64, count:i64, value:i8)`
    byte_copy,          // `fn byte_copy(src_addr:i64, dst_addr:i64, length:i64)`

    //
    // comparsion
    //

    // for the binary operations, the first one popped from the
    // operand stack is the right-hand-side value, e.g.
    //
    // |                 | --> stack end
    // | right hand side | --> 1st pop: RHS
    // | left hand side  | --> 2nd pop: LHS
    // \-----------------/ --> stack start
    //
    // it is the same order as the function parameter, e.g.
    // function `add (a, b)`
    // the parameters in the operand stack is:
    //
    //  |   |
    //  | b |
    //  | a |
    //  \---/

    // the result of the comparison is a logical TRUE or FALSE, when
    // the result is TRUE, the number `1` is pushed into the operand stack,
    // and vice versa the number `0` is pushed into.

    // instruction `i32_lt_u` example:
    //
    // ```
    // ;; load 2 numbers on to the stack
    // (i32.imm 11)
    // (i32.imm 22)
    //
    // ;; now the operand stack layout is:
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
    // ;; now the operand stack layout is:
    // ;;
    // ;; |    |
    // ;; | 1  |
    // ;; \----/
    // ```

    i32_eqz = 0x500,
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

    i32_add = 0x600,
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

    i32_and = 0x700,    // bitwise AND
    i32_or,             // bitwise OR
    i32_xor,            // bitwise XOR
    i32_not,            // bitwise NOT
    i32_clz,            // count leading zeros
    i32_ctz,            // count trailing zeros
    i32_popcnt,         // count the total amount of value `1` bits
    i32_shl,            // shift left                   (param: move_bits:int16)
    i32_shr_s,          // arithmetic right shift       (param: move_bits:int16)
    i32_shr_u,          // logical right shift          (param: move_bits:int16)
    i32_rotl,           // left rotate                  (param: move_bits:int16)
    i32_rotr,           // right rotate                 (param: move_bits:int16)


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
    i64_shl,
    i64_shr_s,
    i64_shr_u,
    i64_rotl,
    i64_rotr,
    i64_clz,
    i64_ctz,
    i64_popcnt,

    //
    // math functions
    //

    f32_abs = 0x800,
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
    // ;; now the operand stack layout is:
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
    // ;; now the operand stack layout is:
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
    // conversion
    //

    // note::
    //
    // in the default XiaoXuan VM implement,
    // the data type of operand (on the operand stack) is 64-bits raw data
    // and do NOT check the type of the operand it present.
    //
    // so some instructions do the same thing, e.g.
    // `i64_extend_i32_s` and `i64_sign_extend32_s`,
    // `i32_trunc_f32_s` and `i32_trunc_f32_u`.
    //
    // and some instructions are simply ignored, e.g.
    // `i32_reinterpret_f32` and other reinterpret instructions.
    //
    // but all these instructions are preserved for consistency, and
    // enable some VM implement for data type checking.

    // convert (wrap) i64 to i32
    // discard the high 32 bits of a i64 number
    i32_wrap_i64 = 0x900,

    // convert (extend) i32 to i64
    i64_extend_i32_s,
    i64_extend_i32_u,

    // float to int
    // convert (truncate fractional part) floating points to integers
    i32_trunc_f32_s,
    i32_trunc_f32_u,
    i32_trunc_f64_s,
    i32_trunc_f64_u,
    i64_trunc_f32_s,
    i64_trunc_f32_u,
    i64_trunc_f64_s,
    i64_trunc_f64_u,

    // float to int without exception
    // the semantics are the same as the corresponding non `_sat` instructions, except:
    // - instead of trapping on positive or negative overflow,
    //   they return the maximum or minimum integer value,
    //   respectively, and do not trap.
    //   this behavior is also referred to as "saturating".
    // - instead of trapping on NaN, they return 0 and do not trap.
    i32_trunc_sat_f32_s,
    i32_trunc_sat_f32_u,
    i32_trunc_sat_f64_s,
    i32_trunc_sat_f64_u,
    i64_trunc_sat_f32_s,
    i64_trunc_sat_f32_u,
    i64_trunc_sat_f64_s,
    i64_trunc_sat_f64_u,

    // int to float
    // convert integers to floating points
    f32_convert_i32_s,
    f32_convert_i32_u,
    f32_convert_i64_s,
    f32_convert_i64_u,
    f64_convert_i32_s,
    f64_convert_i32_u,
    f64_convert_i64_s,
    f64_convert_i64_u,

    // demote
    // convert f64 to f32
    f32_demote_f64,

    // promote
    // convert f32 to f64
    f64_promote_f32,

    // reinterpret
    // reinterpret the bytes of integers as floating points and vice versa
    i32_reinterpret_f32,
    i64_reinterpret_f64,
    f32_reinterpret_i32,
    f64_reinterpret_i64,

    // sign extend i32 to i32
    i32_sign_extend8_s,
    i32_sign_extend16_s,

    // sign extend i64 to i64
    i64_sign_extend8_s,
    i64_sign_extend16_s,
    i64_sign_extend32_s,

    //
    // control flow
    //

    block = 0x1000,     // (param: func_type:int16)
    block_nez,          // (param: func_type:int16, end_addr_offset:int32)
                        //
                        // P.S.
                        // zero (i32 `0` and i64 `0`) is treated as logic FALSE and
                        // non-zero is treated as logic TRUE.

    end,

    break_,             // (param block_level:int16, end_addr_offset:int32)
                        //
                        // note:
                        // the 'break' instruction is used for jumping to the specified 'end' instruction.
                        //
                        // it can be used for the 'if' structure, e.g. (the following are the bytecode)
                        //
                        // ```bytecode
                        // block_nez 0 <the_break_inst_pos>
                        //   ...
                        //   break 0 <the_end_inst_pos>
                        //   ...
                        // end
                        // ```
                        //
                        // the 'level' param is used to break and carry out a value from a loop structure, e.g.
                        //
                        // ```rust
                        // let i = loop {
                        //   ...
                        //   if ... break 100;
                        //   ...
                        // }
                        // ```
                        //
                        // the equivalent bytecode are:
                        //
                        // ```bytecode
                        // block 0
                        //   ...
                        //   block_nez 0 the_1st_end_inst_pos
                        //     i32.imm 100
                        //     break 1 the_2nd_end_inst_pos
                        //   end
                        //   ...
                        // end
                        // ```
                        //
                        // some operands should be copied if the 'break' level is greater than 0 to
                        // meet the return type of the target block .
                        //
                        // the assembly pesudo 'return' instruction will be translated to the 'break' instruction.

    recur,              // (param block_level:int16, start_addr_offset:int32)


    //
    // function
    //

    call = 0x1100,          // call function                    (param func_index:int32)
    dcall,                  // closure/dynamic function call

    ecall,                  // environment call                 (param env_func_num:int32)

    scall,                  // syscall                          (param sys_call_num:int32)
                            // https://chromium.googlesource.com/chromiumos/docs/+/master/constants/syscalls.md

    ccall,                  // external C function call         (param c_func_index:int32)

    // the cfcall operand "cfunc_item_addr" is a vm struct:
    // closure_function_item: {[ref_count], func_idx:i32, closure_items:i64_ref}
    //
    // "closure_items" is a linked list:
    // closure_item: {[ref_count], previous_item_addr:i64_ref, data_type:i32, data_value:db, data_value_ref:i64_ref}
    //
    // the closure function will append a param automatically when it is compiled to assembly, e.g.
    //
    // `fn (i32 a, i32 b) {...}`
    // will be transformed into
    // `fn (i32 a, i32 b, pointer closure_item_link_list) {...}`
    //
    // when pass a general function as a parameter, the target function also must be wrapped
    // into closure function.
    // when invoke a "function pointer", the "cfcall" instruction should be adopted, e.g.
    //
    // ```xiaoxuan
    // function filter(List data,
    //                 signature (i32, i32) result boolean predicate) {
    //    ...
    //    let pass = predicate(1, 2)
    //    ...
    // }
    // ```
    //
    // the equivalent assembly
    //
    // ```clojure
    // (func (param $data i64) (param $predicate i64)
    //    ...
    //    (local.get $predicate)
    //    (i32.imm 2)
    //    (i32.imm 1)
    //    dcall
    //    ...
    // )
    // ```

    //
    // global shared memory
    //

    shared_memory_size = 0x1200,    // the result is the amount of the memory pages, each page is 64 KiB
    shared_memory_grow,             // `fn shared_memory_grow(pages:i64)`

    //
    // address
    //

    addr_local = 0x1300,    // (param local_variable_index:int16)
    addr_data,              // (param data_index:int16)
    addr_function,          // (param func_index:int32)
                            // note:
                            // a host function will be created when `addr_function` is executed, as well as
                            // the specified VM function will be appended to the "function pointer table" to
                            // prevent duplicate creation.

    addr_func_fp,           // get function frame pointer
    addr_block_fp,          // get block frame pointer
    addr_sp,                // get operand stack pointer

    //
    // atomic
    //

    i32_cas = 0x1400,       // compare and swap, `fn i32_cas(addr:i64, old_for_compare:i32, new_for_set:i32) -> 0|1`
                            //
                            //  |                 |
                            //  | new_for_set     |
                            //  | old_for_compare |
                            //  | addr            |
                            //  \-----------------/

    i64_cas,                // compare and swap, `fn i64_cas(addr:i64, old_for_compare:i64, new_for_set:i64) -> 0|1`

}
