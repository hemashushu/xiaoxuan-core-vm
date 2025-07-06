// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use crate::{memory_access::MemoryAccess, MemoryError};

/// In the XiaoXuam Core VM, local variables, data, and allocator memory are accessed
/// using an index instead of a memory address (pointer).
///
/// This mechanism enhances safety during data access and improves the efficiency
/// of high-level programming languages, as it eliminates the need for boundary
/// checks when accessing arrays.
pub trait IndexedMemoryAccess: MemoryAccess {
    /// Retrieves the start address of the data associated with the given index.
    ///
    /// Some memory implementations (e.g., data sections) may use a large block
    /// of underlying memory to simulate each memory item. In such cases, the
    /// "start address" refers to the location of the memory item. However, other
    /// implementations may not have a "start address" and handle memory items differently.
    fn get_start_address_by_index(&self, idx: usize) -> usize;

    /// Retrieves the length of the data associated with the given index.
    ///
    /// Indexed data contains size, capacity, and other information.
    fn get_data_length(&self, idx: usize) -> usize;

    fn read_idx(
        &self,
        idx: usize,
        src_offset_in_bytes: usize,
        length_in_bytes: usize,
        dst_ptr: *mut u8,
    ) {
        self.read(
            self.get_start_address_by_index(idx),
            src_offset_in_bytes,
            length_in_bytes,
            dst_ptr,
        );
    }

    /// Reads a 64-bit integer from the memory at the specified index and offset.
    fn read_idx_i64(&self, idx: usize, src_offset: usize, dst_ptr_64: *mut u64) {
        self.read_i64(self.get_start_address_by_index(idx), src_offset, dst_ptr_64);
    }

    /// Reads a signed 32-bit integer from the memory at the specified index and offset.
    fn read_idx_i32_s_to_i64(&self, idx: usize, src_offset: usize, dst_ptr_64: *mut i64) {
        self.read_i32_s_to_i64(self.get_start_address_by_index(idx), src_offset, dst_ptr_64);
    }

    /// Reads an unsigned 32-bit integer from the memory at the specified index and offset.
    fn read_idx_i32_u_to_u64(&self, idx: usize, src_offset: usize, dst_ptr_64: *mut u64) {
        self.read_i32_u_to_u64(self.get_start_address_by_index(idx), src_offset, dst_ptr_64);
    }

    /// Reads a signed 16-bit integer from the memory at the specified index and offset.
    fn read_idx_i16_s_to_i64(&self, idx: usize, src_offset: usize, dst_ptr_64: *mut i64) {
        self.read_i16_s_to_i64(self.get_start_address_by_index(idx), src_offset, dst_ptr_64)
    }

    /// Reads an unsigned 16-bit integer from the memory at the specified index and offset.
    fn read_idx_i16_u_to_u64(&self, idx: usize, src_offset: usize, dst_ptr_64: *mut u64) {
        self.read_i16_u_to_u64(self.get_start_address_by_index(idx), src_offset, dst_ptr_64)
    }

    /// Reads a signed 8-bit integer from the memory at the specified index and offset.
    fn read_idx_i8_s_to_i64(&self, idx: usize, src_offset: usize, dst_ptr_64: *mut i64) {
        self.read_i8_s_to_i64(self.get_start_address_by_index(idx), src_offset, dst_ptr_64)
    }

    /// Reads an unsigned 8-bit integer from the memory at the specified index and offset.
    fn read_idx_i8_u_to_u64(&self, idx: usize, src_offset: usize, dst_ptr_64: *mut u64) {
        self.read_i8_u_to_u64(self.get_start_address_by_index(idx), src_offset, dst_ptr_64)
    }

    /// Reads a 64-bit floating-point number from the memory at the specified index and offset.
    fn read_idx_f64(
        &self,
        idx: usize,
        src_offset: usize,
        dst_ptr_64: *mut f64,
    ) -> Result<(), MemoryError> {
        self.read_f64(self.get_start_address_by_index(idx), src_offset, dst_ptr_64)
    }

    /// Reads a 32-bit floating-point number from the memory at the specified index and offset.
    fn read_idx_f32(
        &self,
        idx: usize,
        src_offset: usize,
        dst_ptr_32: *mut f32,
    ) -> Result<(), MemoryError> {
        self.read_f32(self.get_start_address_by_index(idx), src_offset, dst_ptr_32)
    }

    fn write_idx(
        &mut self,
        src_ptr: *const u8,
        idx: usize,
        dst_offset_in_bytes: usize,
        length_in_bytes: usize,
    ) {
        self.write(
            src_ptr,
            self.get_start_address_by_index(idx),
            dst_offset_in_bytes,
            length_in_bytes,
        );
    }

    /// Writes a 64-bit integer to the memory at the specified index and offset.
    /// This function is also used to write 64-bit floating-point numbers.
    /// Note: No validation checks are performed during memory write operations.
    fn write_idx_i64(&mut self, src_ptr: *const u8, idx: usize, dst_offset_in_bytes: usize) {
        self.write_i64(
            src_ptr,
            self.get_start_address_by_index(idx),
            dst_offset_in_bytes,
        );
    }

    /// Writes a 32-bit integer to the memory at the specified index and offset.
    /// This function is also used to write 32-bit floating-point numbers.
    /// Note: No validation checks are performed during memory write operations.
    fn write_idx_i32(&mut self, src_ptr: *const u8, idx: usize, dst_offset_in_bytes: usize) {
        self.write_i32(
            src_ptr,
            self.get_start_address_by_index(idx),
            dst_offset_in_bytes,
        );
    }

    /// Writes a 16-bit integer to the memory at the specified index and offset.
    fn write_idx_i16(&mut self, src_ptr: *const u8, idx: usize, dst_offset_in_bytes: usize) {
        self.write_i16(
            src_ptr,
            self.get_start_address_by_index(idx),
            dst_offset_in_bytes,
        );
    }

    /// Writes an 8-bit integer to the memory at the specified index and offset.
    fn write_idx_i8(&mut self, src_ptr: *const u8, idx: usize, dst_offset_in_bytes: usize) {
        self.write_i8(
            src_ptr,
            self.get_start_address_by_index(idx),
            dst_offset_in_bytes,
        );
    }
}
