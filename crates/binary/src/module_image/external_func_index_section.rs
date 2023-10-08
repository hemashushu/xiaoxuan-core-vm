// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// the purpose of these sections are to remove duplicate external functon items.
// - "external function index section"
// - "unified external library section"
// - "unified external library section"

// "external function index section" binary layout
//
//         |--------------------------------------|
//         | item count (u32) | (4 bytes padding) |
//         |--------------------------------------|
// range 0 | offset 0 (u32) | count 0 (u32)       | <-- table 0
// range 1 | offset 1       | count 1             |
//         | ...                                  |
//         |--------------------------------------|
//
//         |---------------------------------------------------------------|
//         | external func idx 0 (u32) | unified external func idx 0 (u32) | <-- table 1
//         | external func idx 1       | unified external func idx 1       |
//         | ...                                                           |
//         |---------------------------------------------------------------|

use crate::utils::{load_section_with_two_tables, save_section_with_two_tables};

use super::{ModuleSectionId, RangeItem, SectionEntry};

#[derive(Debug, PartialEq)]
pub struct ExternalFuncIndexSection<'a> {
    pub ranges: &'a [RangeItem],
    pub items: &'a [ExternalFuncIndexItem],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct ExternalFuncIndexItem {
    pub external_func_index: u32,
    pub unified_external_func_index: u32,
}

impl<'a> SectionEntry<'a> for ExternalFuncIndexSection<'a> {
    fn load(section_data: &'a [u8]) -> Self {
        let (ranges, items) =
            load_section_with_two_tables::<RangeItem, ExternalFuncIndexItem>(section_data);

        ExternalFuncIndexSection { ranges, items }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_two_tables(self.ranges, self.items, writer)
    }

    fn id(&'a self) -> ModuleSectionId {
        ModuleSectionId::ExternalFuncIndex
    }
}

impl ExternalFuncIndexItem {
    pub fn new(external_func_index: u32, unified_external_func_index: u32) -> Self {
        Self {
            external_func_index,
            unified_external_func_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::module_image::{
        external_func_index_section::{ExternalFuncIndexItem, ExternalFuncIndexSection},
        RangeItem, SectionEntry,
    };

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
            2, 0, 0, 0, // external func idx 0, item 0 (little endian)
            3, 0, 0, 0, // uni external func idx 0
            //
            5, 0, 0, 0, // external func idx 1, item 1
            7, 0, 0, 0, // uni external func idx 1
            //
            11, 0, 0, 0, // external func idx 2, item 2
            13, 0, 0, 0, // uni external func idx 2
        ];

        let section = ExternalFuncIndexSection::load(&section_data);

        let ranges = section.ranges;

        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0], RangeItem::new(0, 2,));
        assert_eq!(ranges[1], RangeItem::new(2, 1,));

        let items = section.items;

        assert_eq!(items.len(), 3);
        assert_eq!(items[0], ExternalFuncIndexItem::new(2, 3,));
        assert_eq!(items[1], ExternalFuncIndexItem::new(5, 7,));
        assert_eq!(items[2], ExternalFuncIndexItem::new(11, 13,));
    }

    #[test]
    fn test_save_section() {
        let mut ranges: Vec<RangeItem> = Vec::new();

        ranges.push(RangeItem::new(0, 2));
        ranges.push(RangeItem::new(2, 1));

        let mut items: Vec<ExternalFuncIndexItem> = Vec::new();

        items.push(ExternalFuncIndexItem::new(2, 3));
        items.push(ExternalFuncIndexItem::new(5, 7));
        items.push(ExternalFuncIndexItem::new(11, 13));

        let section = ExternalFuncIndexSection {
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
                2, 0, 0, 0, // external func idx 0, item 0 (little endian)
                3, 0, 0, 0, // uni external func idx 0
                //
                5, 0, 0, 0, // external func idx 1, item 1
                7, 0, 0, 0, // uni external func idx 1
                //
                11, 0, 0, 0, // external func idx 2, item 2
                13, 0, 0, 0, // uni external func idx 2
            ]
        );
    }
}
