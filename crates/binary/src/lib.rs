// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::{fmt::Display, fs::File};

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

pub fn load_modules_files(
    app_cache_path: &str,
    shared_modules_path: &str,
) -> Result<File, BinaryError> {
    // a script application may consist of a single script file, or several script files.
    // in either case, these script files will be compiled into a single module image file
    // named 'main.ancbc', and this file will be copied to the 'application cache' directory.
    //
    // the dependent modules of application are copied to this cache directory also, but
    // the shared modules (such as the standard library) are located at the runtime directory, and
    // they will not be copied to this directory.
    //
    // the structure of application cache directory
    //
    // app cache dir
    //   |-- cache.info (the cache infomations, such as the last modified time and content hash of script file)
    //   |-- main.ancbc
    //   |-- dependency1.ancbc
    //   |-- dependencyN.ancbc

    todo!()
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
