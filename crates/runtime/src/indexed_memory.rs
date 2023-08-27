// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::memory::Memory;

pub trait IndexedMemory: Memory {
    // it's recommended that add annotation "#[inline]" to the implementation
    fn get_offset_and_length_by_index(&self, idx: u32) -> (usize, usize);

    /// read i32,i64,f32,f64 and so on
    fn read_by_index<T>(&self, idx: u32) -> T {
        let (addr, _) = self.get_offset_and_length_by_index(idx);
        let tp = self.get_ptr(addr) as *const T;
        unsafe { std::ptr::read(tp) }
    }

    /// write i32,i64,f32,f64 and so on
    fn write_by_index<T>(&mut self, idx: u32, value: T) {
        let (addr, _) = self.get_offset_and_length_by_index(idx);
        let tp = self.get_mut_ptr(addr) as *mut T;
        unsafe { std::ptr::write(tp, value) }
    }
}
