// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// about the callback function
//
// on the XiaoXuan Core Script application, pass VM function as a callback function to the external C library.
//
//                                      runtime (native)
//                                   /------------------------\
//                                   |                        |
//                                   | external func list     |
//                                   | |--------------------| |
//  XiaoXuan core application        | | idx | lib  | name  | |
// /------------------------\   /--> | | 0   | ".." | ".."  | | --\
// |                        |   |    | |--------------------| |   |
// | fn $demo () -> ()      |   |    |                        |   |
// |   extcall do_something | --/    | callback func table    |   |
// | end                    |        | |--------------------| |   |
// |                        |        | | mod idx | func idx | |   |      libxyz.so
// | fn $callback () -> ()  | <----- | | 0       | 0        | |   |    /----------------------\
// |   ...                  |        | | ...     | ...      | |   \--> | void do_something (  |
// | end                    |        | |--------------------| |        |     void* () cb) {   |
// |                        |        |                        |        |     ...              |
// \------------------------/        | callback func code 0   | <----- |     (cb)(11, 13)     |
//                                   | 0x0000 0xb8, 0x34,     |        | }                    |
//                                   | 0x0000 0x12, 0x00...   |        |                      |
//                                   |                        |        \----------------------/
//                                   | callback func code 1   |
//                                   | ...                    |
//                                   |                        |
//                                   \------------------------/
//

use ancvm_thread::thread_context::ThreadContext;

use crate::{
    interpreter::process_callback_function_call, jit_util::build_host_to_vm_delegate_function,
};

fn get_callback_function<T>(
    thread_context: &mut ThreadContext,
    target_module_index: usize,
    function_internal_index: usize,
) -> Result<T, &'static str> {
    // check if the specified (target_module_index, function_internal_index) already
    // exists in the callback function table
    let opt_callback_function_ptr =
        thread_context.find_callback_function(target_module_index, function_internal_index);

    if let Some(callback_function_ptr) = opt_callback_function_ptr {
        return Ok(unsafe { std::mem::transmute_copy::<*const u8, T>(&callback_function_ptr) });
    }

    let type_index = thread_context.program_ref.modules[target_module_index]
        .func_section
        .items[function_internal_index]
        .type_index;
    let (params, results) = thread_context.program_ref.modules[target_module_index]
        .type_section
        .get_params_and_results_list(type_index as usize);

    if results.len() > 1 {
        return Err("The number of return values of the specified function is more than 1.");
    }

    let delegate_function_addr = process_callback_function_call as *const u8 as usize;
    let thread_context_addr = thread_context as *const ThreadContext as *const u8 as usize;
    let callback_function_ptr = build_host_to_vm_delegate_function(
        delegate_function_addr,
        thread_context_addr,
        target_module_index,
        function_internal_index,
        params,
        results,
    );

    // store the function pointer into table
    thread_context.insert_callback_function(
        target_module_index,
        function_internal_index,
        callback_function_ptr,
    );

    Ok(unsafe { std::mem::transmute_copy::<*const u8, T>(&callback_function_ptr) })
}
