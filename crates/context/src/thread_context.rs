// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::sync::Mutex;

use anc_allocator::{allocator::Allocator, mimallocator::MiMAllocator};
use anc_image::module_image::{ModuleImage, Visibility};
use anc_isa::DataSectionType;
use anc_memory::indexed_memory_access::IndexedMemoryAccess;
use anc_stack::{nostd_stack::NostdStack, stack::Stack, ProgramCounter};
use cranelift_jit::JITModule;

use crate::{
    bridge_function_table::BridgeFunctionTable,
    callback_delegate_function_table::CallbackDelegateFunctionTable, code_generator::Generator,
    external_function_table::ExternalFunctionTable, module_common_instance::ModuleCommonInstance,
    module_linking_instance::ModuleLinkingInstance, process_property::ProcessProperty,
    thread_resources::ThreadResources,
};

// The index of the most significant bit for memory data access.
// This bit is used to indicate that the data is dynamically allocated memory.
// If this bit is set, the data access index is treated as a dynamically allocated memory index
// (the MSB is set to 1), and the actual index is the remaining bits (the MSB is cleared).
// If this bit is not set, the data access index is treated as a public index in the module's data section.
// The MSB is used to distinguish between dynamically allocated memory and pre-defined data item.
pub const MEMORY_DATA_ACCESS_INDEX_MSB: usize = 1 << 63;
pub const MEMORY_DATA_ACCESS_INDEX_MASK: usize = !MEMORY_DATA_ACCESS_INDEX_MSB;

/// Represents the thread context of the VM, containing the stack, allocator, program counter,
/// function tables, module instances, and process properties.
pub struct ThreadContext<'a> {
    pub stack: Box<dyn Stack>, // The stack used for function calls and local variables.
    pub pc: ProgramCounter,    // The program counter, tracking the current instruction.
    pub allocator: Box<dyn Allocator>, // Allocator for dynamic memory management.

    // External function table, shared across threads and protected by a mutex.
    pub external_function_table: &'a Mutex<ExternalFunctionTable>,

    // Table for callback delegate functions, used for callback function calls.
    pub callback_delegate_function_table: CallbackDelegateFunctionTable,

    // Table for bridge functions, used for calling functions from outside the VM.
    pub bridge_function_table: BridgeFunctionTable,

    pub thread_resources: ThreadResources,

    pub jit_generator: &'a Mutex<Generator<JITModule>>,

    // Instances of "linking sections".
    pub module_linking_instance: ModuleLinkingInstance<'a>,

    // Instances of common modules.
    pub module_common_instances: Vec<ModuleCommonInstance<'a>>,

    // Properties of the process, such as configuration and runtime state.
    pub process_property: &'a Mutex<ProcessProperty>,
}

/// Represents a target data object, including its module index, data section type,
/// internal index within the section, and a mutable accessor for memory operations.
pub struct TargetDataObject<'a> {
    pub module_index: usize, // Index of the module containing the data.
    pub data_section_type: DataSectionType, // Type of the data section (e.g., ReadOnly, ReadWrite).
    pub data_internal_index_in_section: usize, // Internal index of the data within the section.
    pub accessor: &'a mut dyn IndexedMemoryAccess, // Accessor for memory operations.
}

/// Represents a target function object, including its module index and internal function index.
pub struct TargetFunctionObject {
    pub module_index: usize, // Index of the module containing the function.
    pub function_internal_index: usize, // Internal index of the function within the module.
}

/// Contains metadata about a function, such as its type, local variables, and code offset.
pub struct FunctionInfo {
    pub type_index: usize, // Index of the function's type in the type section.
    pub local_variable_list_index: usize, // Index of the local variable list.
    pub code_offset: usize, // Offset of the function's code in the code section.
    pub local_variables_with_arguments_allocated_bytes: usize, // Total allocated bytes for local variables and arguments.
}

impl<'a> ThreadContext<'a> {
    /// Creates a new `ThreadContext` instance, initializing its components.
    pub fn new(
        module_images: &'a [ModuleImage<'a>],
        process_property: &'a Mutex<ProcessProperty>,
        external_function_table: &'a Mutex<ExternalFunctionTable>,
        jit_generator: &'a Mutex<Generator<JITModule>>,
    ) -> Self {
        // Initialize the stack and allocator.
        let stack = NostdStack::new();
        let allocator = MiMAllocator::new(); // alternative: VecAllocator::new();

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

        let callback_delegate_function_table = CallbackDelegateFunctionTable::new();
        let bridge_function_table = BridgeFunctionTable::new();
        let resources = ThreadResources::new();

        Self {
            stack: Box::new(stack),
            allocator: Box::new(allocator),
            pc,
            external_function_table,
            callback_delegate_function_table,
            bridge_function_table,
            thread_resources: resources,
            jit_generator,
            module_linking_instance,
            module_common_instances,
            process_property,
        }
    }

    /// Retrieves a target data object, performing bounds checks if enabled.
    pub fn get_target_data_object(
        &mut self,
        module_index: usize,
        data_access_index: usize,
        expect_offset_bytes: usize, // Expected offset in bytes for bounds checking.
        expect_data_length_in_bytes: usize, // Expected data length in bytes for bounds checking.
    ) -> TargetDataObject {
        if data_access_index & MEMORY_DATA_ACCESS_INDEX_MSB != 0 {
            // it is dynamically allocated memory
            // boundary check is implemented in the allocator

            let data_internal_index = data_access_index & MEMORY_DATA_ACCESS_INDEX_MASK; // clear the MSB bit

            //             let opt_size = self.allocator.get_size(data_internal_index);
            //
            //             let data_actual_length = if let Some(size) = opt_size {
            //                 size
            //             } else {
            //                 panic!(
            //                     "Out of bounds of the dynamically allocated data index, request data index: {}.",
            //                     data_internal_index
            //                 );
            //             };
            //
            //             // bounds check
            //             #[cfg(feature = "bounds_check")]
            //             {
            //                 if expect_data_length_in_bytes + expect_offset_bytes > data_actual_length {
            //                     panic!(
            //                         "Access exceeds the length of the dynamically allocated data.
            // function internal index: {}, instruction address: 0x{:04x},
            // data internal index: {},
            // data actual length (in bytes): {}, access offset (in bytes): 0x{:02x}, expect length (in bytes): {}.",
            //
            //                         self.pc.function_internal_index,
            //                         self.pc.instruction_address,
            //                         data_internal_index,
            //                         data_actual_length,
            //                         expect_offset_bytes,
            //                         expect_data_length_in_bytes,
            //                     );
            //                 }
            //             }

            TargetDataObject {
                module_index: 0,
                data_section_type: DataSectionType::ReadWrite,
                data_internal_index_in_section: data_internal_index,
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

                if data_access_index > count {
                    panic!(
                        "Out of bounds of the data public index, module index: {}, total data items: {}, request data index: {}.",
                        module_index, count, data_access_index
                    );
                }
            }

            let data_public_index = data_access_index;

            let (target_module_index, target_data_section_type, data_internal_index_in_section, ) = self
            .module_linking_instance
            .data_index_section
            .get_item_target_module_index_and_data_section_type_and_data_internal_index_in_section(
                module_index,
                data_public_index,
            );

            let target_module = &mut self.module_common_instances[target_module_index];
            let accessor = target_module.datas[target_data_section_type as usize].as_mut();

            // bounds check
            #[cfg(feature = "bounds_check")]
            {
                let data_actual_length = accessor.get_data_length(data_internal_index_in_section);

                if expect_data_length_in_bytes + expect_offset_bytes > data_actual_length {
                    panic!(
                        "Access exceeds the length of the data.
module index: {}, function internal index: {}, instruction address: 0x{:04x},
data section type: {}, data public index: {}, data internal index: {},
data actual length (in bytes): {}, access offset (in bytes): 0x{:02x}, expect length (in bytes): {}.",
                        module_index,
                        self.pc.function_internal_index,
                        self.pc.instruction_address,
                        target_data_section_type,
                        data_access_index,
                        data_internal_index_in_section,
                        data_actual_length,
                        expect_offset_bytes,
                        expect_data_length_in_bytes,
                    );
                }
            }

            TargetDataObject {
                module_index: target_module_index,
                data_section_type: target_data_section_type,
                data_internal_index_in_section,
                accessor,
            }
        }
    }

    /// Retrieves a target function object, performing bounds checks if enabled.
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
            module_index: target_module_index,
            function_internal_index,
        }
    }

    /// Retrieves metadata about a function, such as its type and local variable information.
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

    /// Calculates the start address of a local variable within the stack frame.
    pub fn get_local_variable_start_address(
        &self,
        layers: u16,                 // The number of layers in the reverse stack frames.
        local_variable_index: usize, // Index of the local variable.
        // offset_bytes: usize,         // Offset in bytes for bounds checking.
        expect_data_length_in_bytes: usize, // Expected data length in bytes for bounds checking.
    ) -> usize {
        // get the local variable info
        let ProgramCounter {
            instruction_address,
            function_internal_index,
            module_index,
        } = self.pc;

        let (local_variable_list_index, local_variables_start_address) = self
            .stack
            .get_local_variable_list_index_and_start_address_by_layers(layers);

        let variable_item = &self.module_common_instances[module_index]
            .local_variable_section
            .get_local_variable_list(local_variable_list_index)[local_variable_index];

        // bounds check
        #[cfg(feature = "bounds_check")]
        {
            if expect_data_length_in_bytes // + offset_bytes
                > variable_item.variable_actual_size_in_bytes as usize
            {
                panic!(
                    "Access exceeds the length of the local variable.
module index: {}, function internal index: {}, instruction address: 0x{:04x},
layers: {}, local variable index: {},
variable actual length (in bytes): {}, expect length (in bytes): {}.",
                    module_index,
                    function_internal_index,
                    instruction_address,
                    layers,
                    local_variable_index,
                    variable_item.variable_actual_size_in_bytes,
                    // offset_bytes,
                    expect_data_length_in_bytes,
                );
            }
        }

        local_variables_start_address + variable_item.variable_offset as usize
    }

    /// Finds a function by its fully qualified name, returning its module and internal indices.
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
            None
        } else {
            Some((module_index, function_internal_index))
        }
    }

    /// Finds data by its fully qualified name, returning its module index, section type, and internal index.
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
            None
        } else {
            Some((
                module_index,
                data_section_type,
                data_internal_index_in_section,
            ))
        }
    }

    /// Retrieves a 16-bit instruction opcode.
    /// Returns `[opcode]`.
    pub fn get_opcode_num(&self) -> u16 {
        let data = self.get_instruction(0, 2);
        let ptr_u16 = data.as_ptr() as *const u16;
        unsafe { std::ptr::read(ptr_u16) }
    }

    /// Retrieves a 32-bit instruction parameter.
    /// Returns `[opcode + i16]`.
    pub fn get_param_i16(&self) -> u16 {
        let data = self.get_instruction(2, 2);
        let ptr_u16 = data.as_ptr() as *const u16;
        unsafe { std::ptr::read(ptr_u16) }
    }

    /// Retrieves a 64-bit instruction parameter.
    /// Returns `[opcode + padding + i32]`.
    pub fn get_param_i32(&self) -> u32 {
        let data = self.get_instruction(4, 4);
        let ptr_u32 = data.as_ptr() as *const u32;
        unsafe { std::ptr::read(ptr_u32) }
    }

    /// Retrieves a 64-bit instruction parameter variant.
    /// Returns `[opcode + i16 + i32]`.
    pub fn get_param_i16_i32(&self) -> (u16, u32) {
        let data = self.get_instruction(2, 6);

        unsafe {
            let p0 = std::ptr::read(data.as_ptr() as *const u16);
            let p1 = std::ptr::read(data[2..].as_ptr() as *const u32);
            (p0, p1)
        }
    }

    /// Retrieves a 96-bit instruction parameter.
    /// Returns `[opcode + padding + i32 + i32]`.
    pub fn get_param_i32_i32(&self) -> (u32, u32) {
        let data = self.get_instruction(4, 8);

        unsafe {
            let p0 = std::ptr::read(data.as_ptr() as *const u32);
            let p1 = std::ptr::read(data[4..].as_ptr() as *const u32);
            (p0, p1)
        }
    }

    /// Retrieves a 128-bit instruction parameter.
    /// Returns `[opcode + padding + i32 + i32 + i32]`.
    pub fn get_param_i32_i32_i32(&self) -> (u32, u32, u32) {
        let data = self.get_instruction(4, 12);

        unsafe {
            let p0 = std::ptr::read(data.as_ptr() as *const u32);
            let p1 = std::ptr::read(data[4..8].as_ptr() as *const u32);
            let p2 = std::ptr::read(data[8..].as_ptr() as *const u32);
            (p0, p1, p2)
        }
    }

    /// Retrieves a slice of instruction bytes from the code section.
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
