// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct ProcessProperty {
    // the application path
    // it may be a file path of a script file,
    // also a directory path of a module. e.g.
    //
    // - `~/scripts/hello-world.anc`  (source code)
    // - `~/scripts/hello-world.ancr` (IR)
    // - `~/scripts/hello-world.anca` (ASM)
    // - `~/scripts/hello-world.anci` (image)
    // - `~/projects/hello-world`     (the folder of a module)
    //
    // when a user launcher an application by file path,
    // the runtime should check it and its parent folders to determine
    // whether the file is part of a module.
    pub application_path: String,

    // to indicate the application is single-file script.
    pub is_script: bool,

    // program arguments
    pub arguments: Vec<String>,

    // environment variables
    pub environments: HashMap<String, String>,
}

impl ProcessProperty {
    pub fn new(
        application_path: &str,
        is_script: bool,
        arguments: Vec<String>,
        environments: HashMap<String, String>,
    ) -> Self {
        Self {
            application_path: application_path.to_owned(),
            is_script,
            arguments,
            environments,
        }
    }
}
