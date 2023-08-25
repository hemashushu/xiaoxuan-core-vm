// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use std::{
    any::Any,
    fmt::{Debug, Display},
};

pub mod context;
pub mod process;
pub mod stack;
pub mod vm;

pub trait VMErr: Debug + Display {
    fn as_any(&self) -> &dyn Any;
}
