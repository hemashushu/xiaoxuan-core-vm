// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::sync::Mutex;

use anc_image::module_image::ModuleImage;

use crate::{
    external_function_table::ExternalFunctionTable, process_config::ProcessConfig,
    thread_context::ThreadContext,
};

/// `ProcessContext` contains all asserts (environment and module images)
/// when a program is running.
/// `ThreadContext` is produced by `ProcessContext`.
pub struct ProcessContext<'a> {
    pub config: &'a ProcessConfig,
    pub module_images: Vec<ModuleImage<'a>>,

    // since the 'loadlibrary' is process-scope, the external function (pointer) table
    // should be placed at the 'Program' instead of 'ThreadContext'
    pub external_function_table: &'a Mutex<ExternalFunctionTable>,
}

impl<'a> ProcessContext<'a> {
    pub fn new(
        config: &'a ProcessConfig,
        external_function_table: &'a Mutex<ExternalFunctionTable>,
        module_images: Vec<ModuleImage<'a>>,
    ) -> Self {
        let unified_external_library_count = module_images[0]
            .get_optional_unified_external_library_section()
            .map_or(0, |section| section.items.len());

        let unified_external_function_count = module_images[0]
            .get_optional_unified_external_function_section()
            .map_or(0, |section| section.items.len());

        external_function_table.lock().unwrap().init(
            unified_external_library_count,
            unified_external_function_count,
        );

        Self {
            config,
            module_images,
            external_function_table,
        }
    }

    pub fn create_thread_context(&'a self) -> ThreadContext<'a> {
        ThreadContext::new(
            self.config,
            &self.module_images,
            self.external_function_table,
        )
    }
}
