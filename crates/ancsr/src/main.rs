// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// a script application may consist of a single script file, or several script files.
// in either case, these script files will be compiled into a single module image file
// named 'main.ancbc', and this file will be copied to the 'application cache' directory.
//
// the dependent modules of application are copied to this cache directory also, but
// the shared modules (such as the standard library) are located at the runtime directory, and
// they will not be copied to this directory.
//
// the structure of application cache directory
//
// app cache dir
//   |-- cache.index
//   |   the cache infomations, such as the file type, version (dependencies only),
//   |   and the last modified time, content hash (only the application script files), and the module list
//   |
//   |-- main_module.ancbc
//   |-- dependency_module1.ancbc
//   |-- ...
//   |-- dependency_moduleN.ancbc
//
// all module files and the cache dir itself are named with:
// - `dev_num_major + '_' + dev_num_minor + '_' + inode_num`, or
// - the sha256 of the full path.
//
// note that apart from the script files, a (multiple scripts) application may also contains resource files
// and (dynamically linked) shared libraries.
// these files will stay in their original location and will not be copied to the cache directory.
//
// app source file dir
//   |-- module.anon
//   |   the application description file, similar to the Nodejs's package.json
//   |   and the Rust's Cargo.toml
//   |
//   |-- main.ancs
//   |   the main module script file, the first line would be '#!/bin/ancs -d' (ref: https://en.wikipedia.org/wiki/Shebang_(Unix))
//   |
//   |-- sub-module.ancs
//   |-- sub-dir
//   |     |-- sub-module.ancs
//   |     |-- ...
//   |
//   |-- resources
//   |     |-- images_etc
//   |     |-- ...
//   |
//   |-- lib
//   |     |-- shared-library.so -> shared-library.so.1.0.0
//   |     |-- ...

// to launch a script application which is a single file:
// - `$ ancs /path/to/single-file-app.ancs`
// - `$ /path/to/single-file-app.ancs`
// - `$ single-file-app.ancs` (which path is in the environment variable PATH)
// - `$ [/usr/bin/]single-file-app` (a symbolic link to '/path/to/single-file-app.ancs')
//
// to lanuch a script application which is consist of multiple script files:
// - `$ ancs /path/to/app-source-dir`
// - `$ ancs /path/to/app-source-dir/main.ancs`
// - `$ /path/to/app-source-dir/main.ancs`
// - `$ [/usr/bin/]app-name` (a symblic link to '/path/to/app-source-dir/main.ancs')

use ancvm_binary::{
    module_image::type_section::TypeEntry,
    utils::{
        build_module_binary_with_functions_and_external_functions, BytecodeWriter,
        HelperExternalFunctionEntry, HelperSlimFunctionEntry,
    },
};
use ancvm_program::program_source::ProgramSource;
use ancvm_runtime::{
    in_memory_program_source::InMemoryProgramSource, interpreter::process_function,
};
use ancvm_types::{
    ecallcode::ECallCode, opcode::Opcode, DataType, ExternalLibraryType, ForeignValue,
};

fn main() {
    print_uid()
}

fn print_uid() {
    let code0 = BytecodeWriter::new()
        .write_opcode_i32(Opcode::i32_imm, 0) // external func index
        .write_opcode_i32(Opcode::ecall, ECallCode::extcall as u32) // call external function
        //
        .write_opcode(Opcode::end)
        .to_bytes();

    // `man 3 getuid`
    // 'uid_t getuid(void);'

    let binary0 = build_module_binary_with_functions_and_external_functions(
        vec![
            TypeEntry {
                params: vec![],
                results: vec![DataType::I32],
            }, // getuid
            TypeEntry {
                params: vec![],
                results: vec![DataType::I32],
            }, // main
        ], // types
        vec![HelperSlimFunctionEntry {
            type_index: 1,
            local_variable_item_entries_without_args: vec![],
            code: code0,
        }],
        vec![],
        vec![],
        vec![],
        vec![HelperExternalFunctionEntry {
            external_library_type: ExternalLibraryType::System,
            library_name: "libc.so.6".to_string(),
            function_name: "getuid".to_string(),
            type_index: 0,
        }],
    );

    let program_source0 = InMemoryProgramSource::new(vec![binary0]);
    let program0 = program_source0.build_program().unwrap();
    let mut thread_context0 = program0.new_thread_context();

    let result0 = process_function(&mut thread_context0, 0, 0, &vec![]);
    let results0 = result0.unwrap();

    if let ForeignValue::UInt32(uid) = results0[0] {
        println!("uid: {}", uid);
    } else {
        println!("getuid failed.")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_test() {
        //
    }
}
