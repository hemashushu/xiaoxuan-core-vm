// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_binary::module_image::ModuleImage;
use ancvm_types::{DataType, ForeignValue};

use crate::{
    context::Context, heap::Heap, stack::Stack, INIT_HEAP_SIZE_IN_PAGES, INIT_STACK_SIZE_IN_PAGES,
};

pub struct Thread<'a> {
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
    // |         |
    // | resul 1 | <-- the results are copied to the position immediately following the
    // | resul 0 |     function 1 operands, all data between the results and FP 2 will be removed.
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

    pub context: Context<'a>,
}

#[derive(Debug, PartialEq)]
pub struct ProgramCounter {
    pub instruction_address: usize, // the address of instruction
    pub module_index: usize,        // the module index
}

impl<'a> Thread<'a> {
    pub fn new(module_images: &'a [ModuleImage<'a>]) -> Self {
        let stack = Stack::new(INIT_STACK_SIZE_IN_PAGES);
        let heap = Heap::new(INIT_HEAP_SIZE_IN_PAGES);

        let pc = ProgramCounter {
            instruction_address: 0,
            module_index: 0,
        };

        let context = Context::new(module_images);
        Self {
            stack,
            heap,
            pc,
            context,
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
                ForeignValue::UInt32(value) => self.stack.push_u32(*value),
                ForeignValue::UInt64(value) => self.stack.push_u64(*value),
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
                DataType::I32 => ForeignValue::UInt32(self.stack.pop_u32()),
                DataType::I64 => ForeignValue::UInt64(self.stack.pop_u64()),
                DataType::F32 => ForeignValue::Float32(self.stack.pop_f32()),
                DataType::F64 => ForeignValue::Float64(self.stack.pop_f64()),
            })
            .collect::<Vec<ForeignValue>>();
        reversed_results.reverse();
        reversed_results
    }

    /// get (target_module_index, target_internal_function_index)
    ///
    pub fn get_target_function_module_index_and_internal_index(
        &self,
        module_index: u32,
        func_index: u32,
    ) -> (u32, u32) {
        let range_item = &self.context.func_index_section.ranges[module_index as usize];
        let func_index_items = &self.context.func_index_section.items
            [range_item.offset as usize..(range_item.offset + range_item.count) as usize];
        let func_index_item = &func_index_items[func_index as usize];

        let target_module_index = func_index_item.target_module_index;
        let target_internal_function_index = func_index_item.target_internal_function_index;
        (target_module_index, target_internal_function_index)
    }

    /// get (type_index, code_offset, local_variables_allocate_bytes)
    ///
    pub fn get_internal_function_type_code_and_local_variables_allocate_bytes(
        &self,
        target_module_index: u32,
        target_internal_function_index: u32,
    ) -> (u32, u32, u32) {
        let func_item = &self.context.modules[target_module_index as usize]
            .func_section
            .items[target_internal_function_index as usize];

        let type_index = func_item.type_index;
        let code_offset = func_item.code_offset;

        let local_variables_allocate_bytes = self.context.modules[target_module_index as usize]
            .local_variable_section
            .lists[target_internal_function_index as usize]
            .list_allocate_bytes;

        (type_index, code_offset, local_variables_allocate_bytes)
    }

    /// opcode, or
    /// 16 bits instruction
    /// [opcode]
    pub fn get_opcode(&self) -> u16 {
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
            let p1 = std::ptr::read((&data[2..]).as_ptr() as *const u32);
            (p0, p1)
        }
    }

    /// 96 bits instruction
    /// [opcode + padding + i32 + i32]
    pub fn get_param_i32_i32(&self) -> (u32, u32) {
        let data = self.get_instruction(4, 8);

        unsafe {
            let p0 = std::ptr::read(data.as_ptr() as *const u32);
            let p1 = std::ptr::read((&data[4..]).as_ptr() as *const u32);
            (p0, p1)
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
            module_index,
        } = self.pc;
        let codes_data = self.context.modules[module_index].func_section.codes_data;
        let dst = instruction_address + offset;
        &codes_data[dst..(dst + len_in_bytes)]
    }
}
