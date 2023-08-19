// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::mem::size_of;

// data index file binary layout
//
//              |--------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                               |
//              | offset item 0 | offset item 1 | offset item N | ...                |
// offset 0 --> | index item field 0 | index item field 1 | index item field N | ... |
// offset 1 --> | index item field 0 | index item field 1 | index item field N | ... |
//              | ...                                                                |
//              |--------------------------------------------------------------------|

// use the C style struct memory layout
// see also:
// https://doc.rust-lang.org/reference/type-layout.html#reprc-structs
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct OffsetItem {
    pub offset: u32,
    pub count: u32,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct IndexItem {
    pub data_index: u16,          // data item index (in a specified module)
    pub target_module_index: u16, // target module index
    pub target_data_section: u8,  // target data section, i.e. 0=READ_ONLY, 1=READ_WRITE, 2=UNINIT
    pub target_data_index: u16,   // target data item index (in a specified section)
}

pub fn read_offset_items(binary: &[u8]) -> &[OffsetItem] {
    let ptr = binary.as_ptr();
    let item_count = unsafe { std::ptr::read(ptr as *const u32) };

    // there is a "safe" way to read a number from pointer, e.g.
    //
    // ```rust
    //     let mut buf = [0u8; 4];
    //     let data = &binary[0..4];
    //     buf.clone_from_slice(data);
    //     let module_count =  u32::from_le_bytes(buf);
    // ```

    // there are empty 4 bytes that follows the field `item_count`.
    // these padding bytes are used to archieve 8-byte alignment.
    let ptr_items = unsafe { ptr.offset(8) } as *const OffsetItem;

    // https://doc.rust-lang.org/std/ptr/fn.slice_from_raw_parts.html
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

        // the items buffer can be also created by `items_buf = vec![0u8; byte_count]`
        // which does not require invoke `set_len()`.
        // see also:
        // https://doc.rust-lang.org/std/vec/struct.Vec.html#method.set_len
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
    use crate::data_index_file::{read_index_items, write_index_items, IndexItem};

    use super::{read_offset_items, write_offset_items, OffsetItem};

    #[test]
    fn test_read_offset_items() {
        let binary = vec![
            3u8, 0, 0, 0, // item count (little endian)
            0, 0, 0, 0, // 4 bytes padding
            2, 0, 0, 0, // offset 0
            3, 0, 0, 0, // count 0
            5, 0, 0, 0, // offset 1
            7, 0, 0, 0, // count 1
            11, 0, 0, 0, // offset 2
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
            2, 0, // data index (item 0)
            3, 0, // target module index
            5, // target data section
            0, // 1 byte padding
            7, 0, // target data index
            11, 0, // data index (item 1)
            13, 0,  // target module index
            17, // target data section
            0,  // 1 byte padding
            19, 0, // target data index
            23, 0, // data index (item 2)
            29, 0,  // target module index
            31, // target data section
            0,  // 1 byte padding
            37, 0, // target data index
        ];

        let index_items = read_index_items(&binary, 3);

        assert_eq!(index_items.len(), 3);
        assert_eq!(
            index_items[0],
            IndexItem {
                data_index: 2,
                target_module_index: 3,
                target_data_section: 5,
                target_data_index: 7,
            }
        );

        assert_eq!(
            index_items[1],
            IndexItem {
                data_index: 11,
                target_module_index: 13,
                target_data_section: 17,
                target_data_index: 19,
            }
        );

        assert_eq!(
            index_items[2],
            IndexItem {
                data_index: 23,
                target_module_index: 29,
                target_data_section: 31,
                target_data_index: 37,
            }
        );
    }

    #[test]
    fn test_write_index_items() {
        let mut index_items: Vec<IndexItem> = Vec::new();

        index_items.push(IndexItem {
            data_index: 2,
            target_module_index: 3,
            target_data_section: 5,
            target_data_index: 7,
        });

        index_items.push(IndexItem {
            data_index: 17,
            target_module_index: 19,
            target_data_section: 23,
            target_data_index: 29,
        });

        let buf = write_index_items(&index_items, index_items.len());

        assert_eq!(
            buf,
            vec![
                2u8, 0, // item 0 (little endian)
                3, 0, //
                5, 0, //
                7, 0, //
                17, 0, // item 1
                19, 0, //
                23, 0, //
                29, 0, //
            ]
        );
    }
}
