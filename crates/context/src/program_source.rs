// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_image::ImageError;

use crate::process_context::ProcessContext;

/// The `ProgramSource` trait is responsible for managing module binary images
/// and associated `ProcessProperty` data.
/// It provides functionality to generate a `ProcessContext` from the source.
pub trait ProgramSource {
    /// Creates a `ProcessContext` from the program source.
    ///
    /// # Returns
    /// - `Ok(ProcessContext)` if the context is successfully created.
    /// - `Err(ImageError)` if an error occurs during the process.
    fn create_process_context(&self) -> Result<ProcessContext, ImageError>;
}
