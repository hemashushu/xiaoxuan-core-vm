// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// note:
//
// this crate is part of project XiaoXuan Core VM, it is
// not intended to be a standalone library.
// if you need a dynamic load library, please refer to:
// - https://github.com/MaulingMonkey/minidl.git
// - https://github.com/fschutt/libloading-mini.git

// to load a library and get a symbol (function or 'global variable'):
// - linux:
//   dlopen/dlsym/dlerror/dlclose
// - windows:
//   LoadLibraryW(LoadLibraryExW, GetModuleHandleExW)/GetProcAddress/FreeLibrary

mod platform;

use std::{
    ffi::{c_char, CString, OsString},
    os::raw::c_void,
};

#[cfg(target_family = "unix")]
pub use platform::linux::*;

pub fn str_to_osstring(s: &str) -> OsString {
    let oss: OsString = OsString::from(s);
    oss
}

pub fn str_to_cstring(s: &str) -> CString {
    CString::new(s).unwrap()
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn cstr_pointer_to_str(cstr_ptr: *const c_char) -> &'static str {
    unsafe { std::ffi::CStr::from_ptr(cstr_ptr).to_str().unwrap() }
}

pub fn transmute_symbol_to<T>(ptr: *mut c_void) -> T {
    unsafe { std::mem::transmute_copy::<*mut c_void, T>(&ptr) }
}
