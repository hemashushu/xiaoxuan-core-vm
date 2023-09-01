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
//  item 0 -->  | list offset 0 (u32) | list length 0 (u32) | local vars length 0 (u32)      | <-- table
//  item 1 -->  | list offset 1       | list length 1       | local vars length 1            |
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
//              |----------------------------------------------------------------------------|
//  item 0 -->  | var offset 0 (u32) | var length 0 (u32) | data type 0 (u8) | pad 3 bytes   | <-- list
//  item 1 -->  | var offset 1 (u32) | var length 1 (u32) | data type 1 (u8) | pad 3 bytes   |
//              | ...                                                                        |
//              |----------------------------------------------------------------------------|
//
// note that all variables are aligned to 8-byte, and the size is multipled by 8.
// the length of some type of variable (e.g. struct) may be NOT multipled by 8, but
// it still occur the local variables space by the size which is multipled by 8.
// e.g. one struct length is 12 bytes, but it takes up 16 (= 8 * 2) bytes in the local area.

use ancvm_types::DataType;

use crate::utils::{
    load_section_with_one_table, load_section_with_table_and_data_area,
    save_section_with_one_table, save_section_with_table_and_data_area,
};

use super::{SectionEntry, SectionId};

#[derive(Debug, PartialEq)]
pub struct LocalVariableSection<'a> {
    pub items: &'a [VariableListItem],
    pub list_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct VariableListItem {
    pub list_offset: u32,
    pub list_length: u32,
    pub local_variables_length_in_bytes: u32,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct VariableItem {
    pub var_offset: u32, // the offset of a data item in the "local variable slots area"
    pub var_length: u32, // the length (in bytes) of a data item in the "local variable slots area"
    pub data_type: u8, // the data type field is not necessary at runtime, though it is helpful for debugging.
    _padding0: u8,     //
    _padding1: u16,    //
}

#[derive(Debug, PartialEq)]
pub struct VariableListEntry {
    pub variable_items: Vec<VariableItem>,
}
