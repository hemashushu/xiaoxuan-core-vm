// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use crate::handler::Handler;
use anc_context::thread_context::ThreadContext;
use anc_isa::RUNTIME_EDITION;

pub fn runtime_edition(_handler: &Handler, thread_context: &mut ThreadContext) {
    // `fn (buf_ptr: u64) -> len:u32`

    let buf_ptr_value = thread_context.stack.pop_i64_u();

    let len = if let Some(len) = RUNTIME_EDITION.iter().position(|c| *c == 0) {
        len
    } else {
        RUNTIME_EDITION.len()
    };

    let src_ptr = RUNTIME_EDITION.as_ptr();
    let dst_ptr = buf_ptr_value as *mut u8;
    unsafe {
        std::ptr::copy(src_ptr, dst_ptr, len);
    }

    thread_context.stack.push_i32_u(len as u32);
}

pub fn runtime_version(_handler: &Handler, thread_context: &mut ThreadContext) {
    // `fn () -> version:u64`
    //
    // 0x0000_0000_0000_0000
    //        |    |    |
    //        |    |    |patch version
    //        |    |minor
    //        |major

    // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
    // CARGO_PKG_VERSION_MAJOR — The major version of your package.
    // CARGO_PKG_VERSION_MINOR — The minor version of your package.
    // CARGO_PKG_VERSION_PATCH — The patch version of your package.
    let version_patch = env!("CARGO_PKG_VERSION_PATCH").parse::<u16>().unwrap();
    let version_minor = env!("CARGO_PKG_VERSION_MINOR").parse::<u16>().unwrap();
    let version_major = env!("CARGO_PKG_VERSION_MAJOR").parse::<u16>().unwrap();

    let version_number =
        version_patch as u64 | (version_minor as u64) << 16 | (version_major as u64) << 32;

    thread_context.stack.push_i64_u(version_number);
}

#[cfg(test)]
mod tests {

    use anc_context::program_source::ProgramSource;
    use anc_image::{
        bytecode_reader::format_bytecode_as_text, bytecode_writer::BytecodeWriterHelper,
         utils::helper_build_module_binary_with_single_function,
    };
    use anc_isa::{opcode::Opcode, ForeignValue, OperandDataType, RUNTIME_EDITION};

    use crate::{
        envcall_num::EnvCallNum, handler::Handler, in_memory_program_source::InMemoryProgramSource,
        process::process_function,
    };

    #[test]
    fn test_envcall_runtime_version() {
        // () -> (i64)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::envcall, EnvCallNum::runtime_version as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[],                     // params
            &[OperandDataType::I64], // results
            &[],                     // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);

        let version_patch = env!("CARGO_PKG_VERSION_PATCH").parse::<u16>().unwrap();
        let version_minor = env!("CARGO_PKG_VERSION_MINOR").parse::<u16>().unwrap();
        let version_major = env!("CARGO_PKG_VERSION_MAJOR").parse::<u16>().unwrap();

        let expect_version_number =
            version_patch as u64 | (version_minor as u64) << 16 | (version_major as u64) << 32;

        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::U64(expect_version_number)]
        );
    }

    #[test]
    fn test_envcall_runtime_edition() {
        // () -> (i32, i64)
        //        ^    ^
        //        |    |name buffer (8 bytes)
        //        |name length

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::host_addr_local, 0, 0, 0)
            .append_opcode_i32(Opcode::envcall, EnvCallNum::runtime_edition as u32)
            .append_opcode_i16_i32(Opcode::local_load_i64, 0, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[],                                           // params
            &[OperandDataType::I32, OperandDataType::I64], // results
            &[LocalVariableEntry::from_bytes(8, 8)],       // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
        let fvs1 = result0.unwrap();
        let name_len = fvs1[0].as_u32();
        let name_u64 = fvs1[1].as_u64();

        let name_data = name_u64.to_le_bytes();
        assert_eq!(RUNTIME_EDITION, &name_data);
        assert_eq!(
            RUNTIME_EDITION.iter().position(|c| *c == 0).unwrap(),
            name_len as usize
        );
    }
}
