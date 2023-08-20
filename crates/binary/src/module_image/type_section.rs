// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// "type section" binary layout
//
//                    |---------------------------------------------------------------------------------------|
//                    | item count (u32) | (4 bytes padding)                                                  |
//         item 0 --> | param len 0 (u32) | param offset 0 (u32) | result len 0 (u32) | result offset 0 (u32) |
//         item 1 --> | param len 1 (u32) | param offset 1 (u32) | result len 1 (u32) | result offset 1 (u32) |
//                    | ...                                                                                   |
// param offset 0 --> | param list 0                             | result type list 0                         | <-- result offset 0
// param offset 1 --> | param list 1                             | result type list 1                         | <-- result offset 1
//                    | ...                                                                                   |
//                    |---------------------------------------------------------------------------------------|

use std::{mem::size_of, ptr::slice_from_raw_parts, result, slice};

#[derive(Debug, PartialEq)]
pub struct TypeSection<'a> {
    items: &'a [TypeItem],
    types_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct TypeItem {
    pub param_length: u32,
    pub param_offset: u32,
    pub result_length: u32,
    pub result_offset: u32,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataType {
    I32 = 0x0,
    I64,
    F32,
    F64,
    BYTE, // only available for data section
}

// for access function type item conveniently.
// note that this struct is not used for persistance.
#[derive(Debug, PartialEq, Clone)]
pub struct TypeEntry<'a> {
    pub params: &'a [DataType],
    pub results: &'a [DataType],
}

pub fn load_section(section_data: &[u8]) -> TypeSection {
    let ptr = section_data.as_ptr();
    let item_count = unsafe { std::ptr::read(ptr as *const u32) };

    let one_record_length = size_of::<TypeItem>();
    let total_length = one_record_length * item_count as usize;

    // 8 bytes is the length of header,
    // 4 bytes `item_count` + 4 bytes padding.
    let items_data = &section_data[8..(8 + total_length)];
    let items = load_type_items(items_data, item_count);

    let types_data = &section_data[(8 + total_length)..];

    TypeSection { items, types_data }
}

pub fn save_section(section: &TypeSection, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let items = section.items;
    let types_data = section.types_data;

    // write header
    let item_count = items.len();
    writer.write_all(&(item_count as u32).to_le_bytes())?; // item count
    writer.write_all(&[0u8; 4])?; // 4 bytes padding

    save_type_items(items, writer)?;
    writer.write_all(types_data)?;

    Ok(())
}

fn load_type_items(items_data: &[u8], item_count: u32) -> &[TypeItem] {
    let items_ptr = items_data.as_ptr() as *const TypeItem;
    let items_slice = std::ptr::slice_from_raw_parts(items_ptr, item_count as usize);
    unsafe { &*items_slice }
}

fn save_type_items(
    index_items: &[TypeItem],
    writer: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    let item_count = index_items.len();
    let record_length = size_of::<TypeItem>();
    let total_length = record_length * item_count;

    let ptr = index_items.as_ptr() as *const u8;
    let slice = slice_from_raw_parts(ptr, total_length);
    writer.write_all(unsafe { &*slice })?;

    Ok(())
}

pub fn get_entry<'a>(section: &'a TypeSection<'a>, idx: u16) -> Box<TypeEntry<'a>> {
    let items = section.items;
    let types_data = section.types_data;

    let item = &items[idx as usize];

    let params_data =
        &types_data[item.param_offset as usize..(item.param_offset + item.param_length) as usize];
    let results_data = &types_data
        [item.result_offset as usize..(item.result_offset + item.result_length) as usize];

    Box::new(TypeEntry {
        params: unsafe {
            &*slice_from_raw_parts(
                params_data.as_ptr() as *const DataType,
                item.param_length as usize,
            )
        },
        results: unsafe {
            &*slice_from_raw_parts(
                results_data.as_ptr() as *const DataType,
                item.result_length as usize,
            )
        },
    })
}

pub fn convert_to_entries<'a>(section: &'a TypeSection<'a>) -> Vec<Box<TypeEntry<'a>>> {
    (0u16..section.items.len() as u16)
        .map(|idx| get_entry(section, idx))
        .collect::<Vec<Box<TypeEntry>>>()
}

pub fn convert_from_entries(entries: &[TypeEntry]) -> (Vec<TypeItem>, Vec<u8>) {
    let mut type_offset: u32 = 0;

    let items = entries
        .iter()
        .map(|entry| {
            let param_offset = type_offset;
            let param_length = entry.params.len() as u32;
            let result_offset = param_offset + param_length;
            let result_length = entry.results.len() as u32;

            type_offset = result_offset + result_length; // for next offset
            TypeItem {
                param_length,
                param_offset,
                result_length,
                result_offset,
            }
        })
        .collect::<Vec<TypeItem>>();

    let types_data = entries
        .iter()
        .flat_map(|entry| {
            let mut bytes: Vec<u8> = Vec::new();
            let params_bytes =
                slice_from_raw_parts(entry.params.as_ptr() as *const u8, entry.params.len());
            let results_bytes =
                slice_from_raw_parts(entry.results.as_ptr() as *const u8, entry.results.len());
            bytes.extend_from_slice(unsafe { &*params_bytes });
            bytes.extend_from_slice(unsafe { &*results_bytes });
            bytes
        })
        .collect::<Vec<u8>>();

    (items, types_data)
}

#[cfg(test)]
mod tests {
    use crate::module_image::type_section::{
        convert_from_entries, convert_to_entries, get_entry, load_section, save_section, DataType,
        TypeEntry, TypeItem, TypeSection,
    };

    #[test]
    fn test_load_section() {
        let section_data = vec![
            2u8, 0, 0, 0, // item count
            0, 0, 0, 0, // 4 bytes padding
            //
            2, 0, 0, 0, // param length (item 0)
            0, 0, 0, 0, // param offset
            3, 0, 0, 0, // result length
            2, 0, 0, 0, // result offset
            //
            2, 0, 0, 0, // param length (item 1)
            5, 0, 0, 0, // param offset
            3, 0, 0, 0, // result length
            7, 0, 0, 0, // result offset
            //
            1, 2, // params 0
            3, 2, 1, // result 0
            3, 4, // params 1
            1, 2, 3, // result 1
        ];

        let section = load_section(&section_data);

        assert_eq!(section.items.len(), 2);
        assert_eq!(
            section.items[0],
            TypeItem {
                param_length: 2,
                param_offset: 0,
                result_length: 3,
                result_offset: 2,
            }
        );
        assert_eq!(
            section.items[1],
            TypeItem {
                param_length: 2,
                param_offset: 5,
                result_length: 3,
                result_offset: 7,
            }
        );
    }

    #[test]
    fn test_save_section() {
        let mut items: Vec<TypeItem> = Vec::new();

        items.push(TypeItem {
            param_length: 2,
            param_offset: 0,
            result_length: 3,
            result_offset: 2,
        });

        items.push(TypeItem {
            param_length: 2,
            param_offset: 5,
            result_length: 3,
            result_offset: 7,
        });

        let section = TypeSection {
            items: &items,
            types_data: &vec![
                1u8, 2, // params 0
                3, 2, 1, // result 0
                3, 4, // params 1
                1, 2, 3, // result 1
            ],
        };

        let mut section_data: Vec<u8> = Vec::new();
        save_section(&section, &mut section_data).unwrap();

        assert_eq!(
            section_data,
            vec![
                2u8, 0, 0, 0, // item count
                0, 0, 0, 0, // 4 bytes padding
                //
                2, 0, 0, 0, // param length (item 0)
                0, 0, 0, 0, // param offset
                3, 0, 0, 0, // result length
                2, 0, 0, 0, // result offset
                //
                2, 0, 0, 0, // param length (item 1)
                5, 0, 0, 0, // param offset
                3, 0, 0, 0, // result length
                7, 0, 0, 0, // result offset
                //
                1, 2, // params 0
                3, 2, 1, // result 0
                3, 4, // params 1
                1, 2, 3, // result 1
            ]
        );
    }

    #[test]
    fn test_convert() {
        let mut entries: Vec<TypeEntry> = Vec::new();

        let p0 = vec![DataType::I32, DataType::I64];
        let r0 = vec![DataType::I32];
        entries.push(TypeEntry {
            params: &p0,
            results: &r0,
        });

        let p1 = vec![DataType::I64];
        let r1 = vec![DataType::I64, DataType::I32];
        entries.push(TypeEntry {
            params: &p1,
            results: &r1,
        });

        let p2 = vec![];
        let r2 = vec![DataType::F32];
        entries.push(TypeEntry {
            params: &p2,
            results: &r2,
        });

        let p3 = vec![];
        let r3 = vec![];
        entries.push(TypeEntry {
            params: &p3,
            results: &r3,
        });

        let (items, types_data) = convert_from_entries(&entries);
        let section = TypeSection {
            items: &items,
            types_data: &types_data,
        };

        assert_eq!(
            *get_entry(&section, 0),
            TypeEntry {
                params: &vec![DataType::I32, DataType::I64],
                results: &vec![DataType::I32]
            }
        );

        assert_eq!(
            *get_entry(&section, 3),
            TypeEntry {
                params: &vec![],
                results: &vec![]
            }
        );

        let entries_restore = convert_to_entries(&section)
            .iter()
            .map(|e| e.as_ref().clone())
            .collect::<Vec<TypeEntry>>();

        assert_eq!(entries, entries_restore);
    }
}
