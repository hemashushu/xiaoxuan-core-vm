// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct Environment {
    // the application path
    // it may be a file path of a script or image file,
    // also a directory path of a module. e.g.
    //
    // - `~/scripts/hello-world.anc`
    // - `~/scripts/hello-world.anci`
    // - `~/projects/hello-world`
    pub application_path: String,

    // to indicate the application is a module (i.e. not a script file or image)
    // this field is used to determine if 'application_path' is a folder.
    pub is_module: bool,

    // environment variables
    pub variables: HashMap<String, String>,
}

impl Environment {
    pub fn new(
        application_path: &str,
        is_module: bool,
        // remote_cache_path: &str,
        // share_cache_paths: &[&str],
        // runtime_path: &str,
        variables: HashMap<String, String>,
    ) -> Self {
        Self {
            application_path: application_path.to_owned(),
            is_module,
            // remote_cache_path: remote_cache_path.to_owned(),
            // share_cache_paths: share_cache_paths.iter().map(|p| p.to_string()).collect(),
            // runtime_path: runtime_path.to_owned(),
            variables
        }
    }
}
