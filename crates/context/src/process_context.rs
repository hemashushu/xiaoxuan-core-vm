// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::sync::Mutex;

use anc_image::module_image::ModuleImage;
use cranelift_jit::JITModule;

use crate::{
    capability::Capability, code_generator::Generator,
    external_function_table::ExternalFunctionTable, process_property::ProcessProperty,
    thread_context::ThreadContext,
};

/// `ProcessContext` contains the resources required for program execution.
/// It is responsible for producing `ThreadContext` instances.
#[non_exhaustive]
pub struct ProcessContext<'a> {
    /// A collection of module images associated with the process.
    pub module_images: Vec<ModuleImage<'a>>,

    /// Properties of the process, such as configuration and metadata.
    pub process_property: Mutex<ProcessProperty>,

    /// The external function table.
    ///
    /// Since the pointer retained by the `loadlibrary` function is process-scoped,
    /// the "external function table" must reside in `ProcessContext` instead of `ThreadContext`.
    pub external_function_table: Mutex<ExternalFunctionTable>,

    /// The code generator.
    pub jit_generator: Mutex<Generator<JITModule>>,
}

impl<'a> ProcessContext<'a> {
    /// Creates a new `ProcessContext` with the given process properties and module images.
    pub fn new(
        loaded_process_property: ProcessProperty,
        module_images: Vec<ModuleImage<'a>>,
    ) -> Self {
        // Determine the number of unified external libraries from the first module image.
        let unified_external_library_count = module_images[0]
            .get_optional_unified_external_library_section()
            .map_or(0, |section| section.items.len());

        // Determine the number of unified external functions from the first module image.
        let unified_external_function_count = module_images[0]
            .get_optional_unified_external_function_section()
            .map_or(0, |section| section.items.len());

        // Initialize the external function table with the determined counts.
        let external_function_table = Mutex::new(ExternalFunctionTable::new(
            unified_external_library_count,
            unified_external_function_count,
        ));

        // create JIT generator without imported symbols
        let jit_generator = Mutex::new(Generator::<JITModule>::new(vec![]));

        let process_property = Mutex::new(loaded_process_property);

        Self {
            module_images,
            process_property,
            external_function_table,
            jit_generator,
        }
    }

    /// Creates a new `ThreadContext` associated with this `ProcessContext`.
    pub fn create_thread_context(&'a self) -> ThreadContext<'a> {
        ThreadContext::new(
            &self.module_images,
            &self.process_property,
            &self.external_function_table,
            &self.jit_generator,
        )
    }
}
