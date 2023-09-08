// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::io::Write;

use ancvm_types::opcode::Opcode;

pub struct BytecodeWriter {
    buffer: Vec<u8>, // trait std::io::Write
}

impl BytecodeWriter {
    pub fn new() -> Self {
        Self {
            buffer: Vec::<u8>::new(),
        }
    }

    fn start_opcode(mut self, opcode: Opcode) -> Self {
        let data = (opcode as u16).to_le_bytes();
        self.buffer.write_all(&data).unwrap();
        self
    }

    fn start_opcode_with_i16(self, opcode: Opcode, value: i16) -> Self {
        let mut new_self = self.start_opcode(opcode);
        let data = value.to_le_bytes();
        new_self.buffer.write_all(&data).unwrap();
        new_self
    }

    fn start_opcode_with_16bits_padding(self, opcode: Opcode) -> Self {
        self.start_opcode_with_i16(opcode, 0)
    }

    fn append_i32(mut self, value: i32) -> Self {
        let data = value.to_le_bytes();
        self.buffer.write_all(&data).unwrap();
        self
    }

    pub fn write_opcode(self, opcode: Opcode) -> Self {
        self.start_opcode(opcode)
    }

    pub fn write_opcode_i16(self, opcode: Opcode, value: i16) -> Self {
        self.start_opcode_with_i16(opcode, value)
    }

    pub fn write_opcode_i32(self, opcode: Opcode, value: i32) -> Self {
        self.start_opcode_with_16bits_padding(opcode)
            .append_i32(value)
    }

    pub fn write_opcode_i16_i32(self, opcode: Opcode, param0: i16, param1: i32) -> Self {
        self.start_opcode_with_i16(opcode, param0)
            .append_i32(param1)
    }

    pub fn write_opcode_i32_i32(self, opcode: Opcode, param0: i32, param1: i32) -> Self {
        self.start_opcode_with_16bits_padding(opcode)
            .append_i32(param0)
            .append_i32(param1)
    }

    pub fn write_opcode_pesudo_i64(self, opcode: Opcode, value: i64) -> Self {
        let data = value.to_le_bytes();
        let mut new_self = self.start_opcode_with_16bits_padding(opcode);
        new_self.buffer.write_all(&data).unwrap();
        new_self
    }

    pub fn write_opcode_pesudo_f64(self, opcode: Opcode, value: f64) -> Self {
        let data = value.to_le_bytes();
        let mut new_self = self.start_opcode_with_16bits_padding(opcode);
        new_self.buffer.write_all(&data).unwrap();
        new_self
    }

    pub fn to_bytes(self) -> Vec<u8> {
        self.buffer
    }

    pub fn save_bytecodes(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        writer.write_all(&self.buffer)
    }
}

#[cfg(test)]
pub mod test_helper {
    use ancvm_binary::module_image::{
        func_index_section::{FuncIndexItem, FuncIndexSection},
        func_section::{FuncEntry, FuncSection},
        local_variable_section::{LocalVariableSection, VariableItemEntry},
        module_index_section::{ModuleIndexEntry, ModuleIndexSection, ModuleShareType},
        type_section::{TypeEntry, TypeSection},
        ModuleImage, RangeItem, SectionEntry,
    };
    use ancvm_types::DataType;

    pub fn build_module_binary_with_single_function(
        params: Vec<DataType>,
        results: Vec<DataType>,
        codes: Vec<u8>,
        local_variables: Vec<VariableItemEntry>,
    ) -> Vec<u8> {
        // build type section
        let mut type_entries: Vec<TypeEntry> = Vec::new();
        type_entries.push(TypeEntry { params, results });
        let (type_items, types_data) = TypeSection::convert_from_entries(&type_entries);
        let type_section = TypeSection {
            items: &type_items,
            types_data: &types_data,
        };

        // build function section
        let mut func_entries: Vec<FuncEntry> = Vec::new();
        func_entries.push(FuncEntry {
            type_index: 0,
            code: codes,
        });
        let (func_items, codes_data) = FuncSection::convert_from_entries(&func_entries);
        let func_section = FuncSection {
            items: &func_items,
            codes_data: &codes_data,
        };

        // build local variable section
        let mut local_var_entries: Vec<Vec<VariableItemEntry>> = Vec::new();
        local_var_entries.push(local_variables);

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

        // ignore the data index

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
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        load_modules_binary,
        module_image::{
            func_index_section::FuncIndexItem,
            func_section::FuncEntry,
            local_variable_section::{VariableItem, VariableItemEntry},
            module_index_section::ModuleShareType,
            type_section::TypeEntry,
            RangeItem,
        },
    };
    use ancvm_types::{opcode::Opcode, DataType};

    use crate::{
        resizeable_memory::ResizeableMemory,
        thread::{ProgramCounter, Thread},
        utils::test_helper::build_module_binary_with_single_function,
        INIT_HEAP_SIZE_IN_PAGES, INIT_STACK_SIZE_IN_PAGES,
    };

    use super::BytecodeWriter;

    #[test]
    fn test_build_single_function_module_binary() {
        let binary = build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32],
            vec![DataType::I64],
            vec![0u8],
            vec![VariableItemEntry::from_i32()],
        );

        let binaries = vec![&binary[..]];
        let module_images = load_modules_binary(binaries).unwrap(); //.expect("module binary error");
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
                type_index: 0,
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

        assert_eq!(thread.stack.fp, 0);
        assert_eq!(thread.stack.sp, 0);
        assert_eq!(
            thread.stack.get_capacity_in_pages(),
            INIT_STACK_SIZE_IN_PAGES
        );

        /*
         * ## check heap
         */
        assert_eq!(thread.heap.get_capacity_in_pages(), INIT_HEAP_SIZE_IN_PAGES);
    }

    #[test]
    fn test_bytecode_writer() {
        let code0 = BytecodeWriter::new()
            .write_opcode(Opcode::i32_add)
            .to_bytes();

        assert_eq!(code0, vec![0x00, 0x07]);

        let code1 = BytecodeWriter::new()
            .write_opcode_i16(Opcode::i32_shl, 7)
            .to_bytes();

        assert_eq!(
            code1,
            vec![
                0x07, 0x08, // opcode
                07, 0, // param
            ]
        );

        let code2 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::block, 11)
            .to_bytes();

        assert_eq!(
            code2,
            vec![
                0x01, 0x10, // opcode
                0, 0, // padding
                11, 0, 0, 0 // param
            ]
        );

        let code3 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::return_, 13, 17)
            .to_bytes();

        assert_eq!(
            code3,
            vec![
                0x02, 0x10, // opcode
                13, 0, // param 0
                17, 0, 0, 0 // param 1
            ]
        );

        let code4 = BytecodeWriter::new()
            .write_opcode_i32_i32(Opcode::block_nez, 19, 23)
            .to_bytes();

        assert_eq!(
            code4,
            vec![
                0x04, 0x10, // opcode
                0, 0, // padding
                19, 0, 0, 0, // param 0
                23, 0, 0, 0 // param 1
            ]
        );

        let code5 = BytecodeWriter::new()
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x1122334455667788i64)
            .to_bytes();

        assert_eq!(
            code5,
            vec![
                0x01, 0x01, // opcode
                0, 0, // padding
                0x88, 0x77, 0x66, 0x55, // param 0
                0x44, 0x33, 0x22, 0x11 // param 1
            ]
        );

        let code6 = BytecodeWriter::new()
            .write_opcode_pesudo_f64(Opcode::f64_imm, 6.62607015e-34f64)
            .to_bytes();

        // 6.62607015e-34f64 (dec) -> 0x390B860B DE023111 (hex)

        assert_eq!(
            code6,
            vec![
                0x03, 0x01, // opcode
                0, 0, // padding
                0x11, 0x31, 0x02, 0xde, // param 0
                0x0b, 0x86, 0x0b, 0x39, // param 1
            ]
        );
    }
}
