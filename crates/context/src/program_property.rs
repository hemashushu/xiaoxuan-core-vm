// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramSourceType {
    // a module
    Module,
    // a package image
    PackageImage,
    // a script file
    ScriptFile,
}

#[derive(Debug, Clone)]
pub struct ProgramProperty {
    // the application path.
    // it can be a directory path of a module,
    // a file path of package image,
    // or a file path of a script file. e.g.
    //
    // - `~/projects/hello-world`     (a folder of module)
    // - `~/scripts/hello-world.anc`  (source code)
    // - `~/scripts/hello-world.ancr` (IR)
    // - `~/scripts/hello-world.anca` (assembly)
    // - `~/package/hello-world.ancp` (package image)
    //
    // when a user launcher an application by file path,
    // the runtime should check it and its parent folders to determine
    // whether the file is part of a module.
    pub program_path: PathBuf,

    // to indicate the application is single-file script.
    pub program_source_type: ProgramSourceType,

    // program arguments
    pub arguments: Vec<String>,

    // environment variables
    pub environments: HashMap<String, String>,
}

impl ProgramProperty {
    pub fn new(
        program_path: PathBuf,
        program_source_type: ProgramSourceType,
        arguments: Vec<String>,
        environments: HashMap<String, String>,
    ) -> Self {
        Self {
            program_path: program_path.to_owned(),
            program_source_type,
            arguments,
            environments,
        }
    }
}

impl Default for ProgramProperty {
    fn default() -> Self {
        Self {
            program_path: PathBuf::from("."),
            program_source_type: ProgramSourceType::Module,
            arguments: Vec::new(),
            environments: HashMap::new(),
        }
    }
}
