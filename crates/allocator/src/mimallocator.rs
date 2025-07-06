// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::ffi::c_void;

use anc_memory::{indexed_memory_access::IndexedMemoryAccess, memory_access::MemoryAccess};
use libmimalloc_sys::{mi_free, mi_malloc_aligned, mi_realloc_aligned};

use crate::allocator::Allocator;

pub struct MiMAllocator;

impl MiMAllocator {
    pub fn new() -> Self {
        MiMAllocator
    }
}

impl Default for MiMAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl Allocator for MiMAllocator {
    fn allocate(&mut self, size_in_bytes: usize, alignment_in_bytes: usize) -> usize {
        // Allocate `size` bytes aligned by `alignment`.
        //
        // Return pointer to the allocated memory or null if out of memory.
        //
        // Returns a unique pointer if called with `size` 0.
        //
        // see:
        // https://github.com/purpleprotocol/mimalloc_rust/blob/master/libmimalloc-sys/src/lib.rs
        let ptr = unsafe { mi_malloc_aligned(size_in_bytes, alignment_in_bytes) };
        ptr as usize
    }

    fn reallocate(
        &mut self,
        data_internal_index: usize,
        new_size_in_bytes: usize,
        alignment_in_bytes: usize,
    ) -> usize {
        // Re-allocate memory to `newsize` bytes, aligned by `alignment`.
        //
        // Return pointer to the allocated memory or null if out of memory. If null
        // is returned, the pointer `p` is not freed. Otherwise the original
        // pointer is either freed or returned as the reallocated result (in case
        // it fits in-place with the new size).
        //
        // If `p` is null, it behaves as [`mi_malloc_aligned`]. If `newsize` is
        // larger than the original `size` allocated for `p`, the bytes after
        // `size` are uninitialized.
        let ptr = data_internal_index as *mut c_void;
        let new_ptr = unsafe { mi_realloc_aligned(ptr, new_size_in_bytes, alignment_in_bytes) };
        new_ptr as usize
    }

    // Free previously allocated memory.
    //
    // The pointer `p` must have been allocated before (or be null).
    fn free(&mut self, data_internal_index: usize) {
        let ptr = data_internal_index as *mut c_void;
        unsafe { mi_free(ptr) };
    }
}

impl MemoryAccess for MiMAllocator {
    fn get_ptr(&self, address: usize, offset_in_bytes: usize) -> *const u8 {
        let addr = address + offset_in_bytes;
        addr as *const c_void as *const u8
    }

    fn get_mut_ptr(&mut self, address: usize, offset_in_bytes: usize) -> *mut u8 {
        let addr = address + offset_in_bytes;
        addr as *mut c_void as *mut u8
    }
}

impl IndexedMemoryAccess for MiMAllocator {
    fn get_start_address_by_index(&self, idx: usize) -> usize {
        idx // the index is memory address in the allocator
    }

    fn get_data_length(&self, _idx: usize) -> usize {
        panic!("MiMAllocator does not support data length retrieval");
    }
}

#[cfg(test)]
mod tests {
    use anc_memory::{indexed_memory_access::IndexedMemoryAccess, memory_access::MemoryAccess};

    use crate::{allocator::Allocator, mimallocator::MiMAllocator};

    #[test]
    fn test_allocate_reallocate_and_free() {
        let mut allocator = MiMAllocator::new();

        let data0 = [0x02u8, 0x03, 0x05, 0x07];

        // Allocate memory
        let index0 = allocator.allocate(4, 4);

        // Write data to the allocated memory.
        allocator.write(data0.as_ptr(), index0, 0, 4);

        // Read and verify the data.
        let mut buf0 = [0u8; 4];
        allocator.read(index0, 0, 4, buf0.as_mut_ptr());
        assert_eq!(buf0, data0);

        // Reallocate the memory to a larger size
        let new_index0 = allocator.reallocate(index0, 8, 8);

        // Read and verify the data after reallocation.
        let mut buf1 = [0u8; 4];
        allocator.read(new_index0, 0, 4, buf1.as_mut_ptr());
        assert_eq!(buf1, data0);

        // Append additional data to the reallocated memory.
        let data1 = [0x011u8, 0x13, 0x17, 0x19];
        allocator.write(data1.as_ptr(), new_index0, 4, 4);

        // Read and verify the total data after reallocation.
        let mut buf2 = [0u8; 8];
        allocator.read(new_index0, 0, 8, buf2.as_mut_ptr());
        assert_eq!(buf2[0..4], data0);
        assert_eq!(buf2[4..8], data1);

        // Reallocate the memory to a smaller size.
        let new_index1 = allocator.reallocate(new_index0, 2, 2);

        // Read and verify the data after shrinking.
        let mut buf3 = [0u8; 2];
        allocator.read(new_index1, 0, 2, buf3.as_mut_ptr());
        assert_eq!(buf3, [0x02, 0x03]);

        // Free the memory and check the size.
        allocator.free(new_index1);
    }

    #[test]
    fn test_access_out_of_bounds() {
        // No bounds checking in MiMAllocator,
    }

    #[test]
    fn test_access_freed_memory() {
        // No freed memory detection in MiMAllocator,
    }

    #[test]
    fn test_access_non_existent_memory() {
        // Accessing non-existent memory will cause SIGSEGV in MiMAllocator.
    }

    #[test]
    fn test_indexed_access() {
        let mut allocator = MiMAllocator::new();

        let idx0 = allocator.allocate(8, 8);

        // Write i32 data to the allocated memory.
        {
            let i: i32 = 0x19_17_13_11;
            let data = i.to_le_bytes();
            allocator.write_idx(data.as_ptr(), idx0, 0, 4);
        }

        // Read i32 data from the allocated memory.
        {
            let mut buf = [0u8; 4];
            allocator.read_idx(idx0, 0, 4, buf.as_mut_ptr());
            assert_eq!(i32::from_le_bytes(buf), 0x19_17_13_11)
        }

        // Write i32 data with offset to the allocated memory.
        {
            let i: i32 = 0x37_31_29_23;
            let data = i.to_le_bytes();
            allocator.write_idx(data.as_ptr(), idx0, 4, 4);
        }

        // Read i32 data with offset from the allocated memory.
        {
            let mut buf = [0u8; 4];
            allocator.read_idx(idx0, 4, 4, buf.as_mut_ptr());
            assert_eq!(i32::from_le_bytes(buf), 0x37_31_29_23)
        }

        // Read i64 data from the allocated memory.
        {
            let mut buf = [0u8; 8];
            allocator.read_idx(idx0, 0, 8, buf.as_mut_ptr());
            assert_eq!(i64::from_le_bytes(buf), 0x37_31_29_23_19_17_13_11)
        }

        // Change the size of the memory item.
        let new_idx0 = {
            let enlarge_idx = allocator.reallocate(idx0, 16, 8);

            // Read i64 data from the enlarged memory.
            // The data should be the same as before.
            {
                let mut buf = [0u8; 8];
                allocator.read_idx(enlarge_idx, 0, 8, buf.as_mut_ptr());
                assert_eq!(i64::from_le_bytes(buf), 0x37_31_29_23_19_17_13_11)
            }

            // Write i64 data to the enlarged memory.
            {
                let i: i64 = 0x71_67_61_59_53_47_43_41;
                let data = i.to_le_bytes();
                allocator.write_idx(data.as_ptr(), enlarge_idx, 8, 8);
            }

            // Verify the i64 data in the enlarged memory.
            {
                let mut buf = [0u8; 8];
                allocator.read_idx(enlarge_idx, 8, 8, buf.as_mut_ptr());
                assert_eq!(i64::from_le_bytes(buf), 0x71_67_61_59_53_47_43_41);
                allocator.read_idx(enlarge_idx, 0, 8, buf.as_mut_ptr());
                assert_eq!(i64::from_le_bytes(buf), 0x37_31_29_23_19_17_13_11);
            }

            // Reduce the memory item.
            let reduce_idx = allocator.reallocate(enlarge_idx, 4, 8);

            // Read i32 data from the reduced memory.
            {
                let mut buf = [0u8; 4];
                allocator.read_idx(reduce_idx, 0, 4, buf.as_mut_ptr());
                assert_eq!(i32::from_le_bytes(buf), 0x19_17_13_11);
            }

            reduce_idx
        };

        // Create a new memory item
        let idx1 = allocator.allocate(8, 8);

        // Write i32 data to the new memory item.
        {
            let i: i32 = 0x07_05_03_02;
            let data = i.to_le_bytes();
            allocator.write_idx(data.as_ptr(), idx1, 0, 4);
        }

        // Read i32 data from the new memory item.
        {
            let mut buf = [0u8; 4];
            allocator.read_idx(idx1, 0, 4, buf.as_mut_ptr());
            assert_eq!(i32::from_le_bytes(buf), 0x07_05_03_02)
        }

        // Check the previous memory item.
        {
            let mut buf = [0u8; 4];
            allocator.read_idx(new_idx0, 0, 4, buf.as_mut_ptr());
            assert_eq!(i32::from_le_bytes(buf), 0x19_17_13_11)
        }

        // Free the memory items.
        allocator.free(idx1);
        allocator.free(new_idx0);
    }
}
