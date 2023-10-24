// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::{any::Any, fmt::Display};

use ancvm_types::RuntimeError;
use module_image::ModuleImage;

pub mod cache_info;
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

impl RuntimeError for BinaryError {
    fn as_any(&self) -> &dyn Any {
        self
    }

    // fn get_message(&self) -> &str {
    //     &self.message
    // }
}

pub fn load_modules_from_binaries(
    module_binaries: Vec<&[u8]>,
) -> Result<Vec<ModuleImage>, BinaryError> {
    let mut module_images: Vec<ModuleImage> = Vec::new();
    for binary in module_binaries {
        let module_image = ModuleImage::load(binary)?;
        module_images.push(module_image);
    }

    Ok(module_images)
}
