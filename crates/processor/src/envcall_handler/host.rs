// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::thread_context::ThreadContext;

pub fn host_arch(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // `fn (module_index: i32, data_access_index: i64) -> i32`
    let content = std::env::consts::ARCH;
    do_host_information_str(thread_context, content);
}

pub fn host_os(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // `fn (module_index: i32, data_access_index: i64) -> i32`
    let content = std::env::consts::OS;
    do_host_information_str(thread_context, content);
}

pub fn host_family(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // `fn (module_index: i32, data_access_index: i64) -> i32`
    let content = std::env::consts::FAMILY;
    do_host_information_str(thread_context, content);
}

pub fn host_endian(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // `fn () -> i32`

    // ref:
    // https://doc.rust-lang.org/reference/conditional-compilation.html#target_endian

    let endian = if cfg!(target_endian = "little") {
        0
    } else if cfg!(target_endian = "big") {
        1
    } else {
        panic!("Unsupported host endian.")
    };

    thread_context.stack.push_i32_u(endian);
}

pub fn host_memory_width(/* _handler: &Handler, */ thread_context: &mut ThreadContext) {
    // `fn () -> i32`
    let size = size_of::<usize>();
    thread_context.stack.push_i32_u(size as u32);
}

fn do_host_information_str(thread_context: &mut ThreadContext, content: &str) {
    // `fn (module_index: i32, data_access_index: i64) -> i32`
    let data_access_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.stack.pop_i32_u();

    let content_bytes = content.as_bytes();
    let content_length = content_bytes.len();

    let target_data_object = thread_context.get_target_data_object(
        module_index as usize,
        data_access_index as usize,
        0,
        content_length,
    );

    let src_ptr = content_bytes.as_ptr();
    target_data_object
        .accessor
        .write_idx(src_ptr, data_access_index as usize, 0, content_length);

    thread_context.stack.push_i32_u(content_length as u32);
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use anc_context::program_source::ProgramSource;
    use anc_image::{
        bytecode_writer::BytecodeWriterHelper, entry::ReadWriteDataEntry,
        utils::helper_build_module_binary_with_single_function_and_data,
    };
    use anc_isa::{opcode::Opcode, OperandDataType};

    use crate::{
        envcall_num::EnvCallNum, in_memory_program_source::InMemoryProgramSource,
        process::process_function,
    };

    #[test]
    fn test_envcall_host_arch() {
        // ```code
        // fn test () -> i64
        //               ^
        //               |data pointer
        // ```

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::get_data, 0) // get module index and data access index of data 0
            .append_opcode_i32(Opcode::envcall, EnvCallNum::host_arch as u32)
            // get the data pointer
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        const BUFFER_LENGTH: usize = 16;

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[],                     // params
            &[OperandDataType::I64], // results
            &[],                     // local variables
            code0,
            &[],
            &[ReadWriteDataEntry::from_bytes(vec![0u8; BUFFER_LENGTH], 8)],
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
        let data_str = unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(data_ptr, BUFFER_LENGTH))
        };
        assert_eq!(data_str.trim_end_matches('\0'), std::env::consts::ARCH);
    }

    #[test]
    fn test_envcall_host_os() {
        // ```code
        // fn test () -> i64
        //               ^
        //               |data pointer
        // ```

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::get_data, 0) // get module index and data access index of data 0
            .append_opcode_i32(Opcode::envcall, EnvCallNum::host_os as u32)
            // get the data pointer
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        const BUFFER_LENGTH: usize = 16;

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[],                     // params
            &[OperandDataType::I64], // results
            &[],                     // local variables
            code0,
            &[],
            &[ReadWriteDataEntry::from_bytes(vec![0u8; BUFFER_LENGTH], 8)],
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
        let data_str = unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(data_ptr, BUFFER_LENGTH))
        };
        assert_eq!(data_str.trim_end_matches('\0'), std::env::consts::OS);
    }

    #[test]
    fn test_envcall_host_family() {
        // ```code
        // fn test () -> i64
        //               ^
        //               |data pointer
        // ```

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::get_data, 0) // get module index and data access index of data 0
            .append_opcode_i32(Opcode::envcall, EnvCallNum::host_family as u32)
            // get the data pointer
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        const BUFFER_LENGTH: usize = 16;

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[],                     // params
            &[OperandDataType::I64], // results
            &[],                     // local variables
            code0,
            &[],
            &[ReadWriteDataEntry::from_bytes(vec![0u8; BUFFER_LENGTH], 8)],
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
        let data_str = unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(data_ptr, BUFFER_LENGTH))
        };
        assert_eq!(data_str.trim_end_matches('\0'), std::env::consts::FAMILY);
    }

    #[test]
    fn test_envcall_host_endian() {
        // ```code
        // fn test () -> i32
        // ```

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::envcall, EnvCallNum::host_endian as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[],                     // params
            &[OperandDataType::I32], // results
            &[],                     // local variables
            code0,
            &[],
            &[],
            &[],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(/* &handler, */ &mut thread_context0, 0, 0, &[]);
        let fvs0 = result0.unwrap();

        let endian = if cfg!(target_endian = "little") {
            0
        } else if cfg!(target_endian = "big") {
            1
        } else {
            panic!("Unsupported host endian.")
        };

        assert_eq!(fvs0[0].as_u32(), endian);
    }

    #[test]
    fn test_envcall_host_memory_width() {
        // ```code
        // fn test () -> i32
        // ```

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::envcall, EnvCallNum::host_memory_width as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_data(
            &[],                     // params
            &[OperandDataType::I32], // results
            &[],                     // local variables
            code0,
            &[],
            &[],
            &[],
        );

        /* let handler = Handler::new(); */
        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(/* &handler, */ &mut thread_context0, 0, 0, &[]);
        let fvs0 = result0.unwrap();

        let size = size_of::<usize>();
        assert_eq!(fvs0[0].as_u32(), size as u32);
    }
}
