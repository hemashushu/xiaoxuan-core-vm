// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use core::ffi::c_void;
use std::{borrow::Cow, ffi::c_char};

use libc::{dlerror, dlopen, dlsym, RTLD_LAZY};

// when the path contains char '/', this function load the
// library with the relative/absolute path. otherwise loads the
// system wide shared library (see `ldconfig`)
pub fn load_library(path: &str) -> Result<*mut c_void, String> {
    // clear the last error msg, see `$ man 3 dlerror`
    let _last_msg = unsafe { dlerror() };
    let library_ptr = unsafe { dlopen(add_null_terminated(path).as_ptr() as *const i8, RTLD_LAZY) };

    if library_ptr.is_null() {
        let msg = unsafe { dlerror() };
        let msg_string = unsafe { std::ffi::CStr::from_ptr(msg).to_string_lossy().to_string() };
        Err(msg_string)
    } else {
        Ok(library_ptr)
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn load_symbol(library_ptr: *mut c_void, name: &str) -> Result<*mut c_void, String> {
    let _last_msg = unsafe { dlerror() };
    let symbol_ptr = unsafe { dlsym(library_ptr, add_null_terminated(name).as_ptr() as *const i8) };

    if symbol_ptr.is_null() {
        let msg = unsafe { dlerror() };
        Err(convert_from_cstring(msg).to_string())
    } else {
        Ok(symbol_ptr)
    }
}

pub fn add_null_terminated(s: &str) -> Vec<u8> {
    let mut bytes = s.as_bytes().to_vec();
    bytes.push(0);
    bytes
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn convert_from_cstring(string_ptr: *const c_char) -> Cow<'static, str> {
    unsafe { std::ffi::CStr::from_ptr(string_ptr).to_string_lossy() }
}

pub fn convert_symbol_to<T>(ptr: *mut c_void) -> T {
    unsafe { std::mem::transmute_copy::<*mut c_void, T>(&ptr) }
}

#[cfg(test)]
mod tests {
    use std::{env, ffi::c_char};

    use libc::uid_t;

    use crate::platform_linux::{add_null_terminated, convert_from_cstring};

    use super::{convert_symbol_to, load_library, load_symbol};

    #[test]
    fn test_load_library() {
        // ntoe:
        //
        // ensure that the specified version of libc exists first:
        // `$ ldconfig -p|grep libc.so`

        let result0 = load_library("libc.so.6");
        assert!(matches!(result0, Ok(ptr) if !ptr.is_null()));

        let result1 = load_library("lib_not_exist.so");
        assert!(matches!(result1, Err(_)));
    }

    #[test]
    fn test_load_symbol() {
        let library_ptr = load_library("libc.so.6").unwrap();

        let result0 = load_symbol(library_ptr, "getuid");
        assert!(matches!(result0, Ok(ptr) if !ptr.is_null()));

        let result1 = load_symbol(library_ptr, "fn_not_exist");
        assert!(matches!(result1, Err(_)));
    }

    #[test]
    fn test_convert_symbol() {
        let library_ptr = load_library("libc.so.6").unwrap();

        let symbol0 = load_symbol(library_ptr, "getuid").unwrap();
        let getuid: fn() -> uid_t = convert_symbol_to(symbol0);
        let uid0 = getuid();
        assert!(uid0 > 0);

        let symbol1 = load_symbol(library_ptr, "getenv").unwrap();
        let getenv: fn(*const c_char) -> *mut c_char = convert_symbol_to(symbol1);
        let pwd0 = convert_from_cstring(getenv(add_null_terminated("PWD").as_ptr() as _));
        assert!(!pwd0.to_string().is_empty());
    }

    #[test]
    #[allow(dead_code)]
    fn test_load_local_library() {
        // note:
        // run 'resources/compile.sh' to build shared library 'lib-test-0' first.

        let mut pwd = env::current_dir().unwrap();
        pwd.push("resources");
        pwd.push("lib-test-0.so.1.0.0");
        let lib_test_path = pwd.to_str().unwrap();

        let library_ptr = load_library(lib_test_path).unwrap();
        let func_ptr = load_symbol(library_ptr, "add").unwrap();

        let add: extern "C" fn(i32, i32) -> i32 = convert_symbol_to(func_ptr);
        assert_eq!(add(11, 13), 24);
    }
}
