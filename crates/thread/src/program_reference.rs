// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_binary::module_image::{
    data_index_section::{DataIndexItem, DataIndexSection},
    data_section::DataItem,
    func_index_section::FuncIndexSection,
    func_section::FuncSection,
    local_variable_section::LocalVariableSection,
    type_section::TypeSection,
    ModuleImage, ModuleSectionId, RangeItem,
};

use crate::{
    datas::{ReadOnlyDatas, ReadWriteDatas, UninitDatas},
    indexed_memory::IndexedMemory,
};

const EMPTY_DATA: &[u8] = &[];
const EMPTY_DATA_ITEMS: &[DataItem] = &[];
const EMPTY_DATA_INDEX_ITEMS: &[DataIndexItem] = &[];
const EMPTY_DATA_INDEX_OFFSETS: &[RangeItem] = &[];

pub struct ProgramReference<'a> {
    // the indices
    pub data_index_section: DataIndexSection<'a>,
    pub func_index_section: FuncIndexSection<'a>,

    // the modules
    pub modules: Vec<Module<'a>>,
}

pub struct Module<'a> {
    pub name: &'a [u8],
    pub datas: [Box<dyn IndexedMemory + 'a>; 3],
    pub type_section: TypeSection<'a>,
    pub local_variable_section: LocalVariableSection<'a>,
    pub func_section: FuncSection<'a>,
}

impl<'a> ProgramReference<'a> {
    pub fn new(module_images: &'a [ModuleImage<'a>]) -> Self {
        let modules = module_images
            .iter()
            .map(Module::new)
            .collect::<Vec<Module>>();

        let main_module = &module_images[0];

        let func_index_section = main_module.get_func_index_section();
        let data_index_section =
            if let Some(_idx) = main_module.get_section_index_by_id(ModuleSectionId::DataIndex) {
                main_module.get_data_index_section()
            } else {
                DataIndexSection {
                    items: EMPTY_DATA_INDEX_ITEMS,
                    ranges: EMPTY_DATA_INDEX_OFFSETS,
                }
            };

        Self {
            data_index_section,
            func_index_section,
            modules,
        }
    }
}

impl<'a> Module<'a> {
    pub fn new(module_image: &'a ModuleImage<'a>) -> Self {
        let (read_only_data_items, read_only_datas_data) = if let Some(_idx) =
            module_image.get_section_index_by_id(ModuleSectionId::ReadOnlyData)
        {
            let section = module_image.get_read_only_data_section();
            (section.items, section.datas_data)
        } else {
            (EMPTY_DATA_ITEMS, EMPTY_DATA)
        };

        let (read_write_data_items, read_write_datas_data) = if let Some(_idx) =
            module_image.get_section_index_by_id(ModuleSectionId::ReadWriteData)
        {
            let section = module_image.get_read_write_data_section();
            (section.items, section.datas_data.to_vec())
        } else {
            (EMPTY_DATA_ITEMS, Vec::<u8>::new())
        };

        let (uninit_data_items, uninit_datas_data) =
            if let Some(_idx) = module_image.get_section_index_by_id(ModuleSectionId::UninitData) {
                let section = module_image.get_uninit_data_section();
                // calculate the total data size of this section.
                let length = section
                    .items
                    .iter()
                    .map(|item| item.data_length)
                    .sum::<u32>();

                (section.items, vec![0u8; length as usize])
            } else {
                (EMPTY_DATA_ITEMS, Vec::<u8>::new())
            };

        let read_only_data = ReadOnlyDatas::new(read_only_data_items, read_only_datas_data);
        let read_write_data = ReadWriteDatas::new(read_write_data_items, read_write_datas_data);
        let uninit_data = UninitDatas::new(uninit_data_items, uninit_datas_data);

        let type_section = module_image.get_type_section();
        let func_section = module_image.get_func_section();
        let local_variable_section = module_image.get_local_variable_section();

        Self {
            name: module_image.name,
            datas: [
                Box::new(read_only_data),
                Box::new(read_write_data),
                Box::new(uninit_data),
            ],
            type_section,
            func_section,
            local_variable_section,
        }
    }
}
