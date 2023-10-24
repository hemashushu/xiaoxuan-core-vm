// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

pub trait ResizeableMemory {
    fn get_capacity_in_pages(&self) -> usize;
    fn resize(&mut self, new_size_in_pages: usize) -> usize;
}
