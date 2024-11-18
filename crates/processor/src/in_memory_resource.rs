// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::sync::Mutex;

use anc_context::{
    environment::Environment, external_function_table::ExternalFunctionTable,
    process_context::ProcessContext, resource::Resource,
};
use anc_image::{utils::helper_load_modules_from_binaries, BinaryError};

/// An implement of 'Resource' for unit testing
pub struct InMemoryResource {
    environment: Environment,
    module_binaries: Vec<Vec<u8>>,
    external_function_table: Mutex<ExternalFunctionTable>,
}

impl InMemoryResource {
    #[allow(dead_code)]
    pub fn new(module_binaries: Vec<Vec<u8>>) -> Self {
        Self {
            module_binaries,
            environment: Environment::default(),
            external_function_table: Mutex::new(ExternalFunctionTable::default()),
        }
    }

    #[allow(dead_code)]
    pub fn with_environment(module_binaries: Vec<Vec<u8>>, environment: &Environment) -> Self {
        Self {
            module_binaries,
            environment: environment.clone(),
            external_function_table: Mutex::new(ExternalFunctionTable::default()),
        }
    }
}

impl Resource for InMemoryResource {
    fn create_process_context(&self) -> Result<ProcessContext, BinaryError> {
        let binaries_ref = self
            .module_binaries
            .iter()
            .map(|e| &e[..])
            .collect::<Vec<_>>();

        let module_images = helper_load_modules_from_binaries(binaries_ref)?;

        Ok(ProcessContext::new(
            &self.environment,
            &self.external_function_table,
            module_images,
        ))
    }

    // fn get_type(&self) -> ResourceType {
    //     ResourceType::InMemory
    // }
}

#[cfg(test)]
mod tests {
    use anc_context::{
        resizeable_memory::ResizeableMemory, resource::Resource, thread_context::ProgramCounter,
        INIT_HEAP_SIZE_IN_PAGES,
    };
    use anc_image::{
        entry::{InitedDataEntry, LocalVariableEntry, UninitDataEntry},
        utils::helper_build_module_binary_with_single_function_and_data_sections,
    };
    use anc_isa::OperandDataType;

    use crate::in_memory_resource::InMemoryResource;

    #[test]
    fn test_in_memory_program() {
        let binary0 = helper_build_module_binary_with_single_function_and_data_sections(
            vec![OperandDataType::I32, OperandDataType::I32],
            vec![OperandDataType::I64],
            vec![LocalVariableEntry::from_i32()],
            vec![0u8],
            vec![
                InitedDataEntry::from_i32(0x11),
                InitedDataEntry::from_i64(0x13),
            ],
            vec![InitedDataEntry::from_raw(
                vec![0x17u8, 0x19, 0x23, 0x29, 0x31, 0x37],
                8,
            )],
            vec![
                UninitDataEntry::from_i32(),
                UninitDataEntry::from_i64(),
                UninitDataEntry::from_i32(),
            ],
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let thread_context0 = process_context0.create_thread_context();

        let module_index_instance = &thread_context0.module_index_instance;

        // check index sections
        assert_eq!(module_index_instance.data_index_section.ranges.len(), 1);
        assert_eq!(module_index_instance.data_index_section.items.len(), 6);
        assert_eq!(module_index_instance.function_index_section.ranges.len(), 1);
        assert_eq!(module_index_instance.function_index_section.items.len(), 1);

        let module_common_instances = &thread_context0.module_common_instances;

        assert_eq!(module_common_instances.len(), 1);
        let module = &module_common_instances[0];

        // check data sections
        assert_eq!(module.datas.len(), 3);

        let mut data = [0u8; 8];
        let dst_ptr = &mut data as *mut [u8] as *mut u8;

        let ro_datas = &module.datas[0];
        ro_datas.load_idx_i32_u(0, 0, dst_ptr);
        assert_eq!(&data[0..4], [0x11, 0, 0, 0]);
        ro_datas.load_idx_i64(1, 0, dst_ptr);
        assert_eq!(data, [0x13, 0, 0, 0, 0, 0, 0, 0]);

        let rw_datas = &module.datas[1];
        rw_datas.load_idx_i32_u(0, 0, dst_ptr);
        assert_eq!(&data[0..4], &[0x17u8, 0x19, 0x23, 0x29]);
        rw_datas.load_idx_i16_u(0, 4, dst_ptr);
        assert_eq!(&data[0..2], &[0x31u8, 0x37]);

        let uninit_datas = &module.datas[2];
        uninit_datas.load_idx_i32_u(0, 0, dst_ptr);
        assert_eq!(&data[0..4], &[0x0u8, 0, 0, 0]);
        uninit_datas.load_idx_i64(1, 0, dst_ptr);
        assert_eq!(data, [0x0u8, 0, 0, 0, 0, 0, 0, 0]);
        uninit_datas.load_idx_i32_u(2, 0, dst_ptr);
        assert_eq!(&data[0..4], &[0x0u8, 0, 0, 0]);

        // check type section
        assert_eq!(module.type_section.items.len(), 1);

        // check function section
        assert_eq!(module.function_section.items.len(), 1);

        // check local variable section
        assert_eq!(module.local_variable_section.list_items.len(), 1);

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

        // check heap
        assert_eq!(
            thread_context0.heap.get_capacity_in_pages(),
            INIT_HEAP_SIZE_IN_PAGES
        );
    }
}
