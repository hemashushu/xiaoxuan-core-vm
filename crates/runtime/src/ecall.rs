// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_binary::utils::format_bytecodes;
use ancvm_thread::thread_context::ThreadContext;
use ancvm_types::ecallcode::{ECallCode, MAX_ECALLCODE_NUMBER};

use crate::interpreter::InterpretResult;

use self::syscall::init_syscall_handlers;

pub mod callback;
pub mod extcall;
pub mod heap;
pub mod info;
pub mod syscall;

type EnvCallHandlerFunc = fn(&mut ThreadContext);

fn unreachable(thread_context: &mut ThreadContext) {
    let pc = &thread_context.pc;
    let func_item = &thread_context.program_ref.modules[pc.module_index]
        .func_section
        .items[pc.function_internal_index];
    let codes = &thread_context.program_ref.modules[pc.module_index]
        .func_section
        .codes_data
        [func_item.code_offset as usize..(func_item.code_offset + func_item.code_length) as usize];
    let code_text = format_bytecodes(codes);

    unreachable!(
        "Invalid environment call number: 0x{:04x}
Module index: {}
Function index: {}
Instruction address: 0x{:04x}
Bytecode:
{}",
        thread_context.get_param_i32(),
        pc.module_index,
        pc.function_internal_index,
        pc.instruction_address,
        code_text
    );
}

static mut HANDLERS: [EnvCallHandlerFunc; MAX_ECALLCODE_NUMBER] =
    [unreachable; MAX_ECALLCODE_NUMBER];

// note:
//
// ensure this initialization is only called once
pub fn init_ecall_handlers() {
    // init the syscall handlers
    init_syscall_handlers();

    let handlers = unsafe { &mut HANDLERS };

    // info
    handlers[ECallCode::runtime_name as usize] = info::runtime_name;
    handlers[ECallCode::runtime_version as usize] = info::runtime_version;
    handlers[ECallCode::features as usize] = info::features;
    handlers[ECallCode::check_feature as usize] = info::check_feature;

    // heap
    handlers[ECallCode::heap_fill as usize] = heap::heap_fill;
    handlers[ECallCode::heap_copy as usize] = heap::heap_copy;
    handlers[ECallCode::heap_capacity as usize] = heap::heap_capacity;
    handlers[ECallCode::heap_resize as usize] = heap::heap_resize;

    // system
    handlers[ECallCode::syscall as usize] = syscall::syscall;
    // ... extcall, callback_func
}

pub fn ecall(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param env_func_num:i32)

    let env_func_num = thread_context.get_param_i32();
    let func = unsafe { &HANDLERS[env_func_num as usize] };
    func(thread_context);
    InterpretResult::Move(8)
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        module_image::data_section::UninitDataEntry,
        utils::{
            build_module_binary_with_single_function,
            build_module_binary_with_single_function_and_data_sections, BytecodeWriter,
        },
    };
    use ancvm_types::{ecallcode::ECallCode, opcode::Opcode, DataType, ForeignValue};

    use crate::{
        in_memory_program::InMemoryProgram, interpreter::process_function, program::Program,
        RUNTIME_CODE_NAME, RUNTIME_MAJOR_VERSION, RUNTIME_MINOR_VERSION, RUNTIME_PATCH_VERSION,
    };

    #[test]
    fn test_ecall_heap_capacity() {
        // init_runtime();

        // bytecodes
        //
        // 0x0000 ecall                261
        // 0x0008 i32_imm              0x2
        // 0x0010 ecall                262
        // 0x0018 i32_imm              0x4
        // 0x0020 ecall                262
        // 0x0028 i32_imm              0x1
        // 0x0030 ecall                262
        // 0x0038 ecall                261
        // 0x0040 end
        //
        // () -> (i64, i64, i64, i64, i64)

        let code0 = BytecodeWriter::new()
            // get the capacity
            .write_opcode_i32(Opcode::ecall, ECallCode::heap_capacity as u32)
            // resize - increase
            .write_opcode_i32(Opcode::i32_imm, 2)
            .write_opcode_i32(Opcode::ecall, ECallCode::heap_resize as u32)
            // resize - increase
            .write_opcode_i32(Opcode::i32_imm, 4)
            .write_opcode_i32(Opcode::ecall, ECallCode::heap_resize as u32)
            // resize - decrease
            .write_opcode_i32(Opcode::i32_imm, 1)
            .write_opcode_i32(Opcode::ecall, ECallCode::heap_resize as u32)
            // get the capcity
            .write_opcode_i32(Opcode::ecall, ECallCode::heap_capacity as u32)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![], // params
            vec![
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
            ], // results
            vec![], // local varslist which
            code0,
        );

        let program0 = InMemoryProgram::new(vec![binary0]);
        let program_context0 = program0.build_program_context().unwrap();
        let mut thread_context0 = program_context0.new_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &vec![]);

        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt64(0),
                ForeignValue::UInt64(2),
                ForeignValue::UInt64(4),
                ForeignValue::UInt64(1),
                ForeignValue::UInt64(1),
            ]
        );
    }

    #[test]
    fn test_ecall_runtime_version() {
        // init_runtime();

        // bytecodes
        //
        // 0x0000 ecall                257
        // 0x0008 end
        //
        // () -> (i64)

        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::ecall, ECallCode::runtime_version as u32)
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code0).to_text());

        let binary0 = build_module_binary_with_single_function(
            vec![],              // params
            vec![DataType::I64], // results
            vec![],              // local varslist which
            code0,
        );

        let program0 = InMemoryProgram::new(vec![binary0]);
        let program_context0 = program0.build_program_context().unwrap();
        let mut thread_context0 = program_context0.new_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &vec![]);

        let expect_version_number = RUNTIME_PATCH_VERSION as u64
            | (RUNTIME_MINOR_VERSION as u64) << 16
            | (RUNTIME_MAJOR_VERSION as u64) << 32;

        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::UInt64(expect_version_number)]
        );
    }

    #[test]
    fn test_ecall_runtime_name() {
        // bytecodes
        //
        // 0x0000 host_addr_data       0 0
        // 0x0008 ecall                256
        // 0x0010 data_load            0 0
        // 0x0018 end
        //
        // () -> (i32, i64)
        //        ^    ^
        //        |    |name buffer (8 bytes)
        //        |name length

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::host_addr_data, 0, 0)
            .write_opcode_i32(Opcode::ecall, ECallCode::runtime_name as u32)
            .write_opcode_i16_i32(Opcode::data_load, 0, 0)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function_and_data_sections(
            vec![],
            vec![],
            vec![UninitDataEntry::from_i64()],
            vec![],                             // params
            vec![DataType::I32, DataType::I64], // results
            vec![],                             // local varslist which
            code0,
        );

        let program0 = InMemoryProgram::new(vec![binary0]);
        let program_context0 = program0.build_program_context().unwrap();
        let mut thread_context0 = program_context0.new_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &vec![]);
        let fvs1 = result0.unwrap();
        let name_len = if let ForeignValue::UInt32(i) = fvs1[0] {
            i
        } else {
            0
        };
        let name_u64 = if let ForeignValue::UInt64(i) = fvs1[1] {
            i
        } else {
            0
        };

        let name_data = name_u64.to_le_bytes();
        assert_eq!(&RUNTIME_CODE_NAME[..], &name_data[0..name_len as usize]);
    }
}
