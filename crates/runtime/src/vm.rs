// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::process::Proccess;

pub struct VM {
    pub memory: Vec<u8>,
    pub processes: Vec<Proccess>,
}
