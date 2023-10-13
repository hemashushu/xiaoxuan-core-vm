// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_program::thread_context::ThreadContext;

use super::InterpretResult;

pub fn zero(thread_context: &mut ThreadContext) -> InterpretResult {
    thread_context.stack.push_i64_u(0);
    InterpretResult::Move(2)
}

pub fn drop_(thread_context: &mut ThreadContext) -> InterpretResult {
    thread_context.stack.drop_();
    InterpretResult::Move(2)
}

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
    // let value = unsafe { std::mem::transmute::<u32, f32>(i32_value) };
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
    use ancvm_binary::utils::{build_module_binary_with_single_function, BytecodeWriter};
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    #[test]
    fn test_process_operand_zero() {
        // bytecodes
        //
        // 0x0000 zero
        // 0x0002 end
        //
        // (i32) -> (i32, i32)

        let code0 = BytecodeWriter::new()
            .write_opcode(Opcode::zero)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I32],                // params
            vec![DataType::I32, DataType::I32], // results
            vec![],                             // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 =
            process_function(&mut thread_context0, 0, 0, &vec![ForeignValue::UInt32(233)]);
        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::UInt32(233), ForeignValue::UInt32(0)]
        );
    }

    #[test]
    fn test_process_operand_drop() {
        // bytecodes
        //
        // 0x0000 drop
        // 0x0002 end
        //
        // (i32, i32) -> (i32)
        let code0 = BytecodeWriter::new()
            .write_opcode(Opcode::drop)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32], // params
            vec![DataType::I32],                // results
            vec![],                             // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &vec![ForeignValue::UInt32(13), ForeignValue::UInt32(17)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(13)]);
    }

    #[test]
    fn test_process_operand_duplicate() {
        // bytecodes
        //
        // 0x0000 duplicate
        // 0x0002 end
        //
        // (i32) -> (i32, i32)
        let code0 = BytecodeWriter::new()
            .write_opcode(Opcode::duplicate)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I32],                // params
            vec![DataType::I32, DataType::I32], // results
            vec![],                             // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &vec![ForeignValue::UInt32(19)]);
        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::UInt32(19), ForeignValue::UInt32(19)]
        );
    }

    #[test]
    fn test_process_operand_swap() {
        // bytecodes
        //
        // 0x0000 swap
        // 0x0002 end
        //
        // (i32) -> (i32, i32)

        let code0 = BytecodeWriter::new()
            .write_opcode(Opcode::swap)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32], // params
            vec![DataType::I32, DataType::I32], // results
            vec![],                             // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &vec![ForeignValue::UInt32(211), ForeignValue::UInt32(223)],
        );
        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::UInt32(223), ForeignValue::UInt32(211)]
        );
    }

    #[test]
    fn test_process_immediate_int() {
        // bytecodes
        //
        // 0x0000 i32_imm              0x17
        // 0x0008 i64_imm              0x43475359 0x29313741    ;; 0x29313741_43475359
        // 0x0014 i32_imm              0xffffff21               ;; -223
        // 0x001c i64_imm              0xffffff1d 0xffffffff    ;; -227
        // 0x0028 end
        // () -> (i32, i64, i32, i64)
        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 23)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x29313741_43475359u64)
            .write_opcode_i32(Opcode::i32_imm, (0i32 - 223) as u32)
            .write_opcode_pesudo_i64(Opcode::i64_imm, (0i64 - 227) as u64)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![],                                                           // params
            vec![DataType::I32, DataType::I64, DataType::I32, DataType::I64], // results
            vec![],                                                           // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &vec![]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(23),
                ForeignValue::UInt64(0x29313741_43475359u64),
                ForeignValue::UInt32((0i32 - 223) as u32),
                ForeignValue::UInt64((0i64 - 227) as u64)
            ]
        );
    }

    #[test]
    fn test_process_immediate_float() {
        // bytecodes
        //
        // 0x0000 f32_imm              0x40490fda               ;; 3.1415926
        // 0x0008 f64_imm              0xc5445f02 0x390b85f8    ;; 6.626e-34
        // 0x0014 f32_imm              0xc02df84d               ;; -2.71828
        // 0x001c f64_imm              0xb0000000 0xc1b1de6e    ;; -2.9979e8
        // 0x0028 end
        //
        // () -> (f32, f64, f32, f64)
        let code0 = BytecodeWriter::new()
            .write_opcode_pesudo_f32(Opcode::f32_imm, 3.1415926f32)
            .write_opcode_pesudo_f64(Opcode::f64_imm, 6.626e-34f64)
            .write_opcode_pesudo_f32(Opcode::f32_imm, -2.71828f32)
            .write_opcode_pesudo_f64(Opcode::f64_imm, -2.9979e8f64)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![],                                                           // params
            vec![DataType::F32, DataType::F64, DataType::F32, DataType::F64], // results
            vec![],                                                           // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &vec![]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::Float32(3.1415926f32),
                ForeignValue::Float64(6.626e-34f64),
                ForeignValue::Float32(-2.71828f32),
                ForeignValue::Float64(-2.9979e8f64)
            ]
        );
    }
}
