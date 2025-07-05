// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

// Callback Delegate Function
// -------------------------
//
// Callback delegate functions allow XiaoXuan Core VM functions to be passed as callbacks
// to external C/Rust libraries via the 'extcall' instruction.
//
// The following diagram illustrates how a XiaoXuan Core Application invokes external functions
// and how external functions can invoke VM callback functions:
//
// ```diagram
//                                      XiaoXuan Core Runtime
//                                   /------------------------\
//                                   |                        |
//                                   | external func list     |
//                                   | |--------------------| |
//                                   | | idx | lib  | name  | |
//                              /--> | | 0   | ".." | ".."  | |
//                              |    | |--------------------| |
//                              |    |                        |
//                              |    | wrapper func code 0    |
//  XiaoXuan Core Application   |    | 0x0000 0xb8, 0x34,     |
// /------------------------\   |    | 0x000a 0x12, 0x00...   | --\
// |                        |   |    |                        |   |
// | fn $demo () -> ()      |   |    |                        |   |
// |   extcall do_something | --/    | callback func table    |   |
// | end                    |        | |--------------------| |   |      External Library,
// |                        |        | | mod idx | func idx | |   |      such as `libxyz.so`
// | fn $callback () -> ()  | <----- | | 0       | 0        | |   |    /----------------------\
// |   ...                  |        | | ...     | ...      | |   \--> | void do_something (  |
// | end                    |        | |--------------------| |        |     void* () cb) {   |
// |                        |        |                        |        |     ...              |
// \------------------------/        | delegate func code 0   | <----- |     (cb)(11, 13)     |
//                                   | 0x0000 0xb8, 0x34,     |        | }                    |
//                                   | 0x000a 0x12, 0x00...   |        |                      |
//                                   |                        |        \----------------------/
//                                   | delegate func code 1   |
//                                   | ...                    |
//                                   |                        |
//                                   \------------------------/
// ```
//
// How the `extcall` instruction and callback delegate are executed
// ----------------------------------------------------------------
//
// ```diagram
// | module M, function A |             | external func |       | module N, function B |
// |----------------------|             |---------------|       |----------------------|
// |                      |    wrapper  |               | delegate                     |
// | 0x0000 push args     |    function |               | function                     |
// | 0x0004 push delegate |    |        |               |   |   |                      |
// | 0x0008 extcall idx -------O---------> 0x0000       | /-O----> 0x0000 inst_0       |
// |                    /-------------\ |  0x0004       | |     |  0x0004 inst_1       |
// |                    | |           | |  0x0008  -------/     |  0x0008 inst_2       |
// |                    | |           | |               |       |  0x000c inst_3       |
// |                    | |           | |               | /------- 0x0010 end          |
// | 0x0010 ...      <--/ |           | |               | |     |                      |
// |                      |           | |  0x000c  <------/     |----------------------|
// |                      |           \--- 0x0010       |
// |                      |             |---------------|
// ```
//
// Callback delegate functions are generated dynamically at runtime.
// For more details, see the `bridge_function_table` module.

pub struct CallbackDelegateFunctionTable {
    pub functions_by_modules: Vec<CallbackDelegateFunctionsByModule>,
}

impl CallbackDelegateFunctionTable {
    pub fn new() -> Self {
        Self {
            functions_by_modules: Vec::new(),
        }
    }
}

impl Default for CallbackDelegateFunctionTable {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CallbackDelegateFunctionsByModule {
    pub module_index: usize,
    pub callback_delegate_function_items: Vec<CallbackDelegateFunctionItem>,
}

pub struct CallbackDelegateFunctionItem {
    pub function_internal_index: usize,
    pub callback_delegate_function_ptr: *const u8,
}

impl CallbackDelegateFunctionTable {
    pub fn find_callback_delegate_function(
        &self,
        target_module_index: usize,
        target_function_internal_index: usize,
    ) -> Option<*const u8> {
        match self
            .functions_by_modules
            .iter()
            .find(|module_item| module_item.module_index == target_module_index)
        {
            Some(module_item) => module_item
                .callback_delegate_function_items
                .iter()
                .find(|callback_delegate_function_item| {
                    callback_delegate_function_item.function_internal_index
                        == target_function_internal_index
                })
                .map(|callback_delegate_function_item| {
                    callback_delegate_function_item.callback_delegate_function_ptr
                }),
            None => None,
        }
    }
}
