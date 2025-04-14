// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

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

pub struct CallbackDelegateFunctionsByModule {
    pub module_index: usize,
    pub callback_delegate_function_items: Vec<CallbackDelegateFunctionItem>,
}

pub struct CallbackDelegateFunctionItem {
    pub function_internal_index: usize,
    pub callback_delegate_function_ptr: *const u8,
}

// //     pub fn find_bridge_function(
// //         &self,
// //         target_module_index: usize,
// //         function_internal_index: usize,
// //     ) -> Option<*const u8> {
// //         find_delegate_function(
// //             &self.bridge_function_module_items,
// //             target_module_index,
// //             function_internal_index,
// //         )
// //     }
// //
// //     pub fn find_bridge_callback_function(
// //         &self,
// //         target_module_index: usize,
// //         function_internal_index: usize,
// //     ) -> Option<*const u8> {
// //         find_delegate_function(
// //             &self.callback_function_entries,
// //             target_module_index,
// //             function_internal_index,
// //         )
// //     }
// //
// //     pub fn insert_bridge_function(
// //         &mut self,
// //         target_module_index: usize,
// //         function_internal_index: usize,
// //         bridge_function_ptr: *const u8,
// //     ) {
// //         insert_delegate_function(
// //             &mut self.bridge_function_module_items,
// //             target_module_index,
// //             function_internal_index,
// //             bridge_function_ptr,
// //         );
// //     }
// //
// // pub fn insert_callback_function(
// //     &mut self,
// //     target_module_index: usize,
// //     function_internal_index: usize,
// //     bridge_function_ptr: *const u8,
// // ) {
// //     insert_delegate_function(
// //         &mut self.callback_function_entries,
// //         target_module_index,
// //         function_internal_index,
// //         bridge_function_ptr,
// //     );
// // }

// fn find_delegate_function(
//     delegate_function_table: &[DelegateFunctionByModule],
//     target_module_index: usize,
//     function_internal_index: usize,
// ) -> Option<*const u8> {
//     match delegate_function_table
//         .iter()
//         .find(|module_item| module_item.target_module_index == target_module_index)
//     {
//         Some(module_item) => module_item
//             .delegate_function_items
//             .iter()
//             .find(|functione_item| {
//                 functione_item.target_function_internal_index == function_internal_index
//             })
//             .map(|function_item| function_item.delegate_function_ptr),
//         None => None,
//     }
// }
//
// fn insert_delegate_function(
//     delegate_function_table: &mut Vec<DelegateFunctionByModule>,
//     target_module_index: usize,
//     function_internal_index: usize,
//     bridge_function_ptr: *const u8,
// ) {
//     let idx_m = delegate_function_table
//         .iter()
//         .position(|module_item| module_item.target_module_index == target_module_index)
//         .unwrap_or_else(|| {
//             delegate_function_table.push(DelegateFunctionByModule {
//                 target_module_index,
//                 delegate_function_items: vec![],
//             });
//             delegate_function_table.len() - 1
//         });
//
//     let module_item = &mut delegate_function_table[idx_m];
//
//     module_item
//         .delegate_function_items
//         .push(DelegateFunctionItem {
//             target_function_internal_index: function_internal_index,
//             delegate_function_ptr: bridge_function_ptr,
//         })
// }
