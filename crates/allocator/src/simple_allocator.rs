// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_memory::{indexed_memory_access::IndexedMemoryAccess, memory_access::MemoryAccess};

use crate::allocator::Allocator;

struct MemoryItem {
    // The actual size of the memory item.
    //
    // In some cases, the memory item may be reduced to a smaller size
    // after the user resizes it. However, for performance reasons, the
    // allocated memory is not reallocated. As a result, the size of the
    // memory item may be smaller than the allocated memory.
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

pub struct SimpleAllocator {
    // A collection of memory items, where each item is either allocated
    // (Some) or freed (None).
    items: Vec<Option<MemoryItem>>,
}

impl SimpleAllocator {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}

impl Allocator for SimpleAllocator {
    fn allocate(&mut self, _align_in_bytes: usize, size_in_bytes: usize) -> usize {
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

    fn resize(&mut self, data_public_index: usize, new_size_in_bytes: usize) -> usize {
        if let Some(item) = self.items.get_mut(data_public_index) {
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
                data_public_index
            } else {
                panic!(
                    "Attempted to access a freed memory item. Index: {}",
                    data_public_index
                );
            }
        } else {
            panic!(
                "Invalid index for accessing memory. Index: {}",
                data_public_index
            );
        }
    }

    fn get_size(&self, data_public_index: usize) -> Option<usize> {
        // Retrieve the size of the memory item at the specified index, if it exists.
        self.items
            .get(data_public_index)
            .and_then(|item| item.as_ref().map(|item| item.size))
    }

    fn free(&mut self, data_public_index: usize) {
        // Mark the memory item at the specified index as freed.
        self.items[data_public_index] = None;
    }
}

impl MemoryAccess for SimpleAllocator {
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

impl IndexedMemoryAccess for SimpleAllocator {
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

    fn get_data_length(&self, idx: usize) -> usize {
        if let Some(opt_item) = self.items.get(idx) {
            if let Some(item) = opt_item {
                item.size
            } else {
                panic!("Attempted to access a freed memory item. Index: {}", idx);
            }
        } else {
            panic!("Invalid index for accessing memory. Index: {}", idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::Allocator;

    #[test]
    fn test_allocat_resize_and_free() {
        let mut allocator = SimpleAllocator::new();

        // Allocate memory and check the size.
        let index = allocator.allocate(8, 16);
        assert_eq!(allocator.get_size(index), Some(16));

        // Enlarge the memory and check the size.
        let new_index0 = allocator.resize(index, 32);
        assert_eq!(allocator.get_size(new_index0), Some(32));

        // Reduce the memory and check the size.
        let new_index1 = allocator.resize(new_index0, 24);
        assert_eq!(allocator.get_size(new_index1), Some(24));

        // Free the memory and check the size.
        allocator.free(new_index1);
        assert_eq!(allocator.get_size(new_index1), None);
    }

    #[test]
    fn test_access_freed_memory() {
        let mut allocator = SimpleAllocator::new();

        // Allocate memory and check the size.
        let index = allocator.allocate(8, 8);
        assert_eq!(allocator.get_size(index), Some(8));

        // Free the memory and check the size.
        allocator.free(index);
        assert_eq!(allocator.get_size(index), None);

        // Access the freed memory
        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let mut buf = [0u8; 8];
            // should panic
            allocator.read_idx_i64(index, 0, buf.as_mut_ptr());
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_access_non_existent_memory() {
        let allocator = SimpleAllocator::new();

        let prev_hook = std::panic::take_hook(); // silent panic
        std::panic::set_hook(Box::new(|_| {}));

        let result = std::panic::catch_unwind(move || {
            let mut buf = [0u8; 8];
            // should panic
            allocator.read_idx_i64(99, 0, buf.as_mut_ptr());
        });

        std::panic::set_hook(prev_hook);

        assert!(result.is_err());
    }

    #[test]
    fn test_indexed_access() {
        let mut allocator = SimpleAllocator::new();

        let idx0 = allocator.allocate(8, 8);

        // Write i32 data to the allocated memory.
        {
            let i: i32 = 0x19_17_13_11;
            let data = i.to_le_bytes();
            allocator.write_idx_i32(data.as_ptr(), idx0, 0);
        }

        // Read i32 data from the allocated memory.
        {
            let mut buf = [0u8; 4];
            allocator.read_idx_i32_u(idx0, 0, buf.as_mut_ptr());
            assert_eq!(i32::from_le_bytes(buf), 0x19_17_13_11)
        }

        // Write i32 data with offset to the allocated memory.
        {
            let i: i32 = 0x37_31_29_23;
            let data = i.to_le_bytes();
            allocator.write_idx_i32(data.as_ptr(), idx0, 4);
        }

        // Read i32 data with offset from the allocated memory.
        {
            let mut buf = [0u8; 4];
            allocator.read_idx_i32_u(idx0, 4, buf.as_mut_ptr());
            assert_eq!(i32::from_le_bytes(buf), 0x37_31_29_23)
        }

        // Read i64 data from the allocated memory.
        {
            let mut buf = [0u8; 8];
            allocator.read_idx_i64(idx0, 0, buf.as_mut_ptr());
            assert_eq!(i64::from_le_bytes(buf), 0x37_31_29_23_19_17_13_11)
        }

        // Enlarge the memory item and then reduce it.
        let idx1 = {
            let enlarge_idx = allocator.resize(idx0, 16);
            assert_eq!(allocator.get_size(enlarge_idx), Some(16));

            // Read i64 data from the enlarged memory.
            // The data should be the same as before.
            {
                let mut buf = [0u8; 8];
                allocator.read_idx_i64(enlarge_idx, 0, buf.as_mut_ptr());
                assert_eq!(i64::from_le_bytes(buf), 0x37_31_29_23_19_17_13_11)
            }

            // Write i64 data to the enlarged memory.
            {
                let i: i64 = 0x71_67_61_59_53_47_43_41;
                let data = i.to_le_bytes();
                allocator.write_idx_i64(data.as_ptr(), enlarge_idx, 8);
            }

            // Verify the i64 data in the enlarged memory.
            {
                let mut buf = [0u8; 8];
                allocator.read_idx_i64(enlarge_idx, 8, buf.as_mut_ptr());
                assert_eq!(i64::from_le_bytes(buf), 0x71_67_61_59_53_47_43_41);
                allocator.read_idx_i64(enlarge_idx, 0, buf.as_mut_ptr());
                assert_eq!(i64::from_le_bytes(buf), 0x37_31_29_23_19_17_13_11);
            }

            // Reduce the memory item and check the size.
            let reduce_idx = allocator.resize(enlarge_idx, 4);

            // Read i32 data from the reduced memory.
            {
                let mut buf = [0u8; 4];
                allocator.read_idx_i32_u(reduce_idx, 0, buf.as_mut_ptr());
                assert_eq!(i32::from_le_bytes(buf), 0x19_17_13_11);
            }

            reduce_idx
        };

        // Create a new memory item and check the size.
        let idx2 = allocator.allocate(8, 8);
        assert_eq!(allocator.get_size(idx2), Some(8));

        // Write i32 data to the new memory item.
        {
            let i: i32 = 0x07_05_03_02;
            let data = i.to_le_bytes();
            allocator.write_idx_i32(data.as_ptr(), idx2, 0);
        }

        // Read i32 data from the new memory item.
        {
            let mut buf = [0u8; 4];
            allocator.read_idx_i32_u(idx2, 0, buf.as_mut_ptr());
            assert_eq!(i32::from_le_bytes(buf), 0x07_05_03_02)
        }

        // Check the previous memory item.
        {
            assert_eq!(allocator.get_size(idx1), Some(4));

            let mut buf = [0u8; 4];
            allocator.read_idx_i32_u(idx1, 0, buf.as_mut_ptr());
            assert_eq!(i32::from_le_bytes(buf), 0x19_17_13_11)
        }
    }
}
