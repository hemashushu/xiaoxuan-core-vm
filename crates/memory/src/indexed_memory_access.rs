// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use crate::memory_access::MemoryAccess;

/// In the XiaoXuam Core VM, local variables, data, and allocator memory are accessed
/// using an index instead of a memory address (pointer).
///
/// This mechanism enhances safety during data access and improves the efficiency
/// of high-level programming languages, as it eliminates the need for boundary
/// checks when accessing arrays.
pub trait IndexedMemoryAccess: MemoryAccess {
    /// Retrieves the start address and length of the data associated with the given index.
    ///
    /// Some memory implementations (e.g., data sections) may use a large block
    /// of underlying memory to simulate each memory item. In such cases, the
    /// "start address" refers to the location of the memory item. However, other
    /// implementations may not have a "start address" and handle memory items differently.
    fn get_start_address_and_length_by_index(&self, idx: usize) -> (usize, usize);

    /// Returns the address of the data for the given index and offset.
    /// If the implementation does not have a "start address," the offset itself is returned.
    #[inline]
    fn get_data_address_by_index_and_offset(&self, idx: usize, offset: usize) -> usize {
        let (start, _length) = self.get_start_address_and_length_by_index(idx);
        start + offset
    }

    /// Reads a 64-bit integer from the memory at the specified index and offset.
    fn read_idx_i64(&self, idx: usize, offset: usize, dst_ptr: *mut u8) {
        self.read_i64(
            self.get_data_address_by_index_and_offset(idx, offset),
            dst_ptr,
        );
    }

    /// Reads a signed 32-bit integer from the memory at the specified index and offset.
    fn read_idx_i32_s(&self, idx: usize, offset: usize, dst_ptr: *mut u8) {
        self.read_i32_s(
            self.get_data_address_by_index_and_offset(idx, offset),
            dst_ptr,
        );
    }

    /// Reads an unsigned 32-bit integer from the memory at the specified index and offset.
    fn read_idx_i32_u(&self, idx: usize, offset: usize, dst_ptr: *mut u8) {
        self.read_i32_u(
            self.get_data_address_by_index_and_offset(idx, offset),
            dst_ptr,
        );
    }

    /// Reads a signed 8-bit integer from the memory at the specified index and offset.
    fn read_idx_i8_s(&self, idx: usize, offset: usize, dst_ptr: *mut u8) {
        self.read_i8_s(
            self.get_data_address_by_index_and_offset(idx, offset),
            dst_ptr,
        )
    }

    /// Reads an unsigned 8-bit integer from the memory at the specified index and offset.
    fn read_idx_i8_u(&self, idx: usize, offset: usize, dst_ptr: *mut u8) {
        self.read_i8_u(
            self.get_data_address_by_index_and_offset(idx, offset),
            dst_ptr,
        )
    }

    /// Reads a signed 16-bit integer from the memory at the specified index and offset.
    fn read_idx_i16_s(&self, idx: usize, offset: usize, dst_ptr: *mut u8) {
        self.read_i16_s(
            self.get_data_address_by_index_and_offset(idx, offset),
            dst_ptr,
        )
    }

    /// Reads an unsigned 16-bit integer from the memory at the specified index and offset.
    fn read_idx_i16_u(&self, idx: usize, offset: usize, dst_ptr: *mut u8) {
        self.read_i16_u(
            self.get_data_address_by_index_and_offset(idx, offset),
            dst_ptr,
        )
    }

    /// Reads a 64-bit floating-point number from the memory at the specified index and offset.
    fn read_idx_f64(&self, idx: usize, offset: usize, dst_ptr: *mut u8) -> Result<(), ()> {
        self.read_f64(
            self.get_data_address_by_index_and_offset(idx, offset),
            dst_ptr,
        )
    }

    /// Reads a 32-bit floating-point number from the memory at the specified index and offset.
    fn read_idx_f32(&self, idx: usize, offset: usize, dst_ptr: *mut u8) -> Result<(), ()> {
        self.read_f32(
            self.get_data_address_by_index_and_offset(idx, offset),
            dst_ptr,
        )
    }

    /// Writes a 64-bit integer to the memory at the specified index and offset.
    /// This function is also used to write 64-bit floating-point numbers.
    /// Note: No validation checks are performed during memory write operations.
    fn write_idx_i64(&mut self, src_ptr: *const u8, idx: usize, offset: usize) {
        self.write_i64(
            src_ptr,
            self.get_data_address_by_index_and_offset(idx, offset),
        );
    }

    /// Writes a 32-bit integer to the memory at the specified index and offset.
    /// This function is also used to write 32-bit floating-point numbers.
    /// Note: No validation checks are performed during memory write operations.
    fn write_idx_i32(&mut self, src_ptr: *const u8, idx: usize, offset: usize) {
        self.write_i32(
            src_ptr,
            self.get_data_address_by_index_and_offset(idx, offset),
        );
    }

    /// Writes a 16-bit integer to the memory at the specified index and offset.
    fn write_idx_i16(&mut self, src_ptr: *const u8, idx: usize, offset: usize) {
        self.write_i16(
            src_ptr,
            self.get_data_address_by_index_and_offset(idx, offset),
        );
    }

    /// Writes an 8-bit integer to the memory at the specified index and offset.
    fn write_idx_i8(&mut self, src_ptr: *const u8, idx: usize, offset: usize) {
        self.write_i8(
            src_ptr,
            self.get_data_address_by_index_and_offset(idx, offset),
        );
    }
}
