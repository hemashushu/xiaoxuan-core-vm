// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::path::PathBuf;

use crate::capability::Capability;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramSourceType {
    // Represents a module.
    Module,
    // Represents a script file.
    ScriptFile,
    // Represents a program from memory.
    Memory,
    // Represents a package image.
    PackageImage,
}

#[derive(Debug, Clone)]
pub struct ProcessProperty {
    // The path to the application.
    // This can be:
    // - A directory path for a module.
    // - A file path for a package image.
    // - A file path for a script file.
    //
    // Examples:
    // - `/path/to/projects/hello-world`     (a module directory)
    // - `/path/to/scripts/hello-world.anc`  (source code file)
    // - `/path/to/scripts/hello-world.ancr` (intermediate representation file)
    // - `/path/to/scripts/hello-world.anca` (assembly file)
    // - `/path/to/package/hello-world.ancp` (package image file)
    pub program_path: PathBuf,

    // Indicates the type of application source (e.g., module, package image, or script file).
    pub program_source_type: ProgramSourceType,

    // The arguments passed to the program.
    pub arguments: Vec<String>,

    // The environment variables for the program.
    pub environments: Vec<(String, String)>,

    /// The capability of the process, which defines what operations it can perform.
    pub capability: Capability,
}

impl ProcessProperty {
    pub fn new(
        program_path: PathBuf,
        program_source_type: ProgramSourceType,
        arguments: Vec<String>,
        environments: Vec<(String, String)>,
        capability: Capability,
    ) -> Self {
        Self {
            program_path: program_path.to_owned(),
            program_source_type,
            arguments,
            environments,
            capability,
        }
    }
}

impl Default for ProcessProperty {
    fn default() -> Self {
        Self {
            // Default program path is the current directory.
            program_path: PathBuf::from("."),
            // Default source type is a module.
            program_source_type: ProgramSourceType::Module,
            // Default arguments are an empty list.
            arguments: Vec::new(),
            // Default environment variables are an empty map.
            environments: Vec::new(),
            // Default capability is an empty capability.
            capability: Capability::default(),
        }
    }
}
