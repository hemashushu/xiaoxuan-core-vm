// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;
use anc_isa::RUNTIME_EDITION;

pub fn runtime_edition(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // `fn (module_index: i32, data_access_index: i64)`

    const CONTENT_LENGTH_IN_BYTES: usize = RUNTIME_EDITION.len();

    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();

    let target_data_object = thread_context.get_target_data_object(
        module_index as usize,
        data_access_index as usize,
        0,
        CONTENT_LENGTH_IN_BYTES,
    );

    let src_ptr = RUNTIME_EDITION.as_ptr();

    target_data_object.accessor.write_idx(
        src_ptr,
        data_access_index as usize,
        0,
        CONTENT_LENGTH_IN_BYTES,
    );
}

pub fn runtime_version(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // `fn () -> i64`
    //
    // 0x0000_0000_0000_0000
    //        |    |    |
    //        |    |    |patch version
    //        |    |minor
    //        |major

    // Cargo environment variables provide the version information of the package.
    // - `CARGO_PKG_VERSION_MAJOR`: The major version of your package.
    // - `CARGO_PKG_VERSION_MINOR`: The minor version of your package.
    // - `CARGO_PKG_VERSION_PATCH`: The patch version of your package.
    //
    // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
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
        bytecode_writer::BytecodeWriterHelper,
        entry::ReadWriteDataEntry,
        utils::{
            helper_build_module_binary_with_single_function,
            helper_build_module_binary_with_single_function_and_data,
        },
    };
    use anc_isa::{opcode::Opcode, ForeignValue, OperandDataType, RUNTIME_EDITION};

    use crate::{
        envcall_num::EnvCallNum, in_memory_program_source::InMemoryProgramSource,
        process::process_function,
    };

    #[test]
    fn test_envcall_runtime_edition() {
        // ```code
        // fn test () -> i64
        //               ^
        //               |data pointer
        // ```

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::get_data, 0) // get module index and data access index of data 0
            .append_opcode_i32(Opcode::envcall, EnvCallNum::runtime_edition as u32)
            // get the data pointer
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[],                     // params
            &[OperandDataType::I64], // results
            &[],                     // local variables
            code0,
            &[],
            &[ReadWriteDataEntry::from_bytes(vec![0u8; 8], 8)],
            &[],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(/* &handler, */ &mut thread_context0, 0, 0, &[]);
        let fvs0 = result0.unwrap();
        let data_ptr_value = fvs0[0].as_u64();
        let data_ptr = data_ptr_value as *const u8;

        let mut data = vec![0u8; 8];
        unsafe {
            std::ptr::copy(data_ptr, data.as_mut_ptr(), 8);
        }

        assert_eq!(RUNTIME_EDITION, &data[..RUNTIME_EDITION.len()]);
    }

    #[test]
    fn test_envcall_runtime_version() {
        // `fn test () -> i64`

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

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(/* &handler, */ &mut thread_context0, 0, 0, &[]);

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
}
