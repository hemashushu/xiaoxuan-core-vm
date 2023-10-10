// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_binary::module_image::{
    data_index_section::DataIndexSection, external_func_index_section::ExternalFuncIndexSection,
    func_index_section::FuncIndexSection,
    unified_external_func_section::UnifiedExternalFuncSection,
    unified_external_library_section::UnifiedExternalLibrarySection, ModuleImage,
};

use crate::program_module::ProgramModule;

// const EMPTY_RANGE_ITEMS: &[RangeItem] = &[];
// const EMPTY_DATA_INDEX_ITEMS: &[DataIndexItem] = &[];
// const EMPTY_UNIFIED_EXTERNAL_LIBRARY_ITEM: &[UnifiedExternalLibraryItem] = &[];
// const EMPTY_UNIFIED_EXTERNAL_FUNC_ITEM: &[UnifiedExternalFuncItem] = &[];
// const EMPTY_EXTERNAL_FUNC_INDEX_ITEM: &[ExternalFuncIndexItem] = &[];

pub struct ProgramContext<'a> {
    // the indices
    pub func_index_section: FuncIndexSection<'a>,
    pub data_index_section: DataIndexSection<'a>,
    pub unified_external_library_section: UnifiedExternalLibrarySection<'a>,
    pub unified_external_func_section: UnifiedExternalFuncSection<'a>,
    pub external_func_index_section: ExternalFuncIndexSection<'a>,

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

        let func_index_section = main_module.get_func_index_section();
        let data_index_section = main_module
            .get_optional_data_index_section()
            .unwrap_or_default();

        let unified_external_library_section = main_module
            .get_optional_unified_external_library_section()
            .unwrap_or_default();

        let unified_external_func_section = main_module
            .get_optional_unified_external_func_section()
            .unwrap_or_default();

        let external_func_index_section = main_module
            .get_optional_external_func_index_section()
            .unwrap_or_default();

        Self {
            data_index_section,
            func_index_section,
            unified_external_library_section,
            unified_external_func_section,
            external_func_index_section,
            program_modules,
        }
    }
}
