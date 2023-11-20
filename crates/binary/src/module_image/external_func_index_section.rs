// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// the purpose of these sections is to remove duplicated external functon items.
// - "external function index section"
// - "unified external library section"
// - "unified external function section"

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
//         |------------------------------------------------------------------------------------|
//         | external func idx 0 (u32) | unified external func idx 0 (u32) | type index 0 (u32) | <-- table 1
//         | external func idx 1       | unified external func idx 1       | type index 1       |
//         | ...                                                                                |
//         |------------------------------------------------------------------------------------|

use crate::utils::{load_section_with_two_tables, save_section_with_two_tables};

use super::{ModuleSectionId, RangeItem, SectionEntry};

#[derive(Debug, PartialEq, Default)]
pub struct ExternalFuncIndexSection<'a> {
    pub ranges: &'a [RangeItem],
    pub items: &'a [ExternalFuncIndexItem],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct ExternalFuncIndexItem {
    // this field is REDUNDANT because its value always starts
    // from 0 to the total number of items (within a certain range)/(within a module).
    pub external_func_index: u32,

    pub unified_external_func_index: u32,

    // copy the type_index from ExternalFuncSection of the specific module,
    // so that the ExternalFuncSection can be omitted at runtime.
    pub type_index: u32,
}

impl ExternalFuncIndexItem {
    pub fn new(
        external_func_index: u32,
        unified_external_func_index: u32,
        type_index: u32,
    ) -> Self {
        Self {
            external_func_index,
            unified_external_func_index,
            type_index,
        }
    }
}

#[derive(Debug)]
pub struct ExternalFuncIndexEntry {
    pub external_func_index: usize,
    pub unified_external_func_index: usize,
    pub type_index: usize,
}

impl ExternalFuncIndexEntry {
    pub fn new(
        external_func_index: usize,
        unified_external_func_index: usize,
        type_index: usize,
    ) -> Self {
        Self {
            external_func_index,
            unified_external_func_index,
            type_index,
        }
    }
}

#[derive(Debug)]
pub struct ExternalFuncIndexModuleEntry {
    pub index_entries: Vec<ExternalFuncIndexEntry>,
}

impl ExternalFuncIndexModuleEntry {
    pub fn new(index_entries: Vec<ExternalFuncIndexEntry>) -> Self {
        Self { index_entries }
    }
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

impl<'a> ExternalFuncIndexSection<'a> {
    pub fn get_item_unified_external_func_index_and_type_index(
        &self,
        module_index: usize,
        external_function_index: usize,
    ) -> (usize, usize) {
        let range = &self.ranges[module_index];

        // bounds check
        #[cfg(debug_assertions)]
        {
            if external_function_index > range.count as usize {
                panic!("Out of bounds of the external function index, module index:{}, total external functions: {}, external function index: {}",
                    module_index,
                    range.count,
                    external_function_index);
            }
        }

        let item_index = range.offset as usize + external_function_index;
        let item = &self.items[item_index];
        (
            item.unified_external_func_index as usize,
            item.type_index as usize,
        )
    }

    pub fn convert_from_entries(
        sorted_external_func_index_module_entries: &[ExternalFuncIndexModuleEntry],
    ) -> (Vec<RangeItem>, Vec<ExternalFuncIndexItem>) {
        let mut range_start_offset: u32 = 0;
        let range_items = sorted_external_func_index_module_entries
            .iter()
            .map(|index_module_entry| {
                let count = index_module_entry.index_entries.len() as u32;
                let range_item = RangeItem::new(range_start_offset, count);
                range_start_offset += count;
                range_item
            })
            .collect::<Vec<_>>();

        let external_func_index_items = sorted_external_func_index_module_entries
            .iter()
            .flat_map(|index_module_entry| {
                index_module_entry.index_entries.iter().map(|entry| {
                    ExternalFuncIndexItem::new(
                        entry.external_func_index as u32,
                        entry.unified_external_func_index as u32,
                        entry.type_index as u32,
                    )
                })
            })
            .collect::<Vec<_>>();

        (range_items, external_func_index_items)
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
            5, 0, 0, 0, // type index 0
            //
            7, 0, 0, 0, // external func idx 1, item 1
            11, 0, 0, 0, // uni external func idx 1
            13, 0, 0, 0, // type index 1
            //
            17, 0, 0, 0, // external func idx 2, item 2
            19, 0, 0, 0, // uni external func idx 2
            23, 0, 0, 0, // type index 2
        ];

        let section = ExternalFuncIndexSection::load(&section_data);

        let ranges = section.ranges;

        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0], RangeItem::new(0, 2,));
        assert_eq!(ranges[1], RangeItem::new(2, 1,));

        let items = section.items;

        assert_eq!(items.len(), 3);
        assert_eq!(items[0], ExternalFuncIndexItem::new(2, 3, 5));
        assert_eq!(items[1], ExternalFuncIndexItem::new(7, 11, 13));
        assert_eq!(items[2], ExternalFuncIndexItem::new(17, 19, 23));

        // test get index item
        assert_eq!(
            section.get_item_unified_external_func_index_and_type_index(0, 0),
            (3, 5)
        );

        assert_eq!(
            section.get_item_unified_external_func_index_and_type_index(0, 1),
            (11, 13)
        );

        assert_eq!(
            section.get_item_unified_external_func_index_and_type_index(1, 0),
            (19, 23)
        );
    }

    #[test]
    fn test_save_section() {
        let ranges = vec![RangeItem::new(0, 2), RangeItem::new(2, 1)];

        let items = vec![
            ExternalFuncIndexItem::new(2, 3, 5),
            ExternalFuncIndexItem::new(7, 11, 13),
            ExternalFuncIndexItem::new(17, 19, 23),
        ];

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
                5, 0, 0, 0, // type index 0
                //
                7, 0, 0, 0, // external func idx 1, item 1
                11, 0, 0, 0, // uni external func idx 1
                13, 0, 0, 0, // type index 1
                //
                17, 0, 0, 0, // external func idx 2, item 2
                19, 0, 0, 0, // uni external func idx 2
                23, 0, 0, 0, // type index 2
            ]
        );
    }

    #[test]
    fn test_convert() {
        // todo
    }
}
