// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::{cell::RefCell, sync::Arc};

use ancvm_binary::module_image::ModuleImage;
use ancvm_types::{DataType, ForeignValue};

use crate::{
    external_function::ExtenalFunctionTable, heap::Heap, indexed_memory::IndexedMemory,
    program_reference::ProgramReference, program_settings::ProgramSettings, stack::Stack,
    INIT_HEAP_SIZE_IN_PAGES, INIT_STACK_SIZE_IN_PAGES,
};

/// one ThreadContext per thread.
///
/// the ThreadContext is NOT thread safe, do NOT call functions of ThreadContext
/// from multiple threads.
pub struct ThreadContext<'a> {
    // operand stack also includes the function/block frame info
    // when call a function or enter a block,
    //
    // the default stack capacity is 32 KiB, when a new stack frame is created, the
    // VM will check the capacity of the stack and ensure there is at lease 32 KiB
    // for the current frame.
    // the capacity of stack will be incremented in 32 KiB, i.e. the capacity will be
    // 32, 64, 96, 128 KiB and so on.
    //
    // the following diagram demostrates the stack changing when entering or leaving a function/block.
    //
    // 1. function 1 is going to call function 2, the arguments are ready.
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
    // |  arg 1  | <-- arguments will be moved to the top of stack, follows the frame info and local variables.
    // |  arg 0  |
    // |---------|
    // | local 1 |
    // | local 0 | <-- allocates the local variable area
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
    // 3. function 2 is going to return function 1 with two results
    //
    // |         |
    // | resul 1 |
    // | resul 0 | <-- results
    // |---------|
    // |   %%%   | <-- other operands of function 2
    // |---------|
    // |  arg 1  |
    // |  arg 0  |
    // |---------|
    // | local 1 |
    // | local 0 |
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
    // 4. returned
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
    pub stack: Stack,

    // in XiaoXuan VM, the data sections (read-only, read-write, uninit) are all thread-local,
    // and the heap is thread-local also.
    // threads/processes can communicated through the MessageBox/MessagePipe or the SharedMemory
    //
    // note that the initial capacity of heap is 0 byte
    pub heap: Heap,

    // the position of the next executing instruction (a.k.a. IP/PC)
    // the XiaoXuan VM load multiple modules for a application, thus the
    // "complete IP" consists of the module index and the instruction position.
    pub pc: ProgramCounter,

    pub bridge_function_table: Vec<DelegateModuleItem>,
    pub callback_function_table: Vec<DelegateModuleItem>,
    pub external_function_table: Arc<RefCell<ExtenalFunctionTable>>,

    pub program_reference: ProgramReference<'a>,
    pub program_settings: &'a ProgramSettings,
}

/// unlike the ELF and Linux runting environment, which all data and code of execuatable binary
/// are loaded into one memory space, and the execution state of instruction can be represented
/// by a single number -- program counter (PC) or instruction pointer (IP).
/// XiaoXuan application is consisted of several modules, each module contains its data and code,
/// and code are seperated into several piece which called 'function'.
/// so the PC in XiaoXuan VM is represented by a tuple of
/// (module index, function index, instruction address)
///
/// note that in the default VM implementation, the code of functions are join together,
/// so the address of first instruction of a function is not always start with 0.
/// for example, the 'instruction address' of the first function in a moudle is of course 0,
/// the second will be N if the length of the first function's code is N, and
/// the third will be N+M if the length of the second function's code is M, and so on.
///
/// on the other hand, a PC can only consists of 'module index' and 'instruction address', because
/// the 'instruction address' implies the code starting position of a function, but for the sake
/// of clarity, the field 'function index' is preserved here.
#[derive(Debug, PartialEq)]
pub struct ProgramCounter {
    pub instruction_address: usize,     // the address of instruction
    pub function_internal_index: usize, // the function internal index
    pub module_index: usize,            // the module index
}

pub struct DelegateModuleItem {
    pub target_module_index: usize,
    pub birdge_function_items: Vec<DelegateFunctionItem>,
}

pub struct DelegateFunctionItem {
    pub function_internal_index: usize,
    pub bridge_function_ptr: *const u8,
}

impl<'a> ThreadContext<'a> {
    pub fn new(
        external_function_table: Arc<RefCell<ExtenalFunctionTable>>,
        program_settings: &'a ProgramSettings,
        module_images: &'a [ModuleImage<'a>],
    ) -> Self {
        let stack = Stack::new(INIT_STACK_SIZE_IN_PAGES);
        let heap = Heap::new(INIT_HEAP_SIZE_IN_PAGES);

        let pc = ProgramCounter {
            instruction_address: 0,
            function_internal_index: 0,
            module_index: 0,
        };

        let program_reference = ProgramReference::new(module_images);

        Self {
            stack,
            heap,
            pc,
            bridge_function_table: vec![],
            callback_function_table: vec![],
            external_function_table,
            program_reference,
            program_settings,
        }
    }

    /// push values onto the stack
    ///
    /// note that the first value will be inserted into the stack bottom:
    ///
    /// array [0, 1, 2] -> |  2  |
    ///                    |  1  |
    ///                    |  0  |
    ///                    \-----/
    pub fn push_values(&mut self, values: &[ForeignValue]) {
        for value in values {
            match value {
                ForeignValue::UInt32(value) => self.stack.push_i32_u(*value),
                ForeignValue::UInt64(value) => self.stack.push_i64_u(*value),
                ForeignValue::Float32(value) => self.stack.push_f32(*value),
                ForeignValue::Float64(value) => self.stack.push_f64(*value),
            }
        }
    }

    /// pop values off the stack
    ///
    /// note that the values on the stack top will be poped first and became
    /// the LAST element of the array
    ///
    /// |  2  | -> array [0, 1, 2]
    /// |  1  |
    /// |  0  |
    /// \-----/
    pub fn pop_values(&mut self, data_types: &[DataType]) -> Vec<ForeignValue> {
        let mut reversed_results = data_types
            .iter()
            .rev()
            .map(|data_type| match data_type {
                DataType::I32 => ForeignValue::UInt32(self.stack.pop_i32_u()),
                DataType::I64 => ForeignValue::UInt64(self.stack.pop_i64_u()),
                DataType::F32 => ForeignValue::Float32(self.stack.pop_f32()),
                DataType::F64 => ForeignValue::Float64(self.stack.pop_f64()),
            })
            .collect::<Vec<ForeignValue>>();
        reversed_results.reverse();
        reversed_results
    }

    pub fn find_function_public_index_and_module_index_by_name() -> (usize, usize) {
        // todo
        (0, 0)
    }

    pub fn find_data_public_index_and_module_index_by_name() -> (usize, usize) {
        // todo
        (0, 0)
    }

    /// return:
    /// (dyn IndexedMemory, target_module_index:usize, data_internal_index:usize)
    pub fn get_current_module_data_internal_index_and_datas_object(
        &mut self,
        data_public_index: usize,
    ) -> (&mut dyn IndexedMemory, usize, usize) {
        let module_index = self.pc.module_index;

        let range = &self.program_reference.data_index_section.ranges[module_index];
        let data_index_item = &self.program_reference.data_index_section.items
            [range.offset as usize + data_public_index];

        let target_module_index = data_index_item.target_module_index as usize;
        let target_module = &mut self.program_reference.modules[target_module_index];
        let data_internal_index = data_index_item.data_internal_index as usize;
        let datas = target_module.datas[data_index_item.target_data_section_type as usize].as_mut();

        (datas, target_module_index, data_internal_index)
    }

    /// return:
    /// (target_module_index, function_internal_index)
    pub fn get_function_internal_index_and_module_index(
        &self,
        module_index: usize,
        function_public_index: usize,
    ) -> (usize, usize) {
        let range_item = &self.program_reference.func_index_section.ranges[module_index];
        let func_index_items = &self.program_reference.func_index_section.items
            [range_item.offset as usize..(range_item.offset + range_item.count) as usize];
        let func_index_item = &func_index_items[function_public_index];

        let target_module_index = func_index_item.target_module_index as usize;
        let function_internal_index = func_index_item.function_internal_index as usize;
        (target_module_index, function_internal_index)
    }

    /// return:
    /// (type_index, local_variables_list_index, code_offset, local_variables_allocate_bytes)
    pub fn get_function_type_and_local_index_and_code_offset_and_local_variables_allocate_bytes(
        &self,
        module_index: usize,
        function_internal_index: usize,
    ) -> (usize, usize, usize, u32) {
        let func_item = &self.program_reference.modules[module_index]
            .func_section
            .items[function_internal_index];

        let type_index = func_item.type_index as usize;
        let local_index = func_item.local_index as usize;
        let code_offset = func_item.code_offset as usize;

        let local_variables_allocate_bytes = self.program_reference.modules[module_index]
            .local_variable_section
            .lists[local_index]
            .list_allocate_bytes;

        (
            type_index,
            local_index,
            code_offset,
            local_variables_allocate_bytes,
        )
    }

    pub fn get_local_variable_address_by_index_and_offset(
        &self,
        reversed_index: u16,
        local_variable_index: usize,
        offset_bytes: usize,
    ) -> usize {
        // get the local variable info
        let ProgramCounter {
            instruction_address: _,
            function_internal_index: _,
            module_index,
        } = self.pc;

        let (fp, list_index) = {
            let frame_pack = self.stack.get_frame_pack(reversed_index);
            (
                frame_pack.address,
                frame_pack.frame_info.local_variables_list_index,
            )
        };

        let variable_item = &self.program_reference.modules[module_index]
            .local_variable_section
            .get_variable_list(list_index as usize)[local_variable_index];

        let local_start_address = self.stack.get_frame_local_variables_start_address(fp);
        local_start_address + variable_item.var_offset as usize + offset_bytes
    }

    pub fn find_bridge_function(
        &self,
        target_module_index: usize,
        function_internal_index: usize,
    ) -> Option<*const u8> {
        find_delegate_function(
            &self.bridge_function_table,
            target_module_index,
            function_internal_index,
        )
    }

    pub fn find_callback_function(
        &self,
        target_module_index: usize,
        function_internal_index: usize,
    ) -> Option<*const u8> {
        find_delegate_function(
            &self.callback_function_table,
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
            &mut self.bridge_function_table,
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
            &mut self.callback_function_table,
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

    /// 128 bits instruction
    /// [opcode + padding + i32 + i32 + i32]
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

        let codes_data = self.program_reference.modules[module_index]
            .func_section
            .codes_data;
        let dst = instruction_address + offset;
        &codes_data[dst..(dst + len_in_bytes)]
    }
}

fn find_delegate_function(
    delegate_function_table: &[DelegateModuleItem],
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
    delegate_function_table: &mut Vec<DelegateModuleItem>,
    target_module_index: usize,
    function_internal_index: usize,
    bridge_function_ptr: *const u8,
) {
    let idx_m = delegate_function_table
        .iter()
        .position(|module_item| module_item.target_module_index == target_module_index)
        .unwrap_or_else(|| {
            delegate_function_table.push(DelegateModuleItem {
                target_module_index,
                birdge_function_items: vec![],
            });
            delegate_function_table.len() - 1
        });

    let module_item = &mut delegate_function_table[idx_m];

    // note:
    //
    // there is no checking here to see if the specified function already
    // exists, so make sure don't add a function duplicated.
    module_item
        .birdge_function_items
        .push(DelegateFunctionItem {
            function_internal_index,
            bridge_function_ptr,
        })
}
