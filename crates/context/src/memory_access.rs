// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

/// in XiaoXuan Core VM, there are several objects belong to the memory class,
/// includes the local variable area, (thread-local) data sections, stack,
/// (thread-local) memory. this trait provides the ability of data loading and
/// storing in memory, but does not include the primitive data reading and writing.
///
/// |---------------|
/// | ReadOnlyData  |
/// | ReadWriteData | ----\
/// | UninitData    |     |
/// |---------------|     |
///                       |               |-------|         |------|
/// |---------------|     |     load      |       |   pop   |      |
/// | memory        | ----| ------------> | Stack | ------> |  VM  |
/// |---------------|     | <------------ |       | <------ |      |
///                       |     store     |       |   push  |      |
/// |---------------|     |               |-------|         |------|
/// | Local Vars    | ----/
/// | (virtual)     |
/// |---------------|
pub trait MemoryAccess {
    // it's recommended that add annotation "#[inline]" to the implementation
    fn get_ptr(&self, address: usize) -> *const u8;

    // it's recommended that add annotation "#[inline]" to the implementation
    fn get_mut_ptr(&mut self, address: usize) -> *mut u8;

    #[inline]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn load_to(&self, src_address: usize, dst_ptr: *mut u8, length_in_bytes: usize) {
        let src = self.get_ptr(src_address);
        unsafe {
            std::ptr::copy(src, dst_ptr, length_in_bytes);
        }
    }

    fn load_i64(&self, src_address: usize, dst_ptr: *mut u8) {
        self.load_to(src_address, dst_ptr, 8);
    }

    fn load_i8_s(&self, src_address: usize, dst_ptr: *mut u8) {
        let tp_src = self.get_ptr(src_address) as *const i8;
        unsafe {
            let val_64 = std::ptr::read(tp_src) as i64;
            let dst_ptr_64 = dst_ptr as *mut i64;
            std::ptr::write(dst_ptr_64, val_64);
        }
    }

    fn load_i8_u(&self, src_address: usize, dst_ptr: *mut u8) {
        let tp_src = self.get_ptr(src_address);
        unsafe {
            let val_64 = std::ptr::read(tp_src) as u64;
            let dst_ptr_64 = dst_ptr as *mut u64;
            std::ptr::write(dst_ptr_64, val_64);
        }
    }

    fn load_i16_s(&self, src_address: usize, dst_ptr: *mut u8) {
        let tp_src = self.get_ptr(src_address) as *const i16;
        unsafe {
            let val_64 = std::ptr::read(tp_src) as i64;
            let dst_ptr_64 = dst_ptr as *mut i64;
            std::ptr::write(dst_ptr_64, val_64);
        }
    }

    fn load_i16_u(&self, src_address: usize, dst_ptr: *mut u8) {
        let tp_src = self.get_ptr(src_address) as *const u16;
        unsafe {
            let val_64 = std::ptr::read(tp_src) as u64;
            let dst_ptr_64 = dst_ptr as *mut u64;
            std::ptr::write(dst_ptr_64, val_64);
        }
    }

    fn load_i32_s(&self, src_address: usize, dst_ptr: *mut u8) {
        let tp_src = self.get_ptr(src_address) as *const i32;
        unsafe {
            let val_64 = std::ptr::read(tp_src) as i64;
            let dst_ptr_64 = dst_ptr as *mut i64;
            std::ptr::write(dst_ptr_64, val_64);
        }
    }

    fn load_i32_u(&self, src_address: usize, dst_ptr: *mut u8) {
        let tp_src = self.get_ptr(src_address) as *const u32;
        unsafe {
            let val_64 = std::ptr::read(tp_src) as u64;
            let dst_ptr_64 = dst_ptr as *mut u64;
            std::ptr::write(dst_ptr_64, val_64);
        }
    }

    // load 64-bit data with extra check
    // because VM does support some IEEE 754 variants.
    fn load_f64(&self, src_address: usize, dst_ptr: *mut u8) -> bool {
        let tp = self.get_ptr(src_address) as *const f64;
        let val = unsafe { std::ptr::read(tp) };
        if val.is_normal() || val.is_subnormal() || val == 0.0f64 {
            self.load_i64(src_address, dst_ptr);
            true
        } else {
            false
        }
    }

    // load 32-bit data with extra check
    // because VM does support some IEEE 754 variants.
    fn load_f32(&self, src_addr: usize, dst_ptr: *mut u8) -> bool {
        let tp = self.get_ptr(src_addr) as *const f32;
        let val = unsafe { std::ptr::read(tp) };
        if val.is_normal() || val.is_subnormal() || val == 0.0f32 {
            self.load_i32_u(src_addr, dst_ptr);
            true
        } else {
            false
        }
    }

    #[inline]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn store_from(&mut self, src_ptr: *const u8, dst_address: usize, length_in_bytes: usize) {
        let dst = self.get_mut_ptr(dst_address);
        unsafe {
            std::ptr::copy(src_ptr, dst, length_in_bytes);
        }
    }

    fn store_i64(&mut self, src_ptr: *const u8, dst_address: usize) {
        self.store_from(src_ptr, dst_address, 8);
    }

    fn store_i32(&mut self, src_ptr: *const u8, dst_address: usize) {
        self.store_from(src_ptr, dst_address, 4);
    }

    fn store_i16(&mut self, src_ptr: *const u8, dst_address: usize) {
        self.store_from(src_ptr, dst_address, 2);
    }

    fn store_i8(&mut self, src_ptr: *const u8, dst_address: usize) {
        self.store_from(src_ptr, dst_address, 1);
    }
}
