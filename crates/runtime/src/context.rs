// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

const EMPTY_DATA: &[u8] = &[];
const EMPTY_DATA_ITEMS: &[DataItem] = &[];

use ancvm_binary::module_image::{
    data_index_section::DataIndexSection, data_section::DataItem,
    func_index_section::FuncIndexSection, func_section::FuncSection,
    module_index_section::ModuleIndexSection, type_section::TypeSection,
};

use crate::vm::VM;

pub struct Context<'a> {
    // in XiaoXuan VM, the data sections (read-only, read-write, uninit) are all thread-local,
    // and the heap is thread-local also.
    // threads/processes can communicated through the MessageBox/MessagePipe or the SharedMemory
    //
    // note that the initial capacity of heap is 0 byte
    pub heap: Vec<u8>,

    pub module_index_section: ModuleIndexSection<'a>,
    pub data_index_section: DataIndexSection<'a>,
    pub func_index_section: FuncIndexSection<'a>,

    pub modules: Vec<Module<'a>>,

    pub vm: VM,
}

pub struct Module<'a> {
    pub data_items: [&'a DataItem; 3],
    pub read_only_datas: &'a [u8],
    pub read_write_datas: Vec<u8>,
    pub uninit_datas: Vec<u8>,
    pub type_section: TypeSection<'a>,
    pub func_section: FuncSection<'a>,
}
