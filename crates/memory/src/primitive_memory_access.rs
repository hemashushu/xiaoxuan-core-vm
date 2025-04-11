// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use crate::{memory_access::MemoryAccess, MemoryError, MemoryErrorType};

/// Read/write primitive data from/to memory.
pub trait PrimitiveMemoryAccess: MemoryAccess {
    fn read_primitive_i64_s(&self, address: usize, offset: usize) -> i64 {
        let tp = self.get_ptr(address, offset) as *const i64;
        unsafe { std::ptr::read(tp) }
    }

    fn read_primitive_i64_u(&self, address: usize, offset: usize) -> u64 {
        let tp = self.get_ptr(address, offset) as *const u64;
        unsafe { std::ptr::read(tp) }
    }

    fn read_primitive_i32_s(&self, address: usize, offset: usize) -> i32 {
        let tp = self.get_ptr(address, offset) as *const i32;
        unsafe { std::ptr::read(tp) }
    }

    fn read_primitive_i32_u(&self, address: usize, offset: usize) -> u32 {
        let tp = self.get_ptr(address, offset) as *const u32;
        unsafe { std::ptr::read(tp) }
    }

    // load 64-bit floating-point with validation check.
    // the VM does support some IEEE 754 variants, for more details, see the ISA document.
    fn read_primitive_f64(&self, address: usize, offset: usize) -> Result<f64, MemoryError> {
        let tp = self.get_ptr(address, offset) as *const f64;
        let val = unsafe { std::ptr::read(tp) };
        if val.is_nan() || val.is_infinite() {
            // NaN, +Inf, -Inf
            Err(MemoryError::new(
                MemoryErrorType::UnsupportedFloatingPointVariants,
            ))
        } else {
            Ok(val)
        }
    }

    // load 32-bit floating-point with validation check.
    // the VM does support some IEEE 754 variants, for more details, see the ISA document.
    fn read_primitive_f32(&self, address: usize, offset: usize) -> Result<f32, MemoryError> {
        let tp = self.get_ptr(address, offset) as *const f32;
        let val = unsafe { std::ptr::read(tp) };
        if val.is_nan() || val.is_infinite() {
            // NaN, +Inf, -Inf
            Err(MemoryError::new(
                MemoryErrorType::UnsupportedFloatingPointVariants,
            ))
        } else {
            Ok(val)
        }
    }

    fn write_primitive_i64_s(&mut self, address: usize, offset: usize, value: i64) {
        let tp = self.get_mut_ptr(address, offset) as *mut i64;
        unsafe { std::ptr::write(tp, value) }
    }

    fn write_primitive_i64_u(&mut self, address: usize, offset: usize, value: u64) {
        let tp = self.get_mut_ptr(address, offset) as *mut u64;
        unsafe { std::ptr::write(tp, value) }
    }

    fn write_primitive_i32_s(&mut self, address: usize, offset: usize, value: i32) {
        let tp = self.get_mut_ptr(address, offset) as *mut i32;
        unsafe { std::ptr::write(tp, value) }
    }

    fn write_primitive_i32_u(&mut self, address: usize, offset: usize, value: u32) {
        let tp = self.get_mut_ptr(address, offset) as *mut u32;
        unsafe { std::ptr::write(tp, value) }
    }

    fn write_primitive_f64(&mut self, address: usize, offset: usize, value: f64) {
        let tp = self.get_mut_ptr(address, offset) as *mut f64;
        unsafe { std::ptr::write(tp, value) }
    }

    fn write_primitive_f32(&mut self, address: usize, offset: usize, value: f32) {
        let tp = self.get_mut_ptr(address, offset) as *mut f32;
        unsafe { std::ptr::write(tp, value) }
    }
}
