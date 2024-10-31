// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// "import module section" binary layout
//
//              |---------------------------------------------------------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                                                                                |
//              |---------------------------------------------------------------------------------------------------------------------|
//  item 0 -->  | mod name off 0 (u32) | mod name len 0 (u32) | mod type 0 (u8) | pad 3 bytes | ver major 0 (u16) | ver minor 0 (u16) | <-- table
//  item 1 -->  | mod name off 1       | mod name len 1       | mod type 1      |             | ver major 1       | ver minor 1       |
//              | ...                                                                                                                 |
//              |---------------------------------------------------------------------------------------------------------------------|
// offset 0 --> | name string 0 (UTF-8)                                                                                               | <-- data area
// offset 1 --> | name string 1                                                                                                       |
//              | ...                                                                                                                 |
//              |---------------------------------------------------------------------------------------------------------------------|

use ancvm_isa::{entry::ImportModuleEntry, ModuleShareType};

use crate::{
    module_image::{ModuleSectionId, SectionEntry},
    tableaccess::{load_section_with_table_and_data_area, save_section_with_table_and_data_area},
};

#[derive(Debug, PartialEq)]
pub struct ImportModuleSection<'a> {
    pub items: &'a [ImportModuleItem],
    pub names_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct ImportModuleItem {
    pub name_offset: u32, // the offset of the name string in data area
    pub name_length: u32, // the length (in bytes) of the name string in data area
    pub module_share_type: ModuleShareType, // u8
    _padding0: [u8; 3],
    pub version_major: u16,
    pub version_minor: u16,
}

impl ImportModuleItem {
    pub fn new(
        name_offset: u32,
        name_length: u32,
        module_share_type: ModuleShareType,
        version_major: u16,
        version_minor: u16,
    ) -> Self {
        Self {
            name_offset,
            name_length,
            module_share_type,
            _padding0: [0; 3],
            version_major,
            version_minor,
        }
    }
}

impl<'a> SectionEntry<'a> for ImportModuleSection<'a> {
    fn load(section_data: &'a [u8]) -> Self {
        let (items, names_data) =
            load_section_with_table_and_data_area::<ImportModuleItem>(section_data);
        ImportModuleSection { items, names_data }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_table_and_data_area(self.items, self.names_data, writer)
    }

    fn id(&'a self) -> ModuleSectionId {
        ModuleSectionId::ImportModule
    }
}

impl<'a> ImportModuleSection<'a> {
    pub fn get_item_name_and_module_share_type_and_version(
        &'a self,
        idx: usize,
    ) -> (&'a str, ModuleShareType, u16, u16) {
        let items = self.items;
        let names_data = self.names_data;

        let item = &items[idx];
        let name_data =
            &names_data[item.name_offset as usize..(item.name_offset + item.name_length) as usize];

        (
            std::str::from_utf8(name_data).unwrap(),
            item.module_share_type,
            item.version_major,
            item.version_minor,
        )
    }

    pub fn convert_from_entries(entries: &[ImportModuleEntry]) -> (Vec<ImportModuleItem>, Vec<u8>) {
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

                ImportModuleItem::new(
                    name_offset,
                    name_length,
                    entry.module_share_type,
                    entry.module_version.major,
                    entry.module_version.minor,
                )
            })
            .collect::<Vec<ImportModuleItem>>();

        let names_data = name_bytes
            .iter()
            .flat_map(|bytes| bytes.to_vec())
            .collect::<Vec<u8>>();

        (items, names_data)
    }
}

#[cfg(test)]
mod tests {
    use ancvm_isa::{entry::ImportModuleEntry, EffectiveVersion, ModuleShareType};

    use crate::{common_sections::import_module_section::{ImportModuleItem, ImportModuleSection}, module_image::SectionEntry};

    #[test]
    fn test_load_section() {
        let mut section_data = vec![
            2u8, 0, 0, 0, // item count
            0, 0, 0, 0, // 4 bytes padding
            //
            0, 0, 0, 0, // name offset (item 0)
            3, 0, 0, 0, // name length
            0, // module share type
            0, 0, 0, // padding
            11, 0, // ver major
            13, 0, // ver minor
            //
            3, 0, 0, 0, // name offset (item 1)
            5, 0, 0, 0, // name length
            1, // module share type
            0, 0, 0, // padding
            17, 0, // ver major
            19, 0, // ver minor
        ];

        section_data.extend_from_slice(b"foo");
        section_data.extend_from_slice(b"hello");

        let section = ImportModuleSection::load(&section_data);

        assert_eq!(section.items.len(), 2);
        assert_eq!(
            section.items[0],
            ImportModuleItem::new(0, 3, ModuleShareType::User, 11, 13)
        );
        assert_eq!(
            section.items[1],
            ImportModuleItem::new(3, 5, ModuleShareType::Share, 17, 19)
        );
        assert_eq!(section.names_data, "foohello".as_bytes())
    }

    #[test]
    fn test_save_section() {
        let items = vec![
            ImportModuleItem::new(0, 3, ModuleShareType::User, 11, 13),
            ImportModuleItem::new(3, 5, ModuleShareType::Share, 17, 19),
        ];

        let section = ImportModuleSection {
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
            0, // module share type
            0, 0, 0, // padding
            11, 0, // ver major
            13, 0, // ver minor
            //
            3, 0, 0, 0, // name offset (item 1)
            5, 0, 0, 0, // name length
            1, // module share type
            0, 0, 0, // padding
            17, 0, // ver major
            19, 0, // ver minor
        ];

        expect_data.extend_from_slice(b"foo");
        expect_data.extend_from_slice(b"hello");

        assert_eq!(section_data, expect_data);
    }

    #[test]
    fn test_convert() {
        let entries = vec![
            ImportModuleEntry::new(
                "foobar".to_string(),
                ModuleShareType::User,
                EffectiveVersion::new(23, 29),
            ),
            ImportModuleEntry::new(
                "helloworld".to_string(),
                ModuleShareType::Share,
                EffectiveVersion::new(31, 37),
            ),
        ];

        let (items, names_data) = ImportModuleSection::convert_from_entries(&entries);
        let section = ImportModuleSection {
            items: &items,
            names_data: &names_data,
        };

        assert_eq!(
            section.get_item_name_and_module_share_type_and_version(0),
            ("foobar", ModuleShareType::User, 23, 29)
        );

        assert_eq!(
            section.get_item_name_and_module_share_type_and_version(1),
            ("helloworld", ModuleShareType::Share, 31, 37)
        );
    }
}
