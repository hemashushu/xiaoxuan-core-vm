// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

pub trait MemoryAccess {
    // Returns a constant pointer to the memory at the specified address.
    fn get_ptr(&self, address: usize) -> *const u8;

    // Returns a mutable pointer to the memory at the specified address.
    fn get_mut_ptr(&mut self, address: usize) -> *mut u8;

    // Copies a block of memory from the source address to the destination pointer.
    // `length_in_bytes` specifies the number of bytes to copy.
    #[inline]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn read(&self, src_address: usize, dst_ptr: *mut u8, length_in_bytes: usize) {
        let src = self.get_ptr(src_address);
        unsafe {
            std::ptr::copy(src, dst_ptr, length_in_bytes);
        }
    }

    // Reads an i64 value from the source address and writes it to the destination pointer.
    fn read_i64(&self, src_address: usize, dst_ptr: *mut u8) {
        self.read(src_address, dst_ptr, 8);
    }

    // Reads a signed i32 value from the source address, extends it to i64, and writes it to the destination pointer.
    fn read_i32_s(&self, src_address: usize, dst_ptr: *mut u8) {
        let tp_src = self.get_ptr(src_address) as *const i32;
        unsafe {
            let val_64 = std::ptr::read(tp_src) as i64;
            let dst_ptr_64 = dst_ptr as *mut i64;
            std::ptr::write(dst_ptr_64, val_64);
        }
    }

    // Reads an unsigned i32 value from the source address, extends it to u64, and writes it to the destination pointer.
    fn read_i32_u(&self, src_address: usize, dst_ptr: *mut u8) {
        let tp_src = self.get_ptr(src_address) as *const u32;
        unsafe {
            let val_64 = std::ptr::read(tp_src) as u64;
            let dst_ptr_64 = dst_ptr as *mut u64;
            std::ptr::write(dst_ptr_64, val_64);
        }
    }

    // Reads a signed i16 value from the source address, extends it to i64, and writes it to the destination pointer.
    fn read_i16_s(&self, src_address: usize, dst_ptr: *mut u8) {
        let tp_src = self.get_ptr(src_address) as *const i16;
        unsafe {
            let val_64 = std::ptr::read(tp_src) as i64;
            let dst_ptr_64 = dst_ptr as *mut i64;
            std::ptr::write(dst_ptr_64, val_64);
        }
    }

    // Reads an unsigned i16 value from the source address, extends it to u64, and writes it to the destination pointer.
    fn read_i16_u(&self, src_address: usize, dst_ptr: *mut u8) {
        let tp_src = self.get_ptr(src_address) as *const u16;
        unsafe {
            let val_64 = std::ptr::read(tp_src) as u64;
            let dst_ptr_64 = dst_ptr as *mut u64;
            std::ptr::write(dst_ptr_64, val_64);
        }
    }

    // Reads a signed i8 value from the source address, extends it to i64, and writes it to the destination pointer.
    fn read_i8_s(&self, src_address: usize, dst_ptr: *mut u8) {
        let tp_src = self.get_ptr(src_address) as *const i8;
        unsafe {
            let val_64 = std::ptr::read(tp_src) as i64;
            let dst_ptr_64 = dst_ptr as *mut i64;
            std::ptr::write(dst_ptr_64, val_64);
        }
    }

    // Reads an unsigned i8 value from the source address, extends it to u64, and writes it to the destination pointer.
    fn read_i8_u(&self, src_address: usize, dst_ptr: *mut u8) {
        let tp_src = self.get_ptr(src_address);
        unsafe {
            let val_64 = std::ptr::read(tp_src) as u64;
            let dst_ptr_64 = dst_ptr as *mut u64;
            std::ptr::write(dst_ptr_64, val_64);
        }
    }

    // Reads a 64-bit floating-point value from the source address, validates it, and writes it to the destination pointer.
    // Returns Ok if the value is valid (normal, subnormal, or zero), otherwise Err.
    fn read_f64(&self, src_address: usize, dst_ptr: *mut u8) -> Result<(), ()> {
        let tp = self.get_ptr(src_address) as *const f64;
        let val = unsafe { std::ptr::read(tp) };
        if val.is_nan() || val.is_infinite() {
            // NaN, +Inf, -Inf
            Err(())
        } else {
            self.read_i64(src_address, dst_ptr);
            Ok(())
        }
    }

    // Reads a 32-bit floating-point value from the source address, validates it, and writes it to the destination pointer.
    // Returns true if the value is valid (normal, subnormal, or zero), otherwise false.
    fn read_f32(&self, src_addr: usize, dst_ptr: *mut u8) -> Result<(), ()> {
        let tp = self.get_ptr(src_addr) as *const f32;
        let val = unsafe { std::ptr::read(tp) };
        if val.is_nan() || val.is_infinite() {
            // NaN, +Inf, -Inf
            Err(())
        } else {
            self.read_i32_u(src_addr, dst_ptr);
            Ok(())
        }
    }

    // Copies a block of memory from the source pointer to the destination address.
    // `length_in_bytes` specifies the number of bytes to copy.
    #[inline]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn write(&mut self, src_ptr: *const u8, dst_address: usize, length_in_bytes: usize) {
        let dst = self.get_mut_ptr(dst_address);
        unsafe {
            std::ptr::copy(src_ptr, dst, length_in_bytes);
        }
    }

    // Writes an i64 value from the source pointer to the destination address.
    fn write_i64(&mut self, src_ptr: *const u8, dst_address: usize) {
        self.write(src_ptr, dst_address, 8);
    }

    // Writes an i32 value from the source pointer to the destination address.
    fn write_i32(&mut self, src_ptr: *const u8, dst_address: usize) {
        self.write(src_ptr, dst_address, 4);
    }

    // Writes an i16 value from the source pointer to the destination address.
    fn write_i16(&mut self, src_ptr: *const u8, dst_address: usize) {
        self.write(src_ptr, dst_address, 2);
    }

    // Writes an i8 value from the source pointer to the destination address.
    fn write_i8(&mut self, src_ptr: *const u8, dst_address: usize) {
        self.write(src_ptr, dst_address, 1);
    }
}
