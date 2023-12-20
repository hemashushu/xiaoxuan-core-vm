// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// "exit function list section" binary layout
//
// |--------------------------------------|
// | item count (u32) | (4 bytes padding) |
// |--------------------------------------|
// | func public idx 0 (u32)              |
// | func public idx 1                    |
// | ...                                  |
// |--------------------------------------|

use crate::utils::{load_section_with_one_table, save_section_with_one_table};

use super::{ModuleSectionId, SectionEntry};

#[derive(Debug, PartialEq)]
pub struct ExitFunctionListSection<'a> {
    pub items: &'a [u32],
}

impl<'a> SectionEntry<'a> for ExitFunctionListSection<'a> {
    fn id(&'a self) -> ModuleSectionId {
        ModuleSectionId::ExitFunctionList
    }

    fn load(section_data: &'a [u8]) -> Self
    where
        Self: Sized,
    {
        let items = load_section_with_one_table::<u32>(section_data);
        ExitFunctionListSection { items }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_one_table(self.items, writer)
    }
}

impl<'a> ExitFunctionListSection<'a> {
    pub fn convert_from_entries(entries: &[u32]) -> Vec<u32> {
        entries.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use crate::module_image::{exit_function_list_section::ExitFunctionListSection, SectionEntry};

    #[test]
    fn test_load_section() {
        let section_data = vec![
            4u8, 0, 0, 0, // item count (little endian)
            0, 0, 0, 0, // 4 bytes padding
            //
            11, 0, 0, 0, // func pub idx 0
            13, 0, 0, 0, // func pub idx 1
            17, 0, 0, 0, // func pub idx 2
            19, 0, 0, 0, // func pub idx 3
        ];

        let section = ExitFunctionListSection::load(&section_data);
        let items = section.items;

        assert_eq!(items.len(), 4);
        assert_eq!(items[0], 11);
        assert_eq!(items[1], 13);
        assert_eq!(items[2], 17);
        assert_eq!(items[3], 19);
    }

    #[test]
    fn test_save_section() {
        let items = vec![11, 13, 17, 19];

        let section = ExitFunctionListSection { items: &items };

        let mut section_data: Vec<u8> = Vec::new();
        section.save(&mut section_data).unwrap();

        assert_eq!(
            section_data,
            vec![
                4u8, 0, 0, 0, // item count (little endian)
                0, 0, 0, 0, // 4 bytes padding
                //
                11, 0, 0, 0, // func pub idx 0
                13, 0, 0, 0, // func pub idx 1
                17, 0, 0, 0, // func pub idx 2
                19, 0, 0, 0, // func pub idx 3
            ]
        );
    }
}
