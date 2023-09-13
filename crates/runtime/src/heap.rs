// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

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
        (&self.data[address..]).as_ptr()
    }

    #[inline]
    fn get_mut_ptr(&mut self, address: usize) -> *mut u8 {
        (&mut self.data[address..]).as_mut_ptr()
    }
}

impl ResizeableMemory for Heap {
    fn get_capacity_in_pages(&self) -> usize {
        self.data.len() / MEMORY_PAGE_SIZE_IN_BYTES
    }

    fn resize(&mut self, new_size_in_pages: usize) {
        let new_len = new_size_in_pages * MEMORY_PAGE_SIZE_IN_BYTES;
        self.data.resize(new_len, 0);
    }
}

// impl HostAccessableMemory for Heap {
//     #[inline]
//     fn get_host_address(&self, offset: usize) -> usize {
//         (&self.data[offset..]).as_ptr() as usize
//     }
// }
