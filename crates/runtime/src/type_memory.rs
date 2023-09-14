// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::memory::Memory;

/// memory with primitive data type support, it's used for
/// interactive with the VM.
///
/// in XiaoXuan VM, only the Stack implement this trait.
pub trait TypeMemory: Memory {
    fn read_i32_s(&self, address: usize) -> i32 {
        let tp = self.get_ptr(address) as *const i32;
        unsafe { std::ptr::read(tp) }
    }

    fn read_i32_u(&self, address: usize) -> u32 {
        let tp = self.get_ptr(address) as *const u32;
        unsafe { std::ptr::read(tp) }
    }

    fn read_i64_s(&self, address: usize) -> i64 {
        let tp = self.get_ptr(address) as *const i64;
        unsafe { std::ptr::read(tp) }
    }

    fn read_i64_u(&self, address: usize) -> u64 {
        let tp = self.get_ptr(address) as *const u64;
        unsafe { std::ptr::read(tp) }
    }

    fn read_f32(&self, address: usize) -> f32 {
        let tp = self.get_ptr(address) as *const f32;
        unsafe { std::ptr::read(tp) }
    }

    fn read_f64(&self, address: usize) -> f64 {
        let tp = self.get_ptr(address) as *const f64;
        unsafe { std::ptr::read(tp) }
    }

    fn write_i32_s(&mut self, address: usize, value: i32) {
        let tp = self.get_mut_ptr(address) as *mut i32;
        unsafe { std::ptr::write(tp, value) }
    }

    // although unsigned-integers and signed-integers are stored in the
    // same way in memory, two different naming functions are still provided
    // here for the name consisstency.
    fn write_i32_u(&mut self, address: usize, value: u32) {
        let tp = self.get_mut_ptr(address) as *mut u32;
        unsafe { std::ptr::write(tp, value) }
    }

    fn write_i64_s(&mut self, address: usize, value: i64) {
        let tp = self.get_mut_ptr(address) as *mut i64;
        unsafe { std::ptr::write(tp, value) }
    }

    fn write_i64_u(&mut self, address: usize, value: u64) {
        let tp = self.get_mut_ptr(address) as *mut u64;
        unsafe { std::ptr::write(tp, value) }
    }

    fn write_f32(&mut self, address: usize, value: f32) {
        let tp = self.get_mut_ptr(address) as *mut f32;
        unsafe { std::ptr::write(tp, value) }
    }

    fn write_f64(&mut self, address: usize, value: f64) {
        let tp = self.get_mut_ptr(address) as *mut f64;
        unsafe { std::ptr::write(tp, value) }
    }
}
