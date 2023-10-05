// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::cell::Cell;

use ancvm_binary::{load_modules_from_binaries, module_image::ModuleImage};
use ancvm_thread::thread::Thread;

use crate::program::Program;

pub struct InMemoryProgram<'a> {
    module_binaries: Vec<Vec<u8>>,
    module_images: Vec<ModuleImage<'a>>,
}

impl<'a> InMemoryProgram<'a> {
    pub fn new(module_binaries: Vec<Vec<u8>>) -> Self {
        let program = Self {
            module_binaries,
            module_images: vec![],
        };

        program
    }

    pub fn build_modules(&'a mut self) {
        let binaries_ref = self
            .module_binaries
            .iter()
            .map(|e| &e[..])
            .collect::<Vec<_>>();
        let mut module_images = load_modules_from_binaries(binaries_ref).unwrap();
        self.module_images.append(&mut module_images);
    }
}

impl<'a> Program<'a> for InMemoryProgram<'a> {
    fn new_thread(&'a mut self) -> ancvm_thread::thread::Thread<'a> {
        let images = &self.module_images;
        Thread::new(images)
    }
}
