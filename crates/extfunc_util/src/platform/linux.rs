// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use core::ffi::c_void;

use libc::{dlerror, dlopen, dlsym, RTLD_LAZY};

use crate::{cstr_pointer_to_str, str_to_cstring};

// when the path contains char '/', this function load the
// library with the relative/absolute path. otherwise loads the
// system wide shared library (see `ldconfig`)
pub fn load_library(file_path_or_name: &str) -> Result<*mut c_void, &'static str> {
    // clear the last error msg, see `$ man 3 dlerror`
    let _last_msg = unsafe { dlerror() };
    let library_ptr = unsafe { dlopen(str_to_cstring(file_path_or_name).as_ptr(), RTLD_LAZY) };

    if library_ptr.is_null() {
        let msg = unsafe { dlerror() };
        Err(cstr_pointer_to_str(msg))
    } else {
        Ok(library_ptr)
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn load_symbol(library_ptr: *mut c_void, name: &str) -> Result<*mut c_void, &'static str> {
    let _last_msg = unsafe { dlerror() };
    let symbol_ptr = unsafe { dlsym(library_ptr, str_to_cstring(name).as_ptr()) };

    if symbol_ptr.is_null() {
        let msg = unsafe { dlerror() };
        Err(cstr_pointer_to_str(msg))
    } else {
        Ok(symbol_ptr)
    }
}

#[cfg(test)]
mod tests {
    use std::{env, ffi::c_char};

    use libc::uid_t;

    use crate::{
        cstr_pointer_to_str, load_library, load_symbol, str_to_cstring, transmute_symbol_to,
    };

    #[test]
    fn test_load_library() {
        // ntoe:
        //
        // ensure that the specified version of libc exists first:
        // `$ ldconfig -p|grep libc.so`

        let result0 = load_library("libc.so.6");
        assert!(matches!(result0, Ok(ptr) if !ptr.is_null()));

        let result1 = load_library("lib_not_exist.so");
        assert!(result1.is_err());
    }

    #[test]
    fn test_load_symbol() {
        let library_ptr = load_library("libc.so.6").unwrap();

        let result0 = load_symbol(library_ptr, "getuid");
        assert!(matches!(result0, Ok(ptr) if !ptr.is_null()));

        let result1 = load_symbol(library_ptr, "fn_not_exist");
        assert!(result1.is_err());
    }

    #[test]
    fn test_convert_symbol() {
        let library_ptr = load_library("libc.so.6").unwrap();

        let symbol0 = load_symbol(library_ptr, "getuid").unwrap();
        let getuid: extern "C" fn() -> uid_t = transmute_symbol_to(symbol0);
        let uid0 = getuid();
        assert!(uid0 > 0);

        let symbol1 = load_symbol(library_ptr, "getenv").unwrap();
        let getenv: extern "C" fn(*const c_char) -> *mut c_char = transmute_symbol_to(symbol1);
        let pwd0 = cstr_pointer_to_str(getenv(str_to_cstring("PWD").as_ptr()));
        assert!(!pwd0.to_string().is_empty());
    }

    #[test]
    fn test_load_user_library() {
        // note:
        // run 'test/lib/compile.sh' to build shared library 'lib-test-0' first.

        let mut pwd = env::current_dir().unwrap();
        pwd.push("tests");
        pwd.push("lib");
        pwd.push("lib-test-0.so.1");
        let lib_test_path = pwd.to_str().unwrap();

        let library_ptr = load_library(lib_test_path).unwrap();

        let func_add_ptr = load_symbol(library_ptr, "add").unwrap();
        let func_add: extern "C" fn(i32, i32) -> i32 = transmute_symbol_to(func_add_ptr);
        assert_eq!(func_add(11, 13), 11 + 13);

        let func_mul_add_ptr = load_symbol(library_ptr, "mul_add").unwrap();
        let func_mul_add: extern "C" fn(i32, i32, i32) -> i32 =
            transmute_symbol_to(func_mul_add_ptr);
        assert_eq!(func_mul_add(11, 13, 17), 11 * 13 + 17);

        let func_do_something_ptr = load_symbol(library_ptr, "do_something").unwrap();
        let func_do_something: extern "C" fn(*const u8, i32, i32) -> i32 =
            transmute_symbol_to(func_do_something_ptr);
        assert_eq!(
            func_do_something(user_callback as *const u8, 13, 17),
            13 * 2 + 17
        );
    }

    extern "C" fn user_callback(n: i32) -> i32 {
        n * 2
    }
}
