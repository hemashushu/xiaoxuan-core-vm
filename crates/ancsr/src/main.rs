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
//   |-- cache.anon (the cache infomations, such as the last modified time and content hash of script file)
//   |-- main.ancbc
//   |-- dependency1.ancbc
//   |-- dependencyN.ancbc
//
//
// note that apart from the script files, a script application may also contains resource files
// and (dynamically linked) shared libraries.
// these files will stay in their original location and will not be copied to the cache directory.
//
// app source file dir
//   |-- module.anon (the application description file, similar to the Nodejs's package.json
//   |                and the Rust's Cargo.toml)
//   |-- main.ancs (the main module script file)
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
//   |     |-- shared-library.so
//   |     |-- ...

// to launch a script application which is a single file:
// - `$ ancs /path/to/single-file-app.ancs`
// - `$ /path/to/single-file-app.ancs`
// - `$ single-file-app.ancs` (which path is in the environment variable PATH)
//
// to lanuch a script application which is consist of multiple script files:
// - `$ ancs /path/to/app-source-dir`
// - `$ ancs /path/to/app-source-dir/app-name.ancs`
// - `$ /path/to/app-source-dir/app-name.ancs`
// - `$ app-name.ancs` (a symblic link to '/path/to/app-source-dir/app-name.ancs')

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
