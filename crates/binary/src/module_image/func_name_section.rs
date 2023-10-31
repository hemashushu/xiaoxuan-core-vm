// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// "func name section" binary layout
//
//              |----------------------------------------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                                                               |
//              |----------------------------------------------------------------------------------------------------|
//  item 0 -->  | name offset 0 (u32) | name length 0 (u32) | func_pub_index 0 (u32) | exported 0 (u8) | pad 3 bytes | <-- table
//  item 1 -->  | name offset 1       | name length 1       | func_pub_index 1       | exported 1      | pad 3 bytes |
//              | ...                                                                                                |
//              |----------------------------------------------------------------------------------------------------|
// offset 0 --> | name string 0 (UTF-8)                                                                              | <-- data area
// offset 1 --> | name string 1                                                                                      |
//              | ...                                                                                                |
//              |----------------------------------------------------------------------------------------------------|

use crate::utils::{load_section_with_table_and_data_area, save_section_with_table_and_data_area};

use super::{ModuleSectionId, SectionEntry};

#[derive(Debug, PartialEq)]
pub struct FuncNameSection<'a> {
    pub items: &'a [FuncNameItem],
    pub names_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct FuncNameItem {
    pub name_offset: u32,
    pub name_length: u32,
    pub func_pub_index: u32,
    pub exported: u8, // 0=false, 1=true
    _padding0: [u8; 3],
}

impl FuncNameItem {
    pub fn new(name_offset: u32, name_length: u32, func_pub_index: u32, exported: u8) -> Self {
        Self {
            name_offset,
            name_length,
            func_pub_index,
            exported,
            _padding0: [0, 0, 0],
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FuncNameEntry {
    pub name: String,
    pub func_pub_index: usize,
    pub exported: bool,
}

impl FuncNameEntry {
    pub fn new(name: String, func_pub_index: usize, exported: bool) -> Self {
        Self {
            name,
            func_pub_index,
            exported,
        }
    }
}

impl<'a> SectionEntry<'a> for FuncNameSection<'a> {
    fn load(section_data: &'a [u8]) -> Self {
        let (items, names_data) =
            load_section_with_table_and_data_area::<FuncNameItem>(section_data);
        FuncNameSection { items, names_data }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_table_and_data_area(self.items, self.names_data, writer)
    }

    fn id(&'a self) -> ModuleSectionId {
        ModuleSectionId::FuncName
    }
}

impl<'a> FuncNameSection<'a> {
    pub fn get_item_index_and_exported(&'a self, expected_name: &str) -> Option<(usize, bool)> {
        let items = self.items;
        let names_data = self.names_data;

        let expected_name_data = expected_name.as_bytes();

        let opt_idx = self.items.iter().position(|item| {
            let name_data = &names_data
                [item.name_offset as usize..(item.name_offset + item.name_length) as usize];
            name_data == expected_name_data
        });

        opt_idx.map(|idx| {
            let item = &items[idx];
            (idx, item.exported != 0)
        })
    }

    pub fn convert_from_entries(entries: &[FuncNameEntry]) -> (Vec<FuncNameItem>, Vec<u8>) {
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

                FuncNameItem::new(
                    name_offset,
                    name_length,
                    entry.func_pub_index as u32,
                    if entry.exported { 1 } else { 0 },
                )
            })
            .collect::<Vec<FuncNameItem>>();

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
        func_name_section::{FuncNameEntry, FuncNameItem, FuncNameSection},
        SectionEntry,
    };

    #[test]
    fn test_save_section() {
        let items: Vec<FuncNameItem> = vec![
            FuncNameItem::new(0, 3, 11, 0),
            FuncNameItem::new(3, 5, 13, 1),
        ];

        let section = FuncNameSection {
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
            0, // exported
            0, 0, 0, // padding
            //
            3, 0, 0, 0, // name offset (item 1)
            5, 0, 0, 0, // name length
            13, 0, 0, 0, // func pub index
            1, // exported
            0, 0, 0, // padding
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
            0, // exported
            0, 0, 0, // padding
            //
            3, 0, 0, 0, // name offset (item 1)
            5, 0, 0, 0, // name length
            13, 0, 0, 0, // func pub index
            1, // exported
            0, 0, 0, // padding
        ];

        section_data.extend_from_slice("foo".as_bytes());
        section_data.extend_from_slice("hello".as_bytes());

        let section = FuncNameSection::load(&section_data);

        assert_eq!(section.items.len(), 2);
        assert_eq!(section.items[0], FuncNameItem::new(0, 3, 11, 0));
        assert_eq!(section.items[1], FuncNameItem::new(3, 5, 13, 1));
        assert_eq!(section.names_data, "foohello".as_bytes())
    }

    #[test]
    fn test_convert() {
        let entries: Vec<FuncNameEntry> = vec![
            FuncNameEntry::new("foo".to_string(), 11, false),
            FuncNameEntry::new("hello".to_string(), 13, true),
        ];

        let (items, names_data) = FuncNameSection::convert_from_entries(&entries);
        let section = FuncNameSection {
            items: &items,
            names_data: &names_data,
        };

        assert_eq!(section.get_item_index_and_exported("foo"), Some((0, false)));

        assert_eq!(
            section.get_item_index_and_exported("hello"),
            Some((1, true))
        );

        assert_eq!(section.get_item_index_and_exported("bar"), None);
    }
}
