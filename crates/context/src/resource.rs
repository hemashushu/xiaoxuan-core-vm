// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use anc_image::ImageError;

use crate::process_context::ProcessContext;

/// `ProcessContext` is produced by `Resource`.
pub trait Resource {
    fn create_process_context(&self) -> Result<ProcessContext, ImageError>;
}
