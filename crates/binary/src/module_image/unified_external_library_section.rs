// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// "unified external library section" binary layout
//
//              |-----------------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                                        |
//              |-----------------------------------------------------------------------------|
//  item 0 -->  | lib name off 0 (u32) | lib name len 0 (u32) | lib type 0 (u8) | pad 3 bytes | <-- table
//  item 1 -->  | lib name off 1       | lib name len 1       | lib type 1      |             |
//              | ...                                                                         |
//              |-----------------------------------------------------------------------------|
// offset 0 --> | name string 0 (UTF-8)                                                       | <-- data area
// offset 1 --> | name string 1                                                               |
//              | ...                                                                         |
//              |-----------------------------------------------------------------------------|

use ancvm_types::{entry::UnifiedExternalLibraryEntry, ExternalLibraryType};

use crate::utils::{load_section_with_table_and_data_area, save_section_with_table_and_data_area};

use super::{ModuleSectionId, SectionEntry};

#[derive(Debug, PartialEq, Default)]
pub struct UnifiedExternalLibrarySection<'a> {
    pub items: &'a [UnifiedExternalLibraryItem],
    pub names_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct UnifiedExternalLibraryItem {
    pub name_offset: u32, // the offset of the name string in data area
    pub name_length: u32, // the length (in bytes) of the name string in data area
    pub external_library_type: ExternalLibraryType, // u8
    _padding0: [u8; 3],
}

impl UnifiedExternalLibraryItem {
    pub fn new(
        name_offset: u32,
        name_length: u32,
        external_library_type: ExternalLibraryType,
    ) -> Self {
        Self {
            name_offset,
            name_length,
            external_library_type,
            _padding0: [0; 3],
        }
    }
}

impl<'a> SectionEntry<'a> for UnifiedExternalLibrarySection<'a> {
    fn load(section_data: &'a [u8]) -> Self {
        let (items, names_data) =
            load_section_with_table_and_data_area::<UnifiedExternalLibraryItem>(section_data);
        UnifiedExternalLibrarySection { items, names_data }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_table_and_data_area(self.items, self.names_data, writer)
    }

    fn id(&'a self) -> ModuleSectionId {
        ModuleSectionId::UnifiedExternalLibrary
    }
}

impl<'a> UnifiedExternalLibrarySection<'a> {
    pub fn get_item_name_and_external_library_type(
        &'a self,
        idx: usize,
    ) -> (&'a str, ExternalLibraryType) {
        let items = self.items;
        let names_data = self.names_data;

        let item = &items[idx];
        let name_data =
            &names_data[item.name_offset as usize..(item.name_offset + item.name_length) as usize];

        (
            std::str::from_utf8(name_data).unwrap(),
            item.external_library_type,
        )
    }

    pub fn convert_from_entries(
        entries: &[UnifiedExternalLibraryEntry],
    ) -> (Vec<UnifiedExternalLibraryItem>, Vec<u8>) {
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

                UnifiedExternalLibraryItem::new(
                    name_offset,
                    name_length,
                    entry.external_library_type,
                )
            })
            .collect::<Vec<UnifiedExternalLibraryItem>>();

        let names_data = name_bytes
            .iter()
            .flat_map(|bytes| bytes.to_vec())
            .collect::<Vec<u8>>();

        (items, names_data)
    }
}

#[cfg(test)]
mod tests {
    use ancvm_types::ExternalLibraryType;

    use crate::module_image::{
        unified_external_library_section::{
            UnifiedExternalLibraryEntry, UnifiedExternalLibraryItem, UnifiedExternalLibrarySection,
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
            0, // external library type
            0, 0, 0, // padding
            //
            3, 0, 0, 0, // name offset (item 1)
            5, 0, 0, 0, // name length
            1, // external library type
            0, 0, 0, // padding
        ];

        section_data.extend_from_slice(b"foo");
        section_data.extend_from_slice(b"hello");

        let section = UnifiedExternalLibrarySection::load(&section_data);

        assert_eq!(section.items.len(), 2);
        assert_eq!(
            section.items[0],
            UnifiedExternalLibraryItem::new(0, 3, ExternalLibraryType::User,)
        );
        assert_eq!(
            section.items[1],
            UnifiedExternalLibraryItem::new(3, 5, ExternalLibraryType::Share,)
        );
        assert_eq!(section.names_data, "foohello".as_bytes())
    }

    #[test]
    fn test_save_section() {
        let items = vec![
            UnifiedExternalLibraryItem::new(0, 3, ExternalLibraryType::User),
            UnifiedExternalLibraryItem::new(3, 5, ExternalLibraryType::Share),
        ];

        let section = UnifiedExternalLibrarySection {
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
            0, // external library type
            0, 0, 0, // padding
            //
            3, 0, 0, 0, // name offset (item 1)
            5, 0, 0, 0, // name length
            1, // external library type
            0, 0, 0, // padding
        ];

        expect_data.extend_from_slice(b"foo");
        expect_data.extend_from_slice(b"hello");

        assert_eq!(section_data, expect_data);
    }

    #[test]
    fn test_convert() {
        let entries = vec![
            UnifiedExternalLibraryEntry::new("foobar".to_string(), ExternalLibraryType::User),
            UnifiedExternalLibraryEntry::new("helloworld".to_string(), ExternalLibraryType::Share),
        ];

        let (items, names_data) = UnifiedExternalLibrarySection::convert_from_entries(&entries);
        let section = UnifiedExternalLibrarySection {
            items: &items,
            names_data: &names_data,
        };

        assert_eq!(
            section.get_item_name_and_external_library_type(0),
            ("foobar", ExternalLibraryType::User,)
        );

        assert_eq!(
            section.get_item_name_and_external_library_type(1),
            ("helloworld", ExternalLibraryType::Share,)
        );
    }
}
