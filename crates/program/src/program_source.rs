// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_binary::BinaryError;

use crate::{program::Program, ProgramSourceType};

pub trait ProgramSource {
    fn build_program(&self) -> Result<Program, BinaryError>;
    fn get_source_type(&self) -> ProgramSourceType;
}
