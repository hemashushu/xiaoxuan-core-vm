// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::{cell::RefCell, rc::Rc};

use ancvm_binary::module_image::ModuleImage;

use crate::{
    external_function::ExtenalFunctionTable, program_settings::ProgramSettings,
    thread_context::ThreadContext,
};

pub struct Program<'a> {
    program_settings: &'a ProgramSettings,

    // since the 'loadlibrary' is process-scope, the external function (pointer) table
    // should be placed at the 'Program' instead of 'ThreadContext'
    external_function_table: Rc<RefCell<ExtenalFunctionTable>>,
    module_images: Vec<ModuleImage<'a>>,
}

impl<'a> Program<'a> {
    pub fn new(program_settings: &'a ProgramSettings, module_images: Vec<ModuleImage<'a>>) -> Self {
        let external_library_count = module_images[0]
            .get_optional_unified_external_library_section()
            .map_or(0, |section| section.items.len());
        let external_func_count = module_images[0]
            .get_optional_unified_external_func_section()
            .map_or(0, |section| section.items.len());
        let external_function_table = Rc::new(RefCell::new(ExtenalFunctionTable::new(
            external_library_count,
            external_func_count,
        )));

        Self {
            program_settings,
            module_images,
            external_function_table,
        }
    }

    pub fn new_thread_context(&'a self) -> ThreadContext<'a> {
        let external_function_table = Rc::clone(&self.external_function_table);
        ThreadContext::new(
            external_function_table,
            self.program_settings,
            &self.module_images,
        )
    }
}
