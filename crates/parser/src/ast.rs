// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ModuleNode {
    pub name: String,

    pub runtime_version_major: u16,
    pub runtime_version_minor: u16,

    pub shared_packages: Vec<String>,
}