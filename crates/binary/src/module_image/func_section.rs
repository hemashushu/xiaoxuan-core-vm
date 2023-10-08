// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// "function section" binary layout
//
//              |--------------------------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                                                 |
//              |--------------------------------------------------------------------------------------|
//   item 0 --> | code offset 0 (u32) | code length 0 (u32) | type index 0 (u32) | local index 0 (u32) |  <-- table
//   item 1 --> | code offset 1       | code length 1       | type index 1       | local index 1       |
//              | ...                                                                                  |
//              |--------------------------------------------------------------------------------------|
// offset 0 --> | code 0                                                                               | <-- data area
// offset 1 --> | code 1                                                                               |
//              | ...                                                                                  |
//              |--------------------------------------------------------------------------------------|

use crate::utils::{load_section_with_table_and_data_area, save_section_with_table_and_data_area};

use super::{ModuleSectionId, SectionEntry};

#[derive(Debug, PartialEq)]
pub struct FuncSection<'a> {
    pub items: &'a [FuncItem],
    pub codes_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct FuncItem {
    pub code_offset: u32, // the offset of the code in data area
    pub code_length: u32, // the length (in bytes) of the code in data area
    pub type_index: u32,  // the index of the type (of function)
    pub local_index: u32, // the index of the 'local variables list'
}

#[derive(Debug, PartialEq)]
pub struct FuncEntry {
    pub type_index: usize,
    pub local_index: usize,
    pub code: Vec<u8>,
}

impl<'a> SectionEntry<'a> for FuncSection<'a> {
    fn load(section_data: &'a [u8]) -> Self {
        let (items, codes_data) = load_section_with_table_and_data_area::<FuncItem>(section_data);
        FuncSection { items, codes_data }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_table_and_data_area(self.items, self.codes_data, writer)
    }

    fn id(&'a self) -> ModuleSectionId {
        ModuleSectionId::Func
    }
}

impl<'a> FuncSection<'a> {
    pub fn get_entry(&'a self, idx: u32) -> FuncEntry {
        let items = self.items;
        let codes_data = self.codes_data;

        let item = &items[idx as usize];
        let code_data =
            &codes_data[item.code_offset as usize..(item.code_offset + item.code_length) as usize];

        FuncEntry {
            type_index: item.type_index as usize,
            local_index: item.local_index as usize,
            code: code_data.to_vec(),
        }
    }

    pub fn convert_from_entries(entries: &[FuncEntry]) -> (Vec<FuncItem>, Vec<u8>) {
        let mut next_offset: u32 = 0;

        let items = entries
            .iter()
            .map(|entry| {
                let code_offset = next_offset;
                let code_length = entry.code.len() as u32;
                next_offset += code_length; // for next offset
                FuncItem::new(
                    code_offset,
                    code_length,
                    entry.type_index as u32,
                    entry.local_index as u32,
                )
            })
            .collect::<Vec<FuncItem>>();

        let codes_data = entries
            .iter()
            .flat_map(|entry| entry.code.clone())
            .collect::<Vec<u8>>();

        (items, codes_data)
    }
}

impl FuncItem {
    pub fn new(code_offset: u32, code_length: u32, type_index: u32, local_index: u32) -> Self {
        Self {
            code_offset,
            code_length,
            type_index,
            local_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::module_image::{
        func_section::{FuncEntry, FuncItem, FuncSection},
        SectionEntry,
    };

    #[test]
    fn test_load_section() {
        let mut section_data = vec![
            2u8, 0, 0, 0, // item count
            0, 0, 0, 0, // 4 bytes padding
            //
            3, 0, 0, 0, // code offset (item 0)
            5, 0, 0, 0, // code length
            7, 0, 0, 0, // func type index
            11, 0, 0, 0, // local variable list index
            //
            13, 0, 0, 0, // code offset (item 1)
            17, 0, 0, 0, // code length
            19, 0, 0, 0, // func type index
            23, 0, 0, 0, // local variable list index
        ];

        section_data.extend_from_slice(b"hello0123456789a");

        let section = FuncSection::load(&section_data);

        assert_eq!(section.items.len(), 2);
        assert_eq!(section.items[0], FuncItem::new(3, 5, 7, 11));
        assert_eq!(section.items[1], FuncItem::new(13, 17, 19, 23));
        assert_eq!(section.codes_data, b"hello0123456789a")
    }

    #[test]
    fn test_save_section() {
        let mut items: Vec<FuncItem> = Vec::new();

        items.push(FuncItem::new(3, 5, 7, 11));
        items.push(FuncItem::new(13, 17, 19, 23));

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
            3, 0, 0, 0, // code offset (item 0)
            5, 0, 0, 0, // code length
            7, 0, 0, 0, // func type index
            11, 0, 0, 0, // local variable list index
            //
            13, 0, 0, 0, // code offset (item 1)
            17, 0, 0, 0, // code length
            19, 0, 0, 0, // func type index
            23, 0, 0, 0, // local variable list index
        ];

        expect_data.extend_from_slice(b"hello0123456789a");

        assert_eq!(section_data, expect_data);
    }

    #[test]
    fn test_convert() {
        let mut entries: Vec<FuncEntry> = Vec::new();

        let code0 = b"bar".to_vec();
        let code1 = b"world".to_vec();

        entries.push(FuncEntry {
            type_index: 7,
            local_index: 9,
            code: code0.clone(),
        });

        entries.push(FuncEntry {
            type_index: 11,
            local_index: 13,
            code: code1.clone(),
        });

        let (items, codes_data) = FuncSection::convert_from_entries(&entries);
        let section = FuncSection {
            items: &items,
            codes_data: &codes_data,
        };

        assert_eq!(
            section.get_entry(0),
            FuncEntry {
                type_index: 7,
                local_index: 9,
                code: code0
            }
        );

        assert_eq!(
            section.get_entry(1),
            FuncEntry {
                type_index: 11,
                local_index: 13,
                code: code1
            }
        );
    }
}
