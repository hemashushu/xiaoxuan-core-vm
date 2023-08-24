// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_binary::module_image::{data_section::DataSection, ModuleImage};

use crate::context::{Context, Module};

pub fn build_context<'a>(module_images: &'a [ModuleImage<'a>]) -> Context<'a> {
    // let modules = module_images.iter()
    //     .map(|m| {
    //         let data_sections = [
    //             DataSection{data: &vec![]},
    //             DataSection{data: &vec![]},
    //             DataSection{data: &vec![]},
    //         ];
    //         Module{
    //             data_sections,
    //             type_section: m.,
    //             func_section: todo!()
    //         }
    //     });

    // Context { module_index_section: (), data_index_section: (), func_index_section: (), modules: () }
    todo!()
}
