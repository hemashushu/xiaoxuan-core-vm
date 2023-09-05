// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// "local variable section" binary layout
//
//              |----------------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                                       |
//              |----------------------------------------------------------------------------|
//  item 0 -->  | list offset 0 (u32) | list item count 0 (u32) | allocated_in_bytes 0 (u32) | <-- table
//  item 1 -->  | list offset 1       | list item count 1       | allocated_in_bytes 1       |
//              | ...                                                                        |
//              |----------------------------------------------------------------------------|
// offset 0 --> | list 0                                                                     | <-- data area
// offset 1 --> | list 1                                                                     |
//              | ...                                                                        |
//              |----------------------------------------------------------------------------|
//
//
// the variable "list" is a table too, the "list" layout:
//
//              |-------------------------------------------------------------------------------------------------------------|
//  item 0 -->  | var offset 0 (u32) | var actual length 0 (u32) | data type 0 (u8) | pad (1 byte) | var actual align 0 (u16) | <-- list
//  item 1 -->  | var offset 1       | var actual length 1       | data type 1      |              | var actual align 1       |
//              | ...                                                                                                         |
//              |-------------------------------------------------------------------------------------------------------------|
//
// note that all variables in the 'local variable area' MUST be 8-byte aligned,
// and their size are padded to a multiple of 8.
// for example, an i32 will be padded to 8 bytes, and a struct which is 12 bytes will
// be padded to 16 (= 8 * 2) bytes.
// this is because the current VM allocates 'local variable area' on the stack frame,
// and the stack address is 8-byte aligned.

use std::mem::size_of;

use ancvm_types::{DataType, OPERAND_SIZE_IN_BYTES};

use crate::utils::{load_section_with_table_and_data_area, save_section_with_table_and_data_area};

use super::{SectionEntry, SectionId};

#[derive(Debug, PartialEq)]
pub struct LocalVariableSection<'a> {
    pub lists: &'a [VariableList],
    pub list_data: &'a [u8],
}

// one function one list
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct VariableList {
    pub list_offset: u32,
    pub list_item_count: u32,

    // note that all variables in the 'local variable area' MUST be 8-byte aligned,
    // and their size are padded to a multiple of 8.
    // so the value of this filed will be always the multiple of 8.
    pub list_allocated_in_bytes: u32,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct VariableItem {
    pub var_offset: u32, // the offset of a data item in the "local variables area"

    // the actual length (in bytes) of a variable/data
    // not the length of the item in the local variable area.
    // e.g.
    // - i32 is 4 bytes, but in the local variable area, i32 occurs 8 byets.
    // - i64 is 8 bytes.
    // - for the struct, is the size_of(struct), it means the size contains the padding.
    //   e.g. the size of 'struct {u8, u16}' is 1 + 1 padding + 2 = 4 bytes,
    //   this is the actual size, but in the local variable area this struct occurs 8 bytes.
    //
    // note that all variable is allocated with the length (in bytes) of multiple of 8,
    // for example, an i32 will be padded to 8 bytes, and a struct which is 12 bytes will
    // be padded to 16 (= 8 * 2) bytes.
    pub var_actual_length: u32,

    // the data_type field is not necessary at runtime, though it is helpful for debugging.
    pub data_type: DataType,

    _padding0: u8,

    // the var_actual_align field is not necessary either at runtime because the local variable is always
    // 8-byte aligned, it is only needed when copying data into other memory, such as
    // copying a struct from data section into heap.
    pub var_actual_align: u16,
}

// one function one VariableListEntry
// #[derive(Debug, PartialEq)]
// pub struct VariableListEntry {
//     pub variables: Vec<VariableItemEntry>,
// }

#[derive(Debug, PartialEq)]
pub struct VariableItemEntry {
    pub data_type: DataType,

    // actual length of the variable/data
    pub length: u32,

    pub align: u16,
}

impl VariableItemEntry {
    pub fn from_i32() -> Self {
        Self {
            data_type: DataType::I32,
            length: 4,
            align: 4,
        }
    }

    pub fn from_i64() -> Self {
        Self {
            data_type: DataType::I64,
            length: 8,
            align: 8,
        }
    }

    pub fn from_f32() -> Self {
        Self {
            data_type: DataType::F32,
            length: 4,
            align: 4,
        }
    }

    pub fn from_f64() -> Self {
        Self {
            data_type: DataType::F64,
            length: 8,
            align: 8,
        }
    }

    pub fn from_bytes(length: u32, align: u16) -> Self {
        Self {
            data_type: DataType::BYTE,
            length,
            align,
        }
    }
}

impl VariableItem {
    pub fn new(
        var_offset: u32,
        var_actual_length: u32,
        data_type: DataType,
        var_actual_align: u16,
    ) -> Self {
        Self {
            var_offset,
            var_actual_length,
            data_type,
            _padding0: 0,
            var_actual_align,
        }
    }
}

impl<'a> SectionEntry<'a> for LocalVariableSection<'a> {
    fn id(&'a self) -> SectionId {
        SectionId::LocalVariable
    }

    fn load(section_data: &'a [u8]) -> Self
    where
        Self: Sized,
    {
        let (items, datas) = load_section_with_table_and_data_area::<VariableList>(section_data);
        LocalVariableSection {
            lists: items,
            list_data: datas,
        }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        save_section_with_table_and_data_area(self.lists, self.list_data, writer)
    }
}

impl<'a> LocalVariableSection<'a> {
    pub fn get_variable_list(&'a self, idx: u32) -> &'a [VariableItem] {
        let list = &self.lists[idx as usize];
        let offset = list.list_offset as usize;
        let item_count = list.list_item_count as usize;

        let items_data = &self.list_data[offset..(offset + item_count * size_of::<VariableItem>())];

        let items_ptr = items_data.as_ptr() as *const VariableItem;
        let items = std::ptr::slice_from_raw_parts(items_ptr, item_count);
        unsafe { &*items }
    }

    pub fn convert_from_entries(
        entires: &[Vec<VariableItemEntry>],
    ) -> (Vec<VariableList>, Vec<u8>) {
        let var_item_length_in_bytes = size_of::<VariableItem>();

        // generate a list of (list, list_allocated_in_bytes)
        let list_vec = entires
            .iter()
            .map(|list_entry| {
                // one entry per function

                // offset in one list entry
                let mut next_offset: u32 = 0;

                let items = list_entry
                    .iter()
                    .map(|entry| {
                        let item = VariableItem::new(
                            next_offset,
                            entry.length,
                            entry.data_type,
                            entry.align,
                        );

                        // pad the length of variable/data to the multiple of 8
                        let padding = {
                            let remainder = entry.length % OPERAND_SIZE_IN_BYTES as u32; // remainder
                            if remainder != 0 {
                                OPERAND_SIZE_IN_BYTES as u32 - remainder
                            } else {
                                0
                            }
                        };

                        let var_allocated_in_bytes = entry.length + padding;
                        next_offset += var_allocated_in_bytes;
                        item
                    })
                    .collect::<Vec<VariableItem>>();

                // here 'next_offset' is the list_allocated_in_bytes
                (items, next_offset)
            })
            .collect::<Vec<(Vec<VariableItem>, u32)>>();

        // make lists
        let mut next_offset: u32 = 0;
        let lists = list_vec
            .iter()
            .map(|(list, list_allocated_in_bytes)| {
                let list_offset = next_offset;
                let list_item_count = list.len() as u32;
                next_offset += list_item_count * var_item_length_in_bytes as u32;

                VariableList {
                    list_offset,
                    list_item_count,
                    list_allocated_in_bytes: *list_allocated_in_bytes,
                }
            })
            .collect::<Vec<VariableList>>();

        // make data
        let list_data = list_vec
            .iter()
            .flat_map(|(list, _)| {
                let list_item_count = list.len();
                let total_length_in_bytes = list_item_count * var_item_length_in_bytes;

                // let mut buf: Vec<u8> = vec![0u8; total_length_in_bytes];
                let mut buf: Vec<u8> = Vec::with_capacity(total_length_in_bytes);
                let dst = buf.as_mut_ptr() as *mut u8;
                let src = list.as_ptr() as *const u8;

                unsafe {
                    std::ptr::copy(src, dst, total_length_in_bytes);
                    buf.set_len(total_length_in_bytes);
                }

                buf
            })
            .collect::<Vec<u8>>();

        (lists, list_data)
    }
}

#[cfg(test)]
mod tests {
    use ancvm_types::DataType;

    use crate::module_image::{
        local_variable_section::{VariableItem, VariableList},
        SectionEntry,
    };

    use super::{LocalVariableSection, VariableItemEntry};

    #[test]
    fn test_save_section() {
        let mut entires: Vec<Vec<VariableItemEntry>> = Vec::new();

        entires.push(vec![
            VariableItemEntry::from_i32(),
            VariableItemEntry::from_i64(),
            VariableItemEntry::from_f32(),
            VariableItemEntry::from_f64(),
        ]);

        entires.push(vec![
            VariableItemEntry::from_i32(),
            VariableItemEntry::from_bytes(1, 2),
            VariableItemEntry::from_i32(),
            VariableItemEntry::from_bytes(6, 12),
            VariableItemEntry::from_bytes(12, 16),
            VariableItemEntry::from_i32(),
        ]);

        entires.push(vec![]);
        entires.push(vec![VariableItemEntry::from_bytes(1, 4)]);
        entires.push(vec![]);
        entires.push(vec![]);
        entires.push(vec![VariableItemEntry::from_i32()]);

        let (lists, list_data) = LocalVariableSection::convert_from_entries(&entires);

        let section = LocalVariableSection {
            lists: &lists,
            list_data: &list_data,
        };

        let mut section_data: Vec<u8> = Vec::new();
        section.save(&mut section_data).unwrap();

        assert_eq!(
            section_data,
            vec![
                //
                // header
                //
                7u8, 0, 0, 0, // item count
                0, 0, 0, 0, // 4 bytes padding
                //
                // table
                //
                0, 0, 0, 0, // offset
                4, 0, 0, 0, // count
                32, 0, 0, 0, // alloc bytes
                //
                48, 0, 0, 0, // offset = 4 (count) * 12 (bytes/record)
                6, 0, 0, 0, // count
                56, 0, 0, 0, // alloc bytes
                //
                120, 0, 0, 0, // offset = 48 + (6 * 12)
                0, 0, 0, 0, // count
                0, 0, 0, 0, // alloc bytes
                //
                120, 0, 0, 0, // offset = 120 + 0
                1, 0, 0, 0, // count
                8, 0, 0, 0, // alloc bytes
                //
                132, 0, 0, 0, // offset = 120 + (1 * 12)
                0, 0, 0, 0, // count
                0, 0, 0, 0, // alloc bytes
                //
                132, 0, 0, 0, // offset = 132 + 0
                0, 0, 0, 0, // count
                0, 0, 0, 0, // alloc bytes
                //
                132, 0, 0, 0, // offset = 132 + 0
                1, 0, 0, 0, // count
                8, 0, 0, 0, // alloc bytes
                //
                // data
                //
                // list 0
                0, 0, 0, 0, // var offset (i32)
                4, 0, 0, 0, // var len
                0, // data type
                0, // padding
                4, 0, // align
                //
                8, 0, 0, 0, // var offset (i64)
                8, 0, 0, 0, // var len
                1, // data type
                0, // padding
                8, 0, // align
                //
                16, 0, 0, 0, // var offset (f32)
                4, 0, 0, 0, // var len
                2, // data type
                0, // padding
                4, 0, // align
                //
                24, 0, 0, 0, // var offset (f64)
                8, 0, 0, 0, // var len
                3, // data type
                0, // padding
                8, 0, // align
                //
                // list 1
                0, 0, 0, 0, // var offset (i32)
                4, 0, 0, 0, // var len
                0, // data type
                0, // padding
                4, 0, // align
                //
                8, 0, 0, 0, // var offset (byte len 1 align 2)
                1, 0, 0, 0, // var len
                4, // data type
                0, // padding
                2, 0, // align
                //
                16, 0, 0, 0, // var offset (i32)
                4, 0, 0, 0, // var len
                0, // data type
                0, // padding
                4, 0, // align
                //
                24, 0, 0, 0, // var offset (byte len 6 align 12)
                6, 0, 0, 0, // var len
                4, // data type
                0, // padding
                12, 0, // align
                //
                32, 0, 0, 0, // var offset (byte len 12 align 16)
                12, 0, 0, 0, // var len
                4, // data type
                0, // padding
                16, 0, // align
                //
                48, 0, 0, 0, // var offset (i32)
                4, 0, 0, 0, // var len
                0, // data type
                0, // padding
                4, 0, // align
                //
                // list 3
                0, 0, 0, 0, // var offset (byte len 1 align 4)
                1, 0, 0, 0, // var len
                4, // data type
                0, // padding
                4, 0, // align
                //
                // list 6
                0, 0, 0, 0, // var offset (i32)
                4, 0, 0, 0, // var len
                0, // data type
                0, // padding
                4, 0 // align
            ]
        );
    }

    #[test]
    fn test_load_section() {
        let section_data = vec![
            //
            // header
            //
            7u8, 0, 0, 0, // item count
            0, 0, 0, 0, // 4 bytes padding
            //
            // table
            //
            0, 0, 0, 0, // offset
            4, 0, 0, 0, // count
            32, 0, 0, 0, // alloc bytes
            //
            48, 0, 0, 0, // offset = 4 (count) * 12 (bytes/record)
            6, 0, 0, 0, // count
            56, 0, 0, 0, // alloc bytes
            //
            120, 0, 0, 0, // offset = 48 + (6 * 12)
            0, 0, 0, 0, // count
            0, 0, 0, 0, // alloc bytes
            //
            120, 0, 0, 0, // offset = 120 + 0
            1, 0, 0, 0, // count
            8, 0, 0, 0, // alloc bytes
            //
            132, 0, 0, 0, // offset = 120 + (1 * 12)
            0, 0, 0, 0, // count
            0, 0, 0, 0, // alloc bytes
            //
            132, 0, 0, 0, // offset = 132 + 0
            0, 0, 0, 0, // count
            0, 0, 0, 0, // alloc bytes
            //
            132, 0, 0, 0, // offset = 132 + 0
            1, 0, 0, 0, // count
            8, 0, 0, 0, // alloc bytes
            //
            // data
            //
            // list 0
            0, 0, 0, 0, // var offset (i32)
            4, 0, 0, 0, // var len
            0, // data type
            0, // padding
            4, 0, // align
            //
            8, 0, 0, 0, // var offset (i64)
            8, 0, 0, 0, // var len
            1, // data type
            0, // padding
            8, 0, // align
            //
            16, 0, 0, 0, // var offset (f32)
            4, 0, 0, 0, // var len
            2, // data type
            0, // padding
            4, 0, // align
            //
            24, 0, 0, 0, // var offset (f64)
            8, 0, 0, 0, // var len
            3, // data type
            0, // padding
            8, 0, // align
            //
            // list 1
            0, 0, 0, 0, // var offset (i32)
            4, 0, 0, 0, // var len
            0, // data type
            0, // padding
            4, 0, // align
            //
            8, 0, 0, 0, // var offset (byte len 1 align 2)
            1, 0, 0, 0, // var len
            4, // data type
            0, // padding
            2, 0, // align
            //
            16, 0, 0, 0, // var offset (i32)
            4, 0, 0, 0, // var len
            0, // data type
            0, // padding
            4, 0, // align
            //
            24, 0, 0, 0, // var offset (byte len 6 align 12)
            6, 0, 0, 0, // var len
            4, // data type
            0, // padding
            12, 0, // align
            //
            32, 0, 0, 0, // var offset (byte len 12 align 16)
            12, 0, 0, 0, // var len
            4, // data type
            0, // padding
            16, 0, // align
            //
            48, 0, 0, 0, // var offset (i32)
            4, 0, 0, 0, // var len
            0, // data type
            0, // padding
            4, 0, // align
            //
            // list 3
            0, 0, 0, 0, // var offset (byte len 1 align 4)
            1, 0, 0, 0, // var len
            4, // data type
            0, // padding
            4, 0, // align
            //
            // list 6
            0, 0, 0, 0, // var offset (i32)
            4, 0, 0, 0, // var len
            0, // data type
            0, // padding
            4, 0, // align
        ];

        let section = LocalVariableSection::load(&section_data);

        assert_eq!(section.lists.len(), 7);

        // check lists

        assert_eq!(
            section.lists[0],
            VariableList {
                list_offset: 0,
                list_item_count: 4,
                list_allocated_in_bytes: 32
            }
        );

        assert_eq!(
            section.lists[1],
            VariableList {
                list_offset: 48,
                list_item_count: 6,
                list_allocated_in_bytes: 56
            }
        );

        assert_eq!(
            section.lists[2],
            VariableList {
                list_offset: 120,
                list_item_count: 0,
                list_allocated_in_bytes: 0
            }
        );

        assert_eq!(
            section.lists[3],
            VariableList {
                list_offset: 120,
                list_item_count: 1,
                list_allocated_in_bytes: 8
            }
        );

        assert_eq!(
            section.lists[4],
            VariableList {
                list_offset: 132,
                list_item_count: 0,
                list_allocated_in_bytes: 0
            }
        );

        assert_eq!(
            section.lists[5],
            VariableList {
                list_offset: 132,
                list_item_count: 0,
                list_allocated_in_bytes: 0
            }
        );

        assert_eq!(
            section.lists[6],
            VariableList {
                list_offset: 132,
                list_item_count: 1,
                list_allocated_in_bytes: 8
            }
        );

        // check var items

        let list0 = section.get_variable_list(0);
        assert_eq!(
            list0,
            &vec![
                VariableItem::new(0, 4, DataType::I32, 4),
                VariableItem::new(8, 8, DataType::I64, 8),
                VariableItem::new(16, 4, DataType::F32, 4),
                VariableItem::new(24, 8, DataType::F64, 8),
            ]
        );

        let list1 = section.get_variable_list(1);
        assert_eq!(
            list1,
            &vec![
                VariableItem::new(0, 4, DataType::I32, 4),
                VariableItem::new(8, 1, DataType::BYTE, 2),
                VariableItem::new(16, 4, DataType::I32, 4),
                VariableItem::new(24, 6, DataType::BYTE, 12),
                VariableItem::new(32, 12, DataType::BYTE, 16),
                VariableItem::new(48, 4, DataType::I32, 4),
            ]
        );

        let list2 = section.get_variable_list(2);
        assert_eq!(list2.len(), 0);

        let list3 = section.get_variable_list(3);
        assert_eq!(list3, &vec![VariableItem::new(0, 1, DataType::BYTE, 4),]);

        let list4 = section.get_variable_list(4);
        assert_eq!(list4.len(), 0);

        let list5 = section.get_variable_list(5);
        assert_eq!(list5.len(), 0);

        let list6 = section.get_variable_list(6);
        assert_eq!(list6, &vec![VariableItem::new(0, 4, DataType::I32, 4),]);
    }
}
