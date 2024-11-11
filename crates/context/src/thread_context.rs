// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::sync::Mutex;

use ancvm_image::module_image::ModuleImage;

use crate::{
    environment::Environment, external_function_table::ExternalFunctionTable, heap::Heap,
    indexed_memory::IndexedMemory, module_common_instance::ModuleCommonInstance,
    module_index_instance::ModuleIndexInstance, stack::Stack, INIT_HEAP_SIZE_IN_PAGES,
    INIT_STACK_SIZE_IN_BYTES, LOCAL_LIST_INDEX_NOT_EXIST,
};

/// the thread context of the VM.
pub struct ThreadContext<'a> {
    // the operand stack also contains the function/block frame information
    // when a function is called or a block is entered,
    //
    // the default stack capacity is 32 KiB, when a new stack frame is created, the
    // VM checks the stack capacity and makes sure there is at least 32 KiB
    // for the current frame.
    // the stack capacity is incremented in 32 KiB increments, i.e. the capacity will be
    // 32, 64, 96, 128 KiB and so on.
    //
    // the following diagram shows how the stack changs when a function/block
    // is entered or exited.
    //
    // 1. function 1 will call function 2, the arguments are ready.
    //
    // |         |
    // |         |
    // |  arg 1  | <-- operands that are to be used as arguments
    // |  arg 0  |
    // |---------|
    // |   ###   | <-- other operands of function 1
    // |---------| <-- current stack frame pointer (FP)
    // |   ...   |
    // \---------/ <-- stack start
    //
    // 2. called function 2.
    //
    // |         |
    // | local 1 |
    // | local 0 | <-- allocates the local variable area
    // |---------|
    // |  arg 1  | <-- arguments will be moved to the top side of stack, follows the frame info and local variables.
    // |  arg 0  |
    // |---------|
    // |   $$$   | <-- the stack frame information, includes the previous FP, return address (instruction address and module index),
    // |   $$$   |     also includes the current function information, such as function type, funcion index, and so on.
    // |   $$$   |     note that the original arguments is moved to the top of stack.
    // |---------| <-- new stack frame pointer (FP of function 2)
    // |   ###   | <-- other operands of function 1
    // |---------| <-- function 1 stack frame pointer (FP of function 1)
    // |   ...   |
    // \---------/
    //
    // 3. function 2 will return function 1 with two results
    //
    // |         |
    // | resul 1 |
    // | resul 0 | <-- results
    // |---------|
    // |   %%%   | <-- other operands of function 2
    // |---------|
    // | local 1 |
    // | local 0 |
    // |---------|
    // |  arg 1  |
    // |  arg 0  |
    // |---------|
    // |   $$$   |
    // |   $$$   |
    // |   $$$   |
    // |---------| <-- FP of function 2
    // |   ###   | <-- other operands of function 1
    // |---------| <-- FP of function 1
    // |   ...   |
    // \---------/
    //
    // 4. returns
    //
    // |         |     the results are copied to the position immediately following the
    // | resul 1 | <-- function 1 operands, all data between the results and FP 2 will be removed or overwrited.
    // | resul 0 |     i.e., the frame info, local variables and operands (and their host address) after this frame will no longer valid.
    // |---------|
    // |   ###   | <-- other operands of function 1
    // |---------| <-- FP of function 1
    // |   ...   |
    // \---------/
    //
    // returns multiple values
    // -----------------------
    //
    // on most architectures, only one value or two values can be returned in a function (e.g.
    // rax/rdx on x86_64, x0/x1 on aarch64, a0/a1 on riscv), but the XiaoXuan Core VM allows
    // returning multiple values in a function or a block, this is a kind of convenience when building
    // functions or control flow blocks.
    // however, when a XiaoXuan program need to interact with other programs built with other languages,
    // it is recommended that keep the function return only one value.
    pub stack: Stack,

    // in XiaoXuan Core VM, the data sections (read-only, read-write, uninit) are all thread-local,
    // and the heap is thread-local also.
    // threads/processes can communicated through the MessageBox/MessagePipe or the SharedMemory
    //
    // note that the initial capacity of heap is 0 byte
    pub heap: Heap,

    // the position of the next executing instruction (a.k.a. IP/PC)
    // the XiaoXuan Core VM load multiple modules for a application, thus the
    // "complete IP" consists of the module index and the instruction position.
    pub pc: ProgramCounter,

    // runtime generated entries
    pub bridge_function_module_items: Vec<DelegateFunctionModuleItem>,
    pub bridge_callback_function_module_items: Vec<DelegateFunctionModuleItem>,
    pub external_function_table: &'a Mutex<ExternalFunctionTable>,

    // application modules
    pub module_index_instance: ModuleIndexInstance<'a>,
    pub module_common_instances: Vec<ModuleCommonInstance<'a>>,

    // application environment
    pub environment: &'a Environment,
}

/// the PC
/// ------
///
/// unlike the ELF and Linux running environment, which all data and code of executable binary
/// are loaded into one memory space, and the execution state of instruction can be represented
/// by a single number -- program counter (PC) or instruction pointer (IP).
/// XiaoXuan application is composed of several modules, each module contains its data and code,
/// and code are separated into several pieces which called 'function'.
/// so the PC in XiaoXuan Core VM is represented by a tuple of
/// (module index, function index, instruction address)
///
/// note that in the default VM implementation, the code of functions are joined together,
/// so the address of the first instruction of a function does not always start with 0.
/// for example, the 'instruction address' of the first function in a module is naturally 0,
/// the second will be N if the length of the code of the first function is N, and
/// the third will be N+M if the length of the code of the second function, and so on.
///
/// on the other hand, a PC can only consist of 'module index' and 'instruction address', because
/// the 'instruction address' implies the code start position of a function, but for the sake
/// of clarity the 'function index' field is kept here.
#[derive(Debug, PartialEq)]
pub struct ProgramCounter {
    pub instruction_address: usize,     // the address of instruction
    pub function_internal_index: usize, // the function internal index
    pub module_index: usize,            // the module index
}

pub struct DelegateFunctionModuleItem {
    pub target_module_index: usize,
    pub birdge_function_items: Vec<DelegateFunctionItem>,
}

pub struct DelegateFunctionItem {
    pub function_internal_index: usize,
    pub bridge_function_ptr: *const u8,
}

impl<'a> ThreadContext<'a> {
    pub fn new(
        environment: &'a Environment,
        module_images: &'a [ModuleImage<'a>],
        external_function_table: &'a Mutex<ExternalFunctionTable>,
    ) -> Self {
        let stack = Stack::new(INIT_STACK_SIZE_IN_BYTES);
        let heap = Heap::new(INIT_HEAP_SIZE_IN_PAGES);

        let pc = ProgramCounter {
            instruction_address: 0,
            function_internal_index: 0,
            module_index: 0,
        };

        let module_index_instance = ModuleIndexInstance::new(module_images);
        let module_common_instances = module_images
            .iter()
            .map(ModuleCommonInstance::new)
            .collect::<Vec<ModuleCommonInstance>>();

        Self {
            stack,
            heap,
            pc,
            bridge_function_module_items: vec![],
            bridge_callback_function_module_items: vec![],
            external_function_table,
            module_index_instance,
            module_common_instances,
            environment,
        }
    }

    /// return:
    /// (target_module_index:usize, data_internal_index:usize, dyn IndexedMemory)
    pub fn get_data_target_module_index_and_internal_index_and_data_object_with_bounds_check(
        &mut self,
        module_index: usize,
        data_public_index: usize,
        expect_offset_bytes: usize, // for checking the expect data length
        expect_data_length_in_bytes: usize, // for checking the expect data length
    ) -> (usize, usize, &mut dyn IndexedMemory) {
        let (target_module_index, data_internal_index, target_data_section_type) = self
            .module_index_instance
            .data_index_section
            .get_item_target_module_index_and_data_internal_index_and_data_section_type(
                module_index,
                data_public_index,
            );

        let target_module = &mut self.module_common_instances[target_module_index];
        let data_object = target_module.datas[target_data_section_type as usize].as_mut();

        // bounds check
        #[cfg(feature = "bounds_check")]
        {
            let (_offset, data_actual_length) =
                data_object.get_offset_and_length_by_index(data_internal_index);
            if expect_data_length_in_bytes + expect_offset_bytes > data_actual_length {
                panic!(
                    "Out of bounds of the data.
module index: {}, function internal index: {}, instruction address: {},
data section type: {}, data public index: {}, data internal index: {},
data actual length in bytes: {}, offset in bytes: {}, expect length in bytes: {}.",
                    module_index,
                    self.pc.function_internal_index,
                    self.pc.instruction_address,
                    target_data_section_type,
                    data_public_index,
                    data_internal_index,
                    data_actual_length,
                    expect_offset_bytes,
                    expect_data_length_in_bytes,
                );
            }
        }

        (target_module_index, data_internal_index, data_object)
    }

    /// return:
    /// (target_module_index, function_internal_index)
    pub fn get_function_target_module_index_and_internal_index(
        &self,
        module_index: usize,
        function_public_index: usize,
    ) -> (usize, usize) {
        let (target_module_index, function_internal_index) = self
            .module_index_instance
            .function_index_section
            .get_item_target_module_index_and_function_internal_index(
                module_index,
                function_public_index,
            );
        (target_module_index, function_internal_index)
    }

    /// return:
    /// (type_index, local_list_index, code_offset, local_variables_allocate_bytes)
    pub fn get_function_type_and_local_list_index_and_code_offset_and_local_variables_allocate_bytes(
        &self,
        module_index: usize,
        function_internal_index: usize,
    ) -> (usize, usize, usize, u32) {
        let function_item = &self.module_common_instances[module_index]
            .function_section
            .items[function_internal_index];

        let type_index = function_item.type_index as usize;
        let local_list_index = function_item.local_list_index as usize;
        let code_offset = function_item.code_offset as usize;

        let local_variables_allocate_bytes = self.module_common_instances[module_index]
            .local_variable_section
            .list_items[local_list_index]
            .list_allocate_bytes;

        (
            type_index,
            local_list_index,
            code_offset,
            local_variables_allocate_bytes,
        )
    }

    pub fn get_local_variable_address_by_index_and_offset_with_bounds_check(
        &self,
        reversed_index: u16,
        local_variable_index: usize, // note that this is different from 'local_list_index'
        offset_bytes: usize,
        expect_data_length_in_bytes: usize, // for checking the expect data length
    ) -> usize {
        // get the local variable info
        let ProgramCounter {
            instruction_address,
            function_internal_index,
            module_index,
        } = self.pc;

        let (fp, local_list_index) = {
            let frame_pack = self.stack.get_frame_pack(reversed_index);
            (frame_pack.address, frame_pack.frame_info.local_list_index)
        };

        if local_list_index == LOCAL_LIST_INDEX_NOT_EXIST {
            panic!(
                "An attempt to access a local variable that does not exist.
module index: {}, function internal index: {}, instruction address: {},
block reversed index: {}, local variable index: {},
offset in bytes: {}, expect length in bytes: {}.",
                module_index,
                function_internal_index,
                instruction_address,
                reversed_index,
                local_variable_index,
                offset_bytes,
                expect_data_length_in_bytes,
            );
        }

        let variable_item = &self.module_common_instances[module_index]
            .local_variable_section
            .get_local_list(local_list_index as usize)[local_variable_index];

        // bounds check
        #[cfg(feature = "bounds_check")]
        {
            if expect_data_length_in_bytes + offset_bytes > variable_item.var_actual_length as usize
            {
                panic!(
                    "Out of bounds of the local variable.
module index: {}, function internal index: {}, instruction address: {},
block reversed index: {}, local variable index: {}, variable actual length in bytes: {},
offset in bytes: {}, expect length in bytes: {}.",
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

        let local_start_address = self.stack.get_frame_local_variables_start_address(fp);
        local_start_address + variable_item.var_offset as usize + offset_bytes
    }

    pub fn find_function_public_index_by_name(
        &self,
        module_name: &str,
        function_name: &str,
    ) -> Option<(usize, usize)> {
        let (module_index, module_instance) = self
            .module_common_instances
            .iter()
            .enumerate()
            .find(|(_, module)| module.name == module_name)?;

        let (internal_index, _) = module_instance
            .function_name_section
            .get_item_index_and_export(function_name)?;

        Some((
            module_index,
            internal_index + module_instance.import_function_count,
        ))
    }

    pub fn find_data_public_index_by_name(
        &self,
        module_name: &str,
        data_name: &str,
    ) -> Option<(usize, usize)> {
        let (module_index, module_instance) = self
            .module_common_instances
            .iter()
            .enumerate()
            .find(|(_, module)| module.name == module_name)?;

        let (internal_index, _) = module_instance
            .data_name_section
            .get_item_index_and_export(data_name)?;

        Some((
            module_index,
            internal_index + module_instance.import_data_count,
        ))
    }

    pub fn find_bridge_function(
        &self,
        target_module_index: usize,
        function_internal_index: usize,
    ) -> Option<*const u8> {
        find_delegate_function(
            &self.bridge_function_module_items,
            target_module_index,
            function_internal_index,
        )
    }

    pub fn find_bridge_callback_function(
        &self,
        target_module_index: usize,
        function_internal_index: usize,
    ) -> Option<*const u8> {
        find_delegate_function(
            &self.bridge_callback_function_module_items,
            target_module_index,
            function_internal_index,
        )
    }

    pub fn insert_bridge_function(
        &mut self,
        target_module_index: usize,
        function_internal_index: usize,
        bridge_function_ptr: *const u8,
    ) {
        insert_delegate_function(
            &mut self.bridge_function_module_items,
            target_module_index,
            function_internal_index,
            bridge_function_ptr,
        );
    }

    pub fn insert_callback_function(
        &mut self,
        target_module_index: usize,
        function_internal_index: usize,
        bridge_function_ptr: *const u8,
    ) {
        insert_delegate_function(
            &mut self.bridge_callback_function_module_items,
            target_module_index,
            function_internal_index,
            bridge_function_ptr,
        );
    }

    /// opcode, or
    /// 16 bits instruction
    /// [opcode]
    pub fn get_opcode_num(&self) -> u16 {
        let data = self.get_instruction(0, 2);
        let ptr_u16 = data.as_ptr() as *const u16;
        unsafe { std::ptr::read(ptr_u16) }
    }

    /// 32 bits instruction
    /// [opcode + i16]
    pub fn get_param_i16(&self) -> u16 {
        let data = self.get_instruction(2, 2);
        let ptr_u16 = data.as_ptr() as *const u16;
        unsafe { std::ptr::read(ptr_u16) }
    }

    /// 64 bits instruction
    /// [opcode + padding + i32]
    ///
    /// note that 'i32' in function name means a 32-bit integer, which is equivalent to
    /// the 'uint32_t' in C or 'u32' in Rust. do not confuse it with 'i32' in Rust.
    /// the same applies to the i8, i16 and i64.
    pub fn get_param_i32(&self) -> u32 {
        let data = self.get_instruction(4, 4);
        let ptr_u32 = data.as_ptr() as *const u32;
        unsafe { std::ptr::read(ptr_u32) }
    }

    /// 64 bits instruction
    /// [opcode + i16 + i32]
    pub fn get_param_i16_i32(&self) -> (u16, u32) {
        let data = self.get_instruction(2, 6);

        unsafe {
            let p0 = std::ptr::read(data.as_ptr() as *const u16);
            let p1 = std::ptr::read(data[2..].as_ptr() as *const u32);
            (p0, p1)
        }
    }

    /// 64 bits instruction
    /// [opcode + i16 + i16 + i16]
    pub fn get_param_i16_i16_i16(&self) -> (u16, u16, u16) {
        let data = self.get_instruction(2, 6);

        unsafe {
            let p0 = std::ptr::read(data.as_ptr() as *const u16);
            let p1 = std::ptr::read(data[2..4].as_ptr() as *const u16);
            let p2 = std::ptr::read(data[4..].as_ptr() as *const u16);
            (p0, p1, p2)
        }
    }

    /// 96 bits instruction
    /// [opcode + padding + i32 + i32]
    pub fn get_param_i32_i32(&self) -> (u32, u32) {
        let data = self.get_instruction(4, 8);

        unsafe {
            let p0 = std::ptr::read(data.as_ptr() as *const u32);
            let p1 = std::ptr::read(data[4..].as_ptr() as *const u32);
            (p0, p1)
        }
    }

//     DEPRECATED
//     /// 128 bits instruction
//     /// [opcode + padding + i32 + i32 + i32]
//     pub fn get_param_i32_i32_i32(&self) -> (u32, u32, u32) {
//         let data = self.get_instruction(4, 12);
//
//         unsafe {
//             let p0 = std::ptr::read(data.as_ptr() as *const u32);
//             let p1 = std::ptr::read(data[4..8].as_ptr() as *const u32);
//             let p2 = std::ptr::read(data[8..].as_ptr() as *const u32);
//             (p0, p1, p2)
//         }
//     }

    #[inline]
    pub fn get_instruction(&self, offset: usize, len_in_bytes: usize) -> &[u8] {
        // the instruction schemes:
        //
        // - [opcode i16]
        // - [opcode i16] - [param i16      ]
        // - [opcode i16] - [param i16      ] + [param i32]
        // - [opcode i16] - [padding 16 bits] + [param i32]
        // - [opcode i16] - [padding 16 bits] + [param i32] + [param i32]

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

fn find_delegate_function(
    delegate_function_table: &[DelegateFunctionModuleItem],
    target_module_index: usize,
    function_internal_index: usize,
) -> Option<*const u8> {
    match delegate_function_table
        .iter()
        .find(|module_item| module_item.target_module_index == target_module_index)
    {
        Some(module_item) => module_item
            .birdge_function_items
            .iter()
            .find(|functione_item| {
                functione_item.function_internal_index == function_internal_index
            })
            .map(|function_item| function_item.bridge_function_ptr),
        None => None,
    }
}

fn insert_delegate_function(
    delegate_function_table: &mut Vec<DelegateFunctionModuleItem>,
    target_module_index: usize,
    function_internal_index: usize,
    bridge_function_ptr: *const u8,
) {
    let idx_m = delegate_function_table
        .iter()
        .position(|module_item| module_item.target_module_index == target_module_index)
        .unwrap_or_else(|| {
            delegate_function_table.push(DelegateFunctionModuleItem {
                target_module_index,
                birdge_function_items: vec![],
            });
            delegate_function_table.len() - 1
        });

    let module_item = &mut delegate_function_table[idx_m];

    // note:
    //
    // there is no validation here to check if the function specified
    // already exists, so make sure you don't add a duplicate function.
    module_item
        .birdge_function_items
        .push(DelegateFunctionItem {
            function_internal_index,
            bridge_function_ptr,
        })
}
