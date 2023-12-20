// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_binary::module_image::{
    data_index_section::DataIndexSection, exit_function_list_section::ExitFunctionListSection,
    external_function_index_section::ExternalFunctionIndexSection,
    function_index_section::FunctionIndexSection,
    start_function_list_section::StartFunctionListSection,
    unified_external_function_section::UnifiedExternalFunctionSection,
    unified_external_library_section::UnifiedExternalLibrarySection, ModuleImage,
};

use crate::program_module::ProgramModule;

pub struct ProgramContext<'a> {
    // the indices
    pub function_index_section: FunctionIndexSection<'a>,
    pub start_function_list_section: StartFunctionListSection<'a>,
    pub exit_function_list_section: ExitFunctionListSection<'a>,
    pub data_index_section: DataIndexSection<'a>,
    pub unified_external_library_section: UnifiedExternalLibrarySection<'a>,
    pub unified_external_function_section: UnifiedExternalFunctionSection<'a>,
    pub external_function_index_section: ExternalFunctionIndexSection<'a>,

    // the modules
    pub program_modules: Vec<ProgramModule<'a>>,
}

impl<'a> ProgramContext<'a> {
    pub fn new(module_images: &'a [ModuleImage<'a>]) -> Self {
        let program_modules = module_images
            .iter()
            .map(ProgramModule::new)
            .collect::<Vec<ProgramModule>>();

        let main_module = &module_images[0];

        let function_index_section = main_module.get_function_index_section();
        let start_function_list_section = main_module.get_start_function_list_section();
        let exit_function_list_section = main_module.get_exit_function_list_section();

        let data_index_section = main_module
            .get_optional_data_index_section()
            .unwrap_or_default();

        let unified_external_library_section = main_module
            .get_optional_unified_external_library_section()
            .unwrap_or_default();

        let unified_external_function_section = main_module
            .get_optional_unified_external_function_section()
            .unwrap_or_default();

        let external_function_index_section = main_module
            .get_optional_external_function_index_section()
            .unwrap_or_default();

        Self {
            function_index_section,
            start_function_list_section,
            exit_function_list_section,
            data_index_section,
            unified_external_library_section,
            unified_external_function_section,
            external_function_index_section,
            program_modules,
        }
    }
}
