// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// https://doc.rust-lang.org/stable/reference/conditional-compilation.html?highlight=cfg#the-cfg-attribute
#[cfg(target_arch = "x86_64")]
pub mod x86_64;
