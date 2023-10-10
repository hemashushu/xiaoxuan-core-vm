// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_program::thread_context::ThreadContext;

pub fn extcall(thread_context: &mut ThreadContext) {
    // `fn (external_func_index:i32)`
    //
    // the 'external_func_index' is the index within a module, it is not
    // the 'unified_external_func_index'.

    let external_func_index = thread_context.stack.pop_i64_u();
    let module_index = thread_context.pc.module_index;

    let (uni_ext_library_index, uni_ext_function_index) = {
        // let table = thread_context.program_context.


        (0,0)
    };

    // table.get_external_function_pointer_and_wrapper_function()

    //
    // the syscall arguments should be pushed on the stack first, e.g.
    //
    // | params_count   |
    // | syscall_num    |
    // | arg6           |
    // | arg5           |
    // | arg4           |
    // | arg3           |
    // | arg2           |                  | error no       |
    // | arg1           |     return -->   | return value   |
    // | ...            |                  | ...            |
    // \----------------/ <-- stack start  \----------------/
    //
    todo!()
}
