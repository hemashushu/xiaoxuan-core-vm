// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// "module index section" binary layout
//
//              |------------------------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                                               |
//              |------------------------------------------------------------------------------------|
//  item 0 -->  | name offset 0 (u32) | name length 0 (u32) | module share type 0 (u8) | pad 3 bytes | <-- table
//  item 1 -->  | name offset 1       | name length 1       | module share type 1      |             |
//              | ...                                                                                |
//              |------------------------------------------------------------------------------------|
// offset 0 --> | name string 0 (UTF-8)                                                              | <-- data area
// offset 1 --> | name string 1                                                                      |
//              | ...                                                                                |
//              |------------------------------------------------------------------------------------|

// note:
// the 1st module is the application main module.

use crate::utils::{load_section_with_table_and_data_area, save_section_with_table_and_data_area};

use super::{SectionEntry, SectionId};

#[derive(Debug, PartialEq)]
pub struct ModuleIndexSection<'a> {
    pub items: &'a [ModuleIndexItem],
    pub names_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct ModuleIndexItem {
    pub name_offset: u32, // the offset of the name string in data area
    pub name_length: u32, // the length (in bytes) of the name string in data area
    pub module_share_type: ModuleShareType, // u8
    _padding0: [u8; 3],
}

// specify the data type of enum
// see also:
// https://doc.rust-lang.org/nomicon/other-reprs.html
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ModuleShareType {
    User = 0x0,
    Shared,
}

// impl From<u8> for ModuleShareType {
//     fn from(value: u8) -> Self {
//         unsafe { std::mem::transmute::<u8, ModuleShareType>(value) }
//     }
// }

impl ModuleIndexItem {
    pub fn new(name_offset: u32, name_length: u32, module_share_type: ModuleShareType) -> Self {
        Self {
            name_offset,
            name_length,
            module_share_type,
            _padding0: [0, 0, 0],
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ModuleIndexEntry {
    pub module_share_type: ModuleShareType,
    // pub name: &'a str,
    pub name: String,
}

impl ModuleIndexEntry {
    pub fn new(module_share_type: ModuleShareType, name: String) -> Self {
        Self {
            module_share_type,
            name,
        }
    }
}

impl<'a> SectionEntry<'a> for ModuleIndexSection<'a> {
    fn load(section_data: &'a [u8]) -> Self {
        let (items, names_data) =
            load_section_with_table_and_data_area::<ModuleIndexItem>(section_data);
        ModuleIndexSection { items, names_data }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_table_and_data_area(self.items, self.names_data, writer)
    }

    fn id(&'a self) -> SectionId {
        SectionId::ModuleIndex
    }
}

impl<'a> ModuleIndexSection<'a> {
    pub fn get_entry(&'a self, idx: u32) -> ModuleIndexEntry {
        let items = self.items;
        let names_data = self.names_data;

        let item = &items[idx as usize];
        let name_data =
            &names_data[item.name_offset as usize..(item.name_offset + item.name_length) as usize];

        ModuleIndexEntry::new(
            item.module_share_type,
            String::from_utf8(name_data.to_vec()).unwrap(),
        )
    }

    pub fn convert_from_entries(entries: &[ModuleIndexEntry]) -> (Vec<ModuleIndexItem>, Vec<u8>) {
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

                ModuleIndexItem::new(name_offset, name_length, entry.module_share_type)
            })
            .collect::<Vec<ModuleIndexItem>>();

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
        module_index_section::{ModuleIndexItem, ModuleIndexSection, ModuleShareType},
        SectionEntry,
    };

    use super::ModuleIndexEntry;

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
            //
            3, 0, 0, 0, // name offset (item 1)
            5, 0, 0, 0, // name length
            1, // module share type
            0, 0, 0, // padding
        ];

        section_data.extend_from_slice("foo".as_bytes());
        section_data.extend_from_slice("hello".as_bytes());

        let section = ModuleIndexSection::load(&section_data);

        assert_eq!(section.items.len(), 2);
        assert_eq!(
            section.items[0],
            ModuleIndexItem::new(0, 3, ModuleShareType::Local,)
        );
        assert_eq!(
            section.items[1],
            ModuleIndexItem::new(3, 5, ModuleShareType::Shared,)
        );
        assert_eq!(section.names_data, "foohello".as_bytes())
    }

    #[test]
    fn test_save_section() {
        let mut items: Vec<ModuleIndexItem> = Vec::new();

        items.push(ModuleIndexItem::new(0, 3, ModuleShareType::Local));
        items.push(ModuleIndexItem::new(3, 5, ModuleShareType::Shared));

        let section = ModuleIndexSection {
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
            0, // module share type
            0, 0, 0, // padding
            //
            3, 0, 0, 0, // name offset (item 1)
            5, 0, 0, 0, // name length
            1, // module share type
            0, 0, 0, // padding
        ];

        expect_data.extend_from_slice(b"foo");
        expect_data.extend_from_slice(b"hello");

        assert_eq!(section_data, expect_data);
    }

    #[test]
    fn test_convert() {
        let mut entries: Vec<ModuleIndexEntry> = Vec::new();

        entries.push(ModuleIndexEntry::new(
            ModuleShareType::Local,
            "helloworld".to_string(),
        ));

        entries.push(ModuleIndexEntry::new(
            ModuleShareType::Shared,
            "foobar".to_string(),
        ));

        let (items, names_data) = ModuleIndexSection::convert_from_entries(&entries);
        let section = ModuleIndexSection {
            items: &items,
            names_data: &names_data,
        };

        assert_eq!(
            section.get_entry(0),
            ModuleIndexEntry::new(ModuleShareType::Local, "helloworld".to_string(),)
        );

        assert_eq!(
            section.get_entry(1),
            ModuleIndexEntry::new(ModuleShareType::Shared, "foobar".to_string(),)
        );
    }
}
