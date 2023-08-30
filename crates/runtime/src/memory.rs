// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

/// in XiaoXuan VM, there are several objects belong to the memory class,
/// such as the thread-local data sections, stack, thread-local heap
/// and the shared-memory.
pub trait Memory {
    // it's recommended that add annotation "#[inline]" to the implementation
    fn get_ptr(&self, addr: usize) -> *const u8;

    // it's recommended that add annotation "#[inline]" to the implementation
    fn get_mut_ptr(&mut self, addr: usize) -> *mut u8;

    fn read_i32(&self, addr: usize) -> i32 {
        let tp = self.get_ptr(addr) as *const i32;
        unsafe { std::ptr::read(tp) }
    }

    fn read_i64(&self, addr: usize) -> i64 {
        let tp = self.get_ptr(addr) as *const i64;
        unsafe { std::ptr::read(tp) }
    }

    fn read_f32(&self, addr: usize) -> f32 {
        let tp = self.get_ptr(addr) as *const f32;
        unsafe { std::ptr::read(tp) }
    }

    fn read_f64(&self, addr: usize) -> f64 {
        let tp = self.get_ptr(addr) as *const f64;
        unsafe { std::ptr::read(tp) }
    }

    fn write_i32(&mut self, addr: usize, value: i32) {
        let tp = self.get_mut_ptr(addr) as *mut i32;
        unsafe { std::ptr::write(tp, value) }
    }

    fn write_i64(&mut self, addr: usize, value: i64) {
        let tp = self.get_mut_ptr(addr) as *mut i64;
        unsafe { std::ptr::write(tp, value) }
    }

    fn write_f32(&mut self, addr: usize, value: f32) {
        let tp = self.get_mut_ptr(addr) as *mut f32;
        unsafe { std::ptr::write(tp, value) }
    }

    fn write_f64(&mut self, addr: usize, value: f64) {
        let tp = self.get_mut_ptr(addr) as *mut f64;
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
