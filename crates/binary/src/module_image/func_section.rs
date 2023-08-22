// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// "function section" binary layout
//
//                   |---------------------------------------------------------------------------------|
//                   | item count (u32) | (4 bytes padding)                                            |
//                   |---------------------------------------------------------------------------------|
//        item 0 --> | func type 0 (u16) | padding (16 bytes) | code offset 0 (u32) | code len 0 (u32) | <-- table
//        item 1 --> | func type 1       | padding (16 bytes) | code offset 1       | code len 1       |
//                   | ...                                                                             |
//                   |---------------------------------------------------------------------------------|
// code offset 0 --> | code 0                                                                          | <-- data area
// code offset 1 --> | code 1                                                                          |
//                   | ...                                                                             |
//                   |---------------------------------------------------------------------------------|

use ancvm_types::{FuncEntry, SectionEntry, SectionId};

use crate::utils::{load_section_with_table_and_data_area, save_section_with_table_and_data_area};

#[derive(Debug, PartialEq)]
pub struct FuncSection<'a> {
    pub items: &'a [FuncItem],
    pub codes_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct FuncItem {
    pub func_type: u16,
    _padding0: u16,
    pub code_offset: u32,
    pub code_length: u32,
}

impl<'a> SectionEntry<'a> for FuncSection<'a> {
    fn load(section_data: &'a [u8]) -> Self {
        let (items, codes_data) = load_section_with_table_and_data_area::<FuncItem>(section_data);
        FuncSection { items, codes_data }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_table_and_data_area(self.items, self.codes_data, writer)
    }

    fn id(&'a self) -> SectionId {
        SectionId::Func
    }
}

impl<'a> FuncSection<'a> {
    pub fn get_entry(&'a self, idx: u16) -> Box<FuncEntry<'a>> {
        let items = self.items;
        let codes_data = self.codes_data;

        let item = &items[idx as usize];
        let code_data =
            &codes_data[item.code_offset as usize..(item.code_offset + item.code_length) as usize];

        Box::new(FuncEntry {
            func_type: item.func_type,
            code: code_data,
        })
    }

    pub fn convert_to_entries(&'a self) -> Vec<Box<FuncEntry<'a>>> {
        (0u16..self.items.len() as u16)
            .map(|idx| self.get_entry(idx))
            .collect::<Vec<Box<FuncEntry>>>()
    }

    pub fn convert_from_entries(entries: &[FuncEntry]) -> (Vec<FuncItem>, Vec<u8>) {
        let mut next_offset: u32 = 0;

        let items = entries
            .iter()
            .map(|entry| {
                let code_offset = next_offset;
                let code_length = entry.code.len() as u32;
                next_offset += code_length; // for next offset
                FuncItem {
                    func_type: entry.func_type,
                    _padding0: 0,
                    code_offset,
                    code_length,
                }
            })
            .collect::<Vec<FuncItem>>();

        let codes_data = entries
            .iter()
            .flat_map(|entry| entry.code.to_vec())
            .collect::<Vec<u8>>();

        (items, codes_data)
    }
}

impl FuncItem {
    pub fn new(func_type: u16, code_offset: u32, code_length: u32) -> Self {
        Self {
            func_type,
            _padding0: 0,
            code_offset,
            code_length,
        }
    }
}

#[cfg(test)]
mod tests {
    use ancvm_types::{FuncEntry, SectionEntry};

    use crate::module_image::func_section::{FuncItem, FuncSection};

    #[test]
    fn test_load_section() {
        let mut section_data = vec![
            2u8, 0, 0, 0, // item count
            0, 0, 0, 0, // 4 bytes padding
            //
            3, 0, // func type (item 0)
            0, 0, // padding
            0, 0, 0, 0, // code offset
            5, 0, 0, 0, // code length
            //
            7, 0, // func type (item 0)
            0, 0, // padding
            5, 0, 0, 0, // code offset
            11, 0, 0, 0, // code length
        ];

        section_data.extend_from_slice(b"hello0123456789a");

        let section = FuncSection::load(&section_data);

        assert_eq!(section.items.len(), 2);
        assert_eq!(
            section.items[0],
            FuncItem {
                code_offset: 0,
                code_length: 5,
                func_type: 3,
                _padding0: 0,
            }
        );
        assert_eq!(
            section.items[1],
            FuncItem {
                code_offset: 5,
                code_length: 11,
                func_type: 7,
                _padding0: 0,
            }
        );
        assert_eq!(section.codes_data, b"hello0123456789a")
    }

    #[test]
    fn test_save_section() {
        let mut items: Vec<FuncItem> = Vec::new();

        items.push(FuncItem {
            code_offset: 0,
            code_length: 5,
            func_type: 3,
            _padding0: 0,
        });

        items.push(FuncItem {
            code_offset: 5,
            code_length: 11,
            func_type: 7,
            _padding0: 0,
        });

        let section = FuncSection {
            items: &items,
            codes_data: b"hello0123456789a",
        };

        let mut section_data: Vec<u8> = Vec::new();
        section.save(&mut section_data).unwrap();

        let mut expect_data = vec![
            2u8, 0, 0, 0, // item count
            0, 0, 0, 0, // 4 bytes padding
            //
            3, 0, // func type (item 0)
            0, 0, // padding
            0, 0, 0, 0, // code offset
            5, 0, 0, 0, // code length
            //
            7, 0, // func type (item 0)
            0, 0, // padding
            5, 0, 0, 0, // code offset
            11, 0, 0, 0, // code length
        ];

        expect_data.extend_from_slice(b"hello0123456789a");

        assert_eq!(section_data, expect_data);
    }

    #[test]
    fn test_convert() {
        let mut entries: Vec<FuncEntry> = Vec::new();

        let code0 = b"bar";
        let code1 = b"world";

        entries.push(FuncEntry {
            func_type: 7,
            code: code0,
        });

        entries.push(FuncEntry {
            func_type: 9,
            code: code1,
        });

        let (items, codes_data) = FuncSection::convert_from_entries(&entries);
        let section = FuncSection {
            items: &items,
            codes_data: &codes_data,
        };

        assert_eq!(
            *section.get_entry(0),
            FuncEntry {
                func_type: 7,
                code: code0
            }
        );

        assert_eq!(
            *section.get_entry(1),
            FuncEntry {
                func_type: 9,
                code: code1
            }
        );

        let entries_restore = section
            .convert_to_entries()
            .iter()
            .map(|e| e.as_ref().clone())
            .collect::<Vec<FuncEntry>>();

        assert_eq!(entries, entries_restore);
    }
}
