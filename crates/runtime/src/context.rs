// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_binary::module_image::{
    data_index_section::DataIndexSection, data_section::DataSection,
    func_index_section::FuncIndexSection, func_section::FuncSection,
    module_index_section::ModuleIndexSection, type_section::TypeSection,
};

pub struct Context<'a> {
    pub module_index_section: ModuleIndexSection<'a>,
    pub data_index_section: DataIndexSection<'a>,
    pub func_index_section: FuncIndexSection<'a>,
    pub modules: &'a [Module<'a>],
}

pub struct Module<'a> {
    pub data_sections: [DataSection<'a>; 3],
    pub type_section: TypeSection<'a>,
    pub func_section: FuncSection<'a>,
}
