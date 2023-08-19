// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::mem::size_of;

// func index file binary layout
//
//              |--------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                               |
//              | offset item 0 | offset item 1 | offset item N | ...                |
// offset 0 --> | index item field 0 | index item field 1 | index item field N | ... |
// offset 1 --> | index item field 0 | index item field 1 | index item field N | ... |
//              | ...                                                                |
//              |--------------------------------------------------------------------|

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct OffsetItem {
    pub offset: u32,
    pub count: u32,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct IndexItem {
    pub func_index: u16,          // data item index (in a specified module)
    pub target_module_index: u16, // target module index
    pub target_func_index: u32,   // target func index
}

pub fn read_offset_items(binary: &[u8]) -> &[OffsetItem] {
    let ptr = binary.as_ptr();
    let item_count = unsafe { std::ptr::read(ptr as *const u32) };

    // 4 bytes padding
    let ptr_items = unsafe { ptr.offset(8) } as *const OffsetItem;

    let offset_items_slice = std::ptr::slice_from_raw_parts(ptr_items, item_count as usize);
    unsafe { &*offset_items_slice }
}

pub fn write_offset_items(offset_items: &[OffsetItem]) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();

    // write header
    let item_count = offset_items.len();
    buf.extend_from_slice(&(item_count as u32).to_le_bytes());
    buf.extend_from_slice(&[0u8; 4]);

    // write offset list
    let one_record_length = size_of::<OffsetItem>();
    let total_length = item_count * one_record_length;

    let mut items_buf: Vec<u8> = Vec::with_capacity(total_length);
    let dst = items_buf.as_mut_ptr() as *mut u8;
    let src = offset_items.as_ptr() as *const u8;

    unsafe {
        std::ptr::copy(src, dst, total_length);
        items_buf.set_len(total_length);
    }

    buf.append(&mut items_buf);
    buf
}

pub fn read_index_items(binary: &[u8], item_count: usize) -> &[IndexItem] {
    let ptr = binary.as_ptr() as *const IndexItem;

    let index_items_slice = std::ptr::slice_from_raw_parts(ptr, item_count);
    unsafe { &*index_items_slice }
}

pub fn write_index_items(index_items: &[IndexItem], item_count: usize) -> Vec<u8> {
    let one_record_length = size_of::<IndexItem>();
    let total_length = item_count * one_record_length;

    let mut buf: Vec<u8> = Vec::with_capacity(total_length);
    let dst = buf.as_mut_ptr() as *mut u8;
    let src = index_items.as_ptr() as *const u8;

    unsafe {
        std::ptr::copy(src, dst, total_length);
        buf.set_len(total_length);
    }

    buf
}

#[cfg(test)]
mod tests {
    use crate::func_index_file::{read_index_items, write_index_items, IndexItem};

    use super::{read_offset_items, write_offset_items, OffsetItem};

    #[test]
    fn test_read_offset_items() {
        let binary = vec![
            3u8, 0, 0, 0, // item count (little endian)
            0, 0, 0, 0, // 4 bytes padding
            2, 0, 0, 0, // offset 0 (item 0)
            3, 0, 0, 0, // count 0
            5, 0, 0, 0, // offset 1 (item 1)
            7, 0, 0, 0, // count 1
            11, 0, 0, 0, // offset 2 (item 2)
            13, 0, 0, 0, // count 2
        ];

        let offset_items = read_offset_items(&binary);

        assert_eq!(offset_items.len(), 3);
        assert_eq!(
            offset_items[0],
            OffsetItem {
                offset: 2,
                count: 3
            }
        );
        assert_eq!(
            offset_items[1],
            OffsetItem {
                offset: 5,
                count: 7
            }
        );
        assert_eq!(
            offset_items[2],
            OffsetItem {
                offset: 11,
                count: 13
            }
        );
    }

    #[test]
    fn test_write_offset_items() {
        let mut offset_items: Vec<OffsetItem> = Vec::new();

        offset_items.push(OffsetItem {
            offset: 0x2,
            count: 0x3,
        });

        offset_items.push(OffsetItem {
            offset: 0x5,
            count: 0x7,
        });

        let buf = write_offset_items(&offset_items);

        assert_eq!(
            buf,
            vec![
                2u8, 0, 0, 0, // item count (little endian)
                0, 0, 0, 0, // 4 bytes padding
                2, 0, 0, 0, // offset 0 (item 0)
                3, 0, 0, 0, // count 0
                5, 0, 0, 0, // offset 1 (item 1)
                7, 0, 0, 0, // count 1
            ]
        );
    }

    #[test]
    fn test_read_index_items() {
        let binary = vec![
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

        let index_items = read_index_items(&binary, 3);

        assert_eq!(index_items.len(), 3);
        assert_eq!(
            index_items[0],
            IndexItem {
                func_index: 2,
                target_module_index: 3,
                target_func_index: 5,
            }
        );

        assert_eq!(
            index_items[1],
            IndexItem {
                func_index: 11,
                target_module_index: 13,
                target_func_index: 17,
            }
        );

        assert_eq!(
            index_items[2],
            IndexItem {
                func_index: 23,
                target_module_index: 29,
                target_func_index: 31,
            }
        );
    }

    #[test]
    fn test_write_index_items() {
        let mut index_items: Vec<IndexItem> = Vec::new();

        index_items.push(IndexItem {
            func_index: 2,
            target_module_index: 3,
            target_func_index: 5,
        });

        index_items.push(IndexItem {
            func_index: 17,
            target_module_index: 19,
            target_func_index: 23,
        });

        let buf = write_index_items(&index_items, index_items.len());

        assert_eq!(
            buf,
            vec![
                2u8, 0, // item 0 (little endian)
                3, 0, //
                5, 0, 0, 0, //
                17, 0, // item 1
                19, 0, //
                23, 0, 0, 0, //
            ]
        );
    }
}
