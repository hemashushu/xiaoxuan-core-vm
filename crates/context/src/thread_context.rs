// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::sync::Mutex;

use anc_allocator::{allocator::Allocator, dummy_allocator::DummyAllocator};
use anc_image::module_image::{ModuleImage, Visibility};
use anc_isa::DataSectionType;
use anc_memory::indexed_memory_access::IndexedMemoryAccess;
use anc_stack::{simple_stack::SimpleStack, stack::Stack, ProgramCounter};

use crate::{
    delegate_function_table::DelegateFunctionTable, external_function_table::ExternalFunctionTable,
    module_common_instance::ModuleCommonInstance, module_linking_instance::ModuleLinkingInstance,
    program_property::ProgramProperty,
};

/// the thread context of the VM.
pub struct ThreadContext<'a> {
    pub stack: Box<dyn Stack>,
    pub allocator: Box<dyn Allocator>,
    pub pc: ProgramCounter,

    // external function table
    pub external_function_table: &'a Mutex<ExternalFunctionTable>,

    // callback function table
    pub callback_function_table: DelegateFunctionTable,

    // program modules
    pub module_linking_instance: ModuleLinkingInstance<'a>,
    pub module_common_instances: Vec<ModuleCommonInstance<'a>>,

    // program property
    pub program_property: &'a ProgramProperty,
}

pub struct TargetDataObject<'a> {
    pub module_index: usize,
    pub data_section_type: DataSectionType,
    pub data_internal_index_in_section: usize,
    pub accessor: &'a mut dyn IndexedMemoryAccess,
}

pub struct TargetFunctionObject {
    pub target_module_index: usize,
    pub function_internal_index: usize,
}

pub struct FunctionInfo {
    pub type_index: usize,
    pub local_variable_list_index: usize,
    pub code_offset: usize,
    pub local_variables_with_arguments_allocated_bytes: usize,
}

impl<'a> ThreadContext<'a> {
    pub fn new(
        program_property: &'a ProgramProperty,
        module_images: &'a [ModuleImage<'a>],
        external_function_table: &'a Mutex<ExternalFunctionTable>,
    ) -> Self {
        let stack = SimpleStack::new();
        let allocator = DummyAllocator::new();

        let pc = ProgramCounter {
            instruction_address: 0,
            function_internal_index: 0,
            module_index: 0,
        };

        let module_linking_instance = ModuleLinkingInstance::new(module_images);
        let module_common_instances = module_images
            .iter()
            .map(ModuleCommonInstance::new)
            .collect::<Vec<ModuleCommonInstance>>();

        let callback_function_table = DelegateFunctionTable::new();
        Self {
            stack: Box::new(stack),
            allocator: Box::new(allocator),
            pc,
            external_function_table,
            callback_function_table,
            module_linking_instance,
            module_common_instances,
            program_property,
        }
    }

    pub fn get_target_data_object(
        &mut self,
        module_index: usize,
        data_public_index: usize,
        expect_offset_bytes: usize,         // for checking the data bounds
        expect_data_length_in_bytes: usize, // for checking the data bounds
    ) -> TargetDataObject {
        const MSB_DATA_PUBLIC_INDEX: usize = 1 << 63;
        const MASK_DATA_PUBLIC_INDEX: usize = !MSB_DATA_PUBLIC_INDEX;

        let target_data_object = if data_public_index & MSB_DATA_PUBLIC_INDEX != 0 {
            // it is dynamically allocated memory
            let allocated_memory_index = data_public_index & MASK_DATA_PUBLIC_INDEX; // clear the MSB bit

            TargetDataObject {
                module_index: 0,
                data_section_type: DataSectionType::ReadWrite,
                data_internal_index_in_section: allocated_memory_index,
                accessor: self.allocator.as_mut(),
            }
        } else {
            // data index bounds check for compilation error
            #[cfg(debug_assertions)]
            {
                let count = self
                    .module_linking_instance
                    .data_index_section
                    .get_items_count(module_index);

                if data_public_index > count as usize {
                    panic!(
                    "Out of bounds of the data public index, module index: {}, total data items: {}, request data index: {}.",
                    module_index, count, data_public_index
                );
                }
            }

            let (target_module_index, target_data_section_type, data_internal_index_in_section, ) = self
            .module_linking_instance
            .data_index_section
            .get_item_target_module_index_and_data_section_type_and_data_internal_index_in_section(
                module_index,
                data_public_index,
            );

            let target_module = &mut self.module_common_instances[target_module_index];
            let accessor = target_module.datas[target_data_section_type as usize].as_mut();

            TargetDataObject {
                module_index: target_module_index,
                data_section_type: target_data_section_type,
                data_internal_index_in_section: data_internal_index_in_section,
                accessor: accessor,
            }
        };

        // bounds check
        #[cfg(feature = "bounds_check")]
        {
            let data_actual_length = target_data_object
                .accessor
                .get_data_length(target_data_object.data_internal_index_in_section);

            if expect_data_length_in_bytes + expect_offset_bytes > data_actual_length {
                panic!(
                    "Access exceeds the length of the data.
module index: {}, function internal index: {}, instruction address: {},
data section type: {}, data public index: {}, data internal index: {},
data actual length (in bytes): {}, access offset (in bytes): {}, expect length (in bytes): {}.",
                    module_index,
                    self.pc.function_internal_index,
                    self.pc.instruction_address,
                    target_data_object.data_section_type,
                    data_public_index,
                    target_data_object.data_internal_index_in_section,
                    data_actual_length,
                    expect_offset_bytes,
                    expect_data_length_in_bytes,
                );
            }
        }

        target_data_object
    }

    pub fn get_target_function_object(
        &self,
        module_index: usize,
        function_public_index: usize,
    ) -> TargetFunctionObject {
        // function index bounds check for compilation error
        #[cfg(debug_assertions)]
        {
            let count = self
                .module_linking_instance
                .function_index_section
                .get_items_count(module_index);

            if function_public_index > count {
                panic!("Out of bounds of the function public index, module index: {}, total functions (includes imported): {}, request function public index: {}.",
                    module_index,
                    count,
                    function_public_index
                );
            }
        }

        let (target_module_index, function_internal_index) = self
            .module_linking_instance
            .function_index_section
            .get_item_target_module_index_and_function_internal_index(
                module_index,
                function_public_index,
            );

        TargetFunctionObject {
            target_module_index,
            function_internal_index,
        }
    }

    pub fn get_function_info(
        &self,
        module_index: usize,
        function_internal_index: usize,
    ) -> FunctionInfo {
        let function_item = &self.module_common_instances[module_index]
            .function_section
            .items[function_internal_index];

        let type_index = function_item.type_index as usize;
        let local_variable_list_index = function_item.local_variable_list_index as usize;
        let code_offset = function_item.code_offset as usize;

        let local_variables_with_arguments_allocated_bytes = self.module_common_instances
            [module_index]
            .local_variable_section
            .lists[local_variable_list_index]
            .allocated_bytes as usize;

        FunctionInfo {
            type_index,
            local_variable_list_index,
            code_offset,
            local_variables_with_arguments_allocated_bytes,
        }
    }

    pub fn get_local_variable_start_address(
        &self,
        reversed_index: u16,
        // the index of a local variable
        local_variable_index: usize,
        offset_bytes: usize,                // for check the local variable bounds
        expect_data_length_in_bytes: usize, // for check the local variable bounds
    ) -> usize {
        // get the local variable info
        let ProgramCounter {
            instruction_address,
            function_internal_index,
            module_index,
        } = self.pc;

        let (local_variable_list_index, local_variables_start_address) = self
            .stack
            .get_frame_local_variable_list_index_and_start_address_by_reversed_index(
                reversed_index,
            );

        let variable_item = &self.module_common_instances[module_index]
            .local_variable_section
            .get_local_variable_list(local_variable_list_index as usize)[local_variable_index];

        // bounds check
        #[cfg(feature = "bounds_check")]
        {
            if expect_data_length_in_bytes + offset_bytes > variable_item.var_actual_length as usize
            {
                panic!(
                    "Access exceeds the length of the local variable.
module index: {}, function internal index: {}, instruction address: {},
block reversed index: {}, local variable index: {},
variable actual length (in bytes): {}, access offset (in bytes): {}, expect length (in bytes): {}.",
                    module_index,
                    function_internal_index,
                    instruction_address,
                    reversed_index,
                    local_variable_index,
                    variable_item.var_actual_length,
                    offset_bytes,
                    expect_data_length_in_bytes,
                );
            }
        }

        local_variables_start_address + variable_item.var_offset as usize
    }

    pub fn find_function_by_full_name(
        &self,
        module_name: &str,
        expected_function_full_name: &str,
    ) -> Option<(
        /* module index */ usize,
        /* function internal index */ usize,
    )> {
        let (module_index, module_common_instance) = self
            .module_common_instances
            .iter()
            .enumerate()
            .find(|(_, module)| module.name == module_name)?;

        let (visibility, function_internal_index) = module_common_instance
            .function_name_section
            .get_item_visibility_and_function_internal_index(expected_function_full_name)?;

        if visibility != Visibility::Public {
            return None;
        } else {
            Some((module_index, function_internal_index))
        }
    }

    pub fn find_data_by_full_name(
        &self,
        module_name: &str,
        expected_data_full_name: &str,
    ) -> Option<(
        /* module index */ usize,
        /* data section type */ DataSectionType,
        /* data internal index in section */ usize,
    )> {
        let (module_index, module_common_instance) = self
            .module_common_instances
            .iter()
            .enumerate()
            .find(|(_, module)| module.name == module_name)?;

        let (visibility, data_section_type, data_internal_index_in_section) =
            module_common_instance
                .data_name_section
                .get_item_visibility_and_section_type_and_data_internal_index_in_section(
                    expected_data_full_name,
                )?;

        if visibility != Visibility::Public {
            return None;
        } else {
            Some((
                module_index,
                data_section_type,
                data_internal_index_in_section,
            ))
        }
    }

    /// get 16 bits instruction
    /// returns `[opcode]`.
    pub fn get_opcode_num(&self) -> u16 {
        let data = self.get_instruction(0, 2);
        let ptr_u16 = data.as_ptr() as *const u16;
        unsafe { std::ptr::read(ptr_u16) }
    }

    /// get 32 bits instruction
    /// returns `[opcode + i16]`
    pub fn get_param_i16(&self) -> u16 {
        let data = self.get_instruction(2, 2);
        let ptr_u16 = data.as_ptr() as *const u16;
        unsafe { std::ptr::read(ptr_u16) }
    }

    /// get 64 bits instruction
    /// returns `[opcode + padding + i32]`
    pub fn get_param_i32(&self) -> u32 {
        let data = self.get_instruction(4, 4);
        let ptr_u32 = data.as_ptr() as *const u32;
        unsafe { std::ptr::read(ptr_u32) }
    }

    /// get 64 bits instruction the second variant
    /// returns `[opcode + i16 + i32]`
    pub fn get_param_i16_i32(&self) -> (u16, u32) {
        let data = self.get_instruction(2, 6);

        unsafe {
            let p0 = std::ptr::read(data.as_ptr() as *const u16);
            let p1 = std::ptr::read(data[2..].as_ptr() as *const u32);
            (p0, p1)
        }
    }

    /// get 64 bits instruction the third variant
    /// returns `[opcode + i16 + i16 + i16]`
    pub fn get_param_i16_i16_i16(&self) -> (u16, u16, u16) {
        let data = self.get_instruction(2, 6);

        unsafe {
            let p0 = std::ptr::read(data.as_ptr() as *const u16);
            let p1 = std::ptr::read(data[2..4].as_ptr() as *const u16);
            let p2 = std::ptr::read(data[4..].as_ptr() as *const u16);
            (p0, p1, p2)
        }
    }

    /// get 96 bits instruction.
    /// returns `[opcode + padding + i32 + i32]`
    pub fn get_param_i32_i32(&self) -> (u32, u32) {
        let data = self.get_instruction(4, 8);

        unsafe {
            let p0 = std::ptr::read(data.as_ptr() as *const u32);
            let p1 = std::ptr::read(data[4..].as_ptr() as *const u32);
            (p0, p1)
        }
    }

    /// get 128 bits instruction.
    /// returns `[opcode + padding + i32 + i32 + i32]`
    pub fn get_param_i32_i32_i32(&self) -> (u32, u32, u32) {
        let data = self.get_instruction(4, 12);

        unsafe {
            let p0 = std::ptr::read(data.as_ptr() as *const u32);
            let p1 = std::ptr::read(data[4..8].as_ptr() as *const u32);
            let p2 = std::ptr::read(data[8..].as_ptr() as *const u32);
            (p0, p1, p2)
        }
    }

    #[inline]
    pub fn get_instruction(&self, offset: usize, len_in_bytes: usize) -> &[u8] {
        // Instruction encoding table:
        //
        // | length  | encoding layout                                                             |
        // |---------|-----------------------------------------------------------------------------|
        // | 16-bit  | [opcode 16-bit]                                                             |
        // | 32-bit  | [opcode 16-bit] - [param i16    ]                                           |
        // | 64-bit  | [opcode 16-bit] - [pading 16-bit] + [param i32]                             |
        // | 64-bit  | [opcode 16-bit] - [param i16    ] + [param i32]                             |
        // | 64-bit  | [opcode 16-bit] - [param i16    ] + [param i16] + [param i16]               |
        // | 96-bit  | [opcode 16-bit] - [pading 16-bit] + [param i32] + [param i32]               |
        // | 128-bit | [opcode 16-bit] - [pading 16-bit] + [param i32] + [param i32] + [param i32] |

        let ProgramCounter {
            instruction_address,
            function_internal_index: _,
            module_index,
        } = self.pc;

        let codes_data = self.module_common_instances[module_index]
            .function_section
            .codes_data;
        let dst = instruction_address + offset;
        &codes_data[dst..(dst + len_in_bytes)]
    }
}
