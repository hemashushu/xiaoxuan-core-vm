// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_image::ImageError;

use crate::process_context::ProcessContext;

/// `ProcessContext` is produced by `ProgramSource`.
pub trait ProgramSource {
    fn create_process_context(&self) -> Result<ProcessContext, ImageError>;
}
