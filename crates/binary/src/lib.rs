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

pub fn load_modules_binary(module_binaries: Vec<&[u8]>) -> Result<Vec<ModuleImage>, BinaryError> {
    let mut module_images: Vec<ModuleImage> = Vec::new();
    for binary in module_binaries {
        let module_image = ModuleImage::load(binary)?;
        module_images.push(module_image);
    }

    Ok(module_images)
}
