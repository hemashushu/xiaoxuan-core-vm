// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_binary::module_image::{
    data_section::DataItem,
    external_func_section::{ExternalFuncItem, ExternalFuncSection},
    external_library_section::{ExternalLibraryItem, ExternalLibrarySection},
    func_section::FuncSection,
    local_variable_section::LocalVariableSection,
    type_section::TypeSection,
    ModuleImage,
};

use crate::{
    datas::{ReadOnlyDatas, ReadWriteDatas, UninitDatas},
    indexed_memory::IndexedMemory,
};

const EMPTY_DATA: &[u8] = &[];
const EMPTY_DATA_ITEMS: &[DataItem] = &[];
const EMPTY_EXTERNAL_LIBRARY_ITEMS: &[ExternalLibraryItem] = &[];
const EMPTY_EXTERNAL_FUNC_ITEMS: &[ExternalFuncItem] = &[];

pub struct ProgramModule<'a> {
    pub name: &'a str,
    pub type_section: TypeSection<'a>,
    pub func_section: FuncSection<'a>,
    pub local_variable_section: LocalVariableSection<'a>,
    pub datas: [Box<dyn IndexedMemory + 'a>; 3],
    pub external_library_section: ExternalLibrarySection<'a>,
    pub external_func_section: ExternalFuncSection<'a>,
}

impl<'a> ProgramModule<'a> {
    pub fn new(module_image: &'a ModuleImage<'a>) -> Self {
        let type_section = module_image.get_type_section();
        let func_section = module_image.get_func_section();
        let local_variable_section = module_image.get_local_variable_section();

        let read_only_data = module_image
            .get_optional_read_only_data_section()
            .map_or_else(
                || ReadOnlyDatas::new(EMPTY_DATA_ITEMS, EMPTY_DATA),
                |section| ReadOnlyDatas::new(section.items, section.datas_data),
            );

        let read_write_data = module_image
            .get_optional_read_write_data_section()
            .map_or_else(
                || ReadWriteDatas::new(EMPTY_DATA_ITEMS, Vec::<u8>::new()),
                |section| ReadWriteDatas::new(section.items, section.datas_data.to_vec()),
            );

        let uninit_data = module_image.get_optional_uninit_data_section().map_or_else(
            || UninitDatas::new(EMPTY_DATA_ITEMS, Vec::<u8>::new()),
            |section| {
                let length = section
                    .items
                    .iter()
                    .map(|item| item.data_length)
                    .sum::<u32>();

                UninitDatas::new(section.items, vec![0u8; length as usize])
            },
        );

        let external_library_section = module_image
            .get_optional_external_library_section()
            .unwrap_or(ExternalLibrarySection {
                items: EMPTY_EXTERNAL_LIBRARY_ITEMS,
                names_data: EMPTY_DATA,
            });

        let external_func_section =
            module_image
                .get_optional_external_func_section()
                .unwrap_or(ExternalFuncSection {
                    items: EMPTY_EXTERNAL_FUNC_ITEMS,
                    names_data: EMPTY_DATA,
                });

        Self {
            name: module_image.name,
            type_section,
            func_section,
            local_variable_section,
            datas: [
                Box::new(read_only_data),
                Box::new(read_write_data),
                Box::new(uninit_data),
            ],
            external_library_section,
            external_func_section,
        }
    }
}
