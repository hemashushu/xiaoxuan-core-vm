// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// "module index section" binary layout
//
//              |-----------------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                                        |
//              |-----------------------------------------------------------------------------|
//  item 0 -->  | name offset 0 (u32) | name length 0 (u16) | module type 0 (u8) | pad 1 byte | <-- table
//  item 1 -->  | name offset 1       | name length 1       | module type 1                   |
//              | ...                                                                         |
//              |-----------------------------------------------------------------------------|
// offset 0 --> | name string 0                                                               | <-- data area
// offset 1 --> | name string 1                                                               |
//              | ...                                                                         |
//              |-----------------------------------------------------------------------------|

// note:
// the 1st module is the application main module.

use ancvm_types::{ModuleIndexEntry, ModuleShareType, SectionEntry, SectionId};

use crate::utils::{load_section_with_table_and_data_area, save_section_with_table_and_data_area};

#[derive(Debug, PartialEq)]
pub struct ModuleIndexSection<'a> {
    pub items: &'a [ModuleIndexItem],
    pub names_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct ModuleIndexItem {
    pub name_offset: u32,
    pub name_length: u16,
    pub module_share_type: ModuleShareType,
    _padding0 :u8
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
    pub fn get_entry(&'a self, idx: u16) -> Box<ModuleIndexEntry<'a>> {
        let items = self.items;
        let names_data = self.names_data;

        let item = &items[idx as usize];
        let name_data = &names_data
            [item.name_offset as usize..(item.name_offset + item.name_length as u32) as usize];

        Box::new(ModuleIndexEntry {
            module_share_type: item.module_share_type,
            name: std::str::from_utf8(name_data).unwrap(),
        })
    }

    pub fn convert_to_entries(&'a self) -> Vec<Box<ModuleIndexEntry<'a>>> {
        (0u16..self.items.len() as u16)
            .map(|idx| self.get_entry(idx))
            .collect::<Vec<Box<ModuleIndexEntry>>>()
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
                let name_length = name_bytes[idx].len() as u16;
                next_offset += name_length as u32; // for next offset
                ModuleIndexItem {
                    // the function `std::mem::transmute` can convert
                    // between `enum` and `u8` date, e.g.
                    // ```rust
                    //     unsafe { std::mem::transmute::<FROM, TO>(FROM) }
                    // ```
                    module_share_type: entry.module_share_type,
                    name_offset,
                    name_length,
                    _padding0:0
                }
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
    use ancvm_types::SectionEntry;

    use crate::index_map::module_index_section::{
        ModuleIndexItem, ModuleIndexSection, ModuleShareType,
    };

    use super::ModuleIndexEntry;

    #[test]
    fn test_load_section() {
        let mut section_data = vec![
            2u8, 0, 0, 0, // item count
            0, 0, 0, 0, // 4 bytes padding
            //
            0, 0, 0, 0, // name offset (item 0)
            3, 0, // name length
            0, // module share type
            0, // padding
            //
            3, 0, 0, 0, // name offset (item 1)
            5, 0, // name length
            1, // module share type
            0, // padding
        ];

        section_data.extend_from_slice("foo".as_bytes());
        section_data.extend_from_slice("hello".as_bytes());

        let section = ModuleIndexSection::load(&section_data);

        assert_eq!(section.items.len(), 2);
        assert_eq!(
            section.items[0],
            ModuleIndexItem {
                name_offset: 0,
                name_length: 3,
                module_share_type: ModuleShareType::Local,
                _padding0:0
            }
        );
        assert_eq!(
            section.items[1],
            ModuleIndexItem {
                name_offset: 3,
                name_length: 5,
                module_share_type: ModuleShareType::Shared,
                _padding0:0
            }
        );
        assert_eq!(section.names_data, "foohello".as_bytes())
    }

    #[test]
    fn test_save_section() {
        let mut items: Vec<ModuleIndexItem> = Vec::new();

        items.push(ModuleIndexItem {
            name_offset: 0,
            name_length: 3,
            module_share_type: ModuleShareType::Local,
            _padding0:0,
        });

        items.push(ModuleIndexItem {
            name_offset: 3,
            name_length: 5,
            module_share_type: ModuleShareType::Shared,
            _padding0:0,
        });

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
            3, 0, // name length
            0, // module share type
            0, // padding
            //
            3, 0, 0, 0, // name offset (item 1)
            5, 0, // name length
            1, // module share type
            0, // padding
        ];

        expect_data.extend_from_slice(b"foo");
        expect_data.extend_from_slice(b"hello");

        assert_eq!(section_data, expect_data);
    }

    #[test]
    fn test_convert() {
        let mut entries: Vec<ModuleIndexEntry> = Vec::new();

        entries.push(ModuleIndexEntry {
            module_share_type: ModuleShareType::Local,
            name: "helloworld",
        });

        entries.push(ModuleIndexEntry {
            module_share_type: ModuleShareType::Shared,
            name: "foobar",
        });

        let (items, names_data) = ModuleIndexSection::convert_from_entries(&entries);
        let section = ModuleIndexSection {
            items: &items,
            names_data: &names_data,
        };

        assert_eq!(
            *section.get_entry(0),
            ModuleIndexEntry {
                module_share_type: ModuleShareType::Local,
                name: "helloworld",
            }
        );

        assert_eq!(
            *section.get_entry(1),
            ModuleIndexEntry {
                module_share_type: ModuleShareType::Shared,
                name: "foobar",
            }
        );

        let entries_restore = section
            .convert_to_entries()
            .iter()
            .map(|e| e.as_ref().clone())
            .collect::<Vec<ModuleIndexEntry>>();

        assert_eq!(entries, entries_restore);
    }
}
