// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// "function index section" binary layout
//
//         |--------------------------------------|
//         | item count (u32) | (4 bytes padding) |
//         |--------------------------------------|
// range 0 | offset 0 (u32) | count 0 (u32)       | <-- table 0
// range 1 | offset 1       | count 1             |
//         | ...                                  |
//         |--------------------------------------|
//
//         |------------------------------------------------------------------------------|
//         | func public idx 0 (u32) | target mod idx 0 (u32) | func internal idx 0 (u32) | <-- table 1
//         | func public idx 1       | target mod idx 1       | func internal idx 1       |
//         | ...                                                                          |
//         |------------------------------------------------------------------------------|

use crate::utils::{load_section_with_two_tables, save_section_with_two_tables};

use super::{ModuleSectionId, RangeItem, SectionEntry};

#[derive(Debug, PartialEq)]
pub struct FuncIndexSection<'a> {
    pub ranges: &'a [RangeItem],
    pub items: &'a [FuncIndexItem],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct FuncIndexItem {
    // the index of function item, includes the imported functions and internal functions.
    // 'function public index' equals to
    // 'amount of imported functions' + 'function internal index'
    //
    // this field is REDUNDANT because its value always starts
    // from 0 to the total number of items (within a certain range)/(within a module).
    pub func_public_index: u32,

    // target module index
    pub target_module_index: u32,

    // the index of the internal function in a specified module
    //
    // this index is the actual index of the internal functions in a specified module
    // i.e., it excludes the imported functions.
    pub function_internal_index: u32,
}

impl FuncIndexItem {
    pub fn new(
        func_public_index: u32,
        target_module_index: u32,
        function_internal_index: u32,
    ) -> Self {
        Self {
            func_public_index,
            target_module_index,
            function_internal_index,
        }
    }
}

#[derive(Debug)]
pub struct FuncIndexEntry {
    pub func_public_index: usize,
    pub target_module_index: usize,
    pub function_internal_index: usize,
}

impl FuncIndexEntry {
    pub fn new(
        func_public_index: usize,
        target_module_index: usize,
        function_internal_index: usize,
    ) -> Self {
        Self {
            func_public_index,
            target_module_index,
            function_internal_index,
        }
    }
}

#[derive(Debug)]
pub struct FuncIndexModuleEntry {
    pub index_entries: Vec<FuncIndexEntry>,
}

impl FuncIndexModuleEntry {
    pub fn new(index_entries: Vec<FuncIndexEntry>) -> Self {
        Self { index_entries }
    }
}

impl<'a> SectionEntry<'a> for FuncIndexSection<'a> {
    fn load(section_data: &'a [u8]) -> Self {
        let (ranges, items) =
            load_section_with_two_tables::<RangeItem, FuncIndexItem>(section_data);

        FuncIndexSection { ranges, items }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_two_tables(self.ranges, self.items, writer)
    }

    fn id(&'a self) -> ModuleSectionId {
        ModuleSectionId::FuncIndex
    }
}

impl<'a> FuncIndexSection<'a> {
    pub fn get_item_target_module_index_and_function_internal_index(
        &self,
        module_index: usize,
        function_public_index: usize,
    ) -> (usize, usize) {
        let range = &self.ranges[module_index];

        // bounds check
        #[cfg(debug_assertions)]
        {
            if function_public_index > range.count as usize {
                panic!("Out of bounds of the function index, module index: {}, total functions (includes imported): {}, request function public index: {}.",
                    module_index,
                    range.count,
                    function_public_index
                );
            }
        }

        let item_index = range.offset as usize + function_public_index;
        let item = &self.items[item_index];
        (
            item.target_module_index as usize,
            item.function_internal_index as usize,
        )
    }

    pub fn convert_from_entries(
        sorted_entries: &[FuncIndexModuleEntry],
    ) -> (Vec<RangeItem>, Vec<FuncIndexItem>) {
        let mut range_start_offset: u32 = 0;
        let range_items = sorted_entries
            .iter()
            .map(|index_module_entry| {
                let count = index_module_entry.index_entries.len() as u32;
                let range_item = RangeItem::new(range_start_offset, count);
                range_start_offset += count;
                range_item
            })
            .collect::<Vec<_>>();

        let func_index_items = sorted_entries
            .iter()
            .flat_map(|index_module_entry| {
                index_module_entry.index_entries.iter().map(|entry| {
                    FuncIndexItem::new(
                        entry.func_public_index as u32,
                        entry.target_module_index as u32,
                        entry.function_internal_index as u32,
                    )
                })
            })
            .collect::<Vec<_>>();

        (range_items, func_index_items)
    }
}

#[cfg(test)]
mod tests {
    use crate::module_image::{
        func_index_section::{FuncIndexItem, FuncIndexSection},
        RangeItem, SectionEntry,
    };

    use super::{FuncIndexEntry, FuncIndexModuleEntry};

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
            1, 0, 0, 0, // func pub idx 0, item 0 (little endian)
            2, 0, 0, 0, // target module idx 0
            3, 0, 0, 0, // func internal idx 0
            //
            5, 0, 0, 0, // func pub idx 1, item 1
            7, 0, 0, 0, // target module idx 1
            11, 0, 0, 0, // func internal idx 1
            //
            13, 0, 0, 0, // func pub idx 2, item 2
            17, 0, 0, 0, // target module idx 2
            19, 0, 0, 0, // func internal idx 2
        ];

        let section = FuncIndexSection::load(&section_data);

        let ranges = section.ranges;

        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0], RangeItem::new(0, 2,));
        assert_eq!(ranges[1], RangeItem::new(2, 1,));

        let items = section.items;

        assert_eq!(items.len(), 3);
        assert_eq!(items[0], FuncIndexItem::new(1, 2, 3,));
        assert_eq!(items[1], FuncIndexItem::new(5, 7, 11,));
        assert_eq!(items[2], FuncIndexItem::new(13, 17, 19,));

        // test get index item
        assert_eq!(
            section.get_item_target_module_index_and_function_internal_index(0, 0),
            (2, 3,)
        );

        assert_eq!(
            section.get_item_target_module_index_and_function_internal_index(0, 1),
            (7, 11,)
        );

        assert_eq!(
            section.get_item_target_module_index_and_function_internal_index(1, 0),
            (17, 19,)
        );
    }

    #[test]
    fn test_save_section() {
        let ranges = vec![RangeItem::new(0, 2), RangeItem::new(2, 1)];

        let items = vec![
            FuncIndexItem::new(1, 2, 3),
            FuncIndexItem::new(5, 7, 11),
            FuncIndexItem::new(13, 17, 19),
        ];

        let section = FuncIndexSection {
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
                1, 0, 0, 0, // func puc idx 0, item 0 (little endian)
                2, 0, 0, 0, // target module idx 0
                3, 0, 0, 0, // func internal idx 0
                //
                5, 0, 0, 0, // func puc idx 1, item 1
                7, 0, 0, 0, // target module idx 1
                11, 0, 0, 0, // func internal idx 1
                //
                13, 0, 0, 0, // func puc idx 2, item 2
                17, 0, 0, 0, // target module idx 2
                19, 0, 0, 0, // func internal idx 2
            ]
        );
    }

    #[test]
    fn test_convert() {
        let entries = vec![
            FuncIndexModuleEntry::new(vec![
                FuncIndexEntry::new(0, 1, 3),
                FuncIndexEntry::new(1, 5, 7),
            ]),
            FuncIndexModuleEntry::new(vec![
                FuncIndexEntry::new(0, 11, 13),
                FuncIndexEntry::new(1, 17, 19),
                FuncIndexEntry::new(2, 23, 29),
            ]),
        ];

        let (ranges, items) = FuncIndexSection::convert_from_entries(&entries);

        let section = FuncIndexSection {
            ranges: &ranges,
            items: &items,
        };

        assert_eq!(
            section.get_item_target_module_index_and_function_internal_index(0, 0),
            (1, 3)
        );

        assert_eq!(
            section.get_item_target_module_index_and_function_internal_index(0, 1),
            (5, 7)
        );

        assert_eq!(
            section.get_item_target_module_index_and_function_internal_index(1, 0),
            (11, 13)
        );

        assert_eq!(
            section.get_item_target_module_index_and_function_internal_index(1, 1),
            (17, 19)
        );

        assert_eq!(
            section.get_item_target_module_index_and_function_internal_index(1, 2),
            (23, 29)
        );
    }
}
