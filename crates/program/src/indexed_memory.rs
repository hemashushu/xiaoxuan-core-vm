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
pub trait IndexedMemory: Memory {
    // it's recommended that add annotation "#[inline]" to the implementation
    /// get (offset, length)
    fn get_offset_and_length_by_index(&self, idx: usize) -> (usize, usize);

    #[inline]
    fn get_data_address_by_index_and_offset(&self, idx: usize, offset: usize) -> usize {
        let (start, _length) = self.get_offset_and_length_by_index(idx);
        start + offset
    }

    fn load_idx_64(&self, idx: usize, offset: usize, dst_ptr: *mut u8) {
        self.load_64(
            self.get_data_address_by_index_and_offset(idx, offset),
            dst_ptr,
        );
    }

    fn load_idx_32(&self, idx: usize, offset: usize, dst_ptr: *mut u8) {
        self.load_32(
            self.get_data_address_by_index_and_offset(idx, offset),
            dst_ptr,
        );
    }

    fn load_idx_64_with_float_check(&self, idx: usize, offset: usize, dst_ptr: *mut u8) {
        self.load_64_with_float_check(
            self.get_data_address_by_index_and_offset(idx, offset),
            dst_ptr,
        );
    }

    fn load_idx_32_with_float_check(&self, idx: usize, offset: usize, dst_ptr: *mut u8) {
        self.load_32_with_float_check(
            self.get_data_address_by_index_and_offset(idx, offset),
            dst_ptr,
        );
    }

    fn load_idx_32_extend_from_i8_s(&self, idx: usize, offset: usize, dst_ptr: *mut u8) {
        self.load_32_extend_from_i8_s(
            self.get_data_address_by_index_and_offset(idx, offset),
            dst_ptr,
        )
    }

    fn load_idx_32_extend_from_i8_u(&self, idx: usize, offset: usize, dst_ptr: *mut u8) {
        self.load_32_extend_from_i8_u(
            self.get_data_address_by_index_and_offset(idx, offset),
            dst_ptr,
        )
    }

    fn load_idx_32_extend_from_i16_s(&self, idx: usize, offset: usize, dst_ptr: *mut u8) {
        self.load_32_extend_from_i16_s(
            self.get_data_address_by_index_and_offset(idx, offset),
            dst_ptr,
        )
    }

    fn load_idx_32_extend_from_i16_u(&self, idx: usize, offset: usize, dst_ptr: *mut u8) {
        self.load_32_extend_from_i16_u(
            self.get_data_address_by_index_and_offset(idx, offset),
            dst_ptr,
        )
    }

    fn store_idx_64(&mut self, src_ptr: *const u8, idx: usize, offset: usize) {
        self.store_64(
            src_ptr,
            self.get_data_address_by_index_and_offset(idx, offset),
        );
    }

    fn store_idx_32(&mut self, src_ptr: *const u8, idx: usize, offset: usize) {
        self.store_32(
            src_ptr,
            self.get_data_address_by_index_and_offset(idx, offset),
        );
    }

    fn store_idx_16(&mut self, src_ptr: *const u8, idx: usize, offset: usize) {
        self.store_16(
            src_ptr,
            self.get_data_address_by_index_and_offset(idx, offset),
        );
    }

    fn store_idx_8(&mut self, src_ptr: *const u8, idx: usize, offset: usize) {
        self.store_8(
            src_ptr,
            self.get_data_address_by_index_and_offset(idx, offset),
        );
    }
}
