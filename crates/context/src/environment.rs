// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

#[derive(Default, Clone)]
pub struct Environment {
    pub source_path: String, // the application source path, e.g. `~/projects/hello-world`, or `~/scripts/hello-world.anc`
    pub is_directory: bool, // to indicate the source path is a directory

    // the user's share library and module repo, e.g.
    // the `~/.anc` of
    // `~/.anc/1.0/modules/foo/1.0.1/{src,target}` and
    // `~/.anc/1.0/libraries/bar/1.0.2/{lib,include}`
    // note the this path does not includes the runtime version.
    pub share_path: String,

    // the runtime's path, e.g.
    // the `/usr/lib/anc/` of
    // `/usr/lib/anc/1.0/modules/http-client/1.0.1/{src, target}`
    pub runtime_path: String,
}

impl Environment {
    pub fn new(
        source_path: &str,
        is_directory: bool,
        share_path: &str,
        runtime_path: &str,
    ) -> Self {
        Self {
            source_path: source_path.to_owned(),
            is_directory,
            share_path: share_path.to_owned(),
            runtime_path: runtime_path.to_owned(),
        }
    }
}
