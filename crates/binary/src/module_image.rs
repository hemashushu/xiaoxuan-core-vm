// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// a module is consists of a header and several sections:
//
// - function type section
//   the signature of the function, as well as the code block
// - import data section (optional)
// - data sections (optional)
//   there are 3 kind of data: read-only, read-write, uninit(ialized)
//   all data are thread-local, so the RW section will be cloned and the
//   uninit section will be allocated when a new thread was created
// - import function section (optional)
// - function section
//   a function is consists of a type, a local variable list, and the instructions
// - export data section (optional)
// - export function section (optional)
// - import external C function section (optional)
//   a list of external C functions
// - auto function index list section (optional)
//   the index of functions that are executed before application start, after application exits and
//   the index of the entry (main) function.
//
// a minimal module can only contains:
//
// - function type section
// - function section
//
// the following sections are not required during the runtime, they are generally used for debuging
//
// - import data section
// - import function section
// - export data section
// - export function section
// - import external C function section
//
// because once all modules (source file) are compiled, all imports and exports will be resolved
// and presisted in a _index map_ file, these sections speeds up the next program loading:
//
// - module index section
// - data index section (optional)
// - func index section
//
// there are also some in-memory tables, they are created at application startup:
//
// - external function index (optional)
// - library list (optional)

pub mod type_section;
pub mod import_data_section;

// the "module image file" binary layout:
//
// |--------------------------------------------------------------|
// | magic number (u32) | version (u32)                           |
// | section id 0 (u32) | section length 0 (u32) | section data 0 | <-- section 0
// | section id 1       | section length 1 (u32) | section data 1 | <-- section 1
// | ...                                                          |
// |--------------------------------------------------------------|

const MAGIC_NUMBER: [u8; 8] = [b'a', b'n', b'c', b's', b'm', b'o', b'd', 0]; // "ancsmod\0"
const VERSION: u32 = 1;

// pub struct ModuleImage<'a> {
    // pub module_index_section: ModuleIndexSection<'a>,
    // pub data_index_section: Option<DataIndexSection<'a>>,
    // pub func_index_section: FuncIndexSection<'a>,
// }

struct SectionHeader {
    id: u32,
    length: u32,
}
