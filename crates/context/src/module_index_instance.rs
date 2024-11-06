// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_image::{
    index_sections::{
        data_index_section::DataIndexSection,
        external_function_index_section::ExternalFunctionIndexSection,
        function_index_section::FunctionIndexSection, index_property_section::IndexPropertySection,
        unified_external_function_section::UnifiedExternalFunctionSection,
        unified_external_library_section::UnifiedExternalLibrarySection,
    },
    module_image::ModuleImage,
};

pub struct ModuleIndexInstance<'a> {
    pub function_index_section: FunctionIndexSection<'a>,
    pub index_property_section: IndexPropertySection,
    pub data_index_section: DataIndexSection<'a>,
    pub unified_external_library_section: UnifiedExternalLibrarySection<'a>,
    pub unified_external_function_section: UnifiedExternalFunctionSection<'a>,
    pub external_function_index_section: ExternalFunctionIndexSection<'a>,
}

impl<'a> ModuleIndexInstance<'a> {
    pub fn new(module_images: &'a [ModuleImage<'a>]) -> Self {
        let main_module = &module_images[0];

        let function_index_section = main_module.get_function_index_section();
        let index_property_section = main_module.get_index_property_section();

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
            index_property_section,
            data_index_section,
            unified_external_library_section,
            unified_external_function_section,
            external_function_index_section,
        }
    }
}
