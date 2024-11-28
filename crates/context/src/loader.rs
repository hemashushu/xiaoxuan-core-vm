// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use anc_isa::{ModuleDependency, ModuleDependencyType};

use crate::ContextError;

pub trait Loader<'a> {
    fn load_application(
        module_type: ModuleDependencyType,
        name_or_path: &str,
    ) -> Result<Vec<&'a [u8]>, ContextError>;

    fn load_module(module_value: ModuleDependency) -> Result<&'a [u8], ContextError>;

    fn compile_module(module_value: ModuleDependency) -> Result<&'a [u8], ContextError>;
}
