// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// in XiaoXuan VM, there are several objects belong to the memory class,
// such as the thread-local data sections, stack, thread-local heap
// and the shared-memory.

// pub struct Memory {
//     data: Vec<u8>,
// }
//
// impl Memory {
//     pub fn get_slice(&self, offset: usize) -> &[u8] {
//         &self.data[offset..]
//     }
//
//     pub fn get_mut_slice(&mut self, offset: usize) -> &mut [u8] {
//         &mut self.data[offset..]
//     }
// }

pub trait Memory {
    // it's recommended that add annotation "#[inline]" to the implementation
    fn get_ptr(&self, addr: usize) -> *const u8;

    // it's recommended that add annotation "#[inline]" to the implementation
    fn get_mut_ptr(&mut self, addr: usize) -> *mut u8;

    /// read i32,i64,f32,f64 and so on
    fn read<T>(&self, addr: usize) -> T {
        let tp = self.get_ptr(addr) as *const T;
        unsafe { std::ptr::read(tp) }
    }

    /// write i32,i64,f32,f64 and so on
    fn write<T>(&mut self, addr: usize, value: T) {
        let tp = self.get_mut_ptr(addr) as *mut T;
        unsafe { std::ptr::write(tp, value) }
    }

    fn fill(&mut self, addr: usize, value: u8, length_in_bytes: usize) {
        let dst = self.get_mut_ptr(addr);
        unsafe { std::ptr::write_bytes(dst, value, length_in_bytes) }
    }

    fn copy_to(&self, addr: usize, dst: *mut u8, length_in_bytes: usize) {
        let src = self.get_ptr(addr);
        unsafe { std::ptr::copy(src, dst, length_in_bytes) }
    }

    fn copy_from(&mut self, addr: usize, src: *const u8, length_in_bytes: usize) {
        let dst = self.get_mut_ptr(addr);
        unsafe { std::ptr::copy(src, dst, length_in_bytes) }
    }
}
