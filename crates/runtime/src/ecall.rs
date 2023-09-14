// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_types::ecallcode::{ECallCode, MAX_ECALLCODE_NUMBER};

use crate::{processor::InterpretResult, thread::Thread};

pub mod heap;
pub mod info;

type EnvCallHandlerFunc = fn(&mut Thread) -> Result<(), usize>;

fn unreachable(_thread: &mut Thread) -> Result<(), usize> {
    unreachable!("Invalid environment call number.")
}

pub static mut HANDLERS: Vec<EnvCallHandlerFunc> = vec![];

pub fn init_ecall_handlers() {
    let placehold: Vec<EnvCallHandlerFunc> = vec![unreachable; MAX_ECALLCODE_NUMBER];

    unsafe {
        HANDLERS.extend(placehold.iter());

        //
        HANDLERS[ECallCode::runtime_name as usize] = info::runtime_name;
        HANDLERS[ECallCode::runtime_version as usize] = info::runtime_version;
    }
}

pub fn ecall(thread: &mut Thread) -> InterpretResult {
    // (param env_func_num:i32)

    let env_func_num = thread.get_param_i32();
    let func = unsafe { HANDLERS[env_func_num as usize] };
    let result = func(thread);

    match result {
        Ok(_) => InterpretResult::MoveOn(8),
        Err(err_code) => InterpretResult::EnvError(err_code),
    }
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{load_modules_binary, module_image::data_section::UninitDataEntry};
    use ancvm_types::{ecallcode::ECallCode, opcode::Opcode, DataType, ForeignValue};

    use crate::{
        processor::Processor,
        thread::Thread,
        utils::{
            test_helper::{
                build_module_binary_with_single_function,
                build_module_binary_with_single_function_and_data_sections,
            },
            BytecodeReader, BytecodeWriter,
        },
        RUNTIME_CODE_NAME, RUNTIME_MAJOR_VERSION, RUNTIME_MINOR_VERSION, RUNTIME_PATCH_VERSION,
    };

    #[test]
    fn test_ecall_runtime_info() {
        let processor = Processor::new();

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

        // let text = BytecodeReader::new(&code0).to_text();
        // println!("{}", text);

        let binary0 = build_module_binary_with_single_function(
            vec![],              // params
            vec![DataType::I64], // results
            code0,
            vec![], // local varslist which
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = processor.process_function(&mut thread0, 0, 0, &vec![]);

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

        let text = BytecodeReader::new(&code1).to_text();
        println!("{}", text);

        let binary1 = build_module_binary_with_single_function_and_data_sections(
            vec![],
            vec![],
            vec![UninitDataEntry::from_i64()],
            vec![],                             // params
            vec![DataType::I32, DataType::I64], // results
            code1,
            vec![], // local varslist which
        );

        let image1 = load_modules_binary(vec![&binary1]).unwrap();
        let mut thread1 = Thread::new(&image1);

        let result1 = processor.process_function(&mut thread1, 0, 0, &vec![]);
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
