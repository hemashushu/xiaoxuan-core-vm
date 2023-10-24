// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// "external function section" binary layout
//
//              |--------------------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                                           |
//              |--------------------------------------------------------------------------------|
//  item 0 -->  | lib name off 0 (u32) | lib name len 0 (u32) | uni external library idx 0 (u32) | <-- table
//  item 1 -->  | lib name off 1       | lib name len 1       | uni external library idx 0       |
//              | ...                                                                            |
//              |--------------------------------------------------------------------------------|
// offset 0 --> | name string 0 (UTF-8)                                                          | <-- data area
// offset 1 --> | name string 1                                                                  |
//              | ...                                                                            |
//              |--------------------------------------------------------------------------------|

use crate::utils::{load_section_with_table_and_data_area, save_section_with_table_and_data_area};

use super::{ModuleSectionId, SectionEntry};

#[derive(Debug, PartialEq, Default)]
pub struct UnifiedExternalFuncSection<'a> {
    pub items: &'a [UnifiedExternalFuncItem],
    pub names_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct UnifiedExternalFuncItem {
    pub name_offset: u32, // the offset of the name string in data area
    pub name_length: u32, // the length (in bytes) of the name string in data area
    pub unified_external_library_index: u32,
}

impl UnifiedExternalFuncItem {
    pub fn new(name_offset: u32, name_length: u32, unified_external_library_index: u32) -> Self {
        Self {
            name_offset,
            name_length,
            unified_external_library_index,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct UnifiedExternalFuncEntry {
    pub name: String,
    pub unified_external_library_index: usize,
}

impl UnifiedExternalFuncEntry {
    pub fn new(name: String, unified_external_library_index: usize) -> Self {
        Self {
            name,
            unified_external_library_index,
        }
    }
}

impl<'a> SectionEntry<'a> for UnifiedExternalFuncSection<'a> {
    fn load(section_data: &'a [u8]) -> Self {
        let (items, names_data) =
            load_section_with_table_and_data_area::<UnifiedExternalFuncItem>(section_data);
        UnifiedExternalFuncSection { items, names_data }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_table_and_data_area(self.items, self.names_data, writer)
    }

    fn id(&'a self) -> ModuleSectionId {
        ModuleSectionId::UnifiedExternalFunc
    }
}

impl<'a> UnifiedExternalFuncSection<'a> {
    pub fn get_item_name_and_unified_external_library_index(
        &'a self,
        idx: usize,
    ) -> (&'a str, usize) {
        let items = self.items;
        let names_data = self.names_data;

        let item = &items[idx];
        let name_data =
            &names_data[item.name_offset as usize..(item.name_offset + item.name_length) as usize];

        (
            std::str::from_utf8(name_data).unwrap(),
            item.unified_external_library_index as usize,
        )
    }

    pub fn convert_from_entries(
        entries: &[UnifiedExternalFuncEntry],
    ) -> (Vec<UnifiedExternalFuncItem>, Vec<u8>) {
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

                UnifiedExternalFuncItem::new(
                    name_offset,
                    name_length,
                    entry.unified_external_library_index as u32,
                )
            })
            .collect::<Vec<UnifiedExternalFuncItem>>();

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
        unified_external_func_section::{
            UnifiedExternalFuncEntry, UnifiedExternalFuncItem, UnifiedExternalFuncSection,
        },
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
            11, 0, 0, 0, // unified external library index
            //
            3, 0, 0, 0, // name offset (item 1)
            5, 0, 0, 0, // name length
            13, 0, 0, 0, // unified external library index
        ];

        section_data.extend_from_slice(b"foo");
        section_data.extend_from_slice(b"hello");

        let section = UnifiedExternalFuncSection::load(&section_data);

        assert_eq!(section.items.len(), 2);
        assert_eq!(section.items[0], UnifiedExternalFuncItem::new(0, 3, 11,));
        assert_eq!(section.items[1], UnifiedExternalFuncItem::new(3, 5, 13,));
        assert_eq!(section.names_data, "foohello".as_bytes())
    }

    #[test]
    fn test_save_section() {
        let mut items: Vec<UnifiedExternalFuncItem> = Vec::new();

        items.push(UnifiedExternalFuncItem::new(0, 3, 11));
        items.push(UnifiedExternalFuncItem::new(3, 5, 13));

        let section = UnifiedExternalFuncSection {
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
            11, 0, 0, 0, // unified external library index
            //
            3, 0, 0, 0, // name offset (item 1)
            5, 0, 0, 0, // name length
            13, 0, 0, 0, // unified external library index
        ];

        expect_data.extend_from_slice(b"foo");
        expect_data.extend_from_slice(b"hello");

        assert_eq!(section_data, expect_data);
    }

    #[test]
    fn test_convert() {
        let mut entries: Vec<UnifiedExternalFuncEntry> = Vec::new();

        entries.push(UnifiedExternalFuncEntry::new("foobar".to_string(), 17));

        entries.push(UnifiedExternalFuncEntry::new("helloworld".to_string(), 19));

        let (items, names_data) = UnifiedExternalFuncSection::convert_from_entries(&entries);
        let section = UnifiedExternalFuncSection {
            items: &items,
            names_data: &names_data,
        };

        assert_eq!(
            section.get_item_name_and_unified_external_library_index(0),
            ("foobar", 17)
        );

        assert_eq!(
            section.get_item_name_and_unified_external_library_index(1),
            ("helloworld", 19)
        );
    }
}
