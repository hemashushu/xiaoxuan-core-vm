// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// to load a library and get a symbol (function or 'global variable'):
// - linux:
//   dlopen/dlsym/dlerror/dlclose
// - windows:
//   LoadLibraryW(LoadLibraryExW, GetModuleHandleExW)/GetProcAddress/FreeLibrary

#[cfg(target_family="unix")]
pub mod platform_linux;

#[cfg(target_family="windows")]
pub mod platform_windows;