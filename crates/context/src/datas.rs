// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_image::common_sections::{
    read_only_data_section, read_write_data_section, uninit_data_section,
};

use crate::{indexed_memory_access::IndexedMemoryAccess, memory_access::MemoryAccess};

pub struct ReadOnlyDatas<'a> {
    data_items: &'a [read_only_data_section::DataItem],
    datas: &'a [u8],
}

pub struct ReadWriteDatas<'a> {
    data_items: &'a [read_write_data_section::DataItem],
    datas: Vec<u8>,
}

pub struct UninitDatas<'a> {
    data_items: &'a [uninit_data_section::DataItem],
    datas: Vec<u8>,
}

impl<'a> ReadOnlyDatas<'a> {
    pub fn new(data_items: &'a [read_only_data_section::DataItem], datas: &'a [u8]) -> Self {
        Self { data_items, datas }
    }
}

impl<'a> ReadWriteDatas<'a> {
    pub fn new(data_items: &'a [read_write_data_section::DataItem], datas: Vec<u8>) -> Self {
        Self { data_items, datas }
    }
}

impl<'a> UninitDatas<'a> {
    pub fn new(data_items: &'a [uninit_data_section::DataItem], datas: Vec<u8>) -> Self {
        Self { data_items, datas }
    }
}

impl MemoryAccess for ReadOnlyDatas<'_> {
    #[inline]
    fn get_ptr(&self, address: usize) -> *const u8 {
        self.datas[address..].as_ptr()
    }

    #[inline]
    fn get_mut_ptr(&mut self, _address: usize) -> *mut u8 {
        panic!("Read-only memory can not be written to.")
    }
}

impl IndexedMemoryAccess for ReadOnlyDatas<'_> {
    #[inline]
    fn get_offset_and_length_by_index(&self, idx: usize) -> (usize, usize) {
        let item = &self.data_items[idx];
        (item.data_offset as usize, item.data_length as usize)
    }
}

impl MemoryAccess for ReadWriteDatas<'_> {
    #[inline]
    fn get_ptr(&self, address: usize) -> *const u8 {
        self.datas[address..].as_ptr()
    }

    #[inline]
    fn get_mut_ptr(&mut self, address: usize) -> *mut u8 {
        self.datas[address..].as_mut_ptr()
    }
}

impl IndexedMemoryAccess for ReadWriteDatas<'_> {
    #[inline]
    fn get_offset_and_length_by_index(&self, idx: usize) -> (usize, usize) {
        let item = &self.data_items[idx];
        (item.data_offset as usize, item.data_length as usize)
    }
}

impl MemoryAccess for UninitDatas<'_> {
    #[inline]
    fn get_ptr(&self, address: usize) -> *const u8 {
        self.datas[address..].as_ptr()
    }

    #[inline]
    fn get_mut_ptr(&mut self, address: usize) -> *mut u8 {
        self.datas[address..].as_mut_ptr()
    }
}

impl IndexedMemoryAccess for UninitDatas<'_> {
    #[inline]
    fn get_offset_and_length_by_index(&self, idx: usize) -> (usize, usize) {
        let item = &self.data_items[idx];
        (item.data_offset as usize, item.data_length as usize)
    }
}
