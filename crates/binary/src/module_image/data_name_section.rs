// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

#[derive(Debug, PartialEq)]
pub struct DataNameEntry {
    pub name: String,
    pub data_pub_index: usize,
    pub exported: bool,
}

impl DataNameEntry {
    pub fn new(name: String, data_pub_index: usize, exported: bool) -> Self {
        Self {
            name,
            data_pub_index,
            exported,
        }
    }
}
