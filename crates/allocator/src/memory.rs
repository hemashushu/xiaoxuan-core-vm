// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use crate::{memory_access::MemoryAccess, resizeable_memory::ResizeableMemory, MEMORY_PAGE_SIZE_IN_BYTES};

pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new(init_size_in_pages: usize) -> Self {
        let len = init_size_in_pages * MEMORY_PAGE_SIZE_IN_BYTES;
        let data: Vec<u8> = vec![0u8; len];
        Self { data }
    }
}

impl MemoryAccess for Memory {
    #[inline]
    fn get_ptr(&self, address: usize) -> *const u8 {
        self.data[address..].as_ptr()
    }

    #[inline]
    fn get_mut_ptr(&mut self, address: usize) -> *mut u8 {
        self.data[address..].as_mut_ptr()
    }
}

impl ResizeableMemory for Memory {
    fn get_capacity_in_pages(&self) -> usize {
        self.data.len() / MEMORY_PAGE_SIZE_IN_BYTES
    }

    fn resize(&mut self, new_size_in_pages: usize) -> usize {
        let new_len = new_size_in_pages * MEMORY_PAGE_SIZE_IN_BYTES;
        self.data.resize(new_len, 0);
        new_size_in_pages
    }
}

impl Memory {
    pub fn fill(&mut self, address: usize, value: u8, count: usize) {
        self.data[address..(address + count)].fill(value);
    }

    pub fn copy(&mut self, dst_address: usize, src_address: usize, count: usize) {
        let (src, dst) = self.data.split_at_mut(dst_address);

        // depending on the location of src_offset and dst_offset, there are 2 situations:
        //
        // index: 0 1 2 3 4 5 | 6 7 8 9
        //            ^       | ^
        //            src     | dst
        //
        // index: 0 1 2 3   | 4 5 6 7 8 9
        //            ^     |     ^
        //            dst   |     src (the value of index 'src' has been changed)

        if src_address < dst_address {
            dst.copy_from_slice(&src[src_address..(src_address + count)]);
        } else {
            let offset = src_address - dst_address;
            dst.copy_from_slice(&src[offset..(offset + count)]);
        }
    }

    pub fn load_data(&self, address: usize, count: usize) -> &[u8] {
        &self.data[address..(address + count)]
    }
}
