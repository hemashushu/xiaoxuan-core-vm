// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::{
    process_context::ProcessContext, process_property::ProcessProperty,
    program_source::ProgramSource,
};
use anc_image::{utils::helper_load_modules_from_binaries, ImageError};

/// An implement of 'ProgramSource' for unit testing only
pub struct InMemoryProgramSource {
    program_proerty: ProcessProperty,
    module_binaries: Vec<Vec<u8>>,
}

impl InMemoryProgramSource {
    #[allow(dead_code)]
    pub fn new(module_binaries: Vec<Vec<u8>>) -> Self {
        Self {
            program_proerty: ProcessProperty::default(),
            module_binaries,
        }
    }

    #[allow(dead_code)]
    pub fn with_property(module_binaries: Vec<Vec<u8>>, program_proerty: ProcessProperty) -> Self {
        Self {
            module_binaries,
            program_proerty,
        }
    }
}

impl ProgramSource for InMemoryProgramSource {
    fn create_process_context(&self) -> Result<ProcessContext, ImageError> {
        let binaries_ref = self
            .module_binaries
            .iter()
            .map(|e| &e[..])
            .collect::<Vec<_>>();

        let module_images = helper_load_modules_from_binaries(&binaries_ref)?;

        Ok(ProcessContext::new(&self.program_proerty, module_images))
    }
}

// #[cfg(test)]
// mod tests {
//
//     use anc_image::{
//         entry::{LocalVariableEntry, ReadOnlyDataEntry, ReadWriteDataEntry, UninitDataEntry},
//         utils::helper_build_module_binary_with_single_function_and_data,
//     };
//     use anc_isa::OperandDataType;
//
//     use crate::in_memory_program_source::InMemoryProgramSource;
//
//     #[test]
//     fn test_in_memory_module_instance() {
//         let binary0 = helper_build_module_binary_with_single_function_and_data(
//             &[OperandDataType::I32, OperandDataType::I32],
//             &[OperandDataType::I64],
//             &[LocalVariableEntry::from_i32()],
//             vec![0u8],
//             &[
//                 ReadOnlyDataEntry::from_i32(0x11),
//                 ReadOnlyDataEntry::from_i64(0x13),
//             ],
//             &[ReadWriteDataEntry::from_bytes(
//                 vec![0x17u8, 0x19, 0x23, 0x29, 0x31, 0x37],
//                 8,
//             )],
//             &[
//                 UninitDataEntry::from_i32(),
//                 UninitDataEntry::from_i64(),
//                 UninitDataEntry::from_i32(),
//             ],
//         );
//
//         let resource0 = InMemoryProgramSource::new(vec![binary0]);
//         let process_context0 = resource0.create_process_context().unwrap();
//         let thread_context0 = process_context0.create_thread_context();
//
//         let module_linking_instance = &thread_context0.module_linking_instance;
//
//         // check index sections
//         assert_eq!(module_linking_instance.data_index_section.ranges.len(), 1);
//         assert_eq!(module_linking_instance.data_index_section.items.len(), 6);
//         assert_eq!(
//             module_linking_instance.function_index_section.ranges.len(),
//             1
//         );
//         assert_eq!(
//             module_linking_instance.function_index_section.items.len(),
//             1
//         );
//
//         let module_common_instances = &thread_context0.module_common_instances;
//         assert_eq!(module_common_instances.len(), 1);
//
//         let module_common_instance0 = &module_common_instances[0];
//
//         // check data sections
//         assert_eq!(module_common_instance0.datas.len(), 3);
//
//         let mut data = [0u8; 8];
//         let dst_ptr = &mut data as *mut [u8] as *mut u8;
//
//         let ro_datas = &module_common_instance0.datas[0];
//         ro_datas.load_idx_i32_u(0, 0, dst_ptr);
//         assert_eq!(&data[0..4], [0x11, 0, 0, 0]);
//         ro_datas.load_idx_i64(1, 0, dst_ptr);
//         assert_eq!(data, [0x13, 0, 0, 0, 0, 0, 0, 0]);
//
//         let rw_datas = &module_common_instance0.datas[1];
//         rw_datas.load_idx_i32_u(0, 0, dst_ptr);
//         assert_eq!(&data[0..4], &[0x17u8, 0x19, 0x23, 0x29]);
//         rw_datas.load_idx_i16_u(0, 4, dst_ptr);
//         assert_eq!(&data[0..2], &[0x31u8, 0x37]);
//
//         let uninit_datas = &module_common_instance0.datas[2];
//         uninit_datas.load_idx_i32_u(0, 0, dst_ptr);
//         assert_eq!(&data[0..4], &[0x0u8, 0, 0, 0]);
//         uninit_datas.load_idx_i64(1, 0, dst_ptr);
//         assert_eq!(data, [0x0u8, 0, 0, 0, 0, 0, 0, 0]);
//         uninit_datas.load_idx_i32_u(2, 0, dst_ptr);
//         assert_eq!(&data[0..4], &[0x0u8, 0, 0, 0]);
//
//         // check type section
//         assert_eq!(module_common_instance0.type_section.items.len(), 1);
//
//         // check function section
//         assert_eq!(module_common_instance0.function_section.items.len(), 1);
//
//         // check local variable section
//         assert_eq!(
//             module_common_instance0.local_variable_section.lists.len(),
//             1
//         );
//
//         // check pc
//         assert_eq!(
//             thread_context0.pc,
//             ProgramCounter {
//                 instruction_address: 0,
//                 function_internal_index: 0,
//                 module_index: 0
//             }
//         );
//
//         // check stack
//         assert_eq!(thread_context0.stack.fp, 0);
//         assert_eq!(thread_context0.stack.sp, 0);
//
//         // check heap
//         assert_eq!(
//             thread_context0.memory.get_capacity_in_pages(),
//             INIT_MEMORY_SIZE_IN_PAGES
//         );
//     }
// }
