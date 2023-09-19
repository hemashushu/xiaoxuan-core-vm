// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_binary::module_image::ModuleImage;
use ancvm_types::{DataType, ForeignValue};

use crate::{
    context::Context, heap::Heap, stack::Stack, INIT_HEAP_SIZE_IN_PAGES, INIT_STACK_SIZE_IN_PAGES, indexed_memory::IndexedMemory,
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

    pub context: Context<'a>,
}

/// unlike the ELF and Linux runting environment, which all data and code of execuatable binary
/// are loaded into one memory space, and the execution state of instruction can be represented
/// by a single number -- program counter (PC) or instruction pointer (IP).
/// XiaoXuan application is consisted of several modules, each module contains its data and code,
/// and code are seperated into several piece which called 'function'.
/// so the PC in XiaoXuan VM is represented by a tuple of
/// (module index, function index, instruction address)
#[derive(Debug, PartialEq)]
pub struct ProgramCounter {
    pub instruction_address: usize,     // the address of instruction
    pub internal_function_index: usize, // the function internal index
    pub module_index: usize,            // the module index
}

impl<'a> Thread<'a> {
    pub fn new(module_images: &'a [ModuleImage<'a>]) -> Self {
        let stack = Stack::new(INIT_STACK_SIZE_IN_PAGES);
        let heap = Heap::new(INIT_HEAP_SIZE_IN_PAGES);

        let pc = ProgramCounter {
            instruction_address: 0,
            internal_function_index: 0,
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

    /// return:
    /// (dyn IndexedMemory, internal_data_index)
    pub fn get_internal_data_index_and_datas_object(
        // thread: &'a mut Thread,
        &mut self,
        data_index: usize,
    ) -> (&mut dyn IndexedMemory, usize) {
        // get the target data item

        // let ProgramCounter {
        //     instruction_address: _instruction_address,
        //     internal_function_index: _internal_function_index,
        //     module_index,
        // } = self.pc;

        let module_index = self.pc.module_index;

        let range = &self.context.data_index_section.ranges[module_index];
        let data_index_item =
            &self.context.data_index_section.items[range.offset as usize + data_index];
        let target_module = &mut self.context.modules[data_index_item.target_module_index as usize];
        let datas = target_module.datas[data_index_item.target_data_section_type as usize].as_mut();
        let internal_data_index = data_index_item.target_data_internal_index;

        (datas, internal_data_index as usize)
    }

    /// return:
    /// (target_module_index, internal_function_index)
    ///
    pub fn get_internal_function_index_and_module_index(
        &self,
        module_index: u32,
        function_index: u32,
    ) -> (u32, u32) {
        let range_item = &self.context.func_index_section.ranges[module_index as usize];
        let func_index_items = &self.context.func_index_section.items
            [range_item.offset as usize..(range_item.offset + range_item.count) as usize];
        let func_index_item = &func_index_items[function_index as usize];

        let target_module_index = func_index_item.target_module_index;
        let internal_function_index = func_index_item.target_internal_function_index;
        (target_module_index, internal_function_index)
    }

    /// return:
    /// (type_index, code_offset, local_variables_allocate_bytes)
    ///
    pub fn get_internal_function_type_and_code_offset_and_local_variables_allocate_bytes(
        &self,
        target_module_index: u32,
        internal_function_index: u32,
    ) -> (u32, u32, u32) {
        let func_item = &self.context.modules[target_module_index as usize]
            .func_section
            .items[internal_function_index as usize];

        let type_index = func_item.type_index;
        let code_offset = func_item.code_offset;

        let local_variables_allocate_bytes = self.context.modules[target_module_index as usize]
            .local_variable_section
            .lists[internal_function_index as usize]
            .list_allocate_bytes;

        (type_index, code_offset, local_variables_allocate_bytes)
    }

    pub fn get_local_variable_address_by_index_and_offset(
        // thread: &Thread,
        &self,
        local_variable_index: usize,
        offset_bytes: usize,
    ) -> usize {
        let local_start_address = self.stack.get_local_variables_start_address();

        // get the local variable info
        let ProgramCounter {
            instruction_address: _instruction_address,
            internal_function_index,
            module_index,
        } = self.pc;

        // let internal_function_index = thread
        //     .stack
        //     .get_function_frame_pack()
        //     .frame_info
        //     .internal_function_index;

        let variable_item = &self.context.modules[module_index]
            .local_variable_section
            .get_variable_list(internal_function_index)[local_variable_index];

        local_start_address + variable_item.var_offset as usize + offset_bytes
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
            internal_function_index,
            module_index,
        } = self.pc;
        let func_item =
            &self.context.modules[module_index].func_section.items[internal_function_index];
        let codes_data = self.context.modules[module_index].func_section.codes_data;
        let dst = instruction_address + func_item.code_offset as usize + offset;
        &codes_data[dst..(dst + len_in_bytes)]
    }
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        load_modules_binary,
        module_image::{
            data_section::{DataEntry, UninitDataEntry},
            local_variable_section::LocalVariableEntry,
        },
        utils::build_module_binary_with_single_function_and_data_sections,
    };
    use ancvm_types::DataType;

    use crate::{
        resizeable_memory::ResizeableMemory,
        thread::{ProgramCounter, Thread},
        INIT_HEAP_SIZE_IN_PAGES, INIT_STACK_SIZE_IN_PAGES,
    };

    #[test]
    fn test_thread_instance() {
        let binary = build_module_binary_with_single_function_and_data_sections(
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
            vec![DataType::I32, DataType::I32],
            vec![DataType::I64],
            vec![0u8],
            vec![LocalVariableEntry::from_i32()],
        );

        let binaries = vec![&binary[..]];
        let module_images = load_modules_binary(binaries).unwrap(); //.expect("module binary error");
        let thread = Thread::new(&module_images);

        let context = &thread.context;

        // check index sections
        assert_eq!(context.module_index_section.items.len(), 1);
        assert_eq!(context.data_index_section.ranges.len(), 1);
        assert_eq!(context.data_index_section.items.len(), 6);
        assert_eq!(context.func_index_section.ranges.len(), 1);
        assert_eq!(context.func_index_section.items.len(), 1);

        assert_eq!(context.modules.len(), 1);
        let module = &context.modules[0];

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
            thread.pc,
            ProgramCounter {
                instruction_address: 0,
                internal_function_index: 0,
                module_index: 0
            }
        );

        // check stack
        assert_eq!(thread.stack.fp, 0);
        assert_eq!(thread.stack.sp, 0);
        assert_eq!(
            thread.stack.get_capacity_in_pages(),
            INIT_STACK_SIZE_IN_PAGES
        );

        // check heap
        assert_eq!(thread.heap.get_capacity_in_pages(), INIT_HEAP_SIZE_IN_PAGES);
    }
}
