// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_binary::module_image::data_section::DataItem;

use crate::{
    host_accessable_memory::HostAccessableMemory, indexed_memory::IndexedMemory, memory::Memory,
};

pub struct ReadOnlyDatas<'a> {
    data_items: &'a [DataItem],
    datas: &'a [u8],
}

pub struct ReadWriteDatas<'a> {
    data_items: &'a [DataItem],
    datas: Vec<u8>,
}

pub struct UninitDatas<'a> {
    data_items: &'a [DataItem],
    datas: Vec<u8>,
}

impl<'a> ReadOnlyDatas<'a> {
    pub fn new(data_items: &'a [DataItem], datas: &'a [u8]) -> Self {
        Self { data_items, datas }
    }
}

impl<'a> ReadWriteDatas<'a> {
    pub fn new(data_items: &'a [DataItem], datas: Vec<u8>) -> Self {
        Self { data_items, datas }
    }
}

impl<'a> UninitDatas<'a> {
    pub fn new(data_items: &'a [DataItem], datas: Vec<u8>) -> Self {
        Self { data_items, datas }
    }
}

impl Memory for ReadOnlyDatas<'_> {
    #[inline]
    fn get_ptr(&self, addr: usize) -> *const u8 {
        (&self.datas[addr..]).as_ptr()
    }

    #[inline]
    fn get_mut_ptr(&mut self, _: usize) -> *mut u8 {
        panic!("Read-only memory can not be written to.")
    }
}

impl IndexedMemory for ReadOnlyDatas<'_> {
    #[inline]
    fn get_offset_and_length_by_index(&self, idx: usize) -> (usize, usize) {
        let item = &self.data_items[idx];
        (item.data_offset as usize, item.data_length as usize)
    }
}

impl HostAccessableMemory for ReadOnlyDatas<'_> {
    #[inline]
    fn get_host_address(&self, offset: usize) -> usize {
        (&self.datas[offset..]).as_ptr() as usize
    }
}

impl Memory for ReadWriteDatas<'_> {
    #[inline]
    fn get_ptr(&self, addr: usize) -> *const u8 {
        (&self.datas[addr..]).as_ptr()
    }

    #[inline]
    fn get_mut_ptr(&mut self, addr: usize) -> *mut u8 {
        (&mut self.datas[addr..]).as_mut_ptr()
    }
}

impl IndexedMemory for ReadWriteDatas<'_> {
    #[inline]
    fn get_offset_and_length_by_index(&self, idx: usize) -> (usize, usize) {
        let item = &self.data_items[idx];
        (item.data_offset as usize, item.data_length as usize)
    }
}

impl HostAccessableMemory for ReadWriteDatas<'_> {
    #[inline]
    fn get_host_address(&self, offset: usize) -> usize {
        (&self.datas[offset..]).as_ptr() as usize
    }
}

impl Memory for UninitDatas<'_> {
    #[inline]
    fn get_ptr(&self, addr: usize) -> *const u8 {
        (&self.datas[addr..]).as_ptr()
    }

    #[inline]
    fn get_mut_ptr(&mut self, addr: usize) -> *mut u8 {
        (&mut self.datas[addr..]).as_mut_ptr()
    }
}

impl IndexedMemory for UninitDatas<'_> {
    #[inline]
    fn get_offset_and_length_by_index(&self, idx: usize) -> (usize, usize) {
        let item = &self.data_items[idx];
        (item.data_offset as usize, item.data_length as usize)
    }
}

impl HostAccessableMemory for UninitDatas<'_> {
    #[inline]
    fn get_host_address(&self, offset: usize) -> usize {
        (&self.datas[offset..]).as_ptr() as usize
    }
}