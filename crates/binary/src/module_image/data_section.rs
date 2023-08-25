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

use ancvm_types::{DataEntry, DataType, SectionEntry, SectionId};

use crate::utils::{
    load_section_with_one_table, load_section_with_table_and_data_area,
    save_section_with_one_table, save_section_with_table_and_data_area,
};

#[derive(Debug, PartialEq)]
pub struct ReadOnlyDataSection<'a> {
    pub items: &'a [DataItem],
    pub datas: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct ReadWriteDataSection<'a> {
    pub items: &'a [DataItem],
    pub datas: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct UninitDataSection<'a> {
    pub items: &'a [DataItem],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct DataItem {
    pub data_offset: u32,
    pub data_length: u32,
    pub data_type: DataType,
    _padding0: u8,
    _padding1: u16,
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
        ReadOnlyDataSection { items, datas }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_table_and_data_area(self.items, self.datas, writer)
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
        ReadWriteDataSection { items, datas }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_table_and_data_area(self.items, self.datas, writer)
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

    let items = entries
        .iter()
        .map(|entry| {
            let data_offset = next_offset;
            let data_length = entry.data.len() as u32;
            next_offset += data_length;

            DataItem::new(data_offset, data_length, entry.data_type)
        })
        .collect::<Vec<DataItem>>();

    let datas = entries
        .iter()
        .flat_map(|entry| entry.data.clone())
        .collect::<Vec<u8>>();

    (items, datas)
}

#[cfg(test)]
mod tests {
    use ancvm_types::{DataEntry, DataType, SectionEntry};

    use crate::module_image::data_section::DataItem;

    use super::{convert_data_entries, ReadWriteDataSection};

    #[test]
    fn test_data_section() {
        let data_entry0 = DataEntry::from_i32(11);
        let data_entry1 = DataEntry::from_i64(13);
        let data_entry2 = DataEntry::from_bytes("hello".as_bytes());
        let data_entry3 = DataEntry::from_f32(3.14);
        let data_entry4 = DataEntry::from_f64(3e8); // 2.9979e8
        let data_entry5 = DataEntry::from_bytes("foobar".as_bytes());

        let (items, datas) = convert_data_entries(&vec![
            data_entry0,
            data_entry1,
            data_entry2,
            data_entry3,
            data_entry4,
            data_entry5,
        ]);

        let data_section = ReadWriteDataSection {
            items: &items,
            datas: &datas,
        };

        let mut section_data: Vec<u8> = Vec::new();
        data_section.save(&mut section_data).unwrap();

        let expect_data = vec![
            6u8, 0, 0, 0, // item count
            0, 0, 0, 0, // padding
            //
            0, 0, 0, 0, // offset 0
            8, 0, 0, 0, // length 0
            0, // data type 0
            0, 0, 0, // padding 0
            //
            8, 0, 0, 0, // offset 1
            8, 0, 0, 0, // length 1
            1, // data type 1
            0, 0, 0, // padding 1
            //
            16, 0, 0, 0, // offset 2
            5, 0, 0, 0, // length 2
            4, // data type 2
            0, 0, 0, // padding 2
            //
            21, 0, 0, 0, // offset 3
            8, 0, 0, 0, // length 3
            2, // data type 3
            0, 0, 0, // padding 3
            //
            29, 0, 0, 0, // offset 4
            8, 0, 0, 0, // length 4
            3, // data type 4
            0, 0, 0, // padding 4
            //
            37, 0, 0, 0, // offset 5
            6, 0, 0, 0, // length 5
            4, // data type 5
            0, 0, 0, // padding 5
            //
            11, 0, 0, 0, 0, 0, 0, 0, // data 0
            13, 0, 0, 0, 0, 0, 0, 0, // data 1
            104, 101, 108, 108, 111, // data 2
            //
            195, 245, 72, 64, 0, 0, 0, 0, // data 3
            // Float (IEEE754 Single precision 32-bit)
            // 0x4048F5C3 = 0 1000000 0  1001000 11110101 11000011
            //              ^ ^--------  ^------------------------
            //         sign | | exponent | 31400....
            //
            // https://www.binaryconvert.com/result_float.html?decimal=051046049052
            //
            0, 0, 0, 0, 163, 225, 177, 65, // data 4
            // Double (IEEE754 Double precision 64-bit)
            // 0x41B1E1A300000000 =
            // 0 1000001 1011 0001 11100001 10100011 00000000 00000000 00000000 00000000
            // ^ ^----------- ^------------------...
            // | | Exponent   | Mantissa
            // |
            // | sign
            //
            // https://www.binaryconvert.com/result_double.html?decimal=051048048048048048048048048
            102, 111, 111, 98, 97, 114, // data 5
            0,   // padding
        ];

        assert_eq!(section_data, expect_data);

        let data_section_restore = ReadWriteDataSection::load(&expect_data);
        assert_eq!(
            data_section_restore.items,
            &vec![
                DataItem::new(0, 8, DataType::I32),
                DataItem::new(8, 8, DataType::I64),
                DataItem::new(16, 5, DataType::BYTE),
                DataItem::new(21, 8, DataType::F32),
                DataItem::new(29, 8, DataType::F64),
                DataItem::new(37, 6, DataType::BYTE),
            ]
        );

        assert_eq!(
            &data_section_restore.datas[0..16],
            &vec![
                11u8, 0, 0, 0, 0, 0, 0, 0, // data 0
                13, 0, 0, 0, 0, 0, 0, 0, // data 1
            ]
        )
    }
}
