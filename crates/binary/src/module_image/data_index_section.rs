// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// "data index section" binary layout
//
// |-----------------------------------------------------------------------------------------------|
// | item count (u32) | (4 bytes padding)                                                          |
// |-----------------------------------------------------------------------------------------------|
// | offset 0 (u32) | count 0 (u32)                                                                | <-- table 0
// | offset 1       | count 1                                                                      |
// | ...                                                                                           |
// |-----------------------------------------------------------------------------------------------|
//
// |-----------------------------------------------------------------------------------------------------------|
// | data idx 0 (u16) | tar mod idx 0 (u16) | tar data section type 0 (u8) | pad 1 byte | tar data idx 0 (u16) | <-- table 1
// | data idx 1       | tar mod idx 1       | tar data section type 1      | pad 1 byte | tar data idx 1       |
// | ...                                                                                                       |
// |-----------------------------------------------------------------------------------------------------------|

use crate::utils::{load_section_with_two_tables, save_section_with_two_tables};

use super::{data_section::DataSectionType, SectionEntry, SectionId};

#[derive(Debug, PartialEq)]
pub struct DataIndexSection<'a> {
    pub offsets: &'a [DataIndexOffset],
    pub items: &'a [DataIndexItem],
}

// one index offset item per module.
// for example, consider the following items:
//
// module 0 ----- index item 0
//            |-- index item 1
//            |-- index item 2
//
// module 1 ----- index item 3
//            |-- index item 4
//
// since there are 2 modules, so there will be
// 2 index offset items as the following:
//
// index offset 0 = {offset:0, count:3}
// index offset 1 = {offset:3, count:2}
//
// use the C style struct memory layout
// see also:
// https://doc.rust-lang.org/reference/type-layout.html#reprc-structs
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct DataIndexOffset {
    pub offset: u32,
    pub count: u32,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct DataIndexItem {
    pub data_index: u16,          // data item index (in a specified module)
    pub target_module_index: u16, // target module index
    pub target_data_section_type: DataSectionType, // target data section, i.e. 0=READ_ONLY, 1=READ_WRITE, 2=UNINIT
    _padding0: u8,
    pub target_data_index: u16, // target data item index (in a specified section)
}

impl<'a> SectionEntry<'a> for DataIndexSection<'a> {
    fn load(section_data: &'a [u8]) -> Self {
        let (offsets, items) =
            load_section_with_two_tables::<DataIndexOffset, DataIndexItem>(section_data);
        DataIndexSection { offsets, items }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_two_tables(self.offsets, self.items, writer)
    }

    fn id(&'a self) -> SectionId {
        SectionId::DataIndex
    }
}

impl DataIndexItem {
    pub fn new(
        data_index: u16,
        target_module_index: u16,
        target_data_section_type: DataSectionType,
        target_data_index: u16,
    ) -> Self {
        Self {
            data_index,
            target_module_index,
            target_data_section_type,
            _padding0: 0,
            target_data_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::module_image::{
        data_index_section::{DataIndexItem, DataIndexOffset, DataIndexSection},
        data_section::DataSectionType,
        SectionEntry,
    };

    #[test]
    fn test_load_section() {
        let section_data = vec![
            2u8, 0, 0, 0, // item count (little endian)
            0, 0, 0, 0, // 4 bytes padding
            //
            2, 0, 0, 0, // offset 0 (item 0)
            3, 0, 0, 0, // count 0
            5, 0, 0, 0, // offset 1 (item 1)
            7, 0, 0, 0, // count 1
            //
            2, 0, // data index, item 0 (little endian)
            3, 0, // t module index
            0, // t data section type
            0, // padding
            5, 0, // t data idx
            //
            7, 0, // data index, item 1 (little endian)
            11, 0, // t module index
            1, // t data section type
            0, // padding
            13, 0, // t data idx
            //
            17, 0, // data index, item 2 (little endian)
            11, 0, // t module index
            1, // t data section type
            0, // padding
            19, 0, // t data idx
        ];

        let section = DataIndexSection::load(&section_data);

        let offsets = section.offsets;

        assert_eq!(offsets.len(), 2);
        assert_eq!(
            offsets[0],
            DataIndexOffset {
                offset: 2,
                count: 3,
            }
        );
        assert_eq!(
            offsets[1],
            DataIndexOffset {
                offset: 5,
                count: 7
            }
        );

        let items = section.items;

        assert_eq!(items.len(), 3);

        assert_eq!(
            items[0],
            DataIndexItem::new(2, 3, DataSectionType::ReadOnly, 5)
        );

        assert_eq!(
            items[1],
            DataIndexItem::new(7, 11, DataSectionType::ReadWrite, 13,)
        );

        assert_eq!(
            items[2],
            DataIndexItem::new(17, 11, DataSectionType::ReadWrite, 19,)
        );
    }

    #[test]
    fn test_save_section() {
        let mut offsets: Vec<DataIndexOffset> = Vec::new();

        offsets.push(DataIndexOffset {
            offset: 2,
            count: 3,
        });

        offsets.push(DataIndexOffset {
            offset: 5,
            count: 7,
        });

        let mut items: Vec<DataIndexItem> = Vec::new();

        items.push(DataIndexItem::new(2, 3, DataSectionType::ReadOnly, 5));
        items.push(DataIndexItem::new(7, 11, DataSectionType::ReadWrite, 13));
        items.push(DataIndexItem::new(17, 11, DataSectionType::ReadWrite, 19));

        let section = DataIndexSection {
            offsets: &offsets,
            items: &items,
        };

        let mut section_data: Vec<u8> = Vec::new();
        section.save(&mut section_data).unwrap();

        assert_eq!(
            section_data,
            vec![
                2u8, 0, 0, 0, // item count (little endian)
                0, 0, 0, 0, // 4 bytes padding
                //
                2, 0, 0, 0, // offset 0 (item 0)
                3, 0, 0, 0, // count 0
                5, 0, 0, 0, // offset 1 (item 1)
                7, 0, 0, 0, // count 1
                //
                2, 0, // data index, item 0 (little endian)
                3, 0, // t module index
                0, // t data section type
                0, // padding
                5, 0, // t data idx
                //
                7, 0, // data index, item 1 (little endian)
                11, 0, // t module index
                1, // t data section type
                0, // padding
                13, 0, // t data idx
                //
                17, 0, // data index, item 2 (little endian)
                11, 0, // t module index
                1, // t data section type
                0, // padding
                19, 0, // t data idx
            ]
        );
    }
}
