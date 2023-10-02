// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

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

#[cfg(target_family="unix")]
pub mod platform_linux;

#[cfg(target_family="windows")]
pub mod platform_windows;

#[cfg(target_family="unix")]
pub use platform_linux::*;