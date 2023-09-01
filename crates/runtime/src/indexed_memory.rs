// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::memory::Memory;

/// in the XiaoXuam VM, local variable memory and data memory is access
/// by the index instead of the "memory address (pointer)".
///
/// this mechanism makes data access by (higher-level) programs more safe, and
/// also makes high-level programming languages more efficient by eliminating
/// the need to check boundaries over and over again when accessing an array
/// using an index.
///
/// e.g.
///
/// ```c
/// int a[] = {10,11,12};
/// int b[] = {13,14,15};
///
/// int main(void){
///         int i = a[2]; // ok
///         int j = a[4]; // b[0] will be accessed.
/// }
/// ```
///
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
