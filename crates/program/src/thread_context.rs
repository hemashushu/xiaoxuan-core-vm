// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::sync::Mutex;

use ancvm_binary::module_image::ModuleImage;

use crate::{
    external_function::ExtenalFunctionTable, heap::Heap, indexed_memory::IndexedMemory,
    program_context::ProgramContext, program_settings::ProgramSettings, stack::Stack,
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
    // returning multiple values:
    //
    // on most architecture, only one value can be returned in a function, or two values (e.g.
    // rax/rdx on x86_64, x0/x1 on aarch64, a0/a1 on riscv), but the XiaoXuan ISA allows
    // return multiple values in a function or a block, this is a kind of convenience when building
    // functions or control flow blocks.
    // however, when a XiaoXuan program need to interact with other programs built with other languages,
    // it is recommended that keep the function return only one value.
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

    pub bridge_function_table: Vec<DelegateLibraryItem>,
    pub callback_function_table: Vec<DelegateLibraryItem>,

    pub external_function_table: &'a Mutex<ExtenalFunctionTable>,
    pub program_context: ProgramContext<'a>,
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

pub struct DelegateLibraryItem {
    pub target_module_index: usize,
    pub birdge_function_items: Vec<DelegateFunctionItem>,
}

pub struct DelegateFunctionItem {
    pub function_internal_index: usize,
    pub bridge_function_ptr: *const u8,
}

impl<'a> ThreadContext<'a> {
    pub fn new(
        external_function_table: &'a Mutex<ExtenalFunctionTable>,
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

        let program_context = ProgramContext::new(module_images);

        Self {
            stack,
            heap,
            pc,
            bridge_function_table: vec![],
            callback_function_table: vec![],
            external_function_table,
            program_context,
            program_settings,
        }
    }

    pub fn find_function_target_module_index_and_public_index_by_name() -> (usize, usize) {
        // todo
        (0, 0)
    }

    pub fn find_data_target_module_index_and_public_index_by_name() -> (usize, usize) {
        // todo
        (0, 0)
    }

    /// return:
    /// (target_module_index:usize, data_internal_index:usize, dyn IndexedMemory)
    pub fn get_current_module_data_target_module_index_and_internal_index_and_data_object_with_bounds_check(
        &mut self,
        data_public_index: usize,
        expect_offset_bytes: usize, // for checking the expect data length
        expect_data_length_in_bytes: usize, // for checking the expect data length
    ) -> (usize, usize, &mut dyn IndexedMemory) {
        let module_index = self.pc.module_index;

        let (target_module_index, data_internal_index, target_data_section_type) = self
            .program_context
            .data_index_section
            .get_item_target_module_index_and_data_internal_index_and_data_section_type(
                module_index,
                data_public_index,
            );

        let target_module = &mut self.program_context.program_modules[target_module_index];
        let data_object = target_module.datas[target_data_section_type as usize].as_mut();

        // bounds check
        #[cfg(debug_assertions)]
        {
            let (_offset, data_actual_length) =
                data_object.get_offset_and_length_by_index(data_internal_index);
            if expect_data_length_in_bytes + expect_offset_bytes > data_actual_length {
                panic!(
                    "Out of bounds of the data.
module index: {}, function internal index: {}, instruction address: {},
data section type: {}, data internal index: {},
data actual length in bytes: {}, offset in bytes: {}, expect length in bytes: {}.",
                    module_index,
                    self.pc.function_internal_index,
                    self.pc.instruction_address,
                    data_internal_index,
                    target_data_section_type,
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
            .program_context
            .func_index_section
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
        let func_item = &self.program_context.program_modules[module_index]
            .func_section
            .items[function_internal_index];

        let type_index = func_item.type_index as usize;
        let local_list_index = func_item.local_list_index as usize;
        let code_offset = func_item.code_offset as usize;

        let local_variables_allocate_bytes = self.program_context.program_modules[module_index]
            .local_variable_section
            .lists[local_list_index]
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
            (
                frame_pack.address,
                frame_pack.frame_info.local_list_index,
            )
        };

        let variable_item = &self.program_context.program_modules[module_index]
            .local_variable_section
            .get_local_list(local_list_index as usize)[local_variable_index];

        // bounds check
        #[cfg(debug_assertions)]
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

        let codes_data = self.program_context.program_modules[module_index]
            .func_section
            .codes_data;
        let dst = instruction_address + offset;
        &codes_data[dst..(dst + len_in_bytes)]
    }
}

fn find_delegate_function(
    delegate_function_table: &[DelegateLibraryItem],
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
    delegate_function_table: &mut Vec<DelegateLibraryItem>,
    target_module_index: usize,
    function_internal_index: usize,
    bridge_function_ptr: *const u8,
) {
    let idx_m = delegate_function_table
        .iter()
        .position(|module_item| module_item.target_module_index == target_module_index)
        .unwrap_or_else(|| {
            delegate_function_table.push(DelegateLibraryItem {
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
