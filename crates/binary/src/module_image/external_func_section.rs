// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// "external function section" binary layout
//
//              |-------------------------------------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                                                            |
//              |-------------------------------------------------------------------------------------------------|
//  item 0 -->  | lib name off 0 (u32) | lib name len 0 (u32) | external library idx 0 (u32) | type index 0 (u32) | <-- table
//  item 1 -->  | lib name off 1       | lib name len 1       | external library idx 1       | type index 1       |
//              | ...                                                                                             |
//              |-------------------------------------------------------------------------------------------------|
// offset 0 --> | name string 0 (UTF-8)                                                                           | <-- data area
// offset 1 --> | name string 1                                                                                   |
//              | ...                                                                                             |
//              |-------------------------------------------------------------------------------------------------|

use crate::utils::{load_section_with_table_and_data_area, save_section_with_table_and_data_area};

use super::{ModuleSectionId, SectionEntry};

#[derive(Debug, PartialEq)]
pub struct ExternalFuncSection<'a> {
    pub items: &'a [ExternalFuncItem],
    pub names_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct ExternalFuncItem {
    pub name_offset: u32, // the offset of the name string in data area
    pub name_length: u32, // the length (in bytes) of the name string in data area
    pub external_library_index: u32,
    pub type_index: u32, // the function type
}

impl ExternalFuncItem {
    pub fn new(
        name_offset: u32,
        name_length: u32,
        external_library_index: u32,
        type_index: u32,
    ) -> Self {
        Self {
            name_offset,
            name_length,
            external_library_index,
            type_index,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExternalFuncEntry {
    pub name: String,
    pub external_library_index: usize,
    pub type_index: usize,
}

impl ExternalFuncEntry {
    pub fn new(name: String, external_library_index: usize, type_index: usize) -> Self {
        Self {
            name,
            external_library_index,
            type_index,
        }
    }
}

impl<'a> SectionEntry<'a> for ExternalFuncSection<'a> {
    fn load(section_data: &'a [u8]) -> Self {
        let (items, names_data) =
            load_section_with_table_and_data_area::<ExternalFuncItem>(section_data);
        ExternalFuncSection { items, names_data }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_table_and_data_area(self.items, self.names_data, writer)
    }

    fn id(&'a self) -> ModuleSectionId {
        ModuleSectionId::ExternalFunc
    }
}

impl<'a> ExternalFuncSection<'a> {
    pub fn get_item_name_and_external_library_index_and_type_index(
        &'a self,
        idx: usize,
    ) -> (&'a str, usize, usize) {
        let items = self.items;
        let names_data = self.names_data;

        let item = &items[idx];
        let name_data =
            &names_data[item.name_offset as usize..(item.name_offset + item.name_length) as usize];

        (
            std::str::from_utf8(name_data).unwrap(),
            item.external_library_index as usize,
            item.type_index as usize,
        )
    }

    pub fn convert_from_entries(entries: &[ExternalFuncEntry]) -> (Vec<ExternalFuncItem>, Vec<u8>) {
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

                ExternalFuncItem::new(
                    name_offset,
                    name_length,
                    entry.external_library_index as u32,
                    entry.type_index as u32,
                )
            })
            .collect::<Vec<ExternalFuncItem>>();

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
        external_func_section::{ExternalFuncEntry, ExternalFuncItem, ExternalFuncSection},
        SectionEntry,
    };

    #[test]
    fn test_load_section() {
        let mut section_data = vec![
            2u8, 0, 0, 0, // item count
            0, 0, 0, 0, // 4 bytes padding
            //
            0, 0, 0, 0, // name offset (item 0)
            3, 0, 0, 0, // name length
            11, 0, 0, 0, // external library index
            13, 0, 0, 0, // type index
            //
            3, 0, 0, 0, // name offset (item 1)
            5, 0, 0, 0, // name length
            15, 0, 0, 0, // external library index
            17, 0, 0, 0, // type index
        ];

        section_data.extend_from_slice(b"foo");
        section_data.extend_from_slice(b"hello");

        let section = ExternalFuncSection::load(&section_data);

        assert_eq!(section.items.len(), 2);
        assert_eq!(section.items[0], ExternalFuncItem::new(0, 3, 11, 13,));
        assert_eq!(section.items[1], ExternalFuncItem::new(3, 5, 15, 17));
        assert_eq!(section.names_data, "foohello".as_bytes())
    }

    #[test]
    fn test_save_section() {
        let mut items: Vec<ExternalFuncItem> = Vec::new();

        items.push(ExternalFuncItem::new(0, 3, 11, 13));
        items.push(ExternalFuncItem::new(3, 5, 15, 17));

        let section = ExternalFuncSection {
            items: &items,
            names_data: b"foohello",
        };

        let mut section_data: Vec<u8> = Vec::new();
        section.save(&mut section_data).unwrap();

        let mut expect_data = vec![
            2u8, 0, 0, 0, // item count
            0, 0, 0, 0, // 4 bytes padding
            //
            0, 0, 0, 0, // name offset (item 0)
            3, 0, 0, 0, // name length
            11, 0, 0, 0, // external library index
            13, 0, 0, 0, // type index
            //
            3, 0, 0, 0, // name offset (item 1)
            5, 0, 0, 0, // name length
            15, 0, 0, 0, // external library index
            17, 0, 0, 0, // type index
        ];

        expect_data.extend_from_slice(b"foo");
        expect_data.extend_from_slice(b"hello");

        assert_eq!(section_data, expect_data);
    }

    #[test]
    fn test_convert() {
        let mut entries: Vec<ExternalFuncEntry> = Vec::new();

        entries.push(ExternalFuncEntry::new("foobar".to_string(), 17, 19));

        entries.push(ExternalFuncEntry::new("helloworld".to_string(), 23, 29));

        let (items, names_data) = ExternalFuncSection::convert_from_entries(&entries);
        let section = ExternalFuncSection {
            items: &items,
            names_data: &names_data,
        };

        assert_eq!(
            section.get_item_name_and_external_library_index_and_type_index(0),
            ("foobar", 17, 19)
        );

        assert_eq!(
            section.get_item_name_and_external_library_index_and_type_index(1),
            ("helloworld", 23, 29)
        );
    }
}
