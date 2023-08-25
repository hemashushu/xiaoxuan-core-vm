// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_binary::{
    module_image::{
         func_section::FuncSection, type_section::TypeSection,
        ModuleImage,
    },
    utils::downcast_section_entry,
};
use ancvm_types::SectionId;

use crate::context::{Context, Module};

pub fn build_context<'a>(module_images: &'a [ModuleImage<'a>]) -> Context<'a> {
    // let m = &module_images[0];

    // let modules = module_images.iter().map(|m| {

    //     // let type_section = ;
    //     // let func_section = ;
    // });

    // Context { module_index_section: (), data_index_section: (), func_index_section: (), modules: () }
    todo!()
}

fn build_module<'a>(module_image: &'a ModuleImage<'a>) -> Module<'a> {
    // let data_sections = [
    //     &DataSection{data: &vec![]},
    //     &DataSection{data: &vec![]},
    //     &DataSection{data: &vec![]},
    // ];

    // let t = module_image
    //     .get_section_entry_by_id(SectionId::Type)
    //     .expect("Section \"type\" could not be found.")
    //     .as_ref();
    // let k = downcast_section_entry::<TypeSection>(t);

    // let module = Module {
    //     // data_sections,
    //     type_section: k,
    //     // func_section: downcast_section_entry::<FuncSection>(
    //     //     module_image.get_section_entry_by_id(SectionId::Func)
    //     //         .expect("Section \"type\" could not be found.")
    //     //         .as_ref(),
    //     // ),
    // };
todo!()

}
