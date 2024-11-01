// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

pub mod bytecode_reader;
pub mod bytecode_writer;
pub mod common_sections;
pub mod entry;
pub mod index_sections;
pub mod module_image;
pub mod tableaccess;
pub mod utils;

use std::fmt::Display;

use ancvm_isa::{RUNTIME_MAJOR_VERSION, RUNTIME_MINOR_VERSION};
use module_image::ModuleImage;

#[derive(Debug)]
pub struct BinaryError {
    message: String,
}

impl BinaryError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_owned(),
        }
    }
}

impl Display for BinaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Binary error: {}", self.message)
    }
}

impl std::error::Error for BinaryError {}

pub fn load_modules_from_binaries(
    module_binaries: Vec<&[u8]>,
) -> Result<Vec<ModuleImage>, BinaryError> {
    let mut module_images: Vec<ModuleImage> = Vec::new();
    for binary in module_binaries {
        let module_image = ModuleImage::load(binary)?;

        let property_section = module_image.get_property_section();
        let require_runtime_version = ((property_section.runtime_major_version as u32) << 16)
            | (property_section.runtime_minor_version as u32);
        let supported_runtime_version =
            ((RUNTIME_MAJOR_VERSION as u32) << 16) | (RUNTIME_MINOR_VERSION as u32);

        // a module will only run if its required major and minor
        // versions match the current runtime version 100%.
        if require_runtime_version != supported_runtime_version {
            return Err(BinaryError::new(
                "The module requires a different version runtime to run.",
            ));
        }

        module_images.push(module_image);
    }

    Ok(module_images)
}
