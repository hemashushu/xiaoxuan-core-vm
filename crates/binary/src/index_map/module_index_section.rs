// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// "module index section" binary layout
//
//              |-----------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                            |
//              | name offset 0 (u32) | name length 0 (u16) | module type 0 (u8)  | <-- item 0
//              | name offset 1       | name length 1       | module type 1       | <-- item 1
//              | ...                                                             |
// offset 0 --> | name string 0                                                   |
// offset 1 --> | name string 1                                                   |
//              | ...                                                             |
//              |-----------------------------------------------------------------|

use std::{mem::size_of, ptr::slice_from_raw_parts};

#[derive(Debug, PartialEq)]
pub struct ModuleIndexSection<'a> {
    items: &'a [ModuleIndexItem],
    names_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct ModuleIndexItem {
    pub name_offset: u32,
    pub name_length: u16,
    pub module_type: ModuleType,
}

// specify the data type of enum
// see also:
// https://doc.rust-lang.org/nomicon/other-reprs.html
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ModuleType {
    Local = 0x0,
    Shared,
}

// for access module index item conveniently.
// note that this struct is not used for persistance.
#[derive(Debug, PartialEq, Clone)]
pub struct ModuleIndexEntry<'a> {
    pub module_type: ModuleType,
    pub name: &'a str,
}

pub fn load_section(section_data: &[u8]) -> ModuleIndexSection {
    let ptr = section_data.as_ptr();
    let item_count = unsafe { std::ptr::read(ptr as *const u32) };

    let one_record_length = size_of::<ModuleIndexItem>();
    let total_length = one_record_length * item_count as usize;

    // 8 bytes is the length of header,
    // 4 bytes `item_count` + 4 bytes padding.
    let items_data = &section_data[8..(8 + total_length)];
    let items = load_index_items(items_data, item_count);

    let names_data = &section_data[(8 + total_length)..];

    ModuleIndexSection { items, names_data }
}

pub fn save_section(
    section: &ModuleIndexSection,
    writer: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    let items = section.items;
    let names_data = section.names_data;

    // write header
    let item_count = items.len();
    writer.write_all(&(item_count as u32).to_le_bytes())?; // item count
    writer.write_all(&[0u8; 4])?; // 4 bytes padding

    save_index_items(items, writer)?;
    writer.write_all(names_data)?;

    Ok(())
}

fn load_index_items(items_data: &[u8], item_count: u32) -> &[ModuleIndexItem] {
    let items_ptr = items_data.as_ptr() as *const ModuleIndexItem;
    let items_slice = std::ptr::slice_from_raw_parts(items_ptr, item_count as usize);
    unsafe { &*items_slice }
}

fn save_index_items(
    index_items: &[ModuleIndexItem],
    writer: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    let item_count = index_items.len();
    let record_length = size_of::<ModuleIndexItem>();
    let total_length = record_length * item_count;

    let ptr = index_items.as_ptr() as *const u8;
    let slice = slice_from_raw_parts(ptr, total_length);
    writer.write_all(unsafe { &*slice })?;

    Ok(())
}

pub fn get_entry<'a>(section: &'a ModuleIndexSection<'a>, idx: u16) -> Box<ModuleIndexEntry<'a>> {
    let items = section.items;
    let names_data = section.names_data;

    let item = &items[idx as usize];
    let name_data = &names_data
        [item.name_offset as usize..(item.name_offset + item.name_length as u32) as usize];

    Box::new(ModuleIndexEntry {
        module_type: item.module_type,
        name: std::str::from_utf8(name_data).unwrap(),
    })
}

pub fn convert_to_entries<'a>(
    section: &'a ModuleIndexSection<'a>,
) -> Vec<Box<ModuleIndexEntry<'a>>> {
    (0u16..section.items.len() as u16)
        .map(|idx| get_entry(section, idx))
        .collect::<Vec<Box<ModuleIndexEntry>>>()
}

pub fn convert_from_entries(entries: &[ModuleIndexEntry]) -> (Vec<ModuleIndexItem>, Vec<u8>) {
    let name_bytes = entries
        .iter()
        .map(|entry| entry.name.as_bytes())
        .collect::<Vec<&[u8]>>();

    let mut name_offset: u32 = 0;

    let items = entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let offset = name_offset;
            let length = name_bytes[idx].len() as u16;
            name_offset += length as u32; // for next offset
            ModuleIndexItem {
                module_type: entry.module_type, // unsafe { std::mem::transmute::<ModuleType, u8>(entry.0) }
                name_offset: offset,
                name_length: length,
            }
        })
        .collect::<Vec<ModuleIndexItem>>();

    let names_data = name_bytes
        .iter()
        .flat_map(|bytes| bytes.to_vec())
        .collect::<Vec<u8>>();

    (items, names_data)
}

#[cfg(test)]
mod tests {
    use crate::index_map::module_index_section::{
        convert_to_entries, get_entry, load_section, save_section, ModuleIndexItem,
        ModuleIndexSection, ModuleType,
    };

    use super::{convert_from_entries, ModuleIndexEntry};

    #[test]
    fn test_load_section() {
        let mut section_data = vec![
            2u8, 0, 0, 0, // item count
            0, 0, 0, 0, // 4 bytes padding
            0, 0, 0, 0, // name offset (item 0)
            5, 0, // name length
            0, // module type
            0, // padding
            5, 0, 0, 0, // name offset (item 1)
            3, 0, // name length
            1, // module type
            0, // padding
        ];

        section_data.extend_from_slice("hello".as_bytes());
        section_data.extend_from_slice("foo".as_bytes());

        let section = load_section(&section_data);

        assert_eq!(section.items.len(), 2);
        assert_eq!(
            section.items[0],
            ModuleIndexItem {
                name_offset: 0,
                name_length: 5,
                module_type: ModuleType::Local // name: "hello".to_string(),
            }
        );
        assert_eq!(
            section.items[1],
            ModuleIndexItem {
                name_offset: 5,
                name_length: 3,
                module_type: ModuleType::Shared
            }
        );
        assert_eq!(section.names_data, "hellofoo".as_bytes())
    }

    #[test]
    fn test_save_section() {
        let mut items: Vec<ModuleIndexItem> = Vec::new();

        items.push(ModuleIndexItem {
            name_offset: 0,
            name_length: 3,
            module_type: ModuleType::Shared,
        });

        items.push(ModuleIndexItem {
            name_offset: 3,
            name_length: 5,
            module_type: ModuleType::Local,
        });

        let section = ModuleIndexSection {
            items: &items,
            names_data: "barworld".as_bytes(),
        };

        let mut section_data: Vec<u8> = Vec::new();
        save_section(&section, &mut section_data).unwrap();

        assert_eq!(
            section_data,
            vec![
                2u8, 0, 0, 0, // item count
                0, 0, 0, 0, // 4 bytes padding
                0, 0, 0, 0, // name offset (item 0)
                3, 0, // name length
                1, // module type
                0, // padding
                3, 0, 0, 0, // name offset (item 1)
                5, 0, // name length
                0, // module type
                0, // padding
                b'b', b'a', b'r', // "bar"
                b'w', b'o', b'r', b'l', b'd', // "world"
            ]
        );
    }

    #[test]
    fn test_convert() {
        let mut entries: Vec<ModuleIndexEntry> = Vec::new();

        entries.push(ModuleIndexEntry {
            module_type: ModuleType::Local,
            name: "helloworld",
        });

        entries.push(ModuleIndexEntry {
            module_type: ModuleType::Shared,
            name: "foobar",
        });

        let (items, names_data) = convert_from_entries(&entries);
        let section = ModuleIndexSection {
            items: &items,
            names_data: &names_data,
        };

        assert_eq!(
            *get_entry(&section, 0),
            ModuleIndexEntry {
                module_type: ModuleType::Local,
                name: "helloworld",
            }
        );

        assert_eq!(
            *get_entry(&section, 1),
            ModuleIndexEntry {
                module_type: ModuleType::Shared,
                name: "foobar",
            }
        );

        let entries_restore = convert_to_entries(&section)
            .iter()
            .map(|e| e.as_ref().clone())
            .collect::<Vec<ModuleIndexEntry>>();

        assert_eq!(entries, entries_restore);
    }
}
