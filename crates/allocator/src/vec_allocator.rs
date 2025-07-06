// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_memory::{indexed_memory_access::IndexedMemoryAccess, memory_access::MemoryAccess};

use crate::allocator::Allocator;

/// A simple allocator implemented using a vector to store memory items.
/// For debugging and testing purposes.
pub struct VecAllocator {
    // A collection of memory items, where each item is either allocated
    // (Some) or freed (None).
    items: Vec<Option<MemoryItem>>,
}

impl VecAllocator {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}

impl Default for VecAllocator {
    fn default() -> Self {
        Self::new()
    }
}

struct MemoryItem {
    // The logical size of the memory item.
    //
    // When a user requests to shrink a memory item (e.g., via reallocation),
    // the logical size may be reduced. However, for performance reasons,
    // the underlying allocated memory is not actually resized.
    // As a result, the logical size of the memory item may be smaller than
    // the physical size of the allocated memory buffer.
    size: usize,

    // The data and capacity of the memory item.
    data: Vec<u8>,
}

impl MemoryItem {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            data: vec![0u8; size],
        }
    }
}

impl Allocator for VecAllocator {
    fn allocate(&mut self, size_in_bytes: usize, _alignment_in_bytes: usize) -> usize {
        // Search for an empty slot in the collection.
        let pos = self.items.iter().position(|item| item.is_none());

        if let Some(pos) = pos {
            // Reuse the empty slot by inserting a new memory item.
            self.items[pos] = Some(MemoryItem::new(size_in_bytes));
            pos
        } else {
            // If no empty slot is found, append a new memory item to the collection.
            let len = self.items.len();
            self.items.push(Some(MemoryItem::new(size_in_bytes)));
            len
        }
    }

    fn reallocate(
        &mut self,
        data_internal_index: usize,
        new_size_in_bytes: usize,
        _alignment_in_bytes: usize,
    ) -> usize {
        if let Some(item) = self.items.get_mut(data_internal_index) {
            if let Some(last_item) = item {
                if new_size_in_bytes > last_item.size {
                    // Replace the memory item with a new one if the new size is larger.
                    let mut new_item = MemoryItem::new(new_size_in_bytes);
                    new_item.data[0..last_item.size]
                        .copy_from_slice(&last_item.data[0..last_item.size]);
                    *last_item = new_item;
                } else {
                    // If the new size is smaller, just update the size.
                    last_item.size = new_size_in_bytes;
                }
                data_internal_index
            } else {
                panic!(
                    "Attempted to access a freed memory item. Index: {}",
                    data_internal_index
                );
            }
        } else {
            panic!(
                "Invalid index for accessing memory. Index: {}",
                data_internal_index
            );
        }
    }

    fn free(&mut self, data_internal_index: usize) {
        // Mark the memory item at the specified index as freed.
        self.items[data_internal_index] = None;
    }
}

impl MemoryAccess for VecAllocator {
    fn get_ptr(&self, address: usize, offset_in_bytes: usize) -> *const u8 {
        // Get a constant pointer to the data of the memory item at the specified address.
        unsafe {
            self.items[address]
                .as_ref()
                .unwrap()
                .data
                .as_ptr()
                .add(offset_in_bytes)
        }
    }

    fn get_mut_ptr(&mut self, address: usize, offset_in_bytes: usize) -> *mut u8 {
        // Get a mutable pointer to the data of the memory item at the specified address.
        unsafe {
            self.items[address]
                .as_mut()
                .unwrap()
                .data
                .as_mut_ptr()
                .add(offset_in_bytes)
        }
    }
}

impl IndexedMemoryAccess for VecAllocator {
    fn get_start_address_by_index(&self, idx: usize) -> usize {
        if let Some(opt_item) = self.items.get(idx) {
            if opt_item.is_some() {
                // Return the index
                idx
            } else {
                panic!("Attempted to access a freed memory item. Index: {}", idx);
            }
        } else {
            panic!("Invalid index for accessing memory. Index: {}", idx);
        }
    }

    fn get_data_length(&self, _idx: usize) -> usize {
        panic!("VecAllocator does not support data length retrieval");
    }
}

#[cfg(test)]
mod tests {
    use anc_memory::{indexed_memory_access::IndexedMemoryAccess, memory_access::MemoryAccess};

    use crate::{allocator::Allocator, vec_allocator::VecAllocator};

    #[test]
    fn test_allocat_reallocate_and_free() {
        let mut allocator = VecAllocator::new();

        let data0 = [0x02u8, 0x03, 0x05, 0x07];

        // Allocate memory
        let index0 = allocator.allocate(4, 1);

        // Write data to the allocated memory.
        allocator.write(data0.as_ptr(), index0, 0, 4);

        // Read and verify the data.
        let mut buf0 = [0u8; 4];
        allocator.read(index0, 0, 4, buf0.as_mut_ptr());
        assert_eq!(buf0, data0);

        // Reallocate the memory to a larger size
        let new_index0 = allocator.reallocate(index0, 8, 1);

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
        let new_index1 = allocator.reallocate(new_index0, 2, 1);

        // Read and verify the data after shrinking.
        let mut buf3 = [0u8; 2];
        allocator.read(new_index1, 0, 2, buf3.as_mut_ptr());
        assert_eq!(buf3, [0x02, 0x03]);

        // Free the memory and check the size.
        allocator.free(new_index1);
    }

    #[test]
    fn test_access_out_of_bounds() {
        // No bounds checking in SimpleAllocator,
    }

    #[test]
    fn test_access_freed_memory() {
        let mut allocator = VecAllocator::new();

        // Allocate memory and check the size.
        let index = allocator.allocate(8, 8);

        // Free the memory and check the size.
        allocator.free(index);

        // Access the freed memory
        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let mut buf = [0u8; 8];
            // should panic
            allocator.read(index, 0, 8, buf.as_mut_ptr());
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_access_non_existent_memory() {
        let allocator = VecAllocator::new();
        let non_existent_index = 1001; // Non-existent index

        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let mut buf = [0u8; 8];
            // should panic
            allocator.read(non_existent_index, 0, 8, buf.as_mut_ptr());
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_indexed_access() {
        let mut allocator = VecAllocator::new();

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
