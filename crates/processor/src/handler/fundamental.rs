// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::thread_context::ThreadContext;

use super::{HandleResult, Handler};

pub fn nop(_handler: &Handler, _thread: &mut ThreadContext) -> HandleResult {
    HandleResult::Move(2)
}

// pub fn zero(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
//     thread_context.stack.push_i64_u(0);
//     HandleResult::Move(2)
// }

/*
pub fn drop_(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    thread_context.stack.drop_();
    HandleResult::Move(2)
}

pub fn duplicate(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    thread_context.stack.duplicate();
    HandleResult::Move(2)
}

pub fn swap(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    let a = thread_context.stack.pop_i64_u();
    let b = thread_context.stack.pop_i64_u();
    thread_context.stack.push_i64_u(a);
    thread_context.stack.push_i64_u(b);
    HandleResult::Move(2)
}
*/

// pub fn select_nez(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
//     // (operand when_true:any when_false:any test:i32) -> any
//     //
//     // | test    | a
//     // | false   | b
//     // | true    | c
//     // | ...     |
//     // \---------/
//     //
//     // pop operands a, b and c, then push c if a!=0, otherwise push b.
//
//     let test = thread_context.stack.pop_i32_u();
//     let alternate = thread_context.stack.pop_i64_u();
//
//     if test == 0 {
//         thread_context.stack.drop_();
//         thread_context.stack.push_i64_u(alternate);
//     }
//
//     HandleResult::Move(2)
// }

pub fn imm_i32(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    // note:
    // all i32 will be signed-extended to i64
    let value = thread_context.get_param_i32();
    thread_context.stack.push_i32_u(value);
    HandleResult::Move(8)
}

pub fn imm_i64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    let (low, high) = thread_context.get_param_i32_i32();
    let mut value: u64 = high as u64;
    value <<= 32;
    value |= low as u64;

    thread_context.stack.push_i64_u(value);
    HandleResult::Move(12)
}

pub fn imm_f32(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    let i32_value = thread_context.get_param_i32();
    let value = f32::from_bits(i32_value);

    thread_context.stack.push_f32(value);
    HandleResult::Move(8)
}

pub fn imm_f64(_handler: &Handler, thread_context: &mut ThreadContext) -> HandleResult {
    let (low, high) = thread_context.get_param_i32_i32();

    let mut bytes = [0u8; 8];
    {
        let (p0, p1) = bytes.split_at_mut(4);
        p0.copy_from_slice(&low.to_le_bytes());
        p1.copy_from_slice(&high.to_le_bytes());
    }

    let value = f64::from_le_bytes(bytes);

    thread_context.stack.push_f64(value);
    HandleResult::Move(12)
}

#[cfg(test)]
mod tests {
    use crate::{
        handler::Handler, in_memory_resource::InMemoryResource, process::process_function,
    };
    use ancvm_context::resource::Resource;
    use ancvm_image::{
        bytecode_writer::BytecodeWriterHelper,
        utils::helper_build_module_binary_with_single_function,
    };
    use ancvm_isa::{opcode::Opcode, ForeignValue, OperandDataType};

    #[test]
    fn test_interpreter_fundamental_nop() {
        // bytecodes
        //
        // 0x0000  00 01                       nop
        // 0x0002  c0 03                       end
        //
        // (i32) -> (i32)
        let code0 = BytecodeWriterHelper::new()
            .append_opcode(Opcode::nop)
            .append_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", format_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function(
            vec![OperandDataType::I32], // params
            vec![OperandDataType::I32], // results
            vec![],                     // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &handler,
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(11)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(11)]);
    }

    /*
    #[test]
    fn test_interpreter_fundamental_zero() {
        // () -> (i32)
        let code0 = BytecodeWriterHelper::new()
            .append_opcode(Opcode::zero)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],              // params
            vec![OperandDataType::I32], // results
            vec![],              // local variables
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(0)]);
    }
    */

    /*
    #[test]
    fn test_interpreter_fundamental_drop() {
        // () -> (i32)
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 13)
            .append_opcode_i32(Opcode::imm_i32, 17)
            .append_opcode(Opcode::drop)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],              // params
            vec![OperandDataType::I32], // results
            vec![],              // local variables
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(13)]);
    }

    #[test]
    fn test_interpreter_fundamental_duplicate() {
        // () -> (i32, i32)
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 19)
            .append_opcode(Opcode::duplicate)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                             // params
            vec![OperandDataType::I32, OperandDataType::I32], // results
            vec![],                             // local variables
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::U32(19), ForeignValue::U32(19)]
        );
    }

    #[test]
    fn test_interpreter_fundamental_swap() {
        // () -> (i32, i32)
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 211)
            .append_opcode_i32(Opcode::imm_i32, 223)
            .append_opcode(Opcode::swap)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                             // params
            vec![OperandDataType::I32, OperandDataType::I32], // results
            vec![],                             // local variables
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::U32(223), ForeignValue::U32(211)]
        );
    }
    */

    /*
    #[test]
    fn test_interpreter_fundamental_select_nez_false() {
        // () -> (i32)
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 11)
            .append_opcode_i32(Opcode::imm_i32, 13)
            .append_opcode(Opcode::zero)
            .append_opcode(Opcode::select_nez)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],              // params
            vec![OperandDataType::I32], // results
            vec![],              // local variables
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(13)]);
    }

    #[test]
    fn test_interpreter_fundamental_select_nez_true() {
        // () -> (i32)
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 11)
            .append_opcode_i32(Opcode::imm_i32, 13)
            .append_opcode_i32(Opcode::imm_i32, 1)
            .append_opcode(Opcode::select_nez)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],              // params
            vec![OperandDataType::I32], // results
            vec![],              // local variables
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(11)]);
    }
     */

    #[test]
    fn test_interpreter_fundamental_immediate_integer() {
        // bytecodes
        //
        // 0x0000  40 01 00 00  17 00 00 00    imm_i32           0x00000017
        // 0x0008  41 01 00 00  59 53 47 43    imm_i64           low:0x43475359  high:0x29313741
        //         41 37 31 29
        // 0x0014  40 01 00 00  21 ff ff ff    imm_i32           0xffffff21
        // 0x001c  41 01 00 00  1d ff ff ff    imm_i64           low:0xffffff1d  high:0xffffffff
        //         ff ff ff ff
        // 0x0028  c0 03                       end
        //
        // () -> (i32, i64, i32, i64)
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::imm_i32, 23)
            .append_opcode_i64(Opcode::imm_i64, 0x29313741_43475359u64)
            .append_opcode_i32(Opcode::imm_i32, (-223_i32) as u32)
            .append_opcode_i64(Opcode::imm_i64, (-227_i32) as u64)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![], // params
            vec![
                OperandDataType::I32,
                OperandDataType::I64,
                OperandDataType::I32,
                OperandDataType::I64,
            ], // results
            vec![], // local variables
            code0,
        );

        let interpreter = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&interpreter, &mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U32(23),
                ForeignValue::U64(0x29313741_43475359u64),
                ForeignValue::U32((-223_i32) as u32),
                ForeignValue::U64((-227_i64) as u64)
            ]
        );
    }

    #[test]
    fn test_interpreter_fundamental_immediate_float() {
        // bytecodes
        //
        // 0x0000  42 01 00 00  db 0f 49 40    imm_f32           0x40490fdb
        // 0x0008  43 01 00 00  cd 3b 7f 66    imm_f64           low:0x667f3bcd  high:0x3ff6a09e
        //         9e a0 f6 3f
        // 0x0014  42 01 00 00  54 f8 2d c0    imm_f32           0xc02df854
        // 0x001c  43 01 00 00  66 73 2d 38    imm_f64           low:0x382d7366  high:0xbfe0c152
        //         52 c1 e0 bf
        // 0x0028  c0 03                       end
        //
        // () -> (f32, f64, f32, f64)
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_f32(Opcode::imm_f32, std::f32::consts::PI)
            .append_opcode_f64(Opcode::imm_f64, std::f64::consts::SQRT_2)
            .append_opcode_f32(Opcode::imm_f32, -std::f32::consts::E)
            .append_opcode_f64(Opcode::imm_f64, -std::f64::consts::FRAC_PI_6)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![], // params
            vec![
                OperandDataType::F32,
                OperandDataType::F64,
                OperandDataType::F32,
                OperandDataType::F64,
            ], // results
            vec![], // local variables
            code0,
        );

        let interpreter = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&interpreter, &mut thread_context0, 0, 0, &[]);
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
