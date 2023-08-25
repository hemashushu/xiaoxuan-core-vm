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

use crate::vm::VM;

const EMPTY_DATA: &[u8] = &[];
const EMPTY_DATA_ITEMS: &[DataItem] = &[];
const EMPTY_DATA_INDEX_ITEMS: &[DataIndexItem] = &[];
const EMPTY_DATA_INDEX_OFFSETS: &[DataIndexOffset] = &[];

pub struct Context<'a> {
    pub vm: VM,

    pub module_index_section: ModuleIndexSection<'a>,
    pub data_index_section: DataIndexSection<'a>,
    pub func_index_section: FuncIndexSection<'a>,
    pub modules: Vec<Module<'a>>,
}

pub struct Module<'a> {
    pub data_items: [&'a [DataItem]; 3],
    pub read_only_datas: &'a [u8],
    pub read_write_datas: Vec<u8>,
    pub uninit_datas: Vec<u8>,

    pub type_section: TypeSection<'a>,
    pub func_section: FuncSection<'a>,
}

pub fn build_context<'a>(module_images: &'a [ModuleImage<'a>]) -> Context<'a> {
    let modules = module_images
        .iter()
        .map(|image| build_module(image))
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

    let vm = VM::new();

    Context {
        vm,
        module_index_section,
        data_index_section,
        func_index_section,
        modules,
    }
}

pub fn build_module<'a>(module_image: &'a ModuleImage<'a>) -> Module<'a> {
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

    let type_section = module_image.get_type_section();
    let func_section = module_image.get_func_section();

    Module {
        data_items: [
            read_only_data_items,
            read_write_data_items,
            uninit_data_items,
        ],
        read_only_datas,
        read_write_datas,
        uninit_datas,
        type_section,
        func_section,
    }
}

#[cfg(test)]
mod tests {
    use ancvm_binary::module_image::{
        type_section::{TypeEntry, TypeSection},
        ModuleImage,
    };
    use ancvm_types::DataType;

    fn build_module_binary_with_single_function(
        params: Vec<DataType>,
        results: Vec<DataType>,
        codes: Vec<u8>,
    ) -> Vec<u8> {
        // build type section
        let mut type_entries: Vec<TypeEntry> = Vec::new();
        type_entries.push(TypeEntry {
            params: params,
            results: results,
        });
        let (type_items, types_data) = TypeSection::convert_from_entries(&type_entries);
        let type_section = TypeSection {
            items: &type_items,
            types_data: &types_data,
        };

        // build function section

        //         let mut func_entries: Vec<FuncEntry> = Vec::new();
        //         let code0: Vec<u8> = vec![1u8, 2, 3, 5, 7];
        //         let code1: Vec<u8> = vec![11u8, 13, 17, 19, 23, 29];
        //
        //         func_entries.push(FuncEntry {
        //             func_type: 0,
        //             code: &code0,
        //         });
        //         func_entries.push(FuncEntry {
        //             func_type: 1,
        //             code: &code1,
        //         });
        //
        //         let (func_items, codes_data) = FuncSection::convert_from_entries(&func_entries);
        //         let func_section = FuncSection {
        //             items: &func_items,
        //             codes_data: &codes_data,
        //                 };

        todo!()
    }
}
