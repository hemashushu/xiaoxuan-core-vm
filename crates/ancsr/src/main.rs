// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// a script application may consist of a single script file, or several script files.
// in either case, these script files will be compiled into a single module image file
// named 'main.ancbc', and this file will be copied to the 'application cache' directory.
//
// the dependent modules of application are copied to this cache directory also, but
// the shared modules (such as the standard library) are located at the runtime directory, and
// they will not be copied to this directory.
//
// the structure of application cache directory
//
// app cache dir
//   |-- cache.index
//   |   the cache infomations, such as the file type, version (dependencies only),
//   |   and the last modified time, content hash (only the application script files), and the module list
//   |
//   |-- main_module.ancbc
//   |-- dependency_module1.ancbc
//   |-- ...
//   |-- dependency_moduleN.ancbc
//
// all module files and the cache dir itself are named with:
// - `dev_num_major + '_' + dev_num_minor + '_' + inode_num`, or
// - the sha256 of the full path.
//
// note that apart from the script files, a (multiple scripts) application may also contains resource files
// and (dynamically linked) shared libraries.
// these files will stay in their original location and will not be copied to the cache directory.
//
// app source file dir
//   |-- module.anon
//   |   the application description file, similar to the Nodejs's package.json
//   |   and the Rust's Cargo.toml
//   |
//   |-- main.ancs
//   |   the main module script file, the first line would be '#!/bin/ancs -d' (ref: https://en.wikipedia.org/wiki/Shebang_(Unix))
//   |
//   |-- sub-module.ancs
//   |-- sub-dir
//   |     |-- sub-module.ancs
//   |     |-- ...
//   |
//   |-- resources
//   |     |-- images_etc
//   |     |-- ...
//   |
//   |-- lib
//   |     |-- shared-library.so -> shared-library.so.1.0.0
//   |     |-- ...

// to launch a script application which is a single file:
// - `$ ancs /path/to/single-file-app.ancs`
// - `$ /path/to/single-file-app.ancs`
// - `$ single-file-app.ancs` (which path is in the environment variable PATH)
// - `$ [/usr/bin/]single-file-app` (a symbolic link to '/path/to/single-file-app.ancs')
//
// to lanuch a script application which is consist of multiple script files:
// - `$ ancs /path/to/app-source-dir`
// - `$ ancs /path/to/app-source-dir/main.ancs`
// - `$ /path/to/app-source-dir/main.ancs`
// - `$ [/usr/bin/]app-name` (a symblic link to '/path/to/app-source-dir/main.ancs')

fn main() {
    println!("Hello");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_test() {
        //
    }
}
