// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use crate::memory_access::MemoryAccess;

/// Read/write primitive data from/to memory.
pub trait TypedMemoryAccess: MemoryAccess {
    fn read_primitive_i64_s(&self, address: usize) -> i64 {
        let tp = self.get_ptr(address) as *const i64;
        unsafe { std::ptr::read(tp) }
    }

    fn read_primitive_i64_u(&self, address: usize) -> u64 {
        let tp = self.get_ptr(address) as *const u64;
        unsafe { std::ptr::read(tp) }
    }

    fn read_primitive_i32_s(&self, address: usize) -> i32 {
        let tp = self.get_ptr(address) as *const i32;
        unsafe { std::ptr::read(tp) }
    }

    fn read_primitive_i32_u(&self, address: usize) -> u32 {
        let tp = self.get_ptr(address) as *const u32;
        unsafe { std::ptr::read(tp) }
    }

    // load 64-bit floating-point with validation check.
    // the VM does support some IEEE 754 variants, for more details, see the ISA document.
    fn read_primitive_f64(&self, address: usize) -> Result<f64, ()> {
        let tp = self.get_ptr(address) as *const f64;
        let val = unsafe { std::ptr::read(tp) };
        if val.is_normal() || val.is_subnormal() || val == 0.0f64 {
            Ok(val)
        } else {
            Err(())
        }
    }

    // load 32-bit floating-point with validation check.
    // the VM does support some IEEE 754 variants, for more details, see the ISA document.
    fn read_primitive_f32(&self, address: usize) -> Result<f32, ()> {
        let tp = self.get_ptr(address) as *const f32;
        let val = unsafe { std::ptr::read(tp) };
        if val.is_normal() || val.is_subnormal() || val == 0.0f32 {
            Ok(val)
        } else {
            Err(())
        }
    }

    fn write_primitive_i64_s(&mut self, address: usize, value: i64) {
        let tp = self.get_mut_ptr(address) as *mut i64;
        unsafe { std::ptr::write(tp, value) }
    }

    fn write_primitive_i64_u(&mut self, address: usize, value: u64) {
        let tp = self.get_mut_ptr(address) as *mut u64;
        unsafe { std::ptr::write(tp, value) }
    }

    fn write_primitive_i32_s(&mut self, address: usize, value: i32) {
        let tp = self.get_mut_ptr(address) as *mut i32;
        unsafe { std::ptr::write(tp, value) }
    }

    fn write_primitive_i32_u(&mut self, address: usize, value: u32) {
        let tp = self.get_mut_ptr(address) as *mut u32;
        unsafe { std::ptr::write(tp, value) }
    }

    fn write_primitive_f64(&mut self, address: usize, value: f64) {
        let tp = self.get_mut_ptr(address) as *mut f64;
        unsafe { std::ptr::write(tp, value) }
    }

    fn write_primitive_f32(&mut self, address: usize, value: f32) {
        let tp = self.get_mut_ptr(address) as *mut f32;
        unsafe { std::ptr::write(tp, value) }
    }
}
