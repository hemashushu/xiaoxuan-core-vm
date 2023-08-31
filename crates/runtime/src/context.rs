// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_binary::module_image::{
    data_index_section::{DataIndexItem, DataIndexOffset, DataIndexSection},
    data_section::DataItem,
    func_index_section::FuncIndexSection,
    func_section::FuncSection,
    module_index_section::ModuleIndexSection,
    type_section::TypeSection,
    ModuleImage, SectionId,
};

use crate::{
    datas::{ReadOnlyDatas, ReadWriteDatas, UninitDatas},
    indexed_memory::IndexedMemory,
};

const EMPTY_DATA: &[u8] = &[];
const EMPTY_DATA_ITEMS: &[DataItem] = &[];
const EMPTY_DATA_INDEX_ITEMS: &[DataIndexItem] = &[];
const EMPTY_DATA_INDEX_OFFSETS: &[DataIndexOffset] = &[];

pub struct Context<'a> {
    // the indices
    pub module_index_section: ModuleIndexSection<'a>,
    pub data_index_section: DataIndexSection<'a>,
    pub func_index_section: FuncIndexSection<'a>,

    // the modules
    pub modules: Vec<Module<'a>>,
}

pub struct Module<'a> {
    pub datas: [Box<dyn IndexedMemory + 'a>; 3],

    pub type_section: TypeSection<'a>,
    pub func_section: FuncSection<'a>,
}

impl<'a> Context<'a> {
    pub fn new(module_images: &'a [ModuleImage<'a>]) -> Self {
        let modules = module_images
            .iter()
            .map(|image| Module::new(image))
            .collect::<Vec<Module>>();

        let main_module = &module_images[0];

        let module_index_section = main_module.get_module_index_section();
        let func_index_section = main_module.get_func_index_section();
        let data_index_section =
            if let Some(_) = main_module.get_section_index_by_id(SectionId::DataIndex) {
                main_module.get_data_index_section()
            } else {
                DataIndexSection {
                    items: EMPTY_DATA_INDEX_ITEMS,
                    offsets: EMPTY_DATA_INDEX_OFFSETS,
                }
            };

        Self {
            module_index_section,
            data_index_section,
            func_index_section,
            modules,
        }
    }
}

impl<'a> Module<'a> {
    pub fn new(module_image: &'a ModuleImage<'a>) -> Self {
        let (read_only_data_items, read_only_datas) =
            if let Some(_) = module_image.get_section_index_by_id(SectionId::ReadOnlyData) {
                let section = module_image.get_read_only_data_section();
                (section.items, section.datas)
            } else {
                (EMPTY_DATA_ITEMS, EMPTY_DATA)
            };

        let (read_write_data_items, read_write_datas) =
            if let Some(_) = module_image.get_section_index_by_id(SectionId::ReadWriteData) {
                let section = module_image.get_read_write_data_section();
                (section.items, section.datas.to_vec())
            } else {
                (EMPTY_DATA_ITEMS, Vec::<u8>::new())
            };

        let (uninit_data_items, uninit_datas) =
            if let Some(_) = module_image.get_section_index_by_id(SectionId::UninitData) {
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

        let read_only_memory = ReadOnlyDatas::new(read_only_data_items, read_only_datas);
        let read_write_memory = ReadWriteDatas::new(read_write_data_items, read_write_datas);
        let uninit_memory = UninitDatas::new(uninit_data_items, uninit_datas);

        let type_section = module_image.get_type_section();
        let func_section = module_image.get_func_section();

        Self {
            datas: [
                Box::new(read_only_memory),
                Box::new(read_write_memory),
                Box::new(uninit_memory),
            ],
            type_section,
            func_section,
        }
    }
}
