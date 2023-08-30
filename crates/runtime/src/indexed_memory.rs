// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_binary::module_image::data_section::DataItem;

use crate::memory::Memory;

pub trait IndexedMemory: Memory {
    // it's recommended that add annotation "#[inline]" to the implementation
    fn get_offset_and_length_by_index(&self, idx: usize) -> (usize, usize);

    fn items_count(&self) -> usize;

    fn read_i32_by_index(&self, idx: usize) -> i32 {
        let (addr, _) = self.get_offset_and_length_by_index(idx);
        self.read_i32(addr)
    }

    fn read_i64_by_index(&self, idx: usize) -> i64 {
        let (addr, _) = self.get_offset_and_length_by_index(idx);
        self.read_i64(addr)
    }

    fn read_f32_by_index(&self, idx: usize) -> f32 {
        let (addr, _) = self.get_offset_and_length_by_index(idx);
        self.read_f32(addr)
    }

    fn read_f64_by_index(&self, idx: usize) -> f64 {
        let (addr, _) = self.get_offset_and_length_by_index(idx);
        self.read_f64(addr)
    }

    fn write_i32_by_index(&mut self, idx: usize, value: i32) {
        let (addr, _) = self.get_offset_and_length_by_index(idx);
        self.write_i32(addr, value)
    }

    fn write_i64_by_index(&mut self, idx: usize, value: i64) {
        let (addr, _) = self.get_offset_and_length_by_index(idx);
        self.write_i64(addr, value)
    }

    fn write_f32_by_index(&mut self, idx: usize, value: f32) {
        let (addr, _) = self.get_offset_and_length_by_index(idx);
        self.write_f32(addr, value)
    }

    fn write_f64_by_index(&mut self, idx: usize, value: f64) {
        let (addr, _) = self.get_offset_and_length_by_index(idx);
        self.write_f64(addr, value)
    }
}

pub struct ReadOnlyMemory<'a> {
    data_items: &'a [DataItem],
    datas: &'a [u8],
}

pub struct ReadWriteMemory<'a> {
    data_items: &'a [DataItem],
    datas: Vec<u8>,
}

pub struct UninitMemory<'a> {
    data_items: &'a [DataItem],
    datas: Vec<u8>,
}

impl<'a> ReadOnlyMemory<'a> {
    pub fn new(data_items: &'a [DataItem], datas: &'a [u8]) -> Self {
        Self { data_items, datas }
    }
}

impl<'a> ReadWriteMemory<'a> {
    pub fn new(data_items: &'a [DataItem], datas: Vec<u8>) -> Self {
        Self { data_items, datas }
    }
}

impl<'a> UninitMemory<'a> {
    pub fn new(data_items: &'a [DataItem], datas: Vec<u8>) -> Self {
        Self { data_items, datas }
    }
}

impl Memory for ReadOnlyMemory<'_> {
    #[inline]
    fn get_ptr(&self, addr: usize) -> *const u8 {
        (&self.datas[addr..]).as_ptr()
    }

    #[inline]
    fn get_mut_ptr(&mut self, addr: usize) -> *mut u8 {
        panic!("Read-only memory can not be written to.")
    }
}

impl IndexedMemory for ReadOnlyMemory<'_> {
    #[inline]
    fn get_offset_and_length_by_index(&self, idx: usize) -> (usize, usize) {
        let item = &self.data_items[idx];
        (item.data_offset as usize, item.data_length as usize)
    }

    fn items_count(&self) -> usize {
        self.data_items.len()
    }
}

impl Memory for ReadWriteMemory<'_> {
    #[inline]
    fn get_ptr(&self, addr: usize) -> *const u8 {
        (&self.datas[addr..]).as_ptr()
    }

    #[inline]
    fn get_mut_ptr(&mut self, addr: usize) -> *mut u8 {
        (&mut self.datas[addr..]).as_mut_ptr()
    }
}

impl IndexedMemory for ReadWriteMemory<'_> {
    #[inline]
    fn get_offset_and_length_by_index(&self, idx: usize) -> (usize, usize) {
        let item = &self.data_items[idx];
        (item.data_offset as usize, item.data_length as usize)
    }

    fn items_count(&self) -> usize {
        self.data_items.len()
    }
}

impl Memory for UninitMemory<'_> {
    #[inline]
    fn get_ptr(&self, addr: usize) -> *const u8 {
        (&self.datas[addr..]).as_ptr()
    }

    #[inline]
    fn get_mut_ptr(&mut self, addr: usize) -> *mut u8 {
        (&mut self.datas[addr..]).as_mut_ptr()
    }
}

impl IndexedMemory for UninitMemory<'_> {
    #[inline]
    fn get_offset_and_length_by_index(&self, idx: usize) -> (usize, usize) {
        let item = &self.data_items[idx];
        (item.data_offset as usize, item.data_length as usize)
    }

    fn items_count(&self) -> usize {
        self.data_items.len()
    }
}
