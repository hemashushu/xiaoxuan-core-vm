// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use anc_context::thread_context::ThreadContext;
use anc_isa::{
    RUNTIME_CODE_NAME, RUNTIME_MAJOR_VERSION, RUNTIME_MINOR_VERSION, RUNTIME_PATCH_VERSION,
};

use crate::handler::Handler;

pub fn runtime_name(_handler: &Handler, thread_context: &mut ThreadContext) {
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

pub fn runtime_version(_handler: &Handler, thread_context: &mut ThreadContext) {
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
    use anc_context::resource::Resource;
    use anc_image::{
        bytecode_reader::format_bytecode_as_text, bytecode_writer::BytecodeWriterHelper,
        entry::LocalVariableEntry, utils::helper_build_module_binary_with_single_function,
    };
    use anc_isa::{
        opcode::Opcode, ForeignValue, OperandDataType, RUNTIME_CODE_NAME, RUNTIME_MAJOR_VERSION,
        RUNTIME_MINOR_VERSION, RUNTIME_PATCH_VERSION,
    };

    use crate::{
        envcall_num::EnvCallNum, handler::Handler, in_memory_resource::InMemoryResource,
        process::process_function,
    };

    #[test]
    fn test_envcall_runtime_version() {
        // () -> (i64)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::envcall, EnvCallNum::runtime_version as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        println!("{}", format_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                     // params
            vec![OperandDataType::I64], // results
            vec![],                     // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);

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

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i16_i16(Opcode::host_addr_local, 0, 0, 0)
            .append_opcode_i32(Opcode::envcall, EnvCallNum::runtime_name as u32)
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        println!("{}", format_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                                           // params
            vec![OperandDataType::I32, OperandDataType::I64], // results
            vec![LocalVariableEntry::from_bytes(8, 8)],         // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        let fvs1 = result0.unwrap();
        let name_len = fvs1[0].as_u32();
        let name_u64 = fvs1[1].as_u64();

        let name_data = name_u64.to_le_bytes();
        assert_eq!(&RUNTIME_CODE_NAME[..], &name_data[0..name_len as usize]);
    }
}
