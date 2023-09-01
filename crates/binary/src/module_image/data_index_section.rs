// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// "data index section" binary layout
//
// |--------------------------------------|
// | item count (u32) | (4 bytes padding) |
// |--------------------------------------|
// | offset 0 (u32) | count 0 (u32)       | <-- table 0
// | offset 1       | count 1             |
// | ...                                  |
// |--------------------------------------|
//
// |---------------------------------------------------------------------------------------------------------------------|
// | data idx 0 (u32) | target mod idx 0 (u32) | target data section type 0 (u8) | pad 3 bytes | target data idx 0 (u32) | <-- table 1
// | data idx 1       | target mod idx 1       | target data section type 1      | pad 3 bytes | target data idx 1       |
// | ...                                                                                                                 |
// |---------------------------------------------------------------------------------------------------------------------|

use crate::utils::{load_section_with_two_tables, save_section_with_two_tables};

use super::{data_section::DataSectionType, RangeItem, SectionEntry, SectionId};

#[derive(Debug, PartialEq)]
pub struct DataIndexSection<'a> {
    pub ranges: &'a [RangeItem],
    pub items: &'a [DataIndexItem],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct DataIndexItem {
    pub data_index: u32,          // data item index (in a specified module)
    pub target_module_index: u32, // target module index
    pub target_data_section_type: DataSectionType, // u8, target data section, i.e. 0=READ_ONLY, 1=READ_WRITE, 2=UNINIT
    _padding0: u8,
    _padding1: u16,
    pub target_data_index: u32, // target data item index (in a specified section)
}

impl<'a> SectionEntry<'a> for DataIndexSection<'a> {
    fn load(section_data: &'a [u8]) -> Self {
        let (ranges, items) =
            load_section_with_two_tables::<RangeItem, DataIndexItem>(section_data);
        DataIndexSection { ranges, items }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_two_tables(self.ranges, self.items, writer)
    }

    fn id(&'a self) -> SectionId {
        SectionId::DataIndex
    }
}

impl DataIndexItem {
    pub fn new(
        data_index: u32,
        target_module_index: u32,
        target_data_section_type: DataSectionType,
        target_data_index: u32,
    ) -> Self {
        Self {
            data_index,
            target_module_index,
            target_data_section_type,
            _padding0: 0,
            _padding1: 0,
            target_data_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::module_image::{
        data_index_section::{DataIndexItem, DataIndexSection, RangeItem},
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
            2, 0, 0, 0, // data index, item 0 (little endian)
            3, 0, 0, 0, // t module index
            0, // t data section type
            0, 0, 0, // padding
            5, 0, 0, 0, // t data idx
            //
            7, 0, 0, 0, // data index, item 1 (little endian)
            11, 0, 0, 0, // t module index
            1, // t data section type
            0, 0, 0, // padding
            13, 0, 0, 0, // t data idx
            //
            17, 0, 0, 0, // data index, item 2 (little endian)
            11, 0, 0, 0, // t module index
            1, // t data section type
            0, 0, 0, // padding
            19, 0, 0, 0, // t data idx
        ];

        let section = DataIndexSection::load(&section_data);

        let ranges = section.ranges;

        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0], RangeItem::new(2, 3,));
        assert_eq!(ranges[1], RangeItem::new(5, 7));

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
        let mut ranges: Vec<RangeItem> = Vec::new();

        ranges.push(RangeItem::new(2, 3));
        ranges.push(RangeItem::new(5, 7));

        let mut items: Vec<DataIndexItem> = Vec::new();

        items.push(DataIndexItem::new(2, 3, DataSectionType::ReadOnly, 5));
        items.push(DataIndexItem::new(7, 11, DataSectionType::ReadWrite, 13));
        items.push(DataIndexItem::new(17, 11, DataSectionType::ReadWrite, 19));

        let section = DataIndexSection {
            ranges: &ranges,
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
                2, 0, 0, 0, // data index, item 0 (little endian)
                3, 0, 0, 0, // t module index
                0, // t data section type
                0, 0, 0, // padding
                5, 0, 0, 0, // t data idx
                //
                7, 0, 0, 0, // data index, item 1 (little endian)
                11, 0, 0, 0, // t module index
                1, // t data section type
                0, 0, 0, // padding
                13, 0, 0, 0, // t data idx
                //
                17, 0, 0, 0, // data index, item 2 (little endian)
                11, 0, 0, 0, // t module index
                1, // t data section type
                0, 0, 0, // padding
                19, 0, 0, 0, // t data idx
            ]
        );
    }
}
