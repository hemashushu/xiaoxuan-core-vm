// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::fmt::Display;

use module_image::ModuleImage;

pub mod module_image;
pub mod utils;

#[derive(Debug)]
pub struct BinaryError {
    message: String,
}

impl Display for BinaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "binary error: {}", self.message)
    }
}

pub fn load_binary<'a>(module_image_binaries: &'a [&'a [u8]]) -> (&'a [ModuleImage<'a>]) {
    todo!()
}
