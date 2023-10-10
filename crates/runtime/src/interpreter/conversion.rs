// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_program::thread_context::ThreadContext;

use super::InterpretResult;

// demote i64 to i32
// discard the high 32 bits of an i64 number directly
pub fn i32_trunc_i64(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_i64_u();
    thread_context.stack.push_i32_u(value as u32);
    InterpretResult::Move(2)
}

// promote i32 to i64
pub fn i64_extend_i32_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_i32_s();
    thread_context.stack.push_i64_s(value as i64);
    InterpretResult::Move(2)
}

pub fn i64_extend_i32_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_i32_u();
    thread_context.stack.push_i64_u(value as u64);
    InterpretResult::Move(2)
}

// demote f64 to f32
pub fn f32_demote_f64(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_f64();
    thread_context.stack.push_f32(value as f32);
    InterpretResult::Move(2)
}

// promote f32 to f64
pub fn f64_promote_f32(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_f32();
    thread_context.stack.push_f64(value as f64);
    InterpretResult::Move(2)
}

// convert float to int
// truncate fractional part
pub fn i32_convert_f32_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_f32();
    thread_context.stack.push_i32_s(value as i32);
    InterpretResult::Move(2)
}

pub fn i32_convert_f32_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_f32();
    thread_context.stack.push_i32_u(value as u32);
    InterpretResult::Move(2)
}

pub fn i32_convert_f64_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_f64();
    thread_context.stack.push_i32_s(value as i32);
    InterpretResult::Move(2)
}

pub fn i32_convert_f64_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_f64();
    thread_context.stack.push_i32_u(value as u32);
    InterpretResult::Move(2)
}

pub fn i64_convert_f32_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_f32();
    thread_context.stack.push_i64_s(value as i64);
    InterpretResult::Move(2)
}

pub fn i64_convert_f32_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_f32();
    thread_context.stack.push_i64_u(value as u64);
    InterpretResult::Move(2)
}

pub fn i64_convert_f64_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_f64();
    thread_context.stack.push_i64_s(value as i64);
    InterpretResult::Move(2)
}

pub fn i64_convert_f64_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_f64();
    thread_context.stack.push_i64_u(value as u64);
    InterpretResult::Move(2)
}

// convert int to float
pub fn f32_convert_i32_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_i32_s();
    thread_context.stack.push_f32(value as f32);
    InterpretResult::Move(2)
}

pub fn f32_convert_i32_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_i32_u();
    thread_context.stack.push_f32(value as f32);
    InterpretResult::Move(2)
}

pub fn f32_convert_i64_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_i64_s();
    thread_context.stack.push_f32(value as f32);
    InterpretResult::Move(2)
}

pub fn f32_convert_i64_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_i64_u();
    thread_context.stack.push_f32(value as f32);
    InterpretResult::Move(2)
}

pub fn f64_convert_i32_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_i32_s();
    thread_context.stack.push_f64(value as f64);
    InterpretResult::Move(2)
}

pub fn f64_convert_i32_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_i32_u();
    thread_context.stack.push_f64(value as f64);
    InterpretResult::Move(2)
}

pub fn f64_convert_i64_s(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_i64_s();
    thread_context.stack.push_f64(value as f64);
    InterpretResult::Move(2)
}

pub fn f64_convert_i64_u(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.stack.pop_i64_u();
    thread_context.stack.push_f64(value as f64);
    InterpretResult::Move(2)
}

#[cfg(test)]
mod tests {
    use crate::{in_memory_program_source::InMemoryProgramSource, interpreter::process_function};
    use ancvm_binary::utils::{build_module_binary_with_single_function, BytecodeWriter};
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    #[test]
    fn test_process_conversion_extend_and_trunc() {
        // init_runtime();

        // bytecodes
        //
        // 0x0000 local_load32         0 1
        // 0x0008 i64_extend_i32_s
        // 0x000a nop
        // 0x000c local_load32         0 1
        // 0x0014 i64_extend_i32_u
        // 0x0016 nop
        // 0x0018 local_load           0 0
        // 0x0020 i32_trunc_i64
        // 0x0022 end
        //
        // (i64, i32)  ->  (i64, i64, i32)
        //  |    |          ^    ^    ^
        //  |    | promote  |0   |1   |2
        //  |    \----------/----/    |
        //  \-------------------------/ demote

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i64_extend_i32_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 1)
            .write_opcode(Opcode::i64_extend_i32_u)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 0)
            .write_opcode(Opcode::i32_trunc_i64)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I64, DataType::I32],                // params
            vec![DataType::I64, DataType::I64, DataType::I32], // results
            vec![],                                            // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.new_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &vec![
                ForeignValue::UInt64(0x19171311_07050302u64),
                ForeignValue::UInt32(0x80706050u32),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt64(0xffffffff_80706050u64),
                ForeignValue::UInt64(0x00000000_80706050u64),
                ForeignValue::UInt32(0x07050302u32),
            ]
        );
    }

    #[test]
    fn test_process_conversion_demote_and_promote() {
        // bytecodes
        //
        // 0x0000 local_load32_f32     0 1
        // 0x0008 f64_promote_f32
        // 0x000a nop
        // 0x000c local_load_f64       0 0
        // 0x0014 f32_demote_f64
        // 0x0016 end
        //
        // (f64, f32)  ->  (f64, f32)
        //  |    |          ^    ^
        //  |    | promote  |0   |2
        //  |    \----------/    |
        //  \--------------------/ demote

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .write_opcode(Opcode::f64_promote_f32)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 0)
            .write_opcode(Opcode::f32_demote_f64)
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code1).to_text());

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::F64, DataType::F32], // params
            vec![DataType::F64, DataType::F32], // results
            vec![],                             // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.new_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &vec![
                ForeignValue::Float64(3.1415926535897931159979634685f64),
                // 0x400921FB54442D18 -> 0x40490FDB (3.1415927410125732421875)
                ForeignValue::Float32(2.71828f32),
                // 0x402DF84D -> 0x4005BF0995AAF790 (2.71828000000000002955857780762016773223876953125)
            ],
        );

        let exp0 = 2.71828f32 as f64;
        let exp1 = 3.1415926535897931159979634685f64 as f32;

        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::Float64(exp0), ForeignValue::Float32(exp1),]
        );
    }

    #[test]
    fn test_process_conversion_float_to_int() {
        // init_runtime();

        // bytecodes
        //
        // 0x0000 local_load32_f32     0 0
        // 0x0008 i32_convert_f32_s
        // 0x000a nop
        // 0x000c local_load32_f32     0 0
        // 0x0014 i32_convert_f32_u
        // 0x0016 nop
        // 0x0018 local_load32_f32     0 0
        // 0x0020 i64_convert_f32_s
        // 0x0022 nop
        // 0x0024 local_load32_f32     0 0
        // 0x002c i64_convert_f32_u
        // 0x002e nop
        // 0x0030 local_load_f64       0 1
        // 0x0038 i32_convert_f64_s
        // 0x003a nop
        // 0x003c local_load_f64       0 1
        // 0x0044 i32_convert_f64_u
        // 0x0046 nop
        // 0x0048 local_load_f64       0 1
        // 0x0050 i64_convert_f64_s
        // 0x0052 nop
        // 0x0054 local_load_f64       0 1
        // 0x005c i64_convert_f64_u
        // 0x005e nop
        // 0x0060 local_load32_f32     0 2
        // 0x0068 i32_convert_f32_s
        // 0x006a nop
        // 0x006c local_load32_f32     0 2
        // 0x0074 i32_convert_f32_u
        // 0x0076 nop
        // 0x0078 local_load32_f32     0 2
        // 0x0080 i64_convert_f32_s
        // 0x0082 nop
        // 0x0084 local_load32_f32     0 2
        // 0x008c i64_convert_f32_u
        // 0x008e nop
        // 0x0090 local_load_f64       0 3
        // 0x0098 i32_convert_f64_s
        // 0x009a nop
        // 0x009c local_load_f64       0 3
        // 0x00a4 i32_convert_f64_u
        // 0x00a6 nop
        // 0x00a8 local_load_f64       0 3
        // 0x00b0 i64_convert_f64_s
        // 0x00b2 nop
        // 0x00b4 local_load_f64       0 3
        // 0x00bc i64_convert_f64_u
        // 0x00be end
        //
        // (f32,              f64,            -f32,             -f64)
        //  |                 |                |                 |
        //  |                 |                |                 |
        //  |                 |                |                 |
        //  |---\---\---\     |---\---\---\    |---\---\---\     |---\---\---\
        //  |   |   |   |     |   |   |   |    |   |   |   |     |   |   |   |
        //  v   v   v   v     v   v   v   v    v   v   v   v     v   v   v   v
        // (i32 i32 i64 i64   i32 i32 i64 i64  i32 i32 i64 i64   i32 i32 i64 i64)

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode(Opcode::i32_convert_f32_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode(Opcode::i32_convert_f32_u)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode(Opcode::i64_convert_f32_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .write_opcode(Opcode::i64_convert_f32_u)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .write_opcode(Opcode::i32_convert_f64_s)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .write_opcode(Opcode::i32_convert_f64_u)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .write_opcode(Opcode::i64_convert_f64_s)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 1)
            .write_opcode(Opcode::i64_convert_f64_u)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 2)
            .write_opcode(Opcode::i32_convert_f32_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 2)
            .write_opcode(Opcode::i32_convert_f32_u)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 2)
            .write_opcode(Opcode::i64_convert_f32_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 2)
            .write_opcode(Opcode::i64_convert_f32_u)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 3)
            .write_opcode(Opcode::i32_convert_f64_s)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 3)
            .write_opcode(Opcode::i32_convert_f64_u)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 3)
            .write_opcode(Opcode::i64_convert_f64_s)
            .write_opcode_i16_i16_i16(Opcode::local_load_f64, 0, 0, 3)
            .write_opcode(Opcode::i64_convert_f64_u)
            //
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::F32, DataType::F64, DataType::F32, DataType::F64], // params
            vec![
                DataType::I32,
                DataType::I32,
                DataType::I64,
                DataType::I64,
                //
                DataType::I32,
                DataType::I32,
                DataType::I64,
                DataType::I64,
                //
                DataType::I32,
                DataType::I32,
                DataType::I64,
                DataType::I64,
                //
                DataType::I32,
                DataType::I32,
                DataType::I64,
                DataType::I64,
                //
            ], // results
            vec![],                                                           // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.new_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &vec![
                ForeignValue::Float32(2.236),
                ForeignValue::Float64(3.162),
                ForeignValue::Float32(-5.099),
                ForeignValue::Float64(-7.071),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(2),
                ForeignValue::UInt32(2),
                ForeignValue::UInt64(2),
                ForeignValue::UInt64(2),
                //
                ForeignValue::UInt32(3),
                ForeignValue::UInt32(3),
                ForeignValue::UInt64(3),
                ForeignValue::UInt64(3),
                //
                ForeignValue::UInt32(-5i32 as u32),
                ForeignValue::UInt32(0),
                ForeignValue::UInt64(-5i64 as u64),
                ForeignValue::UInt64(0),
                //
                ForeignValue::UInt32(-7i32 as u32),
                ForeignValue::UInt32(0),
                ForeignValue::UInt64(-7i64 as u64),
                ForeignValue::UInt64(0),
                //
            ]
        );
    }

    #[test]
    fn test_process_conversion_int_to_float() {
        // bytecodes
        //
        // 0x0000 local_load32         0 0
        // 0x0008 f32_convert_i32_s
        // 0x000a nop
        // 0x000c local_load32         0 0
        // 0x0014 f32_convert_i32_u
        // 0x0016 nop
        // 0x0018 local_load32         0 0
        // 0x0020 f64_convert_i32_s
        // 0x0022 nop
        // 0x0024 local_load32         0 0
        // 0x002c f64_convert_i32_u
        // 0x002e nop
        // 0x0030 local_load           0 1
        // 0x0038 f32_convert_i64_s
        // 0x003a nop
        // 0x003c local_load           0 1
        // 0x0044 f32_convert_i64_u
        // 0x0046 nop
        // 0x0048 local_load           0 1
        // 0x0050 f64_convert_i64_s
        // 0x0052 nop
        // 0x0054 local_load           0 1
        // 0x005c f64_convert_i64_u
        // 0x005e nop
        // 0x0060 local_load32         0 2
        // 0x0068 f32_convert_i32_s
        // 0x006a nop
        // 0x006c local_load32         0 2
        // 0x0074 f32_convert_i32_u
        // 0x0076 nop
        // 0x0078 local_load32         0 2
        // 0x0080 f64_convert_i32_s
        // 0x0082 nop
        // 0x0084 local_load32         0 2
        // 0x008c f64_convert_i32_u
        // 0x008e nop
        // 0x0090 local_load           0 3
        // 0x0098 f32_convert_i64_s
        // 0x009a nop
        // 0x009c local_load           0 3
        // 0x00a4 f32_convert_i64_u
        // 0x00a6 nop
        // 0x00a8 local_load           0 3
        // 0x00b0 f64_convert_i64_s
        // 0x00b2 nop
        // 0x00b4 local_load           0 3
        // 0x00bc f64_convert_i64_u
        // 0x00be end
        //
        // (i32,              i64,            -i32,             -i64)
        //  |                 |                |                 |
        //  |                 |                |                 |
        //  |                 |                |                 |
        //  |---\---\---\     |---\---\---\    |---\---\---\     |---\---\---\
        //  |   |   |   |     |   |   |   |    |   |   |   |     |   |   |   |
        //  v   v   v   v     v   v   v   v    v   v   v   v     v   v   v   v
        // (f32 f32 f64 f64   f32 f32 f64 f64  f32 f32 f64 f64   f32 f32 f64 f64)

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode(Opcode::f32_convert_i32_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode(Opcode::f32_convert_i32_u)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode(Opcode::f64_convert_i32_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 0)
            .write_opcode(Opcode::f64_convert_i32_u)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::f32_convert_i64_s)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::f32_convert_i64_u)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::f64_convert_i64_s)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 1)
            .write_opcode(Opcode::f64_convert_i64_u)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode(Opcode::f32_convert_i32_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode(Opcode::f32_convert_i32_u)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode(Opcode::f64_convert_i32_s)
            .write_opcode_i16_i16_i16(Opcode::local_load32, 0, 0, 2)
            .write_opcode(Opcode::f64_convert_i32_u)
            //
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 3)
            .write_opcode(Opcode::f32_convert_i64_s)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 3)
            .write_opcode(Opcode::f32_convert_i64_u)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 3)
            .write_opcode(Opcode::f64_convert_i64_s)
            .write_opcode_i16_i16_i16(Opcode::local_load, 0, 0, 3)
            .write_opcode(Opcode::f64_convert_i64_u)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I64, DataType::I32, DataType::I64], // params
            vec![
                DataType::F32,
                DataType::F32,
                DataType::F64,
                DataType::F64,
                //
                DataType::F32,
                DataType::F32,
                DataType::F64,
                DataType::F64,
                //
                DataType::F32,
                DataType::F32,
                DataType::F64,
                DataType::F64,
                //
                DataType::F32,
                DataType::F32,
                DataType::F64,
                DataType::F64,
            ], // results
            vec![],                                                           // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.new_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &vec![
                ForeignValue::UInt32(11),
                ForeignValue::UInt64(13),
                ForeignValue::UInt32(-17i32 as u32),
                ForeignValue::UInt64(-19i64 as u64),
            ],
        );

        // -11 -> 0xffffffef (u32)
        // -19 -> 0xffffffffffffffed (u64)
        let exp0 = -17i32 as u32 as f32;
        let exp1 = -17i32 as u32 as f64;
        let exp2 = -19i64 as u64 as f32;
        let exp3 = -19i64 as u64 as f64;

        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::Float32(11.0),
                ForeignValue::Float32(11.0),
                ForeignValue::Float64(11.0),
                ForeignValue::Float64(11.0),
                //
                ForeignValue::Float32(13.0),
                ForeignValue::Float32(13.0),
                ForeignValue::Float64(13.0),
                ForeignValue::Float64(13.0),
                //
                ForeignValue::Float32(-17.0),
                ForeignValue::Float32(exp0),
                ForeignValue::Float64(-17.0),
                ForeignValue::Float64(exp1),
                //
                ForeignValue::Float32(-19.0),
                ForeignValue::Float32(exp2),
                ForeignValue::Float64(-19.0),
                ForeignValue::Float64(exp3),
                //
            ]
        );
    }
}
