// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// "start function list section" binary layout
//
// |-------------------------------------------|
// | item count (u32) | (4 bytes padding)      |
// |-------------------------------------------|
// | mod idx 0 (u32) | func public idx 0 (u32) |
// | mod idx 1       | func public idx 1       |
// | ...                                       |
// |-------------------------------------------|

use ancvm_types::entry::ModuleFunctionIndexEntry;

use crate::utils::{load_section_with_one_table, save_section_with_one_table};

use super::{ModuleFunctionIndexItem, ModuleSectionId, SectionEntry};

#[derive(Debug, PartialEq)]
pub struct StartFunctionListSection<'a> {
    pub items: &'a [ModuleFunctionIndexItem],
}

impl<'a> SectionEntry<'a> for StartFunctionListSection<'a> {
    fn id(&'a self) -> ModuleSectionId {
        ModuleSectionId::StartFunctionList
    }

    fn load(section_data: &'a [u8]) -> Self
    where
        Self: Sized,
    {
        let items = load_section_with_one_table::<ModuleFunctionIndexItem>(section_data);
        StartFunctionListSection { items }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_one_table(self.items, writer)
    }
}

impl<'a> StartFunctionListSection<'a> {
    pub fn convert_from_entries(
        entries: &[ModuleFunctionIndexEntry],
    ) -> Vec<ModuleFunctionIndexItem> {
        entries
            .iter()
            .map(|entry| ModuleFunctionIndexItem {
                module_index: entry.module_index as u32,
                function_public_index: entry.function_public_index as u32,
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use crate::module_image::{
        start_function_list_section::StartFunctionListSection, ModuleFunctionIndexItem,
        SectionEntry,
    };

    #[test]
    fn test_load_section() {
        let section_data = vec![
            2u8, 0, 0, 0, // item count (little endian)
            0, 0, 0, 0, // 4 bytes padding
            //
            11, 0, 0, 0, // mod idx 0 (item 0)
            13, 0, 0, 0, // func pub idc 0
            //
            17, 0, 0, 0, // offset 1 (item 1)
            19, 0, 0, 0, // count 1
        ];

        let section = StartFunctionListSection::load(&section_data);
        let items = section.items;

        assert_eq!(items.len(), 2);
        assert_eq!(items[0], ModuleFunctionIndexItem::new(11, 13));
        assert_eq!(items[1], ModuleFunctionIndexItem::new(17, 19));
    }

    #[test]
    fn test_save_section() {
        let items = vec![
            ModuleFunctionIndexItem::new(11, 13),
            ModuleFunctionIndexItem::new(17, 19),
        ];

        let section = StartFunctionListSection { items: &items };

        let mut section_data: Vec<u8> = Vec::new();
        section.save(&mut section_data).unwrap();

        assert_eq!(
            section_data,
            vec![
                2u8, 0, 0, 0, // item count (little endian)
                0, 0, 0, 0, // 4 bytes padding
                //
                11, 0, 0, 0, // mod idx 0 (item 0)
                13, 0, 0, 0, // func pub idc 0
                //
                17, 0, 0, 0, // offset 1 (item 1)
                19, 0, 0, 0, // count 1
            ]
        );
    }
}
