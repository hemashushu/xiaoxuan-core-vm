// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// "func index section" binary layout
//
// |---------------------------------------------------------------|
// | item count (u32) | (4 bytes padding)                          |
// |---------------------------------------------------------------|
// | offset 0 (u32) | count 0 (u32)                                | <-- table 0
// | offset 1       | count 1                                      |
// | ...                                                           |
// |---------------------------------------------------------------|
//
// |---------------------------------------------------------------------|
// | func idx 0 (u32) | target mod idx 0 (u32) | target func idx 0 (u32) | <-- table 1
// | func idx 1       | target mod idx 1       | target func idx 1       |
// | ...                                                                 |
// |---------------------------------------------------------------------|

use crate::utils::{load_section_with_two_tables, save_section_with_two_tables};

use super::{RangeItem, SectionEntry, SectionId};

#[derive(Debug, PartialEq)]
pub struct FuncIndexSection<'a> {
    pub ranges: &'a [RangeItem],
    pub items: &'a [FuncIndexItem],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct FuncIndexItem {
    pub func_index: u32,          // data item index (in a specified module)
    pub target_module_index: u32, // target module index
    pub target_func_index: u32,   // target func index
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

    fn id(&'a self) -> SectionId {
        SectionId::FuncIndex
    }
}

impl FuncIndexItem {
    pub fn new(func_index: u32, target_module_index: u32, target_func_index: u32) -> Self {
        Self {
            func_index,
            target_module_index,
            target_func_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::module_image::{
        func_index_section::{FuncIndexItem, FuncIndexSection},
        RangeItem, SectionEntry,
    };

    #[test]
    fn test_load_section() {
        let section_data = vec![
            2u8, 0, 0, 0, // item count (little endian)
            0, 0, 0, 0, // 4 bytes padding
            //
            1, 0, 0, 0, // offset 0 (item 0)
            2, 0, 0, 0, // count 0
            3, 0, 0, 0, // offset 1 (item 1)
            5, 0, 0, 0, // count 1
            //
            1, 0, 0, 0, // func idx 0, item 0 (little endian)
            2, 0, 0, 0, // target module idx 0
            3, 0, 0, 0, // target func idx 0
            //
            5, 0, 0, 0, // func idx 1, item 1
            7, 0, 0, 0, // target module idx 1
            11, 0, 0, 0, // target func idx 1
            //
            13, 0, 0, 0, // func idx 2, item 2
            17, 0, 0, 0, // target module idx 2
            19, 0, 0, 0, // target func idx 2
        ];

        let section = FuncIndexSection::load(&section_data);

        let ranges = section.ranges;

        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0], RangeItem::new(1, 2,));
        assert_eq!(ranges[1], RangeItem::new(3, 5,));

        let items = section.items;

        assert_eq!(items.len(), 3);
        assert_eq!(items[0], FuncIndexItem::new(1, 2, 3,));
        assert_eq!(items[1], FuncIndexItem::new(5, 7, 11,));
        assert_eq!(items[2], FuncIndexItem::new(13, 17, 19,));
    }

    #[test]
    fn test_save_section() {
        let mut ranges: Vec<RangeItem> = Vec::new();

        ranges.push(RangeItem::new(1, 2));
        ranges.push(RangeItem::new(3, 5));

        let mut items: Vec<FuncIndexItem> = Vec::new();

        items.push(FuncIndexItem::new(1, 2, 3));
        items.push(FuncIndexItem::new(5, 7, 11));
        items.push(FuncIndexItem::new(13, 17, 19));

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
                1, 0, 0, 0, // offset 0 (item 0)
                2, 0, 0, 0, // count 0
                3, 0, 0, 0, // offset 1 (item 1)
                5, 0, 0, 0, // count 1
                //
                1, 0, 0, 0, // func idx 0, item 0 (little endian)
                2, 0, 0, 0, // target module idx 0
                3, 0, 0, 0, // target func idx 0
                //
                5, 0, 0, 0, // func idx 1, item 1
                7, 0, 0, 0, // target module idx 1
                11, 0, 0, 0, // target func idx 1
                //
                13, 0, 0, 0, // func idx 2, item 2
                17, 0, 0, 0, // target module idx 2
                19, 0, 0, 0, // target func idx 2
            ]
        );
    }
}
