// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_memory::indexed_memory_access::IndexedMemoryAccess;

pub trait Allocator: IndexedMemoryAccess {
    /// Allocates a block of memory with the specified alignment and size.
    /// Returns a "data public index" that can be used to access the allocated memory.
    ///
    /// # Parameters
    /// - `align_in_bytes`: The alignment of the memory block in bytes.
    /// - `size_in_bytes`: The size of the memory block in bytes.
    ///
    /// # Returns
    /// A unique index representing the allocated memory block.
    fn allocate(&mut self, align_in_bytes: usize, size_in_bytes: usize) -> usize;

    /// Resizes the memory block at the specified "data public index" to the new size.
    ///
    /// # Parameters
    /// - `data_internal_index`: The index of the memory block to resize.
    /// - `new_size_in_bytes`: The new size of the memory block in bytes.
    ///
    /// # Returns
    /// A new "data public index" for the resized memory block. The original index may be returned
    /// if the size is unchanged or the new size is smaller than the original size.
    fn resize(&mut self, data_internal_index: usize, new_size_in_bytes: usize) -> usize;

    /// Retrieves the actual size of the memory block at the specified "data public index".
    ///
    /// # Parameters
    /// - `data_internal_index`: The index of the memory block.
    ///
    /// # Returns
    /// An `Option` containing the size of the memory block in bytes, or `None` if the index is invalid.
    fn get_size(&self, data_internal_index: usize) -> Option<usize>;

    /// Frees the memory block at the specified "data public index".
    ///
    /// # Parameters
    /// - `data_internal_index`: The index of the memory block to free.
    fn free(&mut self, data_internal_index: usize);
}
