// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::thread_context::ThreadContext;
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
        bytecode_writer::BytecodeWriter, utils::helper_build_module_binary_with_single_function,
    };
    use ancvm_context::program_resource::ProgramResource;
    use ancvm_types::{
        entry::LocalVariableEntry, envcallcode::EnvCallCode, opcode::Opcode, DataType,
        ForeignValue, RUNTIME_CODE_NAME, RUNTIME_MAJOR_VERSION, RUNTIME_MINOR_VERSION,
        RUNTIME_PATCH_VERSION,
    };

    use crate::{in_memory_program_resource::InMemoryProgramResource, interpreter::process_function};

    #[test]
    fn test_envcall_runtime_version() {
        // () -> (i64)

        // bytecode:
        //
        // 0x0000  02 0b 00 00  01 01 00 00    envcall           idx:257
        // 0x0008  00 0a                       end

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::envcall, EnvCallCode::runtime_version as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", print_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],              // params
            vec![DataType::I64], // results
            vec![],              // local vars
            code0,
        );

        let program_resource0 = InMemoryProgramResource::new(vec![binary0]);
        let process_context0 = program_resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);

        let expect_version_number = RUNTIME_PATCH_VERSION as u64
            | (RUNTIME_MINOR_VERSION as u64) << 16
            | (RUNTIME_MAJOR_VERSION as u64) << 32;

        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::U64(expect_version_number)]
        );
    }

    #[test]
    fn test_envcall_runtime_code_name() {
        // () -> (i32, i64)
        //        ^    ^
        //        |    |name buffer (8 bytes)
        //        |name length

        // bytecode:
        //
        // 0x0000  04 0c 00 00  00 00 00 00    host.addr_local   rev:0   off:0x00  idx:0
        // 0x0008  02 0b 00 00  00 01 00 00    envcall           idx:256
        // 0x0010  00 02 00 00  00 00 00 00    local.load64_i64  rev:0   off:0x00  idx:0
        // 0x0018  00 0a                       end

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::host_addr_local, 0, 0, 0)
            .append_opcode_i32(Opcode::envcall, EnvCallCode::runtime_name as u32)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", print_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                                     // params
            vec![DataType::I32, DataType::I64],         // results
            vec![LocalVariableEntry::from_bytes(8, 8)], // local vars
            code0,
        );

        let program_resource0 = InMemoryProgramResource::new(vec![binary0]);
        let process_context0 = program_resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        let fvs1 = result0.unwrap();
        let name_len = fvs1[0].as_u32().unwrap();
        let name_u64 = fvs1[1].as_u64();

        let name_data = name_u64.unwrap().to_le_bytes();
        assert_eq!(&RUNTIME_CODE_NAME[..], &name_data[0..name_len as usize]);
    }
}
