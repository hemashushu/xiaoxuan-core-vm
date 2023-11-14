// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// "data index section" binary layout
//
//         |--------------------------------------|
//         | item count (u32) | (4 bytes padding) |
//         |--------------------------------------|
// range 0 | offset 0 (u32) | count 0 (u32)       | <-- table 0
// range 1 | offset 1       | count 1             |
//         | ...                                  |
//         |--------------------------------------|
//
//         |--------------------------------------------------------------------------------------------------------------------------------|
//         | data public idx 0 (u32) | target mod idx 0 (u32) | data internal idx 0 (u32) | target data section type 0 (u8) | pad (3 bytes) | <-- table 1
//         | data public idx 1       | target mod idx 1       | data internal idx 1       | target data section type 1      |               |
//         | ...                                                                                                                            |
//         |--------------------------------------------------------------------------------------------------------------------------------|

use ancvm_types::DataSectionType;

use crate::utils::{load_section_with_two_tables, save_section_with_two_tables};

use super::{ModuleSectionId, RangeItem, SectionEntry};

#[derive(Debug, PartialEq, Default)]
pub struct DataIndexSection<'a> {
    pub ranges: &'a [RangeItem],
    pub items: &'a [DataIndexItem],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct DataIndexItem {
    // the index of data item
    //
    // this index includes:
    // - imported read-only data items
    // - internal read-only data items
    // - imported read-write data items
    // - internal read-write data items
    // - imported uninitilized data items
    // - internal uninitilized data items
    //
    // this field is REDUNDANT because its value always starts
    // from 0 to the total number of items within [a certain range | a module].
    pub data_public_index: u32,

    // target module index
    pub target_module_index: u32,

    // the index of the internal data item in a specified data section (in a specified module)
    //
    // this index is the actual index of the internal data item in a specified data section
    // i.e., it excludes the imported data items, and it is section relevant.
    pub data_internal_index: u32,

    // u8, target data section, i.e. 0=READ_ONLY, 1=READ_WRITE, 2=UNINIT
    pub target_data_section_type: DataSectionType,
    _padding0: [u8; 3],
}

impl DataIndexItem {
    pub fn new(
        data_public_index: u32,
        target_module_index: u32,
        data_internal_index: u32,
        target_data_section_type: DataSectionType,
    ) -> Self {
        Self {
            data_public_index,
            target_module_index,
            data_internal_index,
            target_data_section_type,
            _padding0: [0, 0, 0],
        }
    }
}

#[derive(Debug)]
pub struct DataIndexEntry {
    pub data_public_index: usize,
    pub target_module_index: usize,
    pub data_internal_index: usize,
    pub target_data_section_type: DataSectionType,
}

impl DataIndexEntry {
    pub fn new(
        data_public_index: usize,
        target_module_index: usize,
        data_internal_index: usize,
        target_data_section_type: DataSectionType,
    ) -> Self {
        Self {
            data_public_index,
            target_module_index,
            data_internal_index,
            target_data_section_type,
        }
    }
}

#[derive(Debug)]
pub struct DataIndexModuleEntry {
    pub index_entries: Vec<DataIndexEntry>,
}

impl DataIndexModuleEntry {
    pub fn new(index_entries: Vec<DataIndexEntry>) -> Self {
        Self { index_entries }
    }
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

    fn id(&'a self) -> ModuleSectionId {
        ModuleSectionId::DataIndex
    }
}

impl<'a> DataIndexSection<'a> {
    pub fn get_item_target_module_index_and_data_internal_index_and_data_section_type(
        &self,
        module_index: usize,
        data_public_index: usize,
    ) -> (usize, usize, DataSectionType) {
        let range = &self.ranges[module_index];

        // bounds check
        #[cfg(debug_assertions)]
        {
            if data_public_index > range.count as usize {
                panic!(
                    "Out of bounds of the data index, module index: {}, data items: {} data index: {}.",
                    module_index, range.count, data_public_index
                );
            }
        }

        let item_index = range.offset as usize + data_public_index;
        let item = &self.items[item_index];
        (
            item.target_module_index as usize,
            item.data_internal_index as usize,
            item.target_data_section_type,
        )
    }

    pub fn convert_from_entries(
        sorted_module_entries: &[DataIndexModuleEntry],
    ) -> (Vec<RangeItem>, Vec<DataIndexItem>) {
        let mut range_start_offset: u32 = 0;

        let range_items = sorted_module_entries
            .iter()
            .map(|index_module_entry| {
                let count = index_module_entry.index_entries.len() as u32;
                let range_item = RangeItem::new(range_start_offset, count);
                range_start_offset += count;
                range_item
            })
            .collect::<Vec<_>>();

        let data_index_items = sorted_module_entries
            .iter()
            .flat_map(|index_module_entry| {
                index_module_entry.index_entries.iter().map(|entry| {
                    DataIndexItem::new(
                        entry.data_public_index as u32,
                        entry.target_module_index as u32,
                        entry.data_internal_index as u32,
                        entry.target_data_section_type,
                    )
                })
            })
            .collect::<Vec<_>>();

        (range_items, data_index_items)
    }
}

#[cfg(test)]
mod tests {
    use ancvm_types::DataSectionType;

    use crate::module_image::{
        data_index_section::{DataIndexItem, DataIndexSection, RangeItem},
        SectionEntry,
    };

    use super::{DataIndexEntry, DataIndexModuleEntry};

    #[test]
    fn test_load_section() {
        let section_data = vec![
            2u8, 0, 0, 0, // item count (little endian)
            0, 0, 0, 0, // 4 bytes padding
            //
            0, 0, 0, 0, // offset 0 (item 0)
            2, 0, 0, 0, // count 0
            2, 0, 0, 0, // offset 1 (item 1)
            1, 0, 0, 0, // count 1
            //
            2, 0, 0, 0, // data pub index, item 0 (little endian)
            3, 0, 0, 0, // t module index
            5, 0, 0, 0, // data internal idx
            0, // t data section type
            0, 0, 0, // padding
            //
            7, 0, 0, 0, // data pub index, item 1 (little endian)
            11, 0, 0, 0, // t module index
            13, 0, 0, 0, // data internal idx
            1, // t data section type
            0, 0, 0, // padding
            //
            17, 0, 0, 0, // data pub index, item 2 (little endian)
            19, 0, 0, 0, // t module index
            23, 0, 0, 0, // data internal idx
            1, // t data section type
            0, 0, 0, // padding
        ];

        let section = DataIndexSection::load(&section_data);

        let ranges = section.ranges;

        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0], RangeItem::new(0, 2,));
        assert_eq!(ranges[1], RangeItem::new(2, 1));

        let items = section.items;

        assert_eq!(items.len(), 3);

        assert_eq!(
            items[0],
            DataIndexItem::new(2, 3, 5, DataSectionType::ReadOnly,)
        );

        assert_eq!(
            items[1],
            DataIndexItem::new(7, 11, 13, DataSectionType::ReadWrite,)
        );

        assert_eq!(
            items[2],
            DataIndexItem::new(17, 19, 23, DataSectionType::ReadWrite,)
        );

        // test get index item
        assert_eq!(
            section
                .get_item_target_module_index_and_data_internal_index_and_data_section_type(0, 0),
            (3, 5, DataSectionType::ReadOnly)
        );

        assert_eq!(
            section
                .get_item_target_module_index_and_data_internal_index_and_data_section_type(0, 1),
            (11, 13, DataSectionType::ReadWrite,)
        );

        assert_eq!(
            section
                .get_item_target_module_index_and_data_internal_index_and_data_section_type(1, 0),
            (19, 23, DataSectionType::ReadWrite)
        );
    }

    #[test]
    fn test_save_section() {
        let ranges = vec![RangeItem::new(0, 2), RangeItem::new(2, 1)];

        let items = vec![
            DataIndexItem::new(2, 3, 5, DataSectionType::ReadOnly),
            DataIndexItem::new(7, 11, 13, DataSectionType::ReadWrite),
            DataIndexItem::new(17, 19, 23, DataSectionType::ReadWrite),
        ];

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
                0, 0, 0, 0, // offset 0 (item 0)
                2, 0, 0, 0, // count 0
                2, 0, 0, 0, // offset 1 (item 1)
                1, 0, 0, 0, // count 1
                //
                2, 0, 0, 0, // data pub index, item 0 (little endian)
                3, 0, 0, 0, // t module index
                5, 0, 0, 0, // data internal idx
                0, // t data section type
                0, 0, 0, // padding
                //
                7, 0, 0, 0, // data pub index, item 1 (little endian)
                11, 0, 0, 0, // t module index
                13, 0, 0, 0, // datainternal  idx
                1, // t data section type
                0, 0, 0, // padding
                //
                17, 0, 0, 0, // data pub index, item 2 (little endian)
                19, 0, 0, 0, // t module index
                23, 0, 0, 0, // data internal idx
                1, // t data section type
                0, 0, 0, // padding
            ]
        );
    }

    #[test]
    fn test_convert() {
        let entries = vec![
            DataIndexModuleEntry::new(vec![
                DataIndexEntry::new(0, 1, 3, DataSectionType::ReadOnly),
                DataIndexEntry::new(1, 5, 7, DataSectionType::ReadWrite),
                DataIndexEntry::new(2, 11, 13, DataSectionType::Uninit),
            ]),
            DataIndexModuleEntry::new(vec![
                DataIndexEntry::new(0, 17, 19, DataSectionType::ReadWrite),
                DataIndexEntry::new(1, 23, 29, DataSectionType::ReadWrite),
            ]),
        ];

        let (ranges, items) = DataIndexSection::convert_from_entries(&entries);

        let section = DataIndexSection {
            ranges: &ranges,
            items: &items,
        };

        assert_eq!(
            section
                .get_item_target_module_index_and_data_internal_index_and_data_section_type(0, 0),
            (1, 3, DataSectionType::ReadOnly)
        );

        assert_eq!(
            section
                .get_item_target_module_index_and_data_internal_index_and_data_section_type(0, 1),
            (5, 7, DataSectionType::ReadWrite)
        );

        assert_eq!(
            section
                .get_item_target_module_index_and_data_internal_index_and_data_section_type(0, 2),
            (11, 13, DataSectionType::Uninit)
        );

        assert_eq!(
            section
                .get_item_target_module_index_and_data_internal_index_and_data_section_type(1, 0),
            (17, 19, DataSectionType::ReadWrite)
        );

        assert_eq!(
            section
                .get_item_target_module_index_and_data_internal_index_and_data_section_type(1, 1),
            (23, 29, DataSectionType::ReadWrite)
        );
    }
}
