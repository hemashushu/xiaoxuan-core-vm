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

pub struct BytecodeReader<'a> {
    codes: &'a [u8],
    offset: usize,
}

impl<'a> BytecodeReader<'a> {
    pub fn new(codes: &'a [u8]) -> Self {
        Self { codes, offset: 0 }
    }

    /// opcode, or
    /// 16 bits instruction
    /// [opcode]
    fn read_opcode(&mut self) -> Opcode {
        let opcode_data = &self.codes[self.offset..self.offset + 2];
        self.offset += 2;

        let opcode_u16 = u16::from_le_bytes(opcode_data.try_into().unwrap());
        unsafe { std::mem::transmute::<u16, Opcode>(opcode_u16) }
    }

    /// 32 bits instruction
    /// [opcode + i16]
    fn read_param_i16(&mut self) -> u16 {
        let param_data0 = &self.codes[self.offset..self.offset + 2];
        self.offset += 2;

        u16::from_le_bytes(param_data0.try_into().unwrap())
    }

    /// 64 bits instruction
    /// [opcode + padding + i32]
    ///
    /// note that 'i32' in function name means a 32-bit integer, which is equivalent to
    /// the 'uint32_t' in C or 'u32' in Rust. do not confuse it with 'i32' in Rust.
    /// the same applies to the i8, i16 and i64.
    fn read_param_i32(&mut self) -> u32 {
        let param_data0 = &self.codes[self.offset + 2..self.offset + 2 + 4];
        self.offset += 6;

        u32::from_le_bytes(param_data0.try_into().unwrap())
    }

    /// 64 bits instruction
    /// [opcode + i16 + i32]
    fn read_param_i16_i32(&mut self) -> (u16, u32) {
        let param_data0 = &self.codes[self.offset..self.offset + 2];
        let param_data1 = &self.codes[self.offset + 2..self.offset + 2 + 4];
        self.offset += 6;

        (
            u16::from_le_bytes(param_data0.try_into().unwrap()),
            u32::from_le_bytes(param_data1.try_into().unwrap()),
        )
    }

    /// 96 bits instruction
    /// [opcode + padding + i32 + i32]
    fn read_param_i32_i32(&mut self) -> (u32, u32) {
        let param_data0 = &self.codes[self.offset + 2..self.offset + 2 + 4];
        let param_data1 = &self.codes[self.offset + 2 + 4..self.offset + 2 + 4 + 4];
        self.offset += 10;

        (
            u32::from_le_bytes(param_data0.try_into().unwrap()),
            u32::from_le_bytes(param_data1.try_into().unwrap()),
        )
    }

    pub fn to_text(&mut self) -> String {
        let mut lines: Vec<String> = Vec::new();

        let code_len = self.codes.len();
        loop {
            let offset = self.offset;
            if offset == code_len {
                break;
            };

            let opcode = self.read_opcode();

            // format!(...)
            // https://doc.rust-lang.org/std/fmt/
            let mut line = format!("0x{:04x} {:20} ", offset, opcode.get_name());

            match opcode {
                Opcode::nop | Opcode::drop | Opcode::duplicate => {}
                //
                Opcode::i32_imm | Opcode::f32_imm => {
                    let p = self.read_param_i32();
                    line.push_str(&format!("0x{:x}", p));
                }
                Opcode::i64_imm | Opcode::f64_imm => {
                    let (p_low, p_high) = self.read_param_i32_i32();
                    line.push_str(&format!("0x{:x} 0x{:x}", p_low, p_high));
                }
                //
                Opcode::local_load
                | Opcode::local_load32
                | Opcode::local_load32_i16_s
                | Opcode::local_load32_i16_u
                | Opcode::local_load32_i8_s
                | Opcode::local_load32_i8_u
                | Opcode::local_load_f64
                | Opcode::local_load32_f32
                | Opcode::local_store
                | Opcode::local_store32
                | Opcode::local_store16
                | Opcode::local_store8 => {
                    let (offset, index) = self.read_param_i16_i32();
                    line.push_str(&format!("{} {}", offset, index));
                }
                //
                Opcode::data_load
                | Opcode::data_load32
                | Opcode::data_load32_i16_s
                | Opcode::data_load32_i16_u
                | Opcode::data_load32_i8_s
                | Opcode::data_load32_i8_u
                | Opcode::data_load_f64
                | Opcode::data_load32_f32
                | Opcode::data_store
                | Opcode::data_store32
                | Opcode::data_store16
                | Opcode::data_store8 => {
                    let (offset, index) = self.read_param_i16_i32();
                    line.push_str(&format!("{} {}", offset, index));
                }
                //
                Opcode::end => {}
                //
                _ => unreachable!(),
            }

            let line_clear = line.trim_end().to_string();
            lines.push(line_clear);
        }

        lines.join("\n")
    }
}

#[cfg(test)]
pub mod test_helper {
    use ancvm_binary::module_image::{
        data_index_section::{DataIndexItem, DataIndexSection},
        data_section::{
            DataEntry, DataSectionType, ReadOnlyDataSection, ReadWriteDataSection, UninitDataEntry,
            UninitDataSection,
        },
        func_index_section::{FuncIndexItem, FuncIndexSection},
        func_section::{FuncEntry, FuncSection},
        local_variable_section::{LocalVariableSection, VariableItemEntry},
        module_index_section::{ModuleIndexEntry, ModuleIndexSection, ModuleShareType},
        type_section::{TypeEntry, TypeSection},
        ModuleImage, RangeItem, SectionEntry,
    };
    use ancvm_types::DataType;

    pub fn build_module_binary_with_single_function(
        param_datatypes: Vec<DataType>,
        result_datatypes: Vec<DataType>,
        codes: Vec<u8>,
        local_variable_item_entries: Vec<VariableItemEntry>,
    ) -> Vec<u8> {
        build_module_binary_with_single_function_and_data_sections(
            vec![],
            vec![],
            vec![],
            param_datatypes,
            result_datatypes,
            codes,
            local_variable_item_entries,
        )
    }

    pub fn build_module_binary_with_single_function_and_data_sections(
        read_only_data_entries: Vec<DataEntry>,
        read_write_data_entries: Vec<DataEntry>,
        uninit_uninit_data_entries: Vec<UninitDataEntry>,
        param_datatypes: Vec<DataType>,
        result_datatypes: Vec<DataType>,
        codes: Vec<u8>,
        local_variable_item_entries: Vec<VariableItemEntry>,
    ) -> Vec<u8> {
        // build read-only data section
        let (ro_items, ro_data) =
            ReadOnlyDataSection::convert_from_entries(&read_only_data_entries);
        let ro_data_section = ReadOnlyDataSection {
            items: &ro_items,
            datas_data: &ro_data,
        };

        // build read-write data section
        let (rw_items, rw_data) =
            ReadWriteDataSection::convert_from_entries(&read_write_data_entries);
        let rw_data_section = ReadWriteDataSection {
            items: &rw_items,
            datas_data: &rw_data,
        };

        // build uninitilized data section
        let uninit_items = UninitDataSection::convert_from_entries(&uninit_uninit_data_entries);
        let uninit_data_section = UninitDataSection {
            items: &uninit_items,
        };

        // build type section
        let mut type_entries: Vec<TypeEntry> = Vec::new();
        type_entries.push(TypeEntry {
            params: param_datatypes,
            results: result_datatypes,
        });
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
        local_var_entries.push(local_variable_item_entries);

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

        // build data index

        // the data index is ordered by:
        // 1. imported ro data
        // 2. ro data
        // 3. imported rw data
        // 4. rw data
        // 5. imported uninit data
        // 6. uninit data
        let mut data_ranges: Vec<RangeItem> = Vec::new();
        let mut data_index_items: Vec<DataIndexItem> = Vec::new();

        data_ranges.push(RangeItem {
            offset: 0,
            count: (ro_items.len() + rw_items.len() + uninit_items.len()) as u32,
        });

        let ro_iter = ro_items
            .iter()
            .enumerate()
            .map(|(idx, _item)| (idx, DataSectionType::ReadOnly));
        let rw_iter = rw_items
            .iter()
            .enumerate()
            .map(|(idx, _item)| (idx, DataSectionType::ReadWrite));
        let uninit_iter = uninit_items
            .iter()
            .enumerate()
            .map(|(idx, _item)| (idx, DataSectionType::Uninit));
        for (total_data_index, (idx, data_section_type)) in
            ro_iter.chain(rw_iter).chain(uninit_iter).enumerate()
        {
            data_index_items.push(DataIndexItem::new(
                total_data_index as u32,
                0,
                data_section_type,
                idx as u32,
            ));
        }

        let data_index_section = DataIndexSection {
            ranges: &data_ranges,
            items: &data_index_items,
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
            &mod_index_section,
            &data_index_section,
            &func_index_section,
            &ro_data_section,
            &rw_data_section,
            &uninit_data_section,
            &type_section,
            &func_section,
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
            data_index_section::DataIndexItem,
            data_section::{DataEntry, DataSectionType, UninitDataEntry},
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
        utils::test_helper::build_module_binary_with_single_function_and_data_sections,
        INIT_HEAP_SIZE_IN_PAGES, INIT_STACK_SIZE_IN_PAGES,
    };

    use super::BytecodeWriter;

    #[test]
    fn test_build_single_function_module_binary_and_data_sections() {
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

        assert_eq!(context.data_index_section.ranges.len(), 1);
        assert_eq!(context.data_index_section.items.len(), 6);

        assert_eq!(&context.data_index_section.ranges[0], &RangeItem::new(0, 6));

        assert_eq!(
            context.data_index_section.items,
            // 2,1,3
            &vec![
                //
                DataIndexItem::new(0, 0, DataSectionType::ReadOnly, 0),
                DataIndexItem::new(1, 0, DataSectionType::ReadOnly, 1),
                //
                DataIndexItem::new(2, 0, DataSectionType::ReadWrite, 0),
                //
                DataIndexItem::new(3, 0, DataSectionType::Uninit, 0),
                DataIndexItem::new(4, 0, DataSectionType::Uninit, 1),
                DataIndexItem::new(5, 0, DataSectionType::Uninit, 2),
            ]
        );

        // check "function index section"

        assert_eq!(context.func_index_section.ranges.len(), 1);
        assert_eq!(context.func_index_section.items.len(), 1);

        assert_eq!(&context.func_index_section.ranges[0], &RangeItem::new(0, 1));

        assert_eq!(
            context.func_index_section.items,
            &vec![FuncIndexItem::new(0, 0, 0)]
        );

        /*
         * ## check modules
         */

        assert_eq!(context.modules.len(), 1);

        let module = &context.modules[0];

        // check "data section" (s)

        assert_eq!(module.datas.len(), 3);

        let mut data = [0u8; 8];
        let dst_ptr = &mut data as *mut [u8] as *mut u8;

        // let ro_datas = unsafe {
        //     &*(module.datas[0].as_ref() as *const dyn IndexedMemory as *const ReadOnlyDatas)
        // };
        let ro_datas = &module.datas[0];
        ro_datas.load_idx_32(0, 0, dst_ptr);
        assert_eq!(&data[0..4], [0x11, 0, 0, 0]);
        ro_datas.load_idx_64(1, 0, dst_ptr);
        assert_eq!(data, [0x13, 0, 0, 0, 0, 0, 0, 0]);

        let rw_datas = &module.datas[1];
        rw_datas.load_idx_32(0, 0, dst_ptr);
        assert_eq!(&data[0..4], &[0x17u8, 0x19, 0x23, 0x29]);
        rw_datas.load_idx_32_extend_from_u16(0, 4, dst_ptr);
        assert_eq!(&data[0..2], &[0x31u8, 0x37]);

        let uninit_datas = &module.datas[2];
        uninit_datas.load_idx_32(0, 0, dst_ptr);
        assert_eq!(&data[0..4], &[0x0u8, 0, 0, 0]);
        uninit_datas.load_idx_64(1, 0, dst_ptr);
        assert_eq!(data, [0x0u8, 0, 0, 0, 0, 0, 0, 0]);
        uninit_datas.load_idx_32(2, 0, dst_ptr);
        assert_eq!(&data[0..4], &[0x0u8, 0, 0, 0]);

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

        assert_eq!(code0, vec![0x00, 0x09]);

        let code1 = BytecodeWriter::new()
            .write_opcode_i16(Opcode::i32_shl, 7)
            .to_bytes();

        assert_eq!(
            code1,
            vec![
                0x07, 0x0a, // opcode
                07, 0, // param
            ]
        );

        let code2 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::block, 11)
            .to_bytes();

        assert_eq!(
            code2,
            vec![
                0x01, 0x0c, // opcode
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
                0x02, 0x0c, // opcode
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
                0x04, 0x0c, // opcode
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
                0x00, 0x09, //
                // i32_shl 0x5
                0x07, 0x0a, //
                0x5, 0, //
                // i32_shr_s 0x7
                0x08, 0x0a, //
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
                0x01, 0x09, //
                // i32_mul
                0x02, 0x09, //
                // local_load 0x23 0x29
                0x00, 0x03, //
                0x23, 0x00, //
                0x29, 0x00, 0x00, 0x00, //
                // local_store 0x31 0x37
                0x08, 0x03, //
                0x31, 0x00, //
                0x37, 0x00, 0x00, 0x00, //
                // i32_div_s
                0x03, 0x09, //
                // padding nop
                0x00, 0x01, //
                // block 0x41
                0x01, 0x0c, //
                0x00, 0x00, //
                0x41, 0x00, 0x00, 0x00, //
                // call 0x43
                0x00, 0x0d, //
                0x00, 0x00, //
                0x43, 0x00, 0x00, 0x00, //
                // i32_div_u
                0x04, 0x09, //
                // padding nop
                0x00, 0x01, //
                // i64_imm 0x47 0x53
                0x01, 0x02, //
                0x00, 0x00, //
                0x47, 0x00, 0x00, 0x00, //
                0x53, 0x00, 0x00, 0x00, //
                // block_nez 0x59 0x61
                0x04, 0x0c, //
                0x00, 0x00, //
                0x59, 0x00, 0x00, 0x00, //
                0x61, 0x00, 0x00, 0x00, //
            ]
        );
    }
}
