// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

#[derive(Default, Clone)]
pub struct Environment {
    // the application source path, e.g.
    // - `~/projects/hello-world`
    // - `~/scripts/hello-world.anc`
    pub source_path: String,

    // to indicate the source path is a directory
    pub is_directory: bool,

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
    // multiple path is allowed
    pub share_paths: Vec<String>,

    // the runtime's path, e.g.
    //
    // `/usr/lib/anc/`
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
        source_path: &str,
        is_directory: bool,
        remote_cache_path: &str,
        share_paths: &[&str],
        runtime_path: &str,
    ) -> Self {
        Self {
            source_path: source_path.to_owned(),
            is_directory,
            remote_cache_path: remote_cache_path.to_owned(),
            share_paths: share_paths.iter().map(|p| p.to_string()).collect(),
            runtime_path: runtime_path.to_owned(),
        }
    }
}
