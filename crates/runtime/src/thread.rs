// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_binary::module_image::ModuleImage;
use ancvm_types::ForeignValue;

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
    // |   $$$   | <-- the stack frame information, includes the previous FP, return address (instruction addr and module index),
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
    pub addr: usize,       // the address of instruction
    pub module_index: u32, // the module index
}

impl<'a> Thread<'a> {
    pub fn new(module_images: &'a [ModuleImage<'a>]) -> Self {
        let stack = Stack::new(INIT_STACK_SIZE_IN_PAGES);
        let heap = Heap::new(INIT_HEAP_SIZE_IN_PAGES);

        let pc = ProgramCounter {
            addr: 0,
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

    pub fn push_values(&mut self, values: Vec<ForeignValue>) {
        for value in values {
            match value {
                ForeignValue::I32(value) => self.stack.push_i32(value),
                ForeignValue::I64(value) => self.stack.push_i64(value),
                ForeignValue::F32(value) => self.stack.push_f32(value),
                ForeignValue::F64(value) => self.stack.push_f64(value),
            }
        }
    }

    // pop up the return values of the current function
    pub fn pop_values(&mut self) -> Vec<ForeignValue> {
        // ensure the current frame is 'functon frame'
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        load_modules_binary,
        module_image::{
            func_index_section::{FuncIndexItem, FuncIndexSection},
            func_section::{FuncEntry, FuncSection},
            local_variable_section::{LocalVariableSection, VariableItem, VariableItemEntry},
            module_index_section::{ModuleIndexEntry, ModuleIndexSection, ModuleShareType},
            type_section::{TypeEntry, TypeSection},
            ModuleImage, RangeItem, SectionEntry,
        },
    };
    use ancvm_types::DataType;

    use crate::thread::{ProgramCounter, Thread};

    fn build_module_binary_with_single_function(
        params: Vec<DataType>,
        results: Vec<DataType>,
        codes: Vec<u8>,
        locals: Vec<VariableItemEntry>,
    ) -> Vec<u8> {
        // build type section
        let mut type_entries: Vec<TypeEntry> = Vec::new();
        type_entries.push(TypeEntry {
            params: params,
            results: results,
        });
        let (type_items, types_data) = TypeSection::convert_from_entries(&type_entries);
        let type_section = TypeSection {
            items: &type_items,
            types_data: &types_data,
        };

        // build function section
        let mut func_entries: Vec<FuncEntry> = Vec::new();
        func_entries.push(FuncEntry {
            func_type: 0,
            code: codes,
        });
        let (func_items, codes_data) = FuncSection::convert_from_entries(&func_entries);
        let func_section = FuncSection {
            items: &func_items,
            codes_data: &codes_data,
        };

        // build local variable section
        let mut local_var_entries: Vec<Vec<VariableItemEntry>> = Vec::new();
        local_var_entries.push(locals);

        let (local_var_lists, local_var_list_data) =
            LocalVariableSection::convert_from_entries(&local_var_entries);
        let local_var_section = LocalVariableSection {
            lists: &local_var_lists,
            list_data: &local_var_list_data,
        };

        // build module index
        let mut mod_index_entries: Vec<ModuleIndexEntry> = Vec::new();
        mod_index_entries.push(ModuleIndexEntry::new(
            ModuleShareType::Local,
            "main".to_string(),
        ));
        let (module_index_items, names_data) =
            ModuleIndexSection::convert_from_entries(&mod_index_entries);
        let mod_index_section = ModuleIndexSection {
            items: &module_index_items,
            names_data: &names_data,
        };

        // build function index
        let mut func_ranges: Vec<RangeItem> = Vec::new();
        let mut func_index_items: Vec<FuncIndexItem> = Vec::new();

        func_ranges.push(RangeItem {
            offset: 0,
            count: 1,
        });
        func_index_items.push(FuncIndexItem::new(0, 0, 0));
        let func_index_section = FuncIndexSection {
            ranges: &func_ranges,
            items: &func_index_items,
        };

        // build module image
        let section_entries: Vec<&dyn SectionEntry> = vec![
            &type_section,
            &func_section,
            &mod_index_section,
            &func_index_section,
            &local_var_section,
        ];
        let (section_items, sections_data) = ModuleImage::convert_from_entries(&section_entries);
        let module_image = ModuleImage {
            items: &section_items,
            sections_data: &sections_data,
        };

        // build module image binary
        let mut image_data: Vec<u8> = Vec::new();
        module_image.save(&mut image_data).unwrap();

        image_data
    }

    #[test]
    fn test_single_function_thread() {
        let binary = build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32],
            vec![DataType::I64],
            vec![0u8],
            vec![VariableItemEntry::from_i32()],
        );

        let binaries = vec![&binary[..]];
        let module_images = load_modules_binary(binaries).expect("module binary error");
        let thread = Thread::new(&module_images);

        // start checking

        /*
         * # check context
         */

        let context = &thread.context;

        /*
         * ## check index sections
         */

        // check "module index section"
        assert_eq!(context.module_index_section.items.len(), 1);

        let module_index_entry = context.module_index_section.get_entry(0);
        assert_eq!(module_index_entry.name, "main".to_string());
        assert_eq!(module_index_entry.module_share_type, ModuleShareType::Local);

        // check "data index section"
        assert_eq!(context.data_index_section.items.len(), 0);

        // check "function index section"
        assert_eq!(context.func_index_section.ranges.len(), 1);
        assert_eq!(context.func_index_section.items.len(), 1);

        assert_eq!(&context.func_index_section.ranges[0], &RangeItem::new(0, 1));

        assert_eq!(
            &context.func_index_section.items[0],
            &FuncIndexItem::new(0, 0, 0)
        );

        /*
         * ## check modules
         */

        assert_eq!(context.modules.len(), 1);

        let module = &context.modules[0];
        assert_eq!(module.datas.len(), 3);

        // check "type section"
        assert_eq!(module.type_section.items.len(), 1);
        assert_eq!(
            module.type_section.get_entry(0),
            TypeEntry {
                params: vec![DataType::I32, DataType::I32],
                results: vec![DataType::I64]
            }
        );

        // check "func section"
        assert_eq!(module.func_section.items.len(), 1);
        assert_eq!(
            module.func_section.get_entry(0),
            FuncEntry {
                func_type: 0,
                code: vec![0u8]
            }
        );

        // check "local variable section"
        assert_eq!(module.local_variable_section.lists.len(), 1);
        assert_eq!(
            module.local_variable_section.get_variable_list(0),
            &vec![VariableItem::new(0, 4, DataType::I32, 4)]
        );

        /*
         * # check pc
         */

        assert_eq!(
            thread.pc,
            ProgramCounter {
                addr: 0,
                module_index: 0
            }
        );

        /*
         * ## check stack
         */

        // TODO::

        /*
         * ## check heap
         */

        // TODO::
    }
}
