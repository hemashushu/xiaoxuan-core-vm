// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// "func index section" binary layout
//
//              |---------------------------------------------------|
//              | item count (u32) | (4 bytes padding)              |
//              | offset 0 | offset 1 | offset N | ...              |
// offset 0 --> | func idx 0 | tar mod idx 0 | tar func idx 0 | ... | <-- item 0
// offset 1 --> | func idx 1 | tar mod idx 1 | tar func idx 1 | ... | <-- item 1
//              | ...                                               |
//              |---------------------------------------------------|

use std::{mem::size_of, ptr::slice_from_raw_parts};

#[derive(Debug, PartialEq)]
pub struct FuncIndexSection<'a> {
    offsets: &'a [FuncIndexOffset],
    items: &'a [FuncIndexItem],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct FuncIndexOffset {
    pub offset: u32,
    pub count: u32,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct FuncIndexItem {
    pub func_index: u16,          // data item index (in a specified module)
    pub target_module_index: u16, // target module index
    pub target_func_index: u32,   // target func index
}

pub fn load_section(section_data: &[u8]) -> FuncIndexSection {
    let ptr = section_data.as_ptr();
    let item_count = unsafe { std::ptr::read(ptr as *const u32) };

    let one_record_length = size_of::<FuncIndexOffset>();
    let total_length = one_record_length * item_count as usize;

    // 8 bytes is the length of header,
    // 4 bytes `item_count` + 4 bytes padding.
    let offsets_data = &section_data[8..(8 + total_length)];

    let offsets = load_offsets(offsets_data, item_count);

    let items_data = &section_data[(8 + total_length)..];
    let items = load_index_items(items_data, item_count);

    FuncIndexSection { offsets, items }
}

pub fn save_section(
    section: &FuncIndexSection,
    writer: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    let offsets = section.offsets;
    let items = section.items;

    // write header
    let item_count = items.len();
    writer.write_all(&(item_count as u32).to_le_bytes())?; // item count
    writer.write_all(&[0u8; 4])?; // 4 bytes padding

    save_offsets(offsets, writer)?;
    save_index_items(items, writer)?;

    Ok(())
}

pub fn load_offsets(offsets_data: &[u8], item_count: u32) -> &[FuncIndexOffset] {
    let offsets_ptr = offsets_data.as_ptr() as *const FuncIndexOffset;
    let offsets_slice = std::ptr::slice_from_raw_parts(offsets_ptr, item_count as usize);
    unsafe { &*offsets_slice }
}

pub fn save_offsets(
    offsets: &[FuncIndexOffset],
    writer: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    let item_count = offsets.len();
    let record_length = size_of::<FuncIndexOffset>();
    let total_length = record_length * item_count;

    let ptr = offsets.as_ptr() as *const u8;
    let slice = slice_from_raw_parts(ptr, total_length);
    writer.write_all(unsafe { &*slice })?;
    Ok(())
}

pub fn load_index_items(items_data: &[u8], item_count: u32) -> &[FuncIndexItem] {
    let items_ptr = items_data.as_ptr() as *const FuncIndexItem;
    let items_slice = std::ptr::slice_from_raw_parts(items_ptr, item_count as usize);
    unsafe { &*items_slice }
}

pub fn save_index_items(
    items: &[FuncIndexItem],
    writer: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    let item_count = items.len();
    let record_length = size_of::<FuncIndexItem>();
    let total_length = record_length * item_count;

    let ptr = items.as_ptr() as *const u8;
    let slice = slice_from_raw_parts(ptr, total_length);
    writer.write_all(unsafe { &*slice })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::index_map::func_index_section::{
        load_section, save_section, FuncIndexItem, FuncIndexOffset, FuncIndexSection,
    };

    #[test]
    fn test_load_section() {
        let section_data = vec![
            3u8, 0, 0, 0, // item count (little endian)
            0, 0, 0, 0, // 4 bytes padding
            //
            2, 0, 0, 0, // offset 0 (item 0)
            3, 0, 0, 0, // count 0
            5, 0, 0, 0, // offset 1 (item 1)
            7, 0, 0, 0, // count 1
            11, 0, 0, 0, // offset 2 (item 2)
            13, 0, 0, 0, // count 2
            //
            2, 0, // func index (item 0)
            3, 0, // target module index
            5, 0, 0, 0, // target func index
            11, 0, // data index (item 1)
            13, 0, // target module index
            17, 0, 0, 0, // target func index
            23, 0, // data index (item 2)
            29, 0, // target module index
            31, 0, 0, 0, // target func index
        ];

        let section = load_section(&section_data);

        let offsets = section.offsets;

        assert_eq!(offsets.len(), 3);
        assert_eq!(
            offsets[0],
            FuncIndexOffset {
                offset: 2,
                count: 3
            }
        );
        assert_eq!(
            offsets[1],
            FuncIndexOffset {
                offset: 5,
                count: 7
            }
        );
        assert_eq!(
            offsets[2],
            FuncIndexOffset {
                offset: 11,
                count: 13
            }
        );

        let items = section.items;

        assert_eq!(items.len(), 3);
        assert_eq!(
            items[0],
            FuncIndexItem {
                func_index: 2,
                target_module_index: 3,
                target_func_index: 5,
            }
        );

        assert_eq!(
            items[1],
            FuncIndexItem {
                func_index: 11,
                target_module_index: 13,
                target_func_index: 17,
            }
        );

        assert_eq!(
            items[2],
            FuncIndexItem {
                func_index: 23,
                target_module_index: 29,
                target_func_index: 31,
            }
        );
    }

    #[test]
    fn test_save_section() {
        let mut offsets: Vec<FuncIndexOffset> = Vec::new();

        offsets.push(FuncIndexOffset {
            offset: 0x2,
            count: 0x3,
        });

        offsets.push(FuncIndexOffset {
            offset: 0x5,
            count: 0x7,
        });

        let mut items: Vec<FuncIndexItem> = Vec::new();

        items.push(FuncIndexItem {
            func_index: 2,
            target_module_index: 3,
            target_func_index: 5,
        });

        items.push(FuncIndexItem {
            func_index: 17,
            target_module_index: 19,
            target_func_index: 23,
        });

        let section = FuncIndexSection {
            offsets: &offsets,
            items: &items,
        };

        let mut section_data: Vec<u8> = Vec::new();
        save_section(&section, &mut section_data).unwrap();

        assert_eq!(
            section_data,
            vec![
                2u8, 0, 0, 0, // item count (little endian)
                0, 0, 0, 0, // 4 bytes padding
                //
                2, 0, 0, 0, // offset 0 (item 0)
                3, 0, 0, 0, // count 0
                5, 0, 0, 0, // offset 1 (item 1)
                7, 0, 0, 0, // count 1
                //
                2, 0, // item 0 (little endian)
                3, 0, //
                5, 0, 0, 0, //
                17, 0, // item 1
                19, 0, //
                23, 0, 0, 0, //
            ]
        );
    }
}
