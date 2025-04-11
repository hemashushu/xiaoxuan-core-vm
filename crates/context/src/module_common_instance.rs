// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use anc_image::{
    common_sections::{
        data_name_section::DataNameSection, function_name_section::FunctionNameSection,
        function_section::FunctionSection, local_variable_section::LocalVariableSection,
        type_section::TypeSection,
    },
    module_image::ModuleImage,
};
use anc_memory::indexed_memory_access::IndexedMemoryAccess;

use crate::datas::{ReadOnlyDatas, ReadWriteDatas, UninitDatas};

pub struct ModuleCommonInstance<'a> {
    // the name of module/package,
    //
    // note that it CANNOT be the sub-module name even if the current image is
    // the object file of a sub-module, it CANNOT be a name path either.
    //
    // explaintion the "full_name" and "name_path"
    // -------------------------------------
    // - "full_name" = "module_name::name_path"
    // - "name_path" = "namespace::identifier"
    // - "namespace" = "sub_module_name"{0,N}
    pub name: String,

    pub type_section: TypeSection<'a>,
    pub local_variable_section: LocalVariableSection<'a>,
    pub function_section: FunctionSection<'a>,
    pub datas: [Box<dyn IndexedMemoryAccess + 'a>; 3],

    // for bridge function feature
    pub function_name_section: FunctionNameSection<'a>,

    // for bridge function feature
    pub data_name_section: DataNameSection<'a>,
}

impl<'a> ModuleCommonInstance<'a> {
    pub fn new(module_image: &'a ModuleImage<'a>) -> Self {
        let property_section = module_image.get_property_section();
        let type_section = module_image.get_type_section();
        let local_variable_section = module_image.get_local_variable_section();
        let function_section = module_image.get_function_section();

        let read_only_data = module_image
            .get_optional_read_only_data_section()
            .map_or_else(
                || ReadOnlyDatas::new(&[], &[]),
                |section| ReadOnlyDatas::new(section.items, section.datas_data),
            );

        let read_write_data = module_image
            .get_optional_read_write_data_section()
            .map_or_else(
                || ReadWriteDatas::new(&[], Vec::<u8>::new()),
                |section| ReadWriteDatas::new(section.items, section.datas_data.to_vec()),
            );

        let uninit_data = module_image.get_optional_uninit_data_section().map_or_else(
            || UninitDatas::new(&[], Vec::<u8>::new()),
            |section| {
                // allocate the memory for uninit data
                let length = section
                    .items
                    .iter()
                    .map(|item| item.data_length)
                    .sum::<u32>();

                UninitDatas::new(section.items, vec![0u8; length as usize])
            },
        );

        let function_name_section = module_image
            .get_optional_export_function_section()
            .unwrap_or_default();

        let data_name_section = module_image
            .get_optional_export_data_section()
            .unwrap_or_default();

        let name = property_section.get_module_name().to_owned();

        Self {
            name,
            type_section,
            function_section,
            local_variable_section,
            datas: [
                Box::new(read_only_data),
                Box::new(read_write_data),
                Box::new(uninit_data),
            ],
            function_name_section,
            data_name_section,
        }
    }
}
