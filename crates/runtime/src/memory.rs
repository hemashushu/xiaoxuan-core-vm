// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

/// in XiaoXuan VM, there are several objects belong to the memory class,
/// includes the local variable area, (thread-local) data sections, stack, (thread-local) heap.
/// this trait provides the ability of data loading and storing to memory,
/// but does not involve the primitive data reading and writing.
///
/// |---------------|
/// | ReadOnlyData  |
/// | ReadWriteData | ----\
/// | UninitData    |     |
/// |---------------|     |
///                       |               |-------|         |------|
/// |---------------|     |     load      |       |   pop   |      |
/// | Heap          | ----| ------------> | Stack | ------> |  VM  |
/// |---------------|     | <------------ |       | <------ |      |
///                       |     store     |       |   push  |      |
/// |---------------|     |               |-------|         |------|
/// | Local Vars    | ----/
/// | (virtual)     |
/// |---------------|
///
/// there are 2 kinds of length for the operands, 32-bit and 64-bit operands.
/// it makes reading or writing 32-bit data conveniently because when
/// reading or writing i32 data, it doesn't have to sign-extend it to i64.
/// so there 2 versions of function 'load' and 'store', one is load_64/store_64,
/// and the another one is load_32/store_32.
pub trait Memory {
    // it's recommended that add annotation "#[inline]" to the implementation
    fn get_ptr(&self, addr: usize) -> *const u8;

    // it's recommended that add annotation "#[inline]" to the implementation
    fn get_mut_ptr(&mut self, addr: usize) -> *mut u8;

    fn fill(&mut self, addr: usize, value: u8, length_in_bytes: usize) {
        let dst = self.get_mut_ptr(addr);
        unsafe {
            std::ptr::write_bytes(dst, value, length_in_bytes);
        }
    }

    #[inline]
    fn load_to(&self, src_addr: usize, dst_ptr: *mut u8, length_in_bytes: usize) {
        let src = self.get_ptr(src_addr);
        unsafe {
            std::ptr::copy(src, dst_ptr, length_in_bytes);
        }
    }

    fn load_64(&self, src_addr: usize, dst_ptr: *mut u8) {
        self.load_to(src_addr, dst_ptr, 8);
    }

    fn load_32(&self, src_addr: usize, dst_ptr: *mut u8) {
        self.load_to(src_addr, dst_ptr, 4);
    }

    // load 64-bit data with extra check
    // because VM does support some IEEE 754 variants.
    fn load_64_with_float_check(&self, src_addr: usize, dst_ptr: *mut u8) -> bool {
        let tp = self.get_ptr(src_addr) as *const f64;
        let val = unsafe { std::ptr::read(tp) };
        if val.is_normal() || val.is_subnormal() || val == 0.0f64 {
            self.load_64(src_addr, dst_ptr);
            true
        } else {
            false
        }
    }

    // load 32-bit data with extra check
    // because VM does support some IEEE 754 variants.
    fn load_32_with_float_check(&self, src_addr: usize, dst_ptr: *mut u8) -> bool {
        let tp = self.get_ptr(src_addr) as *const f32;
        let val = unsafe { std::ptr::read(tp) };
        if val.is_normal() || val.is_subnormal() || val == 0.0f32 {
            self.load_32(src_addr, dst_ptr);
            true
        } else {
            false
        }
    }

    fn load_32_extend_from_i8(&self, src_addr: usize, dst_ptr: *mut u8) {
        let tp_src = self.get_ptr(src_addr) as *const i8;
        unsafe {
            let val_32 = std::ptr::read(tp_src) as i32;
            let dst_ptr_32 = dst_ptr as *mut i32;
            std::ptr::write(dst_ptr_32, val_32);
        }
    }

    fn load_32_extend_from_u8(&self, src_addr: usize, dst_ptr: *mut u8) {
        let tp_src = self.get_ptr(src_addr) as *const u8;
        unsafe {
            let val_32 = std::ptr::read(tp_src) as u32;
            let dst_ptr_32 = dst_ptr as *mut u32;
            std::ptr::write(dst_ptr_32, val_32);
        }
    }

    fn load_32_extend_from_i16(&self, src_addr: usize, dst_ptr: *mut u8) {
        let tp_src = self.get_ptr(src_addr) as *const i16;
        unsafe {
            let val_32 = std::ptr::read(tp_src) as i32;
            let dst_ptr_32 = dst_ptr as *mut i32;
            std::ptr::write(dst_ptr_32, val_32);
        }
    }

    fn load_32_extend_from_u16(&self, src_addr: usize, dst_ptr: *mut u8) {
        let tp_src = self.get_ptr(src_addr) as *const u16;
        unsafe {
            let val_32 = std::ptr::read(tp_src) as u32;
            let dst_ptr_32 = dst_ptr as *mut u32;
            std::ptr::write(dst_ptr_32, val_32);
        }
    }

    #[inline]
    fn store_from(&mut self, src_ptr: *const u8, dst_addr: usize, length_in_bytes: usize) {
        let dst = self.get_mut_ptr(dst_addr);
        unsafe {
            std::ptr::copy(src_ptr, dst, length_in_bytes);
        }
    }

    fn store_64(&mut self, src_ptr: *const u8, dst_addr: usize) {
        self.store_from(src_ptr, dst_addr, 8);
    }

    fn store_32(&mut self, src_ptr: *const u8, dst_addr: usize) {
        self.store_from(src_ptr, dst_addr, 4);
    }

    fn store_16(&mut self, src_ptr: *const u8, dst_addr: usize) {
        self.store_from(src_ptr, dst_addr, 2);
    }

    fn store_8(&mut self, src_ptr: *const u8, dst_addr: usize) {
        self.store_from(src_ptr, dst_addr, 1);
    }
}
