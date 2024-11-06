// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
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

    pub variables: HashMap<String, String>,

    // the cache folder for the remote shared modules and libraries, e.g.
    // `~/.cache/.anc`
    pub remote_cache_path: String,

    // the local folder for storing the shared modules and libraries which
    // comes from repository, e.g.
    //
    // `~/.anc`
    //
    // note the this path SHOULD NOT include the runtime version number.
    //
    // thus the computed shared module and libraries path are:
    // - `~/.anc/1.0/modules/foo/1.0.1/{src,target}`
    // - `~/.anc/1.0/libraries/bar/1.0.2/{lib,include}`
    //
    // multiple paths are allowed, by default there are 3 paths:
    // - `~/.anc` for users to install from the central repository
    // - `~/.cache/.anc-test` for developers to install the local module (`$ ancm install-local ...`) for testing
    // - `/usr/lib/anc` for system (OS) package managers
    pub share_cache_paths: Vec<String>,

    // the runtime's path, e.g.
    //
    // `/usr/lib/anc`
    //
    // note the this path SHOULD NOT include the runtime version number.
    //
    // thus the computed bulitin modules and libraries path are:
    // - `/usr/lib/anc/1.0/runtime/modules/http-client/{src, target}`
    // - `/usr/lib/anc/1.0/runtime/libraries/lz4/{lib, include}`
    pub runtime_path: String,
}

impl Environment {
    pub fn new(
        application_path: &str,
        is_module: bool,
        remote_cache_path: &str,
        share_cache_paths: &[&str],
        runtime_path: &str,
        variables: HashMap<String, String>,
    ) -> Self {
        Self {
            application_path: application_path.to_owned(),
            is_module,
            remote_cache_path: remote_cache_path.to_owned(),
            share_cache_paths: share_cache_paths.iter().map(|p| p.to_string()).collect(),
            runtime_path: runtime_path.to_owned(),
            variables
        }
    }
}
