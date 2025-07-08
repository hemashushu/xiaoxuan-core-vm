// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_context::{
    process_context::ProcessContext, process_property::ProcessProperty,
    program_source::ProgramSource,
};
use anc_image::{utils::helper_load_modules_from_binaries, ImageError};

/// An implement of 'ProgramSource' for unit testing only
pub struct InMemoryProgramSource {
    program_proerty: ProcessProperty,
    module_binaries: Vec<Vec<u8>>,
}

impl InMemoryProgramSource {
    #[allow(dead_code)]
    pub fn new(module_binaries: Vec<Vec<u8>>) -> Self {
        Self {
            program_proerty: ProcessProperty::default(),
            module_binaries,
        }
    }

    #[allow(dead_code)]
    pub fn with_property(module_binaries: Vec<Vec<u8>>, program_proerty: ProcessProperty) -> Self {
        Self {
            module_binaries,
            program_proerty,
        }
    }
}

impl ProgramSource for InMemoryProgramSource {
    fn create_process_context(&self) -> Result<ProcessContext, ImageError> {
        let binaries_ref = self
            .module_binaries
            .iter()
            .map(|e| &e[..])
            .collect::<Vec<_>>();

        let module_images = helper_load_modules_from_binaries(&binaries_ref)?;

        Ok(ProcessContext::new(
            self.program_proerty.clone(),
            module_images,
        ))
    }
}
