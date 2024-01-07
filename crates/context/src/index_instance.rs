// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_binary::module_image::{
    data_index_section::DataIndexSection, exit_function_list_section::ExitFunctionListSection,
    external_function_index_section::ExternalFunctionIndexSection,
    function_index_section::FunctionIndexSection, property_section::PropertySection,
    start_function_list_section::StartFunctionListSection,
    unified_external_function_section::UnifiedExternalFunctionSection,
    unified_external_library_section::UnifiedExternalLibrarySection, ModuleImage,
};

pub struct IndexInstance<'a> {
    // the indices
    pub function_index_section: FunctionIndexSection<'a>,
    pub start_function_list_section: StartFunctionListSection<'a>,
    pub exit_function_list_section: ExitFunctionListSection<'a>,
    pub property_section: PropertySection,
    //
    pub data_index_section: DataIndexSection<'a>,
    pub unified_external_library_section: UnifiedExternalLibrarySection<'a>,
    pub unified_external_function_section: UnifiedExternalFunctionSection<'a>,
    pub external_function_index_section: ExternalFunctionIndexSection<'a>,
}

impl<'a> IndexInstance<'a> {
    pub fn new(module_images: &'a [ModuleImage<'a>]) -> Self {
        let main_module = &module_images[0];

        let function_index_section = main_module.get_function_index_section();
        let start_function_list_section = main_module.get_start_function_list_section();
        let exit_function_list_section = main_module.get_exit_function_list_section();
        let property_section = main_module.get_property_section();

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
            property_section,
            //
            data_index_section,
            unified_external_library_section,
            unified_external_function_section,
            external_function_index_section,
        }
    }
}
