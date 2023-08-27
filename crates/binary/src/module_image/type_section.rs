// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// "type section" binary layout
//
//                     |---------------------------------------------------------------------------------------|
//                     | item count (u32) | (4 bytes padding)                                                  |
//                     |---------------------------------------------------------------------------------------|
//          item 0 --> | param len 0 (u32) | param offset 0 (u32) | result len 0 (u32) | result offset 0 (u32) | <-- table
//          item 1 --> | param len 1       | param offset 1       | result len 1       | result offset 1       |
//                     | ...                                                                                   |
//                     |---------------------------------------------------------------------------------------|
// param offset 0 -->  | param type list 0                                                                     | <-- data area
// result offset 0 --> | result type list 0                                                                    |
// param offset 1 -->  | param type list 1                                                                     |
// result offset 1 --> | result type list 1                                                                    |
//                     | ...                                                                                   |
//                     |---------------------------------------------------------------------------------------|

use std::ptr::slice_from_raw_parts;

use ancvm_types::DataType;

use crate::utils::{load_section_with_table_and_data_area, save_section_with_table_and_data_area};

use super::{SectionEntry, SectionId};

#[derive(Debug, PartialEq)]
pub struct TypeSection<'a> {
    pub items: &'a [TypeItem],
    pub types_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct TypeItem {
    pub param_length: u32,
    pub param_offset: u32,
    pub result_length: u32,
    pub result_offset: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeEntry {
    // pub params: &'a [DataType],
    // pub results: &'a [DataType],
    pub params: Vec<DataType>,
    pub results: Vec<DataType>,
}

impl<'a> SectionEntry<'a> for TypeSection<'a> {
    fn load(section_data: &'a [u8]) -> Self {
        let (items, types_data) = load_section_with_table_and_data_area::<TypeItem>(section_data);
        TypeSection { items, types_data }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_table_and_data_area(self.items, self.types_data, writer)
    }

    fn id(&'a self) -> SectionId {
        SectionId::Type
    }
}

impl<'a> TypeSection<'a> {
    pub fn get_entry(&'a self, idx: u32) -> TypeEntry {
        let items = self.items;
        let types_data = self.types_data;

        let item = &items[idx as usize];

        let params_data = &types_data
            [item.param_offset as usize..(item.param_offset + item.param_length) as usize];
        let results_data = &types_data
            [item.result_offset as usize..(item.result_offset + item.result_length) as usize];

        let params_slice = unsafe {
            &*slice_from_raw_parts(
                params_data.as_ptr() as *const DataType,
                item.param_length as usize,
            )
        };

        let results_slice = unsafe {
            &*slice_from_raw_parts(
                results_data.as_ptr() as *const DataType,
                item.result_length as usize,
            )
        };

        TypeEntry {
            params: params_slice.to_vec(),
            results: results_slice.to_vec(),
        }
    }

    pub fn convert_to_entries(&'a self) -> Vec<TypeEntry> {
        (0u32..self.items.len() as u32)
            .map(|idx| self.get_entry(idx))
            .collect::<Vec<TypeEntry>>()
    }

    pub fn convert_from_entries(entries: &[TypeEntry]) -> (Vec<TypeItem>, Vec<u8>) {
        let mut next_offset: u32 = 0;

        let items = entries
            .iter()
            .map(|entry| {
                let param_offset = next_offset;
                let param_length = entry.params.len() as u32;
                let result_offset = param_offset + param_length;
                let result_length = entry.results.len() as u32;

                next_offset = result_offset + result_length; // for next offset
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
}

#[cfg(test)]
mod tests {
    use crate::module_image::{
        type_section::{DataType, TypeEntry, TypeItem, TypeSection},
        SectionEntry,
    };

    #[test]
    fn test_load_section() {
        let section_data = vec![
            3u8, 0, 0, 0, // item count
            0, 0, 0, 0, // 4 bytes padding
            //
            2, 0, 0, 0, // param length (item 0)
            0, 0, 0, 0, // param offset
            3, 0, 0, 0, // result length
            2, 0, 0, 0, // result offset
            //
            1, 0, 0, 0, // param length (item 1)
            5, 0, 0, 0, // param offset
            0, 0, 0, 0, // result length
            6, 0, 0, 0, // result offset
            //
            4, 0, 0, 0, // param length (item 1)
            6, 0, 0, 0, // param offset
            1, 0, 0, 0, // result length
            10, 0, 0, 0, // result offset
            //
            1u8, 2, // param types 0
            3, 2, 1, // result types 0
            4, // param types 1
            // result types 1
            4, 3, 2, 1, // param types 2
            1, // result types 2
        ];

        let section = TypeSection::load(&section_data);

        assert_eq!(section.items.len(), 3);
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
                param_length: 1,
                param_offset: 5,
                result_length: 0,
                result_offset: 6,
            }
        );
        assert_eq!(
            section.items[2],
            TypeItem {
                param_length: 4,
                param_offset: 6,
                result_length: 1,
                result_offset: 10,
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
            param_length: 1,
            param_offset: 5,
            result_length: 0,
            result_offset: 6,
        });

        items.push(TypeItem {
            param_length: 4,
            param_offset: 6,
            result_length: 1,
            result_offset: 10,
        });

        let section = TypeSection {
            items: &items,
            types_data: &vec![
                1u8, 2, // param types 0
                3, 2, 1, // result types 0
                4, // param types 1
                // result types 1
                4, 3, 2, 1, // param types 2
                1, // result types 2
            ],
        };

        let mut section_data: Vec<u8> = Vec::new();
        section.save(&mut section_data).unwrap();

        assert_eq!(
            section_data,
            vec![
                3u8, 0, 0, 0, // item count
                0, 0, 0, 0, // 4 bytes padding
                //
                2, 0, 0, 0, // param length (item 0)
                0, 0, 0, 0, // param offset
                3, 0, 0, 0, // result length
                2, 0, 0, 0, // result offset
                //
                1, 0, 0, 0, // param length (item 1)
                5, 0, 0, 0, // param offset
                0, 0, 0, 0, // result length
                6, 0, 0, 0, // result offset
                //
                4, 0, 0, 0, // param length (item 1)
                6, 0, 0, 0, // param offset
                1, 0, 0, 0, // result length
                10, 0, 0, 0, // result offset
                //
                1u8, 2, // param types 0
                3, 2, 1, // result types 0
                4, // param types 1
                // result types 1
                4, 3, 2, 1, // param types 2
                1, // result types 2
                //
                0, // padding for 4-byte align
            ]
        );
    }

    #[test]
    fn test_convert() {
        let mut entries: Vec<TypeEntry> = Vec::new();

        let p0 = vec![DataType::I32, DataType::I64];
        let r0 = vec![DataType::I32];
        entries.push(TypeEntry {
            params: p0,
            results: r0,
        });

        let p1 = vec![DataType::I64];
        let r1 = vec![DataType::I64, DataType::I32];
        entries.push(TypeEntry {
            params: p1,
            results: r1,
        });

        let p2 = vec![];
        let r2 = vec![DataType::F32];
        entries.push(TypeEntry {
            params: p2,
            results: r2,
        });

        let p3 = vec![];
        let r3 = vec![];
        entries.push(TypeEntry {
            params: p3,
            results: r3,
        });

        let (items, types_data) = TypeSection::convert_from_entries(&entries);
        let section = TypeSection {
            items: &items,
            types_data: &types_data,
        };

        assert_eq!(
            section.get_entry(0),
            TypeEntry {
                params: vec![DataType::I32, DataType::I64],
                results: vec![DataType::I32]
            }
        );

        assert_eq!(
            section.get_entry(3),
            TypeEntry {
                params: vec![],
                results: vec![]
            }
        );

        let entries_restore = section.convert_to_entries();

        assert_eq!(entries, entries_restore);
    }
}
