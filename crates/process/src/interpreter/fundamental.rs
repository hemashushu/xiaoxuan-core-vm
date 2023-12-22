// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_program::thread_context::ThreadContext;

use super::InterpretResult;

pub fn nop(_thread: &mut ThreadContext) -> InterpretResult {
    InterpretResult::Move(2)
}

pub fn zero(thread_context: &mut ThreadContext) -> InterpretResult {
    thread_context.stack.push_i64_u(0);
    InterpretResult::Move(2)
}

pub fn drop_(thread_context: &mut ThreadContext) -> InterpretResult {
    thread_context.stack.drop_();
    InterpretResult::Move(2)
}

/*
pub fn duplicate(thread_context: &mut ThreadContext) -> InterpretResult {
    thread_context.stack.duplicate();
    InterpretResult::Move(2)
}

pub fn swap(thread_context: &mut ThreadContext) -> InterpretResult {
    let a = thread_context.stack.pop_i64_u();
    let b = thread_context.stack.pop_i64_u();
    thread_context.stack.push_i64_u(a);
    thread_context.stack.push_i64_u(b);
    InterpretResult::Move(2)
}
*/

pub fn select_nez(thread_context: &mut ThreadContext) -> InterpretResult {
    // (operand when_true:any when_false:any test:i32) -> any
    //
    // | test    | a
    // | false   | b
    // | true    | c
    // | ...     |
    // \---------/
    //
    // pop operands a, b and c, then push c if a!=0, otherwise push b.

    let test = thread_context.stack.pop_i32_u();
    let alternate = thread_context.stack.pop_i64_u();

    if test == 0 {
        thread_context.stack.drop_();
        thread_context.stack.push_i64_u(alternate);
    }

    InterpretResult::Move(2)
}

pub fn i32_imm(thread_context: &mut ThreadContext) -> InterpretResult {
    let value = thread_context.get_param_i32();
    thread_context.stack.push_i32_u(value);
    InterpretResult::Move(8)
}

pub fn i64_imm(thread_context: &mut ThreadContext) -> InterpretResult {
    let (low, high) = thread_context.get_param_i32_i32();
    let mut value: u64 = high as u64;
    value <<= 32;
    value |= low as u64;

    thread_context.stack.push_i64_u(value);
    InterpretResult::Move(12)
}

pub fn f32_imm(thread_context: &mut ThreadContext) -> InterpretResult {
    let i32_value = thread_context.get_param_i32();
    let value = f32::from_bits(i32_value);

    thread_context.stack.push_f32(value);
    InterpretResult::Move(8)
}

pub fn f64_imm(thread_context: &mut ThreadContext) -> InterpretResult {
    let (low, high) = thread_context.get_param_i32_i32();

    let mut bytes = [0u8; 8];
    {
        let (p0, p1) = bytes.split_at_mut(4);
        p0.copy_from_slice(&low.to_le_bytes());
        p1.copy_from_slice(&high.to_le_bytes());
    }

    let value = f64::from_le_bytes(bytes);

    thread_context.stack.push_f64(value);
    InterpretResult::Move(12)
}

#[cfg(test)]
mod tests {
    use crate::{in_memory_program_source::InMemoryProgramSource, interpreter::process_function};
    use ancvm_binary::{
        bytecode_writer::BytecodeWriter, utils::helper_build_module_binary_with_single_function,
    };
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    #[test]
    fn test_interpreter_host_nop() {
        // (i32) -> (i32)

        let code0 = BytecodeWriter::new()
            .append_opcode(Opcode::nop)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![DataType::I32], // params
            vec![DataType::I32], // results
            vec![],              // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::U32(11)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(11)]);
    }

    #[test]
    fn test_interpreter_fundamental_zero() {
        // () -> (i32)
        let code0 = BytecodeWriter::new()
            .append_opcode(Opcode::zero)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],              // params
            vec![DataType::I32], // results
            vec![],              // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(0)]);
    }

    #[test]
    fn test_interpreter_fundamental_drop() {
        // () -> (i32)
        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 13)
            .append_opcode_i32(Opcode::i32_imm, 17)
            .append_opcode(Opcode::drop)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],              // params
            vec![DataType::I32], // results
            vec![],              // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(13)]);
    }

    /*
    #[test]
    fn test_interpreter_fundamental_duplicate() {
        // () -> (i32, i32)
        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 19)
            .append_opcode(Opcode::duplicate)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                             // params
            vec![DataType::I32, DataType::I32], // results
            vec![],                             // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::U32(19), ForeignValue::U32(19)]
        );
    }

    #[test]
    fn test_interpreter_fundamental_swap() {
        // () -> (i32, i32)
        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 211)
            .append_opcode_i32(Opcode::i32_imm, 223)
            .append_opcode(Opcode::swap)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                             // params
            vec![DataType::I32, DataType::I32], // results
            vec![],                             // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::U32(223), ForeignValue::U32(211)]
        );
    }
    */

    #[test]
    fn test_interpreter_fundamental_select_nez_false() {
        // () -> (i32)
        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 11)
            .append_opcode_i32(Opcode::i32_imm, 13)
            .append_opcode(Opcode::zero)
            .append_opcode(Opcode::select_nez)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],              // params
            vec![DataType::I32], // results
            vec![],              // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(13)]);
    }

    #[test]
    fn test_interpreter_fundamental_select_nez_true() {
        // () -> (i32)
        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 11)
            .append_opcode_i32(Opcode::i32_imm, 13)
            .append_opcode_i32(Opcode::i32_imm, 1)
            .append_opcode(Opcode::select_nez)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],              // params
            vec![DataType::I32], // results
            vec![],              // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(11)]);
    }

    #[test]
    fn test_interpreter_fundamental_immediate_int() {
        // bytecodes
        //
        // 0x0000  i32.imm         0x00000017
        // 0x0008  i64.imm         low:0x43475359  high:0x29313741
        // 0x0014  i32.imm         0xffffff21                           ;; -223
        // 0x001c  i64.imm         low:0xffffff1d  high:0xffffffff      ;; -227
        // 0x0028  end
        //
        // () -> (i32, i64, i32, i64)
        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 23)
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x29313741_43475359u64)
            .append_opcode_i32(Opcode::i32_imm, (0i32 - 223) as u32)
            .append_opcode_pesudo_i64(Opcode::i64_imm, (0i64 - 227) as u64)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                                                           // params
            vec![DataType::I32, DataType::I64, DataType::I32, DataType::I64], // results
            vec![],                                                           // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U32(23),
                ForeignValue::U64(0x29313741_43475359u64),
                ForeignValue::U32((-223i32) as u32),
                ForeignValue::U64((-227i64) as u64)
            ]
        );
    }

    #[test]
    fn test_interpreter_fundamental_immediate_float() {
        // bytecodes
        //
        // 0x0000  f32.imm         0x40490fdb                           ;; Pi
        // 0x0008  f64.imm         low:0x667f3bcd  high:0x3ff6a09e      ;; sqrt(2)
        // 0x0014  f32.imm         0xc02df854                           ;; E
        // 0x001c  f64.imm         low:0x382d7366  high:0xbfe0c152      ;; Pi/6
        // 0x0028  end
        //
        // () -> (f32, f64, f32, f64)
        let code0 = BytecodeWriter::new()
            .append_opcode_pesudo_f32(Opcode::f32_imm, std::f32::consts::PI)
            .append_opcode_pesudo_f64(Opcode::f64_imm, std::f64::consts::SQRT_2)
            .append_opcode_pesudo_f32(Opcode::f32_imm, -std::f32::consts::E)
            .append_opcode_pesudo_f64(Opcode::f64_imm, -std::f64::consts::FRAC_PI_6)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                                                           // params
            vec![DataType::F32, DataType::F64, DataType::F32, DataType::F64], // results
            vec![],                                                           // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::F32(std::f32::consts::PI),
                ForeignValue::F64(std::f64::consts::SQRT_2),
                ForeignValue::F32(-std::f32::consts::E),
                ForeignValue::F64(-std::f64::consts::FRAC_PI_6),
            ]
        );
    }
}
