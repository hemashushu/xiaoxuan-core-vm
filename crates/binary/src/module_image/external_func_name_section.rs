// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

#[derive(Debug, PartialEq)]
pub struct ExternalFuncNameEntry {
    pub name: String,
    pub external_func_index: usize,
}

impl ExternalFuncNameEntry {
    pub fn new(name: String, external_func_index: usize) -> Self {
        Self {
            name,
            external_func_index,
        }
    }
}
