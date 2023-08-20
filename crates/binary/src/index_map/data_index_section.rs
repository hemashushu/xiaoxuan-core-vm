// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// "data index section" binary layout
//
//              |--------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                         |
//              | offset 0 | offset 1 | offset N | ...                         |
// offset 0 --> | data idx 0 | tar mod idx 0 | tar data sec 0 | tar data idx 0 | <-- item 0
// offset 1 --> | data idx 1 | tar mod idx 1 | tar data sec 1 | tar data idx 1 | <-- item 1
//              | ...                                                          |
//              |--------------------------------------------------------------|

use std::{mem::size_of, ptr::slice_from_raw_parts};

#[derive(Debug, PartialEq)]
pub struct DataIndexSection<'a> {
    offsets: &'a [DataIndexOffset],
    items: &'a [DataIndexItem],
}

// use the C style struct memory layout
// see also:
// https://doc.rust-lang.org/reference/type-layout.html#reprc-structs
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct DataIndexOffset {
    pub offset: u32,
    pub count: u32,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct DataIndexItem {
    pub data_index: u16,          // data item index (in a specified module)
    pub target_module_index: u16, // target module index
    pub target_data_section: u8,  // target data section, i.e. 0=READ_ONLY, 1=READ_WRITE, 2=UNINIT
    pub target_data_index: u16,   // target data item index (in a specified section)
}

pub fn load_section(section_data: &[u8]) -> DataIndexSection {
    let ptr = section_data.as_ptr();
    let item_count = unsafe { std::ptr::read(ptr as *const u32) };

    // there is a "safe" way to read a number from pointer, e.g.
    //
    // ```rust
    //     let mut buf = [0u8; 4];
    //     let data = &binary[0..4];
    //     buf.clone_from_slice(data);
    //     let module_count =  u32::from_le_bytes(buf);
    // ```

    let one_record_length = size_of::<DataIndexOffset>();
    let total_length = one_record_length * item_count as usize;

    // 8 bytes is the length of header,
    // 4 bytes `item_count` + 4 bytes padding.
    let offsets_data = &section_data[8..(8 + total_length)];

    // there is another method to get the `offsets_data`, e.g.
    // ```rust
    //     let ptr_items = unsafe {
    //         ptr.offset(8)
    //     } as *const DataIndexOffset;
    // ```

    let offsets = load_offsets(offsets_data, item_count);

    let items_data = &section_data[(8 + total_length)..];
    let items = load_index_items(items_data, item_count);

    DataIndexSection { offsets, items }
}

pub fn save_section(
    section: &DataIndexSection,
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

fn load_offsets(offsets_data: &[u8], item_count: u32) -> &[DataIndexOffset] {
    let offsets_ptr = offsets_data.as_ptr() as *const DataIndexOffset;
    // https://doc.rust-lang.org/std/ptr/fn.slice_from_raw_parts.html
    let offsets_slice = std::ptr::slice_from_raw_parts(offsets_ptr, item_count as usize);
    unsafe { &*offsets_slice }
}

fn save_offsets(
    offsets: &[DataIndexOffset],
    writer: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    let item_count = offsets.len();
    let record_length = size_of::<DataIndexOffset>();
    let total_length = record_length * item_count;

    let ptr = offsets.as_ptr() as *const u8;
    let slice = slice_from_raw_parts(ptr, total_length);
    writer.write_all(unsafe { &*slice })?;

    // an example of writing a slice to Vec<u8>
    //
    // ```rust
    //     let record_length = size_of::<SOME_STRUCT>();
    //     let total_length = item_count * record_length;
    //
    //     let mut buf: Vec<u8> = Vec::with_capacity(total_length);
    //     let dst = buf.as_mut_ptr() as *mut u8;
    //     let src = items.as_ptr() as *const u8;
    //
    //     unsafe {
    //         std::ptr::copy(src, dst, total_length);
    //         items_buf.set_len(total_length);
    //     }
    // ```

    Ok(())
}

fn load_index_items(items_data: &[u8], item_count: u32) -> &[DataIndexItem] {
    let items_ptr = items_data.as_ptr() as *const DataIndexItem;
    let items_slice = std::ptr::slice_from_raw_parts(items_ptr, item_count as usize);
    unsafe { &*items_slice }
}

fn save_index_items(
    items: &[DataIndexItem],
    writer: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    let item_count = items.len();
    let record_length = size_of::<DataIndexItem>();
    let total_length = record_length * item_count;

    let ptr = items.as_ptr() as *const u8;
    let slice = slice_from_raw_parts(ptr, total_length);
    writer.write_all(unsafe { &*slice })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::index_map::data_index_section::{
        load_section, save_section, DataIndexItem, DataIndexOffset, DataIndexSection,
    };

    #[test]
    fn test_load_section() {
        let section_data = vec![
            3u8, 0, 0, 0, // item count (little endian)
            0, 0, 0, 0, // 4 bytes padding
            //
            2, 0, 0, 0, // offset 0 (offset 0)
            3, 0, 0, 0, // count 0
            5, 0, 0, 0, // offset 1 (offset 1)
            7, 0, 0, 0, // count 1
            11, 0, 0, 0, // offset 2 (offset 2)
            13, 0, 0, 0, // count 2
            //
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

        let section = load_section(&section_data);

        let offsets = section.offsets;

        assert_eq!(offsets.len(), 3);
        assert_eq!(
            offsets[0],
            DataIndexOffset {
                offset: 2,
                count: 3
            }
        );
        assert_eq!(
            offsets[1],
            DataIndexOffset {
                offset: 5,
                count: 7
            }
        );
        assert_eq!(
            offsets[2],
            DataIndexOffset {
                offset: 11,
                count: 13
            }
        );

        let items = section.items;

        assert_eq!(items.len(), 3);
        assert_eq!(
            items[0],
            DataIndexItem {
                data_index: 2,
                target_module_index: 3,
                target_data_section: 5,
                target_data_index: 7,
            }
        );

        assert_eq!(
            items[1],
            DataIndexItem {
                data_index: 11,
                target_module_index: 13,
                target_data_section: 17,
                target_data_index: 19,
            }
        );

        assert_eq!(
            items[2],
            DataIndexItem {
                data_index: 23,
                target_module_index: 29,
                target_data_section: 31,
                target_data_index: 37,
            }
        );
    }

    #[test]
    fn test_save_section() {
        let mut offsets: Vec<DataIndexOffset> = Vec::new();

        offsets.push(DataIndexOffset {
            offset: 0x2,
            count: 0x3,
        });

        offsets.push(DataIndexOffset {
            offset: 0x5,
            count: 0x7,
        });

        let mut items: Vec<DataIndexItem> = Vec::new();

        items.push(DataIndexItem {
            data_index: 2,
            target_module_index: 3,
            target_data_section: 5,
            target_data_index: 7,
        });

        items.push(DataIndexItem {
            data_index: 17,
            target_module_index: 19,
            target_data_section: 23,
            target_data_index: 29,
        });

        let section = DataIndexSection {
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
