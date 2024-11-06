// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_isa::{GenericError, ModuleDependentType, ModuleDependentValue};

pub trait Loader<'a> {
    fn load_application(
        module_type: ModuleDependentType,
        name_or_path: &str,
    ) -> Result<Vec<&'a [u8]>, GenericError>;

    // fn compile_application(
    //     module_type: ModuleDependentType,
    //     name_or_path: &str,
    // ) -> Result<&'a [u8], GenericError>;

    fn load_module(module_value: ModuleDependentValue) -> Result<&'a [u8], GenericError>;

    fn compile_module(module_value: ModuleDependentValue) -> Result<&'a [u8], GenericError>;
}
