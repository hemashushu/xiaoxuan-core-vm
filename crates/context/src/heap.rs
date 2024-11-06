// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use crate::{memory::Memory, resizeable_memory::ResizeableMemory, MEMORY_PAGE_SIZE_IN_BYTES};

pub struct Heap {
    data: Vec<u8>,
}

impl Heap {
    pub fn new(init_size_in_pages: usize) -> Self {
        let len = init_size_in_pages * MEMORY_PAGE_SIZE_IN_BYTES;
        let data: Vec<u8> = vec![0u8; len];
        Self { data }
    }
}

impl Memory for Heap {
    #[inline]
    fn get_ptr(&self, address: usize) -> *const u8 {
        self.data[address..].as_ptr()
    }

    #[inline]
    fn get_mut_ptr(&mut self, address: usize) -> *mut u8 {
        self.data[address..].as_mut_ptr()
    }
}

impl ResizeableMemory for Heap {
    fn get_capacity_in_pages(&self) -> usize {
        self.data.len() / MEMORY_PAGE_SIZE_IN_BYTES
    }

    fn resize(&mut self, new_size_in_pages: usize) -> usize {
        let new_len = new_size_in_pages * MEMORY_PAGE_SIZE_IN_BYTES;
        self.data.resize(new_len, 0);
        new_size_in_pages
    }
}

impl Heap {
    pub fn fill(&mut self, address: usize, value: u8, count: usize) {
        self.data[address..(address + count)].fill(value);
    }

    pub fn copy(&mut self, dst_address: usize, src_address: usize, length_in_bytes: usize) {
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
            dst.copy_from_slice(&src[src_address..(src_address + length_in_bytes)]);
        } else {
            let offset = src_address - dst_address;
            dst.copy_from_slice(&src[offset..(offset + length_in_bytes)]);
        }
    }

    pub fn load_data(&self, address: usize, count: usize) -> &[u8] {
        &self.data[address..(address + count)]
    }
}
