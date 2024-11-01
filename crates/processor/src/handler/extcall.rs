// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::path::PathBuf;

use crate::jit_util::build_vm_to_external_function;

use ancvm_context::thread_context::ThreadContext;
use ancvm_isa::{ExternalLibraryType, OPERAND_SIZE_IN_BYTES};

use super::HandleResult;

pub fn extcall(thread_context: &mut ThreadContext) -> HandleResult {
    // (operand external_function_index:i32) -> void/i32/i64/f32/f64
    //
    // the 'external_function_index' is the index within a specific module, it is not
    // the 'unified_external_function_index'.

    let external_function_index = thread_context.get_param_i32() as usize;
    let module_index = thread_context.pc.module_index;

    // get the unified external function index
    let (unified_external_function_index, type_index) = thread_context
        .module_index_instance
        .external_function_index_section
        .get_item_unified_external_function_index_and_type_index(
            module_index,
            external_function_index,
        );

    // get the data types of params and results of the external function
    let (param_datatypes, result_datatypes) = thread_context.module_common_instances
        [module_index]
        .type_section
        .get_item_params_and_results(type_index);

    let opt_func_pointer_and_wrapper = {
        let table = thread_context.external_function_table.lock().unwrap();
        table.get_external_function_pointer_and_wrapper_function(unified_external_function_index)
    };

    let func_pointer_and_wrapper = opt_func_pointer_and_wrapper.unwrap_or_else(|| {
        // get the name of the external function and
        // the index of the unified external library
        let (external_function_name, unified_external_library_index) = thread_context
            .module_index_instance
            .unified_external_function_section
            .get_item_name_and_unified_external_library_index(unified_external_function_index);

        // get the file path or name of the external library
        let (external_library_name, external_library_type) = thread_context
            .module_index_instance
            .unified_external_library_section
            .get_item_name_and_external_library_type(unified_external_library_index);

        let external_library_file_path_or_name = match external_library_type {
            ExternalLibraryType::User => {
                let mut path_buf = PathBuf::from(&thread_context.environment.source_path);
                if !thread_context.environment.is_directory {
                    path_buf.pop();
                }
                path_buf.push("lib");
                path_buf.push(external_library_name);
                path_buf.as_os_str().to_string_lossy().to_string()
            }
            ExternalLibraryType::Share => {
                let mut path_buf = PathBuf::from(&thread_context.environment.runtime_path);
                path_buf.push("lib");
                path_buf.push(external_library_name);
                path_buf.as_os_str().to_string_lossy().to_string()
            }
            ExternalLibraryType::System => external_library_name.to_owned(),
        };

        let mut table = thread_context.external_function_table.lock().unwrap();
        table
            .add_external_function(
                unified_external_function_index,
                unified_external_library_index,
                &external_library_file_path_or_name,
                external_function_name,
                param_datatypes,
                result_datatypes,
            )
            .unwrap()
    });

    // call the wrapper function:
    //
    // ```rust
    // type WrapperFunction = extern "C" fn(
    //     external_function_pointer: *const c_void,
    //     params_ptr: *const u8,
    //     results_ptr: *mut u8,
    // );
    // ```

    let params = thread_context.stack.pop_operands(param_datatypes.len());
    let mut results = [0u8; OPERAND_SIZE_IN_BYTES];

    (func_pointer_and_wrapper.1)(
        func_pointer_and_wrapper.0,
        params.as_ptr(),
        results.as_mut_ptr(),
    );

    // push the result on the stack
    // only one or zero result is allowed for 'C' function.
    if !result_datatypes.is_empty() {
        let dst = thread_context.stack.push_operand_from_memory();
        unsafe { std::ptr::copy(results.as_ptr(), dst, OPERAND_SIZE_IN_BYTES) };
    }

    HandleResult::Move(8)
}

impl ExtenalFunctionTable {

    pub fn add_external_function(
        &mut self,
        unified_external_function_index: usize,
        unified_external_library_index: usize,
        external_library_file_path_or_name: &str,
        external_function_name: &str,
        param_datatypes: &[OperandDataType],
        result_datatypes: &[OperandDataType],
    ) -> Result<(*mut c_void, WrapperFunction), &'static str> {
        if result_datatypes.len() > 1 {
            return Err("The specified function has more than 1 return value.");
        }

        // find library pointer
        let library_pointer = if let Some(unified_external_library_pointer_item) =
            &self.unified_external_library_pointer_list[unified_external_library_index]
        {
            unified_external_library_pointer_item.address as *mut c_void
        } else {
            self.add_external_library(
                unified_external_library_index,
                external_library_file_path_or_name,
            )?
        };

        let function_pointer = load_symbol(library_pointer, external_function_name)?;

        // find wrapper function index
        let wrapper_function_index = if let Some(wrapper_function_index) = self
            .wrapper_function_list
            .iter()
            .position(|wrapper_function_item| {
                wrapper_function_item.param_datatypes == param_datatypes
                    && wrapper_function_item.result_datatypes == result_datatypes
            }) {
            wrapper_function_index
        } else {
            self.add_wrapper_function(param_datatypes, result_datatypes)
        };

        // update external_function_pointer_list
        let unified_external_function_pointer_item = UnifiedExternalFunctionPointerItem {
            address: function_pointer as usize,
            wrapper_function_index,
        };

        self.unified_external_function_pointer_list[unified_external_function_index] =
            Some(unified_external_function_pointer_item);

        let wrapper_function = self.wrapper_function_list[wrapper_function_index].wrapper_function;

        Ok((function_pointer, wrapper_function))
    }

    fn add_external_library(
        &mut self,
        unified_external_library_index: usize,
        external_library_file_path_or_name: &str,
    ) -> Result<*mut c_void, &'static str> {
        let library_pointer = load_library(external_library_file_path_or_name)?;
        self.unified_external_library_pointer_list[unified_external_library_index] =
            Some(UnifiedExternalLibraryPointerItem {
                address: library_pointer as usize,
            });
        Ok(library_pointer)
    }

    fn add_wrapper_function(
        &mut self,
        param_types: &[OperandDataType],
        result_types: &[OperandDataType],
    ) -> usize {
        // build wrapper function
        let wrapper_function_index = self.wrapper_function_list.len();

        let wrapper_function_pointer =
            build_vm_to_external_function(wrapper_function_index, param_types, result_types);

        let wrapper_function_item = WrapperFunctionItem {
            param_datatypes: param_types.to_vec(),
            result_datatypes: result_types.to_vec(),
            wrapper_function: transmute_symbol_to::<WrapperFunction>(
                wrapper_function_pointer as *mut c_void,
            ),
        };

        self.wrapper_function_list.push(wrapper_function_item);

        wrapper_function_index
    }
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        bytecode_writer::BytecodeWriter,
        utils::{
            helper_build_module_binary_with_functions_and_external_functions,
            HelperExternalFunctionEntry, HelperFunctionWithCodeAndLocalVariablesEntry,
        },
    };
    use dyncall_util::cstr_pointer_to_str;
    use ancvm_context::{environment::ProgramSettings, program_resource::ProgramResource};
    use ancvm_isa::{
        entry::{InitedDataEntry, TypeEntry},
        opcode::Opcode,
        OperandDataType, ExternalLibraryType, ForeignValue,
    };

        use crate::{
        handler::Handler, in_memory_resource::InMemoryResource, process::process_function,
    };

    #[test]
    fn test_interpreter_extcall_with_system_libc_getuid() {
        let code0 = BytecodeWriterHelper::new()
            // .append_opcode_i32(Opcode::imm_i32, 0) // 0 is the external func index
            .append_opcode_i32(Opcode::extcall, 0) // 0 is the external func index
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        // `man 3 getuid`
        // 'uid_t getuid(void);'

        let binary0 = helper_build_module_binary_with_functions_and_external_functions(
            vec![
                TypeEntry {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                }, // getuid
                TypeEntry {
                    params: vec![],
                    results: vec![OperandDataType::I32],
                }, // main
            ], // types
            vec![HelperFunctionWithCodeAndLocalVariablesEntry {
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

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        let results0 = result0.unwrap();

        assert!(matches!(results0[0], ForeignValue::U32(uid) if uid > 0 ));
    }

    #[test]
    fn test_interpreter_extcall_with_system_libc_getenv() {
        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i32(Opcode::host_addr_data, 0, 0) // external func param 0
            //
            // .append_opcode_i32(Opcode::imm_i32, 0) // external func index
            .append_opcode_i32(Opcode::extcall, 0) // 0 is the external func index
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        // `man 3 getenv`
        // 'char *getenv(const char *name);'

        let binary0 = helper_build_module_binary_with_functions_and_external_functions(
            vec![
                TypeEntry {
                    params: vec![OperandDataType::I64],  // pointer
                    results: vec![OperandDataType::I64], // pointer
                }, // getenv
                TypeEntry {
                    params: vec![],
                    results: vec![OperandDataType::I64], // pointer
                }, // main
            ], // types
            vec![HelperFunctionWithCodeAndLocalVariablesEntry {
                type_index: 1,
                local_variable_item_entries_without_args: vec![],
                code: code0,
            }],
            vec![InitedDataEntry::from_bytes(b"PWD\0".to_vec(), 1)],
            vec![],
            vec![],
            vec![HelperExternalFunctionEntry {
                external_library_type: ExternalLibraryType::System,
                library_name: "libc.so.6".to_string(),
                function_name: "getenv".to_string(),
                type_index: 0,
            }],
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        let results0 = result0.unwrap();

        assert!(matches!(results0[0], ForeignValue::U64(addr) if {
            let pwd0 = cstr_pointer_to_str(addr as *const i8);
            !pwd0.to_string().is_empty()
        }));
    }

    #[test]
    fn test_interpreter_extcall_with_user_lib() {
        // (i32,i32) -> (i32)

        // 'libtest0.so.1'
        // 'int add(int, int)'

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0) // external func param 0
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1) // external func param 1
            //
            // .append_opcode_i32(Opcode::imm_i32, 0) // external func index
            .append_opcode_i32(Opcode::extcall, 0) // 0 is the external func index
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_external_functions(
            vec![
                TypeEntry {
                    params: vec![OperandDataType::I32, OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                }, // getenv
                TypeEntry {
                    params: vec![OperandDataType::I32, OperandDataType::I32],
                    results: vec![OperandDataType::I32],
                }, // main
            ], // types
            vec![HelperFunctionWithCodeAndLocalVariablesEntry {
                type_index: 1,
                local_variable_item_entries_without_args: vec![],
                code: code0,
            }],
            vec![],
            vec![],
            vec![],
            vec![HelperExternalFunctionEntry {
                external_library_type: ExternalLibraryType::User,
                library_name: "libtest0.so.1".to_string(),
                function_name: "add".to_string(),
                type_index: 0,
            }],
        );

        // it can not obtain the name of crate currently, the macro cfg
        // only supports several options:
        // https://doc.rust-lang.org/reference/conditional-compilation.html
        // https://doc.rust-lang.org/reference/attributes.html
        // https://doc.rust-lang.org/cargo/reference/environment-variables.html

        let mut pwd = std::env::current_dir().unwrap();
        let pkg_name = env!("CARGO_PKG_NAME");
        if !pwd.ends_with(pkg_name) {
            // in the VSCode editor `Debug` environment, the `current_dir()` returns
            // the project's root folder.
            // while in both `$ cargo test` and VSCode editor `Run Test` environment,
            // the `current_dir()` returns the current crate path.
            // here canonicalize the test resources path.
            pwd.push("crates");
            pwd.push(pkg_name);
        }
        pwd.push("tests");
        let program_source_path = pwd.to_str().unwrap();

        let program_resource0 = InMemoryProgramResource::with_settings(
            vec![binary0],
            &ProgramSettings::new(program_source_path, true, "", ""),
        );

        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(11), ForeignValue::U32(13)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(24)]);

        let result1 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U32(211), ForeignValue::U32(223)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::U32(434)]);
    }
}
