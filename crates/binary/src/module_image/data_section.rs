// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// "read-only/read-write data section" binary layout
//
//              |------------------------------------------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                                                                 |
//              |------------------------------------------------------------------------------------------------------|
//  item 0 -->  | data offset 0 (u32) | data length 0 (u32) | mem data type 0 (u8) | pad (1 byte) | data align 0 (u16) | <-- table
//  item 1 -->  | data offset 1       | data length 1       | mem data type 1      |              | data align 1       |
//              | ...                                                                                                  |
//              |------------------------------------------------------------------------------------------------------|
// offset 0 --> | data 0 | data 1 | ...                                                                                | <-- data area
//              |          ^                                                                                           |
// offset 1 ----|----------|                                                                                           |
//              |------------------------------------------------------------------------------------------------------|
//
//
// "uninit data section" binary layout
//
//              |------------------------------------------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                                                                 |
//              |------------------------------------------------------------------------------------------------------|
//  item 0 -->  | data offset 0 (u32) | data length 0 (u32) | mem data type 0 (u8) | pad (1 byte) | data align 0 (u16) | <-- table
//  item 1 -->  | data offset 1       | data length 1       | mem data type 1      |              | data align 1       |
//              | ...                                                                                                  |
//              |------------------------------------------------------------------------------------------------------|
//
// data size and alignment
//
// | type  | size | alignment |
// |-------|------|-----------|
// | i32   | 4    | 4         |
// | i64   | 8    | 8         |
// | f32   | 4    | 4         |
// | f64   | 8    | 8         |
// | bytes | -    | -         |
//
// when storing "struct" data, the data type "byte" should be used, as well as
// the alignment should be speicified.

use std::fmt::Display;

use ancvm_types::MemoryDataType;

use crate::utils::{
    load_section_with_one_table, load_section_with_table_and_data_area,
    save_section_with_one_table, save_section_with_table_and_data_area,
};

use super::{ModuleSectionId, SectionEntry};

#[derive(Debug, PartialEq)]
pub struct ReadOnlyDataSection<'a> {
    pub items: &'a [DataItem],
    pub datas_data: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct ReadWriteDataSection<'a> {
    pub items: &'a [DataItem],
    pub datas_data: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct UninitDataSection<'a> {
    pub items: &'a [DataItem],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct DataItem {
    pub data_offset: u32, // the offset of a data item in the section's "data area"
    pub data_length: u32, // the length (in bytes) of a data item in the section's "data area"

    // the data type field is not necessary at runtime, though it is helpful for debugging.
    pub memory_data_type: MemoryDataType,

    _padding0: u8,

    // the align field is not necessary for data loading and storing, because the value of 'data_offset'
    // is aligned at compilation time, it is only needed when copying data into other memory, such as
    // copying a struct from data section into heap.
    //
    // the value of this field should not be '0'
    pub data_align: u16,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataSectionType {
    ReadOnly = 0x0,
    ReadWrite,
    Uninit,
}

impl Display for DataSectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            DataSectionType::ReadOnly => "read-only",
            DataSectionType::ReadWrite => "read-write",
            DataSectionType::Uninit => "unitialized",
        };
        f.write_str(name)
    }
}

#[derive(Debug)]
pub struct DataEntry {
    pub memory_data_type: MemoryDataType,
    pub data: Vec<u8>,
    pub length: u32,
    pub align: u16, // should not be '0'
}

#[derive(Debug)]
pub struct UninitDataEntry {
    pub memory_data_type: MemoryDataType,
    pub length: u32,
    pub align: u16, // should not be '0'
}

impl DataEntry {
    /// note that 'i32' in function name means a 32-bit integer, which is equivalent to
    /// the 'uint32_t' in C or 'u32' in Rust. do not confuse it with 'i32' in Rust.
    /// the same applies to the i8, i16 and i64.
    pub fn from_i32(value: u32) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(8);
        data.extend(value.to_le_bytes().iter());

        Self {
            memory_data_type: MemoryDataType::I32,
            data,
            length: 4,
            align: 4,
        }
    }

    pub fn from_i64(value: u64) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(8);
        data.extend(value.to_le_bytes().iter());

        Self {
            memory_data_type: MemoryDataType::I64,
            data,
            length: 8,
            align: 8,
        }
    }

    pub fn from_f32(value: f32) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(8);
        data.extend(value.to_le_bytes().iter());

        Self {
            memory_data_type: MemoryDataType::F32,
            data,
            length: 4,
            align: 4,
        }
    }

    pub fn from_f64(value: f64) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(8);
        data.extend(value.to_le_bytes().iter());

        Self {
            memory_data_type: MemoryDataType::F64,
            data,
            length: 8,
            align: 8,
        }
    }

    pub fn from_bytes(data: Vec<u8>, align: u16) -> Self {
        let length = data.len() as u32;

        Self {
            memory_data_type: MemoryDataType::BYTES,
            data,
            length,
            align,
        }
    }
}

impl UninitDataEntry {
    pub fn from_i32() -> Self {
        Self {
            memory_data_type: MemoryDataType::I32,
            length: 4,
            align: 4,
        }
    }

    pub fn from_i64() -> Self {
        Self {
            memory_data_type: MemoryDataType::I64,
            length: 8,
            align: 8,
        }
    }

    pub fn from_f32() -> Self {
        Self {
            memory_data_type: MemoryDataType::F32,
            length: 4,
            align: 4,
        }
    }

    pub fn from_f64() -> Self {
        Self {
            memory_data_type: MemoryDataType::F64,
            length: 8,
            align: 8,
        }
    }

    pub fn from_bytes(length: u32, align: u16) -> Self {
        Self {
            memory_data_type: MemoryDataType::BYTES,
            length,
            align,
        }
    }
}

impl DataItem {
    pub fn new(
        data_offset: u32,
        data_length: u32,
        data_type: MemoryDataType,
        data_align: u16,
    ) -> Self {
        DataItem {
            data_offset,
            data_length,
            memory_data_type: data_type,
            _padding0: 0,
            data_align,
        }
    }
}

impl<'a> SectionEntry<'a> for ReadOnlyDataSection<'a> {
    fn id(&'a self) -> ModuleSectionId {
        ModuleSectionId::ReadOnlyData
    }

    fn load(section_data: &'a [u8]) -> Self
    where
        Self: Sized,
    {
        let (items, datas) = load_section_with_table_and_data_area::<DataItem>(section_data);
        ReadOnlyDataSection {
            items,
            datas_data: datas,
        }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_table_and_data_area(self.items, self.datas_data, writer)
    }
}

impl<'a> SectionEntry<'a> for ReadWriteDataSection<'a> {
    fn id(&'a self) -> ModuleSectionId {
        ModuleSectionId::ReadWriteData
    }

    fn load(section_data: &'a [u8]) -> Self
    where
        Self: Sized,
    {
        let (items, datas) = load_section_with_table_and_data_area::<DataItem>(section_data);
        ReadWriteDataSection {
            items,
            datas_data: datas,
        }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_table_and_data_area(self.items, self.datas_data, writer)
    }
}

impl<'a> SectionEntry<'a> for UninitDataSection<'a> {
    fn id(&'a self) -> ModuleSectionId {
        ModuleSectionId::UninitData
    }

    fn load(section_data: &'a [u8]) -> Self
    where
        Self: Sized,
    {
        let items = load_section_with_one_table::<DataItem>(section_data);
        UninitDataSection { items }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_one_table(self.items, writer)
    }
}

impl ReadOnlyDataSection<'_> {
    pub fn convert_from_entries(entries: &[DataEntry]) -> (Vec<DataItem>, Vec<u8>) {
        convert_from_entries_internal(entries)
    }
}

impl ReadWriteDataSection<'_> {
    pub fn convert_from_entries(entries: &[DataEntry]) -> (Vec<DataItem>, Vec<u8>) {
        convert_from_entries_internal(entries)
    }
}

fn convert_from_entries_internal(entries: &[DataEntry]) -> (Vec<DataItem>, Vec<u8>) {
    // | type  | size | alignment |
    // |-------|------|-----------|
    // | i32   | 4    | 4         |
    // | i64   | 8    | 8         |
    // | f32   | 4    | 4         |
    // | f64   | 8    | 8         |
    // | bytes | -    | -         |

    let mut next_offset: u32 = 0;

    // get the position '(padding, data_offset, data_length)'

    let positions = entries
        .iter()
        .map(|entry| {
            let remainder = next_offset % (entry.align as u32); // remainder
            let padding = if remainder != 0 {
                (entry.align as u32) - remainder
            } else {
                0
            };

            let data_offset = next_offset + padding; // the data offset after aligning
            let data_length = entry.length;
            next_offset = data_offset + data_length;
            (padding, data_offset, data_length)
        })
        .collect::<Vec<(u32, u32, u32)>>();

    let items = entries
        .iter()
        .zip(&positions)
        .map(|(entry, (_padding, data_offset, data_length))| {
            DataItem::new(
                *data_offset,
                *data_length,
                entry.memory_data_type,
                entry.align,
            )
        })
        .collect::<Vec<DataItem>>();

    let datas_data = entries
        .iter()
        .zip(&positions)
        .flat_map(|(entry, (padding, _data_offset, _data_length))| {
            let mut data = vec![0u8; *padding as usize];
            data.extend(entry.data.iter());
            data
        })
        .collect::<Vec<u8>>();

    (items, datas_data)
}

impl UninitDataSection<'_> {
    pub fn convert_from_entries(entries: &[UninitDataEntry]) -> Vec<DataItem> {
        // | type  | size | alignment |
        // |-------|------|-----------|
        // | i32   | 4    | 4         |
        // | i64   | 8    | 8         |
        // | f32   | 4    | 4         |
        // | f64   | 8    | 8         |
        // | bytes | -    | -         |

        let mut next_offset: u32 = 0;

        // get the position '(padding, data_offset, data_length)'

        let positions = entries
            .iter()
            .map(|entry| {
                let remainder = next_offset % (entry.align as u32); // remainder
                let padding = if remainder != 0 {
                    (entry.align as u32) - remainder
                } else {
                    0
                };

                let data_offset = next_offset + padding; // the data offset after aligning
                let data_length = entry.length;
                next_offset = data_offset + data_length;
                (padding, data_offset, data_length)
            })
            .collect::<Vec<(u32, u32, u32)>>();

        let items = entries
            .iter()
            .zip(&positions)
            .map(|(entry, (_padding, data_offset, data_length))| {
                DataItem::new(
                    *data_offset,
                    *data_length,
                    entry.memory_data_type,
                    entry.align,
                )
            })
            .collect::<Vec<DataItem>>();

        items
    }
}

#[cfg(test)]
mod tests {
    use ancvm_types::MemoryDataType;

    use crate::module_image::{
        data_section::{DataEntry, DataItem, UninitDataEntry, UninitDataSection},
        SectionEntry,
    };

    use super::ReadWriteDataSection;

    #[test]
    fn test_save_read_write_data_section() {
        let data_entry0 = DataEntry::from_i32(11);
        let data_entry1 = DataEntry::from_i64(13);
        let data_entry2 = DataEntry::from_bytes(b"hello".to_vec(), 1);
        let data_entry3 = DataEntry::from_f32(std::f32::consts::PI);
        let data_entry4 = DataEntry::from_f64(std::f64::consts::E); // deprecated 2.9979e8
        let data_entry5 = DataEntry::from_bytes(b"foo".to_vec(), 8);
        let data_entry6 = DataEntry::from_i64(17);
        let data_entry7 = DataEntry::from_i32(19);

        let (items, datas) = ReadWriteDataSection::convert_from_entries(&[
            data_entry0,
            data_entry1,
            data_entry2,
            data_entry3,
            data_entry4,
            data_entry5,
            data_entry6,
            data_entry7,
        ]);

        let section = ReadWriteDataSection {
            items: &items,
            datas_data: &datas,
        };

        let mut section_data: Vec<u8> = Vec::new();
        section.save(&mut section_data).unwrap();

        let expect_data = vec![
            8u8, 0, 0, 0, // item count
            0, 0, 0, 0, // padding
            //
            0, 0, 0, 0, // offset 0
            4, 0, 0, 0, // length
            0, // type
            0, // padding
            4, 0, // align
            //
            8, 0, 0, 0, // offset 1
            8, 0, 0, 0, // length
            1, // type
            0, // padding
            8, 0, // align
            //
            16, 0, 0, 0, // offset 2
            5, 0, 0, 0, // length
            4, // type
            0, // padding
            1, 0, // align
            //
            24, 0, 0, 0, // offset 3
            4, 0, 0, 0, // length
            2, // type
            0, // padding
            4, 0, // align
            //
            32, 0, 0, 0, // offset 4
            8, 0, 0, 0, // length
            3, // type
            0, // padding
            8, 0, // align
            //
            40, 0, 0, 0, // offset 5
            3, 0, 0, 0, // length
            4, // type
            0, // padding
            8, 0, // align
            //
            48, 0, 0, 0, // offset 6
            8, 0, 0, 0, // length
            1, // type
            0, // padding
            8, 0, // align
            //
            56, 0, 0, 0, // offset 7
            4, 0, 0, 0, // length
            0, // type
            0, // padding
            4, 0, // align
            //
            // datas
            //
            11, 0, 0, 0, // data 0
            0, 0, 0, 0, // padding
            13, 0, 0, 0, 0, 0, 0, 0, // data 1
            104, 101, 108, 108, 111, // data 2, "hello"
            0, 0, 0, // padding
            // Float (IEEE754 Single precision 32-bit)
            // 0x4048F5C3 = 0 1000000 0  1001000 11110101 11000011
            //              ^ ^--------  ^------------------------
            //         sign | | exponent | 31400....
            //
            // https://www.binaryconvert.com/result_float.html?decimal=051046049052
            //
            219, 15, 73, 64, // data 3
            0, 0, 0, 0, // padding
            // Double (IEEE754 Double precision 64-bit)
            // 0x41B1E1A300000000 =
            // 0 1000001 1011 0001 11100001 10100011 00000000 00000000 00000000 00000000
            // ^ ^----------- ^------------------...
            // | | Exponent   | Mantissa
            // |
            // | sign
            //
            // https://www.binaryconvert.com/result_double.html?decimal=051048048048048048048048048
            105, 87, 20, 139, 10, 191, 5, 64, // data 4
            102, 111, 111, // data 5, "bar"
            0, 0, 0, 0, 0, // padding
            17, 0, 0, 0, 0, 0, 0, 0, // data 6
            19, 0, 0, 0, // data 7
        ];

        assert_eq!(section_data, expect_data);
    }

    #[test]
    fn test_load_read_write_data_section() {
        let section_data = vec![
            8u8, 0, 0, 0, // item count
            0, 0, 0, 0, // padding
            //
            0, 0, 0, 0, // offset 0
            4, 0, 0, 0, // length
            0, // type
            0, // padding
            4, 0, // align
            //
            8, 0, 0, 0, // offset 1
            8, 0, 0, 0, // length
            1, // type
            0, // padding
            8, 0, // align
            //
            16, 0, 0, 0, // offset 2
            5, 0, 0, 0, // length
            4, // type
            0, // padding
            1, 0, // align
            //
            24, 0, 0, 0, // offset 3
            4, 0, 0, 0, // length
            2, // type
            0, // padding
            4, 0, // align
            //
            32, 0, 0, 0, // offset 4
            8, 0, 0, 0, // length
            3, // type
            0, // padding
            8, 0, // align
            //
            40, 0, 0, 0, // offset 5
            3, 0, 0, 0, // length
            4, // type
            0, // padding
            8, 0, // align
            //
            48, 0, 0, 0, // offset 6
            8, 0, 0, 0, // length
            1, // type
            0, // padding
            8, 0, // align
            //
            56, 0, 0, 0, // offset 7
            4, 0, 0, 0, // length
            0, // type
            0, // padding
            4, 0, // align
            //
            // datas
            //
            11, 0, 0, 0, // data 0
            0, 0, 0, 0, // padding
            13, 0, 0, 0, 0, 0, 0, 0, // data 1
            104, 101, 108, 108, 111, // data 2, "hello"
            0, 0, 0, // padding
            // Float (IEEE754 Single precision 32-bit)
            // 0x4048F5C3 = 0 1000000 0  1001000 11110101 11000011
            //              ^ ^--------  ^------------------------
            //         sign | | exponent | 31400....
            //
            // https://www.binaryconvert.com/result_float.html?decimal=051046049052
            //
            195, 245, 72, 64, // data 3
            0, 0, 0, 0, // padding
            // Double (IEEE754 Double precision 64-bit)
            // 0x41B1E1A300000000 =
            // 0 1000001 1011 0001 11100001 10100011 00000000 00000000 00000000 00000000
            // ^ ^----------- ^------------------...
            // | | Exponent   | Mantissa
            // |
            // | sign
            //
            // https://www.binaryconvert.com/result_double.html?decimal=051048048048048048048048048
            0, 0, 0, 0, 163, 225, 177, 65, // data 4
            102, 111, 111, // data 5, "bar"
            0, 0, 0, 0, 0, // padding
            17, 0, 0, 0, 0, 0, 0, 0, // data 6
            19, 0, 0, 0, // data 7
        ];

        let section = ReadWriteDataSection::load(&section_data);

        assert_eq!(
            section.items,
            &[
                DataItem::new(0, 4, MemoryDataType::I32, 4),
                DataItem::new(8, 8, MemoryDataType::I64, 8),
                DataItem::new(16, 5, MemoryDataType::BYTES, 1),
                DataItem::new(24, 4, MemoryDataType::F32, 4),
                DataItem::new(32, 8, MemoryDataType::F64, 8),
                DataItem::new(40, 3, MemoryDataType::BYTES, 8),
                DataItem::new(48, 8, MemoryDataType::I64, 8),
                DataItem::new(56, 4, MemoryDataType::I32, 4),
            ]
        );

        // the data area is too long, only check partly here.
        assert_eq!(
            &section.datas_data[0..16],
            &[
                11u8, 0, 0, 0, // data 0
                0, 0, 0, 0, // padding
                13, 0, 0, 0, 0, 0, 0, 0, // data 1
            ]
        )
    }

    #[test]
    fn test_save_uninit_data_section() {
        let data_entry0 = UninitDataEntry::from_i32();
        let data_entry1 = UninitDataEntry::from_i64();
        let data_entry2 = UninitDataEntry::from_bytes(5, 1);
        let data_entry3 = UninitDataEntry::from_f32();
        let data_entry4 = UninitDataEntry::from_f64();
        let data_entry5 = UninitDataEntry::from_bytes(3, 8);
        let data_entry6 = UninitDataEntry::from_i64();
        let data_entry7 = UninitDataEntry::from_i32();

        let items = UninitDataSection::convert_from_entries(&[
            data_entry0,
            data_entry1,
            data_entry2,
            data_entry3,
            data_entry4,
            data_entry5,
            data_entry6,
            data_entry7,
        ]);

        let section = UninitDataSection { items: &items };

        let mut section_data: Vec<u8> = Vec::new();
        section.save(&mut section_data).unwrap();

        let expect_data = vec![
            8u8, 0, 0, 0, // item count
            0, 0, 0, 0, // padding
            //
            0, 0, 0, 0, // offset 0
            4, 0, 0, 0, // length
            0, // type
            0, // padding
            4, 0, // align
            //
            8, 0, 0, 0, // offset 1
            8, 0, 0, 0, // length
            1, // type
            0, // padding
            8, 0, // align
            //
            16, 0, 0, 0, // offset 2
            5, 0, 0, 0, // length
            4, // type
            0, // padding
            1, 0, // align
            //
            24, 0, 0, 0, // offset 3
            4, 0, 0, 0, // length
            2, // type
            0, // padding
            4, 0, // align
            //
            32, 0, 0, 0, // offset 4
            8, 0, 0, 0, // length
            3, // type
            0, // padding
            8, 0, // align
            //
            40, 0, 0, 0, // offset 5
            3, 0, 0, 0, // length
            4, // type
            0, // padding
            8, 0, // align
            //
            48, 0, 0, 0, // offset 6
            8, 0, 0, 0, // length
            1, // type
            0, // padding
            8, 0, // align
            //
            56, 0, 0, 0, // offset 7
            4, 0, 0, 0, // length
            0, // type
            0, // padding
            4, 0, // align
        ];

        assert_eq!(section_data, expect_data);
    }

    #[test]
    fn test_load_uninit_data_section() {
        let section_data = vec![
            8u8, 0, 0, 0, // item count
            0, 0, 0, 0, // padding
            //
            0, 0, 0, 0, // offset 0
            4, 0, 0, 0, // length
            0, // type
            0, // padding
            4, 0, // align
            //
            8, 0, 0, 0, // offset 1
            8, 0, 0, 0, // length
            1, // type
            0, // padding
            8, 0, // align
            //
            16, 0, 0, 0, // offset 2
            5, 0, 0, 0, // length
            4, // type
            0, // padding
            1, 0, // align
            //
            24, 0, 0, 0, // offset 3
            4, 0, 0, 0, // length
            2, // type
            0, // padding
            4, 0, // align
            //
            32, 0, 0, 0, // offset 4
            8, 0, 0, 0, // length
            3, // type
            0, // padding
            8, 0, // align
            //
            40, 0, 0, 0, // offset 5
            3, 0, 0, 0, // length
            4, // type
            0, // padding
            8, 0, // align
            //
            48, 0, 0, 0, // offset 6
            8, 0, 0, 0, // length
            1, // type
            0, // padding
            8, 0, // align
            //
            56, 0, 0, 0, // offset 7
            4, 0, 0, 0, // length
            0, // type
            0, // padding
            4, 0, // align
        ];

        let section = UninitDataSection::load(&section_data);
        assert_eq!(
            section.items,
            &[
                DataItem::new(0, 4, MemoryDataType::I32, 4),
                DataItem::new(8, 8, MemoryDataType::I64, 8),
                DataItem::new(16, 5, MemoryDataType::BYTES, 1),
                DataItem::new(24, 4, MemoryDataType::F32, 4),
                DataItem::new(32, 8, MemoryDataType::F64, 8),
                DataItem::new(40, 3, MemoryDataType::BYTES, 8),
                DataItem::new(48, 8, MemoryDataType::I64, 8),
                DataItem::new(56, 4, MemoryDataType::I32, 4),
            ]
        );
    }
}
