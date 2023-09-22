// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::sync::Mutex;

use ancvm_binary::utils::print_bytecodes;
use ancvm_types::ecallcode::{ECallCode, MAX_ECALLCODE_NUMBER};

use crate::{interpreter::InterpretResult, thread::Thread};

pub mod heap;
pub mod info;

type EnvCallHandlerFunc = fn(&mut Thread) -> Result<(), usize>;

fn unreachable(thread: &mut Thread) -> Result<(), usize> {
    let pc = &thread.pc;
    let func_item = &thread.context.modules[pc.module_index].func_section.items[pc.function_internal_index];
    let codes = &thread.context.modules[pc.module_index]
        .func_section
        .codes_data
        [func_item.code_offset as usize..(func_item.code_offset + func_item.code_length) as usize];
    let code_text = print_bytecodes(codes);

    unreachable!(
        "Invalid environment call number: 0x{:04x}
Module index: {}
Function index: {}
Instruction address: 0x{:04x}
Bytecode:
{}",
        thread.get_param_i32(),
        pc.module_index,
        pc.function_internal_index,
        pc.instruction_address,
        code_text
    );
}

static INIT_LOCK: Mutex<i32> = Mutex::new(0);
static mut IS_INIT: bool = false;
static mut HANDLERS: [EnvCallHandlerFunc; MAX_ECALLCODE_NUMBER] =
    [unreachable; MAX_ECALLCODE_NUMBER];

pub fn init_ecall_handlers() {
    let _lock = INIT_LOCK.lock().unwrap();

    unsafe {
        if IS_INIT {
            return;
        }
        IS_INIT = true;
    }

    let handlers = unsafe { &mut HANDLERS };

    // the initialization can only be called once
    // in the unit test environment (`$ cargo test`), the init procedure
    // runs in parallel.
    // if handlers[ECallCode::runtime_name as usize] == info::runtime_name {
    //     return;
    // }

    // info
    handlers[ECallCode::runtime_name as usize] = info::runtime_name;
    handlers[ECallCode::runtime_version as usize] = info::runtime_version;
    handlers[ECallCode::runtime_features as usize] = info::runtime_features;
    // heap
    handlers[ECallCode::heap_fill as usize] = heap::heap_fill;
    handlers[ECallCode::heap_copy as usize] = heap::heap_copy;
    handlers[ECallCode::heap_capacity as usize] = heap::heap_capacity;
    handlers[ECallCode::heap_resize as usize] = heap::heap_resize;
}

pub fn ecall(thread: &mut Thread) -> InterpretResult {
    // (param env_func_num:i32)

    let env_func_num = thread.get_param_i32();
    let func = unsafe { &HANDLERS[env_func_num as usize] };
    let result = func(thread);

    match result {
        Ok(_) => InterpretResult::Move(8),
        Err(err_code) => InterpretResult::EnvError(err_code),
    }
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        load_modules_binary,
        module_image::data_section::UninitDataEntry,
        utils::{
            build_module_binary_with_single_function,
            build_module_binary_with_single_function_and_data_sections, BytecodeWriter,
        },
    };
    use ancvm_types::{ecallcode::ECallCode, opcode::Opcode, DataType, ForeignValue};

    use crate::{
        init_runtime, interpreter::process_function, thread::Thread, RUNTIME_CODE_NAME,
        RUNTIME_MAJOR_VERSION, RUNTIME_MINOR_VERSION, RUNTIME_PATCH_VERSION,
    };

    #[test]
    fn test_ecall_heap_capacity() {
        init_runtime();

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
            code0,
            vec![], // local varslist which
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = process_function(&mut thread0, 0, 0, &vec![]);

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
    fn test_ecall_runtime_info() {
        init_runtime();

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
            vec![], // params
            vec![DataType::I64], // results
            code0,
            vec![], // local varslist which
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = process_function(&mut thread0, 0, 0, &vec![]);

        let expect_version_number = RUNTIME_PATCH_VERSION as u64
            | (RUNTIME_MINOR_VERSION as u64) << 16
            | (RUNTIME_MAJOR_VERSION as u64) << 32;

        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::UInt64(expect_version_number)]
        );

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

        let code1 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::host_addr_data, 0, 0)
            .write_opcode_i32(Opcode::ecall, ECallCode::runtime_name as u32)
            .write_opcode_i16_i32(Opcode::data_load, 0, 0)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary1 = build_module_binary_with_single_function_and_data_sections(
            vec![],
            vec![],
            vec![UninitDataEntry::from_i64()],
            vec![], // params
            vec![DataType::I32, DataType::I64], // results
            code1,
            vec![], // local varslist which
        );

        let image1 = load_modules_binary(vec![&binary1]).unwrap();
        let mut thread1 = Thread::new(&image1);

        let result1 = process_function(&mut thread1, 0, 0, &vec![]);
        let fvs1 = result1.unwrap();
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
