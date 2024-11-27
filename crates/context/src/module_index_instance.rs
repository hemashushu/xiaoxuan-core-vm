// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use anc_image::{
    index_sections::{
        data_index_section::DataIndexSection,
        external_function_index_section::ExternalFunctionIndexSection,
        external_function_section::UnifiedExternalFunctionSection,
        external_library_section::UnifiedExternalLibrarySection,
        external_type_section::UnifiedExternalTypeSection,
        function_index_section::FunctionIndexSection,
    },
    module_image::ModuleImage,
};

pub struct ModuleIndexInstance<'a> {
    pub runtime_major_version: u16, // only application can specify runtime/compiler version
    pub runtime_minor_version: u16,
    pub entry_function_public_index: u32, // u32::max = none

    // essential
    // pub index_property_section: IndexPropertySection,
    pub function_index_section: FunctionIndexSection<'a>,

    // source optional
    pub data_index_section: DataIndexSection<'a>,
    pub unified_external_type_section: UnifiedExternalTypeSection<'a>,
    pub unified_external_library_section: UnifiedExternalLibrarySection<'a>,
    pub unified_external_function_section: UnifiedExternalFunctionSection<'a>,
    pub external_function_index_section: ExternalFunctionIndexSection<'a>,
}

impl<'a> ModuleIndexInstance<'a> {
    pub fn new(module_images: &'a [ModuleImage<'a>]) -> Self {
        let main_module = &module_images[0];

        let index_property_section = main_module.get_index_property_section();
        let function_index_section = main_module.get_function_index_section();

        let data_index_section = main_module
            .get_optional_data_index_section()
            .unwrap_or_default();

        let unified_external_type_section = main_module
            .get_optional_unified_external_type_section()
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

        let runtime_major_version = index_property_section.runtime_major_version;
        let runtime_minor_version = index_property_section.runtime_minor_version;
        let entry_function_public_index = index_property_section.entry_function_public_index;

        Self {
            runtime_major_version,
            runtime_minor_version,
            entry_function_public_index,
            //
            function_index_section,
            //
            data_index_section,
            //
            unified_external_type_section,
            unified_external_library_section,
            unified_external_function_section,
            external_function_index_section,
        }
    }
}
