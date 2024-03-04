// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_binary::module_image::{
    data_name_section::DataNameSection, function_name_section::FunctionNameSection,
    function_section::FunctionSection, local_variable_section::LocalVariableSection,
    type_section::TypeSection, ModuleImage,
};

use crate::{
    datas::{ReadOnlyDatas, ReadWriteDatas, UninitDatas},
    indexed_memory::IndexedMemory,
};

pub struct ModuleInstance<'a> {
    pub name: String, // &'a str,
    pub type_section: TypeSection<'a>,
    pub local_variable_section: LocalVariableSection<'a>,
    pub function_section: FunctionSection<'a>,
    pub datas: [Box<dyn IndexedMemory + 'a>; 3],
    pub function_name_section: FunctionNameSection<'a>,
    pub data_name_section: DataNameSection<'a>,
}

impl<'a> ModuleInstance<'a> {
    pub fn new(module_image: &'a ModuleImage<'a>) -> Self {
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
                let length = section
                    .items
                    .iter()
                    .map(|item| item.data_length)
                    .sum::<u32>();

                UninitDatas::new(section.items, vec![0u8; length as usize])
            },
        );

        let function_name_section = module_image
            .get_optional_function_name_section()
            .unwrap_or_default();

        let data_name_section = module_image
            .get_optional_data_name_section()
            .unwrap_or_default();

        let property_section =module_image.get_property_section();
        let name_bytes = property_section.module_name_buffer[0..property_section.module_name_length as usize].to_vec();
        let name = String::from_utf8(name_bytes).unwrap();

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
