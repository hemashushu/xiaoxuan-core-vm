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

    /// note that 'i32' in function name means a 32-bit integer, which is equivalent to
    /// the 'uint32_t' in C or 'u32' in Rust. do not confuse it with 'i32' in Rust.
    /// the same applies to the i8, i16 and i64.
    fn start_opcode_with_i16(self, opcode: Opcode, value: u16) -> Self {
        let mut new_self = self.start_opcode(opcode);
        let data = value.to_le_bytes();
        new_self.buffer.write_all(&data).unwrap();
        new_self
    }

    fn start_opcode_with_16bits_padding(self, opcode: Opcode) -> Self {
        self.start_opcode_with_i16(opcode, 0)
    }

    fn append_i32(mut self, value: u32) -> Self {
        let data = value.to_le_bytes();
        self.buffer.write_all(&data).unwrap();
        self
    }

    fn require_4bytes_padding(self) -> Self {
        if self.buffer.len() % 4 != 0 {
            // insert padding instruction
            self.start_opcode(Opcode::nop)
        } else {
            self
        }
    }

    /// 16-bit instruction
    pub fn write_opcode(self, opcode: Opcode) -> Self {
        self.start_opcode(opcode)
    }

    /// (16+16)-bit instruction
    pub fn write_opcode_i16(self, opcode: Opcode, value: u16) -> Self {
        self.start_opcode_with_i16(opcode, value)
    }

    /// 64-bit instruction
    pub fn write_opcode_i32(self, opcode: Opcode, value: u32) -> Self {
        self.require_4bytes_padding()
            .start_opcode_with_16bits_padding(opcode)
            .append_i32(value)
    }

    /// 64-bit instruction
    pub fn write_opcode_i16_i32(self, opcode: Opcode, param0: u16, param1: u32) -> Self {
        self.require_4bytes_padding()
            .start_opcode_with_i16(opcode, param0)
            .append_i32(param1)
    }

    /// 96-bit instruction
    pub fn write_opcode_i32_i32(self, opcode: Opcode, param0: u32, param1: u32) -> Self {
        self.require_4bytes_padding()
            .start_opcode_with_16bits_padding(opcode)
            .append_i32(param0)
            .append_i32(param1)
    }

    /// 96-bit instruction
    pub fn write_opcode_pesudo_i64(self, opcode: Opcode, value: u64) -> Self {
        let data = value.to_le_bytes();
        let mut new_self = self
            .require_4bytes_padding()
            .start_opcode_with_16bits_padding(opcode);
        new_self.buffer.write_all(&data).unwrap();
        new_self
    }

    /// 64-bit instruction
    pub fn write_opcode_pesudo_f32(self, opcode: Opcode, value: f32) -> Self {
        let data = value.to_le_bytes();
        let mut new_self = self
            .require_4bytes_padding()
            .start_opcode_with_16bits_padding(opcode);
        new_self.buffer.write_all(&data).unwrap();
        new_self
    }

    /// 96-bit instruction
    pub fn write_opcode_pesudo_f64(self, opcode: Opcode, value: f64) -> Self {
        let data = value.to_le_bytes();
        let mut new_self = self
            .require_4bytes_padding()
            .start_opcode_with_16bits_padding(opcode);
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
        data_section::{DataEntry, UninitDataEntry},
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
        build_module_binary_with_single_function_and_data_sections(
            vec![],
            vec![],
            vec![],
            params,
            results,
            codes,
            local_variables,
        )
    }

    pub fn build_module_binary_with_single_function_and_data_sections(
        read_only_datas: Vec<DataEntry>,
        read_write_datas: Vec<DataEntry>,
        uninit_datas: Vec<UninitDataEntry>,
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
    use ancvm_types::{opcode::Opcode, DataType, MemoryDataType};

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
            &vec![VariableItem::new(0, 4, MemoryDataType::I32, 4)]
        );

        /*
         * # check pc
         */

        assert_eq!(
            thread.pc,
            ProgramCounter {
                instruction_address: 0,
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
    fn test_bytecode_writer_schemes() {
        let code0 = BytecodeWriter::new()
            .write_opcode(Opcode::i32_add)
            .to_bytes();

        assert_eq!(code0, vec![0x00, 0x08]);

        let code1 = BytecodeWriter::new()
            .write_opcode_i16(Opcode::i32_shl, 7)
            .to_bytes();

        assert_eq!(
            code1,
            vec![
                0x07, 0x09, // opcode
                07, 0, // param
            ]
        );

        let code2 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::block, 11)
            .to_bytes();

        assert_eq!(
            code2,
            vec![
                0x01, 0x0b, // opcode
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
                0x02, 0x0b, // opcode
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
                0x04, 0x0b, // opcode
                0, 0, // padding
                19, 0, 0, 0, // param 0
                23, 0, 0, 0 // param 1
            ]
        );

        let code5 = BytecodeWriter::new()
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x1122334455667788u64)
            .to_bytes();

        assert_eq!(
            code5,
            vec![
                0x01, 0x02, // opcode
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
                0x03, 0x02, // opcode
                0, 0, // padding
                0x11, 0x31, 0x02, 0xde, // param 0
                0x0b, 0x86, 0x0b, 0x39, // param 1
            ]
        );
    }

    #[test]
    fn test_bytecode_writer_padding() {
        let code0 = BytecodeWriter::new()
            .write_opcode(Opcode::i32_add)
            .write_opcode_i16(Opcode::i32_shl, 0x5)
            .write_opcode_i16(Opcode::i32_shr_s, 0x7)
            // padding
            .write_opcode_i16_i32(Opcode::local_load, 0x11, 0x13)
            .write_opcode_i16_i32(Opcode::local_store, 0x17, 0x19)
            .write_opcode(Opcode::i32_sub)
            .write_opcode(Opcode::i32_mul)
            .write_opcode_i16_i32(Opcode::local_load, 0x23, 0x29)
            .write_opcode_i16_i32(Opcode::local_store, 0x31, 0x37)
            .write_opcode(Opcode::i32_div_s)
            // padding
            .write_opcode_i32(Opcode::block, 0x41)
            .write_opcode_i32(Opcode::call, 0x43)
            .write_opcode(Opcode::i32_div_u)
            // padding
            .write_opcode_i32_i32(Opcode::i64_imm, 0x47, 0x53)
            .write_opcode_i32_i32(Opcode::block_nez, 0x59, 0x61)
            .to_bytes();

        assert_eq!(
            code0,
            vec![
                // i32_add
                0x00, 0x08, //
                // i32_shl 0x5
                0x07, 0x09, //
                0x5, 0, //
                // i32_shr_s 0x7
                0x08, 0x09, //
                0x7, 0, //
                // padding nop
                0x00, 0x01, //
                // local_load 0x11 0x13
                0x00, 0x03, //
                0x11, 0x00, //
                0x13, 0x00, 0x00, 0x00, //
                // local_store 0x17 0x19
                0x08, 0x03, //
                0x17, 0x00, //
                0x19, 0x00, 0x00, 0x00, //
                // i32_sub
                0x01, 0x08, //
                // i32_mul
                0x02, 0x08, //
                // local_load 0x23 0x29
                0x00, 0x03, //
                0x23, 0x00, //
                0x29, 0x00, 0x00, 0x00, //
                // local_store 0x31 0x37
                0x08, 0x03, //
                0x31, 0x00, //
                0x37, 0x00, 0x00, 0x00, //
                // i32_div_s
                0x03, 0x08, //
                // padding nop
                0x00, 0x01, //
                // block 0x41
                0x01, 0x0b, //
                0x00, 0x00, //
                0x41, 0x00, 0x00, 0x00, //
                // call 0x43
                0x00, 0x0c, //
                0x00, 0x00, //
                0x43, 0x00, 0x00, 0x00, //
                // i32_div_u
                0x04, 0x08, //
                // padding nop
                0x00, 0x01, //
                // i64_imm 0x47 0x53
                0x01, 0x02, //
                0x00, 0x00, //
                0x47, 0x00, 0x00, 0x00, //
                0x53, 0x00, 0x00, 0x00, //
                // block_nez 0x59 0x61
                0x04, 0x0b, //
                0x00, 0x00, //
                0x59, 0x00, 0x00, 0x00, //
                0x61, 0x00, 0x00, 0x00, //
            ]
        );
    }

    union AB {
        a:DataType,
        b:u8
    }

    #[test]
    fn test_union(){
        let b = unsafe {
            AB{b:3}.a
        };
        println!("{:?}", b);
    }
}
