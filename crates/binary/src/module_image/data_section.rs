// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// "read-only/read-write data section" binary layout
//
//              |----------------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                                       |
//              |----------------------------------------------------------------------------|
//  item 0 -->  | data offset 0 (u32) | data length 0 (u32) | data type 0 (u8) | pad 3 bytes | <-- table
//  item 1 -->  | data offset 1       | data length 1       | data type 1                    |
//              | ...                                                                        |
//              |----------------------------------------------------------------------------|
// offset 0 --> | data 0 | data 1 | ...                                                      | <-- data area
//              |          ^                                                                 |
// offset 1 ----|----------|                                                                 |
//              |----------------------------------------------------------------------------|
//
//
// "uninit data section" binary layout
//
//              |----------------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                                       |
//              |----------------------------------------------------------------------------|
//  item 0 -->  | data offset 0 (u32) | data length 0 (u32) | data type 0 (u8) | pad 3 bytes | <-- table
//  item 1 -->  | data offset 1       | data length 1       | data type 1                    |
//              | ...                                                                        |
//              |----------------------------------------------------------------------------|
//
// data size and alignment
//
// | type  | size | alignment |
// |-------|------|-----------|
// | i32   | 4    | 4         |
// | i64   | 8    | 8         |
// | f32   | 4    | 4         |
// | f64   | 8    | 8         |
// | byte  | -    | -         |
//
// when storing "struct" data the "byte" type can be used, note that
// the alignment should be speicified.

use ancvm_types::DataType;

use crate::utils::{
    load_section_with_one_table, load_section_with_table_and_data_area,
    save_section_with_one_table, save_section_with_table_and_data_area,
};

use super::{SectionEntry, SectionId};

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
    pub data_type: DataType, // the data type field is not necessary at runtime, though it is helpful for debugging.
    _padding0: u8,           //
    _padding1: u16,          //
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataSectionType {
    ReadOnly = 0x0,
    ReadWrite,
    Uninit,
}

impl From<u8> for DataSectionType {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute::<u8, DataSectionType>(value) }
    }
}

#[derive(Debug)]
pub struct DataEntry {
    pub data_type: DataType,
    // the length of data entry is determited by the data type, when the data type is 'byte',
    // the 'vec.len()' is used.
    pub data: Vec<u8>,
    pub align: u32,
}

impl DataEntry {
    pub fn from_i32(value: i32) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(8);
        data.extend(value.to_le_bytes().iter());

        Self {
            data_type: DataType::I32,
            data,
            align: 4,
        }
    }

    pub fn from_i64(value: i64) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(8);
        data.extend(value.to_le_bytes().iter());

        Self {
            data_type: DataType::I64,
            data,
            align: 8,
        }
    }

    pub fn from_f32(value: f32) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(8);
        data.extend(value.to_le_bytes().iter());

        Self {
            data_type: DataType::F32,
            data,
            align: 4,
        }
    }

    pub fn from_f64(value: f64) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(8);
        data.extend(value.to_le_bytes().iter());

        Self {
            data_type: DataType::F64,
            data,
            align: 8,
        }
    }

    pub fn from_bytes(data: Vec<u8>, align: u32) -> Self {
        Self {
            data_type: DataType::BYTE,
            data,
            align,
        }
    }
}

impl DataItem {
    pub fn new(data_offset: u32, data_length: u32, data_type: DataType) -> Self {
        DataItem {
            data_offset,
            data_length,
            data_type,
            _padding0: 0,
            _padding1: 0,
        }
    }
}

impl<'a> SectionEntry<'a> for ReadOnlyDataSection<'a> {
    fn id(&'a self) -> SectionId {
        SectionId::ReadOnlyData
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
    fn id(&'a self) -> SectionId {
        SectionId::ReadWriteData
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
    fn id(&'a self) -> SectionId {
        SectionId::UninitData
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

pub fn convert_data_entries(entries: &[DataEntry]) -> (Vec<DataItem>, Vec<u8>) {
    let mut next_offset: u32 = 0;

    let positions = entries
        .iter()
        .map(|entry| {
            let remainder = next_offset % entry.align; // remainder
            let padding = if remainder != 0 {
                entry.align - remainder
            } else {
                0
            };

            let data_offset = next_offset + padding; // the data offset after aligning
            let data_length = entry.data.len() as u32;
            next_offset = data_offset + data_length;
            (padding, data_offset, data_length)
        })
        .collect::<Vec<_>>();

    let items = entries
        .iter()
        .zip(&positions)
        .map(|(entry, (_, data_offset, data_length))| {
            DataItem::new(*data_offset, *data_length, entry.data_type)
        })
        .collect::<Vec<DataItem>>();

    let datas_data = entries
        .iter()
        .zip(&positions)
        .flat_map(|(entry, (padding, _, _))| {
            let mut data = vec![0u8; *padding as usize];
            data.extend(entry.data.iter());
            data
        })
        .collect::<Vec<u8>>();

    (items, datas_data)
}

#[cfg(test)]
mod tests {
    use ancvm_types::DataType;

    use crate::module_image::{
        data_section::{DataEntry, DataItem},
        SectionEntry,
    };

    use super::{convert_data_entries, ReadWriteDataSection};

    #[test]
    fn test_data_section() {
        let data_entry0 = DataEntry::from_i32(11);
        let data_entry1 = DataEntry::from_i64(13);
        let data_entry2 = DataEntry::from_bytes(b"hello".to_vec(), 1);
        let data_entry3 = DataEntry::from_f32(3.14);
        let data_entry4 = DataEntry::from_f64(3e8); // 2.9979e8
        let data_entry5 = DataEntry::from_bytes(b"foo".to_vec(), 8);
        let data_entry6 = DataEntry::from_i64(17);
        let data_entry7 = DataEntry::from_i32(19);

        let (items, datas) = convert_data_entries(&vec![
            data_entry0,
            data_entry1,
            data_entry2,
            data_entry3,
            data_entry4,
            data_entry5,
            data_entry6,
            data_entry7,
        ]);

        let data_section = ReadWriteDataSection {
            items: &items,
            datas_data: &datas,
        };

        let mut section_data: Vec<u8> = Vec::new();
        data_section.save(&mut section_data).unwrap();

        let expect_data = vec![
            8u8, 0, 0, 0, // item count
            0, 0, 0, 0, // padding
            //
            0, 0, 0, 0, // offset 0
            4, 0, 0, 0, // length
            0, // type
            0, 0, 0, // padding
            //
            8, 0, 0, 0, // offset 1
            8, 0, 0, 0, // length
            1, // type
            0, 0, 0, // padding
            //
            16, 0, 0, 0, // offset 2
            5, 0, 0, 0, // length
            4, // type
            0, 0, 0, // padding
            //
            24, 0, 0, 0, // offset 3
            4, 0, 0, 0, // length
            2, // type
            0, 0, 0, // padding
            //
            32, 0, 0, 0, // offset 4
            8, 0, 0, 0, // length
            3, // type
            0, 0, 0, // padding
            //
            40, 0, 0, 0, // offset 5
            3, 0, 0, 0, // length
            4, // type
            0, 0, 0, // padding
            //
            48, 0, 0, 0, // offset 6
            8, 0, 0, 0, // length
            1, // type
            0, 0, 0, // padding
            //
            56, 0, 0, 0, // offset 7
            4, 0, 0, 0, // length
            0, // type
            0, 0, 0, // padding
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

        assert_eq!(section_data, expect_data);

        let data_section_restore = ReadWriteDataSection::load(&expect_data);
        assert_eq!(
            data_section_restore.items,
            &vec![
                DataItem::new(0, 4, DataType::I32),
                DataItem::new(8, 8, DataType::I64),
                DataItem::new(16, 5, DataType::BYTE),
                DataItem::new(24, 4, DataType::F32),
                DataItem::new(32, 8, DataType::F64),
                DataItem::new(40, 3, DataType::BYTE),
                DataItem::new(48, 8, DataType::I64),
                DataItem::new(56, 4, DataType::I32),
            ]
        );

        // the data area is too long, only check partly here.
        assert_eq!(
            &data_section_restore.datas_data[0..16],
            &vec![
                11u8, 0, 0, 0, // data 0
                0, 0, 0, 0, // padding
                13, 0, 0, 0, 0, 0, 0, 0, // data 1
            ]
        )
    }
}
