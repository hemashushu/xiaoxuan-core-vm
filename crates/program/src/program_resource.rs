// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_binary::BinaryError;

use crate::{program_context::ProgramContext, ProgramResourceType};

// all asserts needed for a program to run.
// includes settings, module images and other essential objects.
pub trait ProgramResource {
    fn build_program_context(&self) -> Result<ProgramContext, BinaryError>;
    fn get_type(&self) -> ProgramResourceType;
}
