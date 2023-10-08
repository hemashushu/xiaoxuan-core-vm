// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

pub struct ProgramSettings {
    pub source_path: String,
    pub is_multiple_scripts: bool,
    pub cache_path: String,
    pub runtime_path: String,
}

impl ProgramSettings {
    pub fn new(
        source_path: &str,
        is_multiple_scripts: bool,
        cache_path: &str,
        runtime_path: &str,
    ) -> Self {
        Self {
            source_path: source_path.to_owned(),
            is_multiple_scripts,
            cache_path: cache_path.to_owned(),
            runtime_path: runtime_path.to_owned(),
        }
    }
}

impl Default for ProgramSettings {
    fn default() -> Self {
        Self {
            source_path: Default::default(),
            is_multiple_scripts: Default::default(),
            cache_path: Default::default(),
            runtime_path: Default::default(),
        }
    }
}
