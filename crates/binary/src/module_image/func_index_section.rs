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

use super::{SectionEntry, SectionId};

#[derive(Debug, PartialEq)]
pub struct FuncIndexSection<'a> {
    pub offsets: &'a [FuncIndexOffset],
    pub items: &'a [FuncIndexItem],
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
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct FuncIndexOffset {
    pub offset: u32,
    pub count: u32,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct FuncIndexItem {
    pub func_index: u32,          // data item index (in a specified module)
    pub target_module_index: u32, // target module index
    // _padding0: u16,
    pub target_func_index: u32, // target func index
}

impl<'a> SectionEntry<'a> for FuncIndexSection<'a> {
    fn load(section_data: &'a [u8]) -> Self {
        let (offsets, items) =
            load_section_with_two_tables::<FuncIndexOffset, FuncIndexItem>(section_data);

        FuncIndexSection { offsets, items }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_two_tables(self.offsets, self.items, writer)
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
            // _padding0: 0,
            target_func_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::module_image::{
        func_index_section::{FuncIndexItem, FuncIndexOffset, FuncIndexSection},
        SectionEntry,
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

        let offsets = section.offsets;

        assert_eq!(offsets.len(), 2);
        assert_eq!(
            offsets[0],
            FuncIndexOffset {
                offset: 1,
                count: 2,
            }
        );
        assert_eq!(
            offsets[1],
            FuncIndexOffset {
                offset: 3,
                count: 5,
            }
        );

        let items = section.items;

        assert_eq!(items.len(), 3);
        assert_eq!(items[0], FuncIndexItem::new(1, 2, 3,));
        assert_eq!(items[1], FuncIndexItem::new(5, 7, 11,));
        assert_eq!(items[2], FuncIndexItem::new(13, 17, 19,));
    }

    #[test]
    fn test_save_section() {
        let mut offsets: Vec<FuncIndexOffset> = Vec::new();

        offsets.push(FuncIndexOffset {
            offset: 1,
            count: 2,
        });

        offsets.push(FuncIndexOffset {
            offset: 3,
            count: 5,
        });

        let mut items: Vec<FuncIndexItem> = Vec::new();

        items.push(FuncIndexItem::new(1, 2, 3));
        items.push(FuncIndexItem::new(5, 7, 11));
        items.push(FuncIndexItem::new(13, 17, 19));

        let section = FuncIndexSection {
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
