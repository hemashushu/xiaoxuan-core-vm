// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::sync::Mutex;

use ancvm_binary::load_modules_from_binaries;
use ancvm_program::{
    external_function::ExtenalFunctionTable, program::Program, program_settings::ProgramSettings,
    program_source::ProgramSource,
};

use crate::interpreter::init_interpreters;

pub struct InMemoryProgramSource {
    program_settings: ProgramSettings,
    module_binaries: Vec<Vec<u8>>,
    extenal_function_table: Mutex<ExtenalFunctionTable>,
}

impl InMemoryProgramSource {
    pub fn new(module_binaries: Vec<Vec<u8>>) -> Self {
        Self {
            module_binaries,
            program_settings: ProgramSettings::default(),
            extenal_function_table: Mutex::new(ExtenalFunctionTable::new()),
        }
    }

    pub fn with_settings(module_binaries: Vec<Vec<u8>>, program_settings: ProgramSettings) -> Self {
        Self {
            module_binaries,
            program_settings,
            extenal_function_table: Mutex::new(ExtenalFunctionTable::new()),
        }
    }
}

impl ProgramSource for InMemoryProgramSource {
    fn build_program(&self) -> Result<Program, ancvm_binary::BinaryError> {
        init_interpreters(); // initialize interpreters

        let binaries_ref = self
            .module_binaries
            .iter()
            .map(|e| &e[..])
            .collect::<Vec<_>>();

        let module_images = load_modules_from_binaries(binaries_ref)?;

        Ok(Program::new(
            &self.program_settings,
            &self.extenal_function_table,
            module_images,
        ))
    }
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        module_image::{
            data_section::{DataEntry, UninitDataEntry},
            local_variable_section::LocalVariableEntry,
        },
        utils::build_module_binary_with_single_function_and_data_sections,
    };
    use ancvm_program::{
        program_source::ProgramSource, resizeable_memory::ResizeableMemory,
        thread_context::ProgramCounter, INIT_HEAP_SIZE_IN_PAGES, INIT_STACK_SIZE_IN_PAGES,
    };
    use ancvm_types::DataType;

    use crate::in_memory_program_source::InMemoryProgramSource;

    #[test]
    fn test_in_memory_program() {
        let binary0 = build_module_binary_with_single_function_and_data_sections(
            vec![DataType::I32, DataType::I32],
            vec![DataType::I64],
            vec![LocalVariableEntry::from_i32()],
            vec![0u8],
            vec![DataEntry::from_i32(0x11), DataEntry::from_i64(0x13)],
            vec![DataEntry::from_bytes(
                vec![0x17u8, 0x19, 0x23, 0x29, 0x31, 0x37],
                8,
            )],
            vec![
                UninitDataEntry::from_i32(),
                UninitDataEntry::from_i64(),
                UninitDataEntry::from_i32(),
            ],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let thread_context0 = program0.create_thread_context();

        let program_ref = &thread_context0.program_context;

        // check index sections
        assert_eq!(program_ref.data_index_section.ranges.len(), 1);
        assert_eq!(program_ref.data_index_section.items.len(), 6);
        assert_eq!(program_ref.func_index_section.ranges.len(), 1);
        assert_eq!(program_ref.func_index_section.items.len(), 1);

        assert_eq!(program_ref.program_modules.len(), 1);
        let module = &program_ref.program_modules[0];

        // check data sections
        assert_eq!(module.datas.len(), 3);

        let mut data = [0u8; 8];
        let dst_ptr = &mut data as *mut [u8] as *mut u8;

        let ro_datas = &module.datas[0];
        ro_datas.load_idx_32(0, 0, dst_ptr);
        assert_eq!(&data[0..4], [0x11, 0, 0, 0]);
        ro_datas.load_idx_64(1, 0, dst_ptr);
        assert_eq!(data, [0x13, 0, 0, 0, 0, 0, 0, 0]);

        let rw_datas = &module.datas[1];
        rw_datas.load_idx_32(0, 0, dst_ptr);
        assert_eq!(&data[0..4], &[0x17u8, 0x19, 0x23, 0x29]);
        rw_datas.load_idx_32_extend_from_i16_u(0, 4, dst_ptr);
        assert_eq!(&data[0..2], &[0x31u8, 0x37]);

        let uninit_datas = &module.datas[2];
        uninit_datas.load_idx_32(0, 0, dst_ptr);
        assert_eq!(&data[0..4], &[0x0u8, 0, 0, 0]);
        uninit_datas.load_idx_64(1, 0, dst_ptr);
        assert_eq!(data, [0x0u8, 0, 0, 0, 0, 0, 0, 0]);
        uninit_datas.load_idx_32(2, 0, dst_ptr);
        assert_eq!(&data[0..4], &[0x0u8, 0, 0, 0]);

        // check type section
        assert_eq!(module.type_section.items.len(), 1);

        // check func section
        assert_eq!(module.func_section.items.len(), 1);

        // check local variable section
        assert_eq!(module.local_variable_section.lists.len(), 1);

        // check pc
        assert_eq!(
            thread_context0.pc,
            ProgramCounter {
                instruction_address: 0,
                function_internal_index: 0,
                module_index: 0
            }
        );

        // check stack
        assert_eq!(thread_context0.stack.fp, 0);
        assert_eq!(thread_context0.stack.sp, 0);
        assert_eq!(
            thread_context0.stack.get_capacity_in_pages(),
            INIT_STACK_SIZE_IN_PAGES
        );

        // check heap
        assert_eq!(
            thread_context0.heap.get_capacity_in_pages(),
            INIT_HEAP_SIZE_IN_PAGES
        );
    }
}
