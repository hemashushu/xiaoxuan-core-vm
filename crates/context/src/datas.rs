// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_image::common_sections::{
    read_only_data_section, read_write_data_section, uninit_data_section,
};
use anc_memory::{indexed_memory_access::IndexedMemoryAccess, memory_access::MemoryAccess};

// Represents a collection of read-only data items and their associated raw data.
pub struct ReadOnlyDatas<'a> {
    data_items: &'a [read_only_data_section::DataItem], // Metadata for each data item.
    datas: &'a [u8],                                    // Raw data buffer.
}

// Represents a collection of read-write data items and their associated raw data.
pub struct ReadWriteDatas<'a> {
    data_items: &'a [read_write_data_section::DataItem], // Metadata for each data item.
    datas: Vec<u8>,                                      // Mutable raw data buffer.
}

// Represents a collection of uninitialized data items and their associated raw data.
pub struct UninitDatas<'a> {
    data_items: &'a [uninit_data_section::DataItem], // Metadata for each data item.
    datas: Vec<u8>, // Mutable raw data buffer for uninitialized data.
}

impl<'a> ReadOnlyDatas<'a> {
    // Creates a new instance of ReadOnlyDatas with the given data items and raw data.
    pub fn new(data_items: &'a [read_only_data_section::DataItem], datas: &'a [u8]) -> Self {
        Self { data_items, datas }
    }
}

impl<'a> ReadWriteDatas<'a> {
    // Creates a new instance of ReadWriteDatas with the given data items and raw data.
    pub fn new(data_items: &'a [read_write_data_section::DataItem], datas: Vec<u8>) -> Self {
        Self { data_items, datas }
    }
}

impl<'a> UninitDatas<'a> {
    // Creates a new instance of UninitDatas with the given data items and raw data.
    pub fn new(data_items: &'a [uninit_data_section::DataItem], datas: Vec<u8>) -> Self {
        Self { data_items, datas }
    }
}

impl MemoryAccess for ReadOnlyDatas<'_> {
    // Returns a pointer to the specified address and offset in the read-only data buffer.
    #[inline]
    fn get_ptr(&self, address: usize, offset_in_bytes: usize) -> *const u8 {
        unsafe { self.datas[address..].as_ptr().add(offset_in_bytes) }
    }

    // Panics because read-only memory cannot be written to.
    #[inline]
    fn get_mut_ptr(&mut self, _address: usize, _offset_in_bytes: usize) -> *mut u8 {
        panic!("Read-only memory cannot be written to.")
    }
}

impl IndexedMemoryAccess for ReadOnlyDatas<'_> {
    // Returns the start address of the data item at the specified index.
    #[inline]
    fn get_start_address_by_index(&self, idx: usize) -> usize {
        let item = &self.data_items[idx];
        item.data_offset as usize
    }

    // Returns the length of the data item at the specified index.
    fn get_data_length(&self, idx: usize) -> usize {
        let item = &self.data_items[idx];
        item.data_length as usize
    }
}

impl MemoryAccess for ReadWriteDatas<'_> {
    // Returns a pointer to the specified address and offset in the read-write data buffer.
    #[inline]
    fn get_ptr(&self, address: usize, offset_in_bytes: usize) -> *const u8 {
        unsafe { self.datas[address..].as_ptr().add(offset_in_bytes) }
    }

    // Returns a mutable pointer to the specified address and offset in the read-write data buffer.
    #[inline]
    fn get_mut_ptr(&mut self, address: usize, offset_in_bytes: usize) -> *mut u8 {
        unsafe { self.datas[address..].as_mut_ptr().add(offset_in_bytes) }
    }
}

impl IndexedMemoryAccess for ReadWriteDatas<'_> {
    // Returns the start address of the data item at the specified index.
    #[inline]
    fn get_start_address_by_index(&self, idx: usize) -> usize {
        let item = &self.data_items[idx];
        item.data_offset as usize
    }

    // Returns the length of the data item at the specified index.
    fn get_data_length(&self, idx: usize) -> usize {
        let item = &self.data_items[idx];
        item.data_length as usize
    }
}

impl MemoryAccess for UninitDatas<'_> {
    // Returns a pointer to the specified address and offset in the uninitialized data buffer.
    #[inline]
    fn get_ptr(&self, address: usize, offset_in_bytes: usize) -> *const u8 {
        unsafe { self.datas[address..].as_ptr().add(offset_in_bytes) }
    }

    // Returns a mutable pointer to the specified address and offset in the uninitialized data buffer.
    #[inline]
    fn get_mut_ptr(&mut self, address: usize, offset_in_bytes: usize) -> *mut u8 {
        unsafe { self.datas[address..].as_mut_ptr().add(offset_in_bytes) }
    }
}

impl IndexedMemoryAccess for UninitDatas<'_> {
    // Returns the start address of the data item at the specified index.
    #[inline]
    fn get_start_address_by_index(&self, idx: usize) -> usize {
        let item = &self.data_items[idx];
        item.data_offset as usize
    }

    // Returns the length of the data item at the specified index.
    fn get_data_length(&self, idx: usize) -> usize {
        let item = &self.data_items[idx];
        item.data_length as usize
    }
}
