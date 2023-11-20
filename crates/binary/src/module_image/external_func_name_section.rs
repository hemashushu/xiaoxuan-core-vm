// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// "external func name section" binary layout
//
//              |-------------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                                    |
//              |-------------------------------------------------------------------------|
//  item 0 -->  | name offset 0 (u32) | name length 0 (u32) | external_func_index 0 (u32) | <-- table
//  item 1 -->  | name offset 1       | name length 1       | external_func_index 1       |
//              | ...                                                                     |
//              |-------------------------------------------------------------------------|
// offset 0 --> | name string 0 (UTF-8)                                                   | <-- data area
// offset 1 --> | name string 1                                                           |
//              | ...                                                                     |
//              |-------------------------------------------------------------------------|

use crate::utils::{load_section_with_table_and_data_area, save_section_with_table_and_data_area};

use super::{ModuleSectionId, SectionEntry};

#[derive(Debug, PartialEq)]
pub struct ExternalFuncNameSection<'a> {
    pub items: &'a [ExternalFuncNameItem],
    pub names_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct ExternalFuncNameItem {
    pub name_offset: u32,
    pub name_length: u32,
    pub external_func_index: u32,
}

impl ExternalFuncNameItem {
    pub fn new(name_offset: u32, name_length: u32, external_func_index: u32) -> Self {
        Self {
            name_offset,
            name_length,
            external_func_index,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ExternalFuncNameEntry {
    pub name: String,
    pub external_func_index: usize,
}

impl ExternalFuncNameEntry {
    pub fn new(name: String, external_func_index: usize) -> Self {
        Self {
            name,
            external_func_index,
        }
    }
}

impl<'a> SectionEntry<'a> for ExternalFuncNameSection<'a> {
    fn load(section_data: &'a [u8]) -> Self {
        let (items, names_data) =
            load_section_with_table_and_data_area::<ExternalFuncNameItem>(section_data);
        ExternalFuncNameSection { items, names_data }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_table_and_data_area(self.items, self.names_data, writer)
    }

    fn id(&'a self) -> ModuleSectionId {
        ModuleSectionId::ExternalFuncName
    }
}

impl<'a> ExternalFuncNameSection<'a> {
    pub fn get_item_index(&'a self, expected_name: &str) -> Option<usize> {
        let names_data = self.names_data;

        let expected_name_data = expected_name.as_bytes();

        self.items.iter().position(|item| {
            let name_data = &names_data
                [item.name_offset as usize..(item.name_offset + item.name_length) as usize];
            name_data == expected_name_data
        })
    }

    pub fn convert_from_entries(
        entries: &[ExternalFuncNameEntry],
    ) -> (Vec<ExternalFuncNameItem>, Vec<u8>) {
        let name_bytes = entries
            .iter()
            .map(|entry| entry.name.as_bytes())
            .collect::<Vec<&[u8]>>();

        let mut next_offset: u32 = 0;

        let items = entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let name_offset = next_offset;
                let name_length = name_bytes[idx].len() as u32;
                next_offset += name_length; // for next offset

                ExternalFuncNameItem::new(
                    name_offset,
                    name_length,
                    entry.external_func_index as u32,
                )
            })
            .collect::<Vec<ExternalFuncNameItem>>();

        let names_data = name_bytes
            .iter()
            .flat_map(|bytes| bytes.to_vec())
            .collect::<Vec<u8>>();

        (items, names_data)
    }
}

#[cfg(test)]
mod tests {
    use crate::module_image::{
        external_func_name_section::{
            ExternalFuncNameEntry, ExternalFuncNameItem, ExternalFuncNameSection,
        },
        SectionEntry,
    };

    #[test]
    fn test_save_section() {
        let items: Vec<ExternalFuncNameItem> = vec![
            ExternalFuncNameItem::new(0, 3, 11),
            ExternalFuncNameItem::new(3, 5, 13),
        ];

        let section = ExternalFuncNameSection {
            items: &items,
            names_data: "foohello".as_bytes(),
        };

        let mut section_data: Vec<u8> = Vec::new();
        section.save(&mut section_data).unwrap();

        let mut expect_data = vec![
            2u8, 0, 0, 0, // item count
            0, 0, 0, 0, // 4 bytes padding
            //
            0, 0, 0, 0, // name offset (item 0)
            3, 0, 0, 0, // name length
            11, 0, 0, 0, // func pub index
            //
            3, 0, 0, 0, // name offset (item 1)
            5, 0, 0, 0, // name length
            13, 0, 0, 0, // func pub index
        ];

        expect_data.extend_from_slice(b"foo");
        expect_data.extend_from_slice(b"hello");

        assert_eq!(section_data, expect_data);
    }

    #[test]
    fn test_load_section() {
        let mut section_data = vec![
            2u8, 0, 0, 0, // item count
            0, 0, 0, 0, // 4 bytes padding
            //
            0, 0, 0, 0, // name offset (item 0)
            3, 0, 0, 0, // name length
            11, 0, 0, 0, // func pub index
            //
            3, 0, 0, 0, // name offset (item 1)
            5, 0, 0, 0, // name length
            13, 0, 0, 0, // func pub index
        ];

        section_data.extend_from_slice("foo".as_bytes());
        section_data.extend_from_slice("hello".as_bytes());

        let section = ExternalFuncNameSection::load(&section_data);

        assert_eq!(section.items.len(), 2);
        assert_eq!(section.items[0], ExternalFuncNameItem::new(0, 3, 11));
        assert_eq!(section.items[1], ExternalFuncNameItem::new(3, 5, 13));
        assert_eq!(section.names_data, "foohello".as_bytes())
    }

    #[test]
    fn test_convert() {
        let entries: Vec<ExternalFuncNameEntry> = vec![
            ExternalFuncNameEntry::new("foo".to_string(), 11),
            ExternalFuncNameEntry::new("hello".to_string(), 13),
        ];

        let (items, names_data) = ExternalFuncNameSection::convert_from_entries(&entries);
        let section = ExternalFuncNameSection {
            items: &items,
            names_data: &names_data,
        };

        assert_eq!(section.get_item_index("foo"), Some(0));

        assert_eq!(section.get_item_index("hello"), Some(1));

        assert_eq!(section.get_item_index("bar"), None);
    }
}
