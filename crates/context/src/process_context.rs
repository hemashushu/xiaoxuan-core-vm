// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::sync::Mutex;

use ancvm_binary::module_image::ModuleImage;

use crate::{
    external_function::ExtenalFunctionTable, program_settings::ProgramSettings,
    thread_context::ThreadContext,
};

// all asserts when a program is running.
// it's the reference of the 'ProgramResource'
pub struct ProcessContext<'a> {
    pub program_settings: &'a ProgramSettings,
    pub module_images: Vec<ModuleImage<'a>>,

    // since the 'loadlibrary' is process-scope, the external function (pointer) table
    // should be placed at the 'Program' instead of 'ThreadContext'
    pub external_function_table: &'a Mutex<ExtenalFunctionTable>,
}

impl<'a> ProcessContext<'a> {
    pub fn new(
        program_settings: &'a ProgramSettings,
        external_function_table: &'a Mutex<ExtenalFunctionTable>,
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
            program_settings,
            module_images,
            external_function_table,
        }
    }

    pub fn create_thread_context(&'a self) -> ThreadContext<'a> {
        ThreadContext::new(
            self.external_function_table,
            self.program_settings,
            &self.module_images,
        )
    }
}
