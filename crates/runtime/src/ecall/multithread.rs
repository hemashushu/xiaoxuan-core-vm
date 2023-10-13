// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_program::thread_context::ThreadContext;

use crate::CURRENT_THREAD_ID;

pub fn thread_id(thread_context: &mut ThreadContext) {
    // `fn () -> thread_id:u64`

    // the function `thread::id()` is unstable, so here implement ourself.
    //
    // ref:
    //
    // ```test
    // error[E0658]: use of unstable library feature 'thread_id_value'
    // #![feature(thread_id_value)]
    // ```

    CURRENT_THREAD_ID.with(|id_cell| {
        let id = *id_cell.borrow();
        thread_context.stack.push_i32_u(id);
    });
}
