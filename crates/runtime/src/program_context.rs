// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_binary::module_image::ModuleImage;
use ancvm_thread::thread_context::ThreadContext;

pub struct ProgramContext<'a> {
    module_images: Vec<ModuleImage<'a>>,
}

impl<'a> ProgramContext<'a> {
    pub fn new(module_images: Vec<ModuleImage<'a>>) -> Self {
        Self { module_images }
    }

    pub fn new_thread_context(&'a self) -> ThreadContext<'a> {
        ThreadContext::new(&self.module_images)
    }
}
