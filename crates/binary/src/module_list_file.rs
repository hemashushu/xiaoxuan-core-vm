// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// module list file binary layout
//
//              |-----------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                            |
//              | name offset 0 (u32) | name length 0 (u16) | module class 0 (u8) | <-- item 0
//              | name offset 1       | name length 1       | module class 1      | <-- item 1
//              | ...                                                             |
// offset 0 --> | name string 0                                                   |
// offset 1 --> | name string 1                                                   |
//              | ...                                                             |
//              |-----------------------------------------------------------------|

use std::mem::size_of;

// specify the data type of enum
// see also:
// https://doc.rust-lang.org/nomicon/other-reprs.html
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ModuleClass {
    Local = 0x0,
    Shared,
}

#[derive(Debug, PartialEq)]
pub struct ModuleEntry {
    pub class: ModuleClass,
    pub name: String,
}

// for internal use only
#[repr(C)]
#[derive(Debug, PartialEq)]
struct ModuleItem {
    name_offset: u32,
    name_length: u16,
    module_class: u8,
}

pub fn read_module_entries(binary: &[u8]) -> Vec<ModuleEntry> {
    let ptr = binary.as_ptr();
    let item_count = unsafe { std::ptr::read(ptr as *const u32) };

    // 4 bytes padding
    let ptr_items = unsafe { ptr.offset(8) } as *const ModuleItem;

    // read items
    let module_items_slice = std::ptr::slice_from_raw_parts(ptr_items, item_count as usize);
    let module_items = unsafe { &*module_items_slice };

    // read name strings and form struct ModuleEntry
    let string_start_pos = 8 + (size_of::<ModuleItem>() * item_count as usize);

    module_items
        .iter()
        .map(|item| {
            let module_class = unsafe { std::mem::transmute::<u8, ModuleClass>(item.module_class) };

            let name_start_pos = string_start_pos + item.name_offset as usize;
            let name_end_pos = name_start_pos + item.name_length as usize;
            let name_slice = &binary[name_start_pos..name_end_pos];
            let name = String::from_utf8(name_slice.to_vec()).unwrap();

            ModuleEntry {
                class: module_class,
                name: name,
            }
        })
        .collect::<Vec<ModuleEntry>>()
}

pub fn write_module_entries(module_entries: &[ModuleEntry]) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();

    // write header
    let item_count = module_entries.len();
    buf.extend_from_slice(&(item_count as u32).to_le_bytes());
    buf.extend_from_slice(&[0u8; 4]);

    let name_bytes = module_entries
        .iter()
        .map(|entry| entry.name.as_bytes())
        .collect::<Vec<&[u8]>>();

    // build item list
    let mut name_offset: u32 = 0;

    let module_items = module_entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let offset = name_offset;
            let length = name_bytes[idx].len() as u16;
            name_offset += length as u32; // for next offset
            ModuleItem {
                module_class: unsafe { std::mem::transmute::<ModuleClass, u8>(entry.class) },
                name_offset: offset,
                name_length: length,
            }
        })
        .collect::<Vec<ModuleItem>>();

    // write item list
    let one_record_length = size_of::<ModuleItem>();
    let total_length = item_count * one_record_length;

    let mut items_buf: Vec<u8> = Vec::with_capacity(total_length);
    let dst = items_buf.as_mut_ptr() as *mut u8;
    let src = module_items.as_ptr() as *const u8;

    unsafe {
        std::ptr::copy(src, dst, total_length);
        items_buf.set_len(total_length);
    }

    buf.append(&mut items_buf);

    // write name strings
    for name in name_bytes {
        buf.extend_from_slice(name);
    }

    buf
}

#[cfg(test)]
mod tests {
    use crate::module_list_file::{ModuleClass, ModuleEntry};

    use super::{read_module_entries, write_module_entries};

    #[test]
    fn test_read_module_entries() {
        let mut binary = vec![
            2u8, 0, 0, 0, // item count
            0, 0, 0, 0, // 4 bytes padding
            0, 0, 0, 0, // name offset (item 0)
            5, 0, // name length
            0, // module class
            0, // padding
            5, 0, 0, 0, // name offset (item 1)
            3, 0, // name length
            1, // module class
            0, // padding
        ];

        binary.extend_from_slice("hello".as_bytes());
        binary.extend_from_slice("foo".as_bytes());

        let module_entries = read_module_entries(&binary);

        assert_eq!(module_entries.len(), 2);
        assert_eq!(
            module_entries[0],
            ModuleEntry {
                name: "hello".to_string(),
                class: ModuleClass::Local
            }
        );
        assert_eq!(
            module_entries[1],
            ModuleEntry {
                name: "foo".to_string(),
                class: ModuleClass::Shared
            }
        );
    }

    #[test]
    fn test_write_module_entries() {
        let mut module_entries: Vec<ModuleEntry> = Vec::new();

        module_entries.push(ModuleEntry {
            class: ModuleClass::Shared,
            name: "bar".to_string(),
        });

        module_entries.push(ModuleEntry {
            class: ModuleClass::Local,
            name: "world".to_string(),
        });

        let buf = write_module_entries(&module_entries);

        assert_eq!(
            buf,
            vec![
                2u8, 0, 0, 0, // item count
                0, 0, 0, 0, // 4 bytes padding
                0, 0, 0, 0, // name offset (item 0)
                3, 0, // name length
                1, // module class
                0, // padding
                3, 0, 0, 0, // name offset (item 1)
                5, 0, // name length
                0, // module class
                0, // padding
                b'b', b'a', b'r', // "bar"
                b'w', b'o', b'r', b'l', b'd', // "world"
            ]
        );
    }
}
