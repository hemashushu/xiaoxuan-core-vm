// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_program::thread_context::ThreadContext;
use ancvm_types::{
    RUNTIME_CODE_NAME, RUNTIME_MAJOR_VERSION, RUNTIME_MINOR_VERSION, RUNTIME_PATCH_VERSION,
};

pub fn runtime_name(thread_context: &mut ThreadContext) {
    // `fn (buf_ptr: u64) -> name_len:u32`

    let buf_ptr_value = thread_context.stack.pop_i64_u();

    let name_len = RUNTIME_CODE_NAME.len();

    let src_ptr = RUNTIME_CODE_NAME.as_ptr();
    let dst_ptr = buf_ptr_value as *mut u8;
    unsafe {
        std::ptr::copy(src_ptr, dst_ptr, name_len);
    }

    thread_context.stack.push_i32_u(name_len as u32);
}

pub fn runtime_version(thread_context: &mut ThreadContext) {
    // `fn () -> version:u64`
    //
    // 0x0000_0000_0000_0000
    //        |    |    |
    //        |    |    |patch version
    //        |    |minor
    //        |major

    let version_number = RUNTIME_PATCH_VERSION as u64
        | (RUNTIME_MINOR_VERSION as u64) << 16
        | (RUNTIME_MAJOR_VERSION as u64) << 32;

    thread_context.stack.push_i64_u(version_number);
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
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{
        ecallcode::ECallCode, opcode::Opcode, DataType, ForeignValue, RUNTIME_CODE_NAME,
        RUNTIME_MAJOR_VERSION, RUNTIME_MINOR_VERSION, RUNTIME_PATCH_VERSION,
    };

    use crate::{in_memory_program_source::InMemoryProgramSource, interpreter::process_function};

    #[test]
    fn test_ecall_runtime_version() {
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

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

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
            vec![],                             // params
            vec![DataType::I32, DataType::I64], // results
            vec![],                             // local varslist which
            code0,
            vec![],
            vec![],
            vec![UninitDataEntry::from_i64()],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

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
