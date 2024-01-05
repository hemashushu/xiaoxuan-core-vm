// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::thread_context::ThreadContext;

pub fn count_start_function(thread_context: &mut ThreadContext) {
    // `fn () -> i32`
    let start_function_count = thread_context
        .module_index_instance
        .start_function_list_section
        .items
        .len();
    thread_context.stack.push_i32_u(start_function_count as u32);
}

pub fn count_exit_function(thread_context: &mut ThreadContext) {
    // `fn () -> i32`
    let exit_function_count = thread_context
        .module_index_instance
        .exit_function_list_section
        .items
        .len();
    thread_context.stack.push_i32_u(exit_function_count as u32);
}

pub fn get_start_function_item(thread_context: &mut ThreadContext) {
    // `fn (index:i32) -> i32`
    // panic if out of index.
    let idx = thread_context.stack.pop_i32_u();
    let function_public_index = thread_context
        .module_index_instance
        .start_function_list_section
        .items[idx as usize];
    thread_context.stack.push_i32_u(function_public_index);
}

pub fn get_exit_function_item(thread_context: &mut ThreadContext) {
    // `fn (index:i32) -> i32`
    // panic if out of index.
    let idx = thread_context.stack.pop_i32_u();
    let function_public_index = thread_context
        .module_index_instance
        .exit_function_list_section
        .items[idx as usize];
    thread_context.stack.push_i32_u(function_public_index);
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        bytecode_writer::BytecodeWriter,
        utils::{
            helper_build_module_binary_with_functions_and_blocks_and_entry_and_start_and_exit_functions,
            HelperFunctionWithCodeAndSignatureAndLocalVariablesEntry,
        },
    };
    use ancvm_context::program_resource::ProgramResource;
    use ancvm_types::{envcallcode::EnvCallCode, opcode::Opcode, DataType, ForeignValue};

    use crate::{
        in_memory_program_resource::InMemoryProgramResource, interpreter::process_function,
    };

    #[test]
    fn test_envcall_count_start_function() {
        // () -> (i32)

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::envcall, EnvCallCode::count_start_function as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_blocks_and_entry_and_start_and_exit_functions(
            vec![HelperFunctionWithCodeAndSignatureAndLocalVariablesEntry{
                params: vec![],
                results: vec![DataType::I32],
                local_variable_item_entries_without_args: vec![],
                code: code0
            }],
            vec![],
            vec![11,13,17,19],
            vec![23,27],
            0
        );

        let program_resource0 = InMemoryProgramResource::new(vec![binary0]);
        let process_context0 = program_resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(4)]);
    }

    #[test]
    fn test_envcall_count_exit_function() {
        // () -> (i32)

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::envcall, EnvCallCode::count_exit_function as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_blocks_and_entry_and_start_and_exit_functions(
            vec![HelperFunctionWithCodeAndSignatureAndLocalVariablesEntry{
                params: vec![],
                results: vec![DataType::I32],
                local_variable_item_entries_without_args: vec![],
                code: code0
            }],
            vec![],
            vec![11,13,17,19],
            vec![23,27],
            0
        );

        let program_resource0 = InMemoryProgramResource::new(vec![binary0]);
        let process_context0 = program_resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::U32(2)]);
    }

    #[test]
    fn test_envcall_get_start_function_item() {
        // () -> (i32,i32,i32,i32)

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 1)
            .append_opcode_i32(Opcode::envcall, EnvCallCode::get_start_function_item as u32)
            .append_opcode_i32(Opcode::i32_imm, 0)
            .append_opcode_i32(Opcode::envcall, EnvCallCode::get_start_function_item as u32)
            .append_opcode_i32(Opcode::i32_imm, 3)
            .append_opcode_i32(Opcode::envcall, EnvCallCode::get_start_function_item as u32)
            .append_opcode_i32(Opcode::i32_imm, 2)
            .append_opcode_i32(Opcode::envcall, EnvCallCode::get_start_function_item as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_blocks_and_entry_and_start_and_exit_functions(
            vec![HelperFunctionWithCodeAndSignatureAndLocalVariablesEntry{
                params: vec![],
                results: vec![DataType::I32,DataType::I32,DataType::I32,DataType::I32],
                local_variable_item_entries_without_args: vec![],
                code: code0
            }],
            vec![],
            vec![11,13,17,19],
            vec![23,27],
            0
        );

        let program_resource0 = InMemoryProgramResource::new(vec![binary0]);
        let process_context0 = program_resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::U32(13),
                ForeignValue::U32(11),
                ForeignValue::U32(19),
                ForeignValue::U32(17),
            ]
        );
    }

    #[test]
    fn test_envcall_get_exit_function_item() {
        // () -> (i32,i32,i32,i32)

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 1)
            .append_opcode_i32(Opcode::envcall, EnvCallCode::get_exit_function_item as u32)
            .append_opcode_i32(Opcode::i32_imm, 0)
            .append_opcode_i32(Opcode::envcall, EnvCallCode::get_exit_function_item as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_functions_and_blocks_and_entry_and_start_and_exit_functions(
            vec![HelperFunctionWithCodeAndSignatureAndLocalVariablesEntry{
                params: vec![],
                results: vec![DataType::I32,DataType::I32,],
                local_variable_item_entries_without_args: vec![],
                code: code0
            }],
            vec![],
            vec![11,13,17,19],
            vec![23,27],
            0
        );

        let program_resource0 = InMemoryProgramResource::new(vec![binary0]);
        let process_context0 = program_resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::U32(27), ForeignValue::U32(23),]
        );
    }
}
