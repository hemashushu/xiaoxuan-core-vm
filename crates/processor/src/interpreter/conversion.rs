// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::thread_context::ThreadContext;

use super::InterpretResult;

// demote i64 to i32
// discard the high 32 bits of an i64 number directly
pub fn i32_truncate_i64(thread_context: &mut ThreadContext) -> InterpretResult {
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
    use crate::{in_memory_program_resource::InMemoryProgramResource, interpreter::process_function};
    use ancvm_binary::{
        bytecode_writer::BytecodeWriter, utils::helper_build_module_binary_with_single_function,
    };
    use ancvm_context::program_resource::ProgramResource;
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    #[test]
    fn test_interpreter_conversion_extend_and_truncate() {
        // (i64, i32)  ->  (i64, i64, i32)
        //  |    |          ^    ^    ^
        //  |    | extend   |    |    |
        //  |    \----------/----/    |
        //  \-------------------------/ truncate

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i64_extend_i32_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i64_extend_i32_u)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 0)
            .append_opcode(Opcode::i32_truncate_i64)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![DataType::I64, DataType::I32],                // params
            vec![DataType::I64, DataType::I64, DataType::I32], // results
            vec![],                                            // local vars
            code0,
        );

        let program_resource0 = InMemoryProgramResource::new(vec![binary0]);
        let process_context0 = program_resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::U64(0x19171311_07050302u64),
                ForeignValue::U32(0x80706050u32),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U64(0xffffffff_80706050u64),
                ForeignValue::U64(0x00000000_80706050u64),
                ForeignValue::U32(0x07050302u32),
            ]
        );
    }

    #[test]
    fn test_interpreter_conversion_demote_and_promote() {
        // (f64, f32)  ->  (f64, f32)
        //  |    |          ^    ^
        //  |    | promote  |    |
        //  |    \----------/    |
        //  \--------------------/ demote

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 1)
            .append_opcode(Opcode::f64_promote_f32)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 0)
            .append_opcode(Opcode::f32_demote_f64)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![DataType::F64, DataType::F32], // params
            vec![DataType::F64, DataType::F32], // results
            vec![],                             // local vars
            code0,
        );

        let program_resource0 = InMemoryProgramResource::new(vec![binary0]);
        let process_context0 = program_resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::F64(std::f64::consts::PI),
                ForeignValue::F32(std::f32::consts::E),
            ],
        );

        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::F64(std::f32::consts::E as f64),
                ForeignValue::F32(std::f64::consts::PI as f32),
            ]
        );
    }

    #[test]
    fn test_interpreter_conversion_float_to_int() {
        // (f32,              f64,            -f32,             -f64)
        //  |                 |                |                 |
        //  |                 |                |                 |
        //  |                 |                |                 |
        //  |---\---\---\     |---\---\---\    |---\---\---\     |---\---\---\
        //  |   |   |   |     |   |   |   |    |   |   |   |     |   |   |   |
        //  v   v   v   v     v   v   v   v    v   v   v   v     v   v   v   v
        // (i32 i32 i64 i64   i32 i32 i64 i64  i32 i32 i64 i64   i32 i32 i64 i64)

        let code0 = BytecodeWriter::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::i32_convert_f32_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::i32_convert_f32_u)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::i64_convert_f32_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 0)
            .append_opcode(Opcode::i64_convert_f32_u)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode(Opcode::i32_convert_f64_s)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode(Opcode::i32_convert_f64_u)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode(Opcode::i64_convert_f64_s)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 1)
            .append_opcode(Opcode::i64_convert_f64_u)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 2)
            .append_opcode(Opcode::i32_convert_f32_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 2)
            .append_opcode(Opcode::i32_convert_f32_u)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 2)
            .append_opcode(Opcode::i64_convert_f32_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_f32, 0, 0, 2)
            .append_opcode(Opcode::i64_convert_f32_u)
            // group 3
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 3)
            .append_opcode(Opcode::i32_convert_f64_s)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 3)
            .append_opcode(Opcode::i32_convert_f64_u)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 3)
            .append_opcode(Opcode::i64_convert_f64_s)
            .append_opcode_i16_i16_i16(Opcode::local_load64_f64, 0, 0, 3)
            .append_opcode(Opcode::i64_convert_f64_u)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", format_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function(
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

        let program_resource0 = InMemoryProgramResource::new(vec![binary0]);
        let process_context0 = program_resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::F32(2.236),
                ForeignValue::F64(3.162),
                ForeignValue::F32(-5.099),
                ForeignValue::F64(-7.071),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::U32(2),
                ForeignValue::U32(2),
                ForeignValue::U64(2),
                ForeignValue::U64(2),
                // group 1
                ForeignValue::U32(3),
                ForeignValue::U32(3),
                ForeignValue::U64(3),
                ForeignValue::U64(3),
                // group 2
                ForeignValue::U32(-5i32 as u32),
                ForeignValue::U32(0),
                ForeignValue::U64(-5i64 as u64),
                ForeignValue::U64(0),
                // group 3
                ForeignValue::U32(-7i32 as u32),
                ForeignValue::U32(0),
                ForeignValue::U64(-7i64 as u64),
                ForeignValue::U64(0),
            ]
        );
    }

    #[test]
    fn test_interpreter_conversion_int_to_float() {
        // (i32,              i64,            -i32,             -i64)
        //  |                 |                |                 |
        //  |                 |                |                 |
        //  |                 |                |                 |
        //  |---\---\---\     |---\---\---\    |---\---\---\     |---\---\---\
        //  |   |   |   |     |   |   |   |    |   |   |   |     |   |   |   |
        //  v   v   v   v     v   v   v   v    v   v   v   v     v   v   v   v
        // (f32 f32 f64 f64   f32 f32 f64 f64  f32 f32 f64 f64   f32 f32 f64 f64)

        let code0 = BytecodeWriter::new()
            // group 0
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode(Opcode::f32_convert_i32_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode(Opcode::f32_convert_i32_u)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode(Opcode::f64_convert_i32_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode(Opcode::f64_convert_i32_u)
            // group 1
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::f32_convert_i64_s)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::f32_convert_i64_u)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::f64_convert_i64_s)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 1)
            .append_opcode(Opcode::f64_convert_i64_u)
            // group 2
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode(Opcode::f32_convert_i32_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode(Opcode::f32_convert_i32_u)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode(Opcode::f64_convert_i32_s)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode(Opcode::f64_convert_i32_u)
            // group 3
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 3)
            .append_opcode(Opcode::f32_convert_i64_s)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 3)
            .append_opcode(Opcode::f32_convert_i64_u)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 3)
            .append_opcode(Opcode::f64_convert_i64_s)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 3)
            .append_opcode(Opcode::f64_convert_i64_u)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
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

        let program_resource0 = InMemoryProgramResource::new(vec![binary0]);
        let process_context0 = program_resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::U32(11),
                ForeignValue::U64(13),
                ForeignValue::U32(-17i32 as u32),
                ForeignValue::U64(-19i64 as u64),
            ],
        );

        // -11 -> 0xffffffef (u32)
        // -19 -> 0xffffffffffffffed (u64)

        assert_eq!(
            result0.unwrap(),
            vec![
                // group 0
                ForeignValue::F32(11.0),
                ForeignValue::F32(11.0),
                ForeignValue::F64(11.0),
                ForeignValue::F64(11.0),
                // group 1
                ForeignValue::F32(13.0),
                ForeignValue::F32(13.0),
                ForeignValue::F64(13.0),
                ForeignValue::F64(13.0),
                // group 2
                ForeignValue::F32(-17.0),
                ForeignValue::F32(-17i32 as u32 as f32),
                ForeignValue::F64(-17.0),
                ForeignValue::F64(-17i32 as u32 as f64),
                // group 3
                ForeignValue::F32(-19.0),
                ForeignValue::F32(-19i64 as u64 as f32),
                ForeignValue::F64(-19.0),
                ForeignValue::F64(-19i64 as u64 as f64),
            ]
        );
    }
}
