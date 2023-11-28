// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// a module consists of two parts, data and code (i.e., instructions), which
// are spread out in the following sections:
//
// - function type section
//   the signature of a function, the types are also applied to the code blocks and external functions.
// - function section
// - local variable section
//   a function is consists of a type, a local variable list, and instructions
// - data sections
//   there are 3 kinds of data sections: read-only, read-write, uninit(ialized)
//   all data are thread-local, so the read-write section will be cloned and the
//   uninitialized section will be allocated when a new thread is created.
// - auto function index list section
//   presists the index of these functions:
//   - which executes before application start (constructor function, one per module)
//   - which executes before application exit (destructor function, one per module)
//   - the entry function (main function)
// - import function section
// - function name section
// - import data section
// - data name section
// - external library section
// - external function section
// - external function name section

// a minimal module only requires 3 sections:
//
// - function type section
// - local variable section
// - function section
//
// and there are optional sections:
// - read-only data section
// - read-write data section
// - uninitial data section
//
// the following sections are not required during the runtime, they are generally used for debuging
// and linking.
//
// - import function section
// - function name section
// - import data section
// - data name section
// - external library section
// - external function section
// - external function name section

// in the stage of linking modules (which follows the stage compiling), all imports and exports
// are resolved and stored the indices in the following sections,
// this help speeding up the program loading:
//
// - func index section
// - data index section (optional)
// - unified external library section (optional)
// - unified external functon section (optional)
// - external function index section (optional)
//
// note that only the application main module contains these sections.

// about the design of module:
//
// the loading and startup of XiaoXuan modules are extremely fast, because:
// - there is no parsing process and zero-copy, the loading process actually does only two things: maps
//   the module image file into memory, and locates the start and end positions of echo
//   sections.
// - instructions are executed directly on the (binary) bytecode and all sores of indices of the module.
//
// these allow the XiaoXuan applications to have almost no startup time.
//
// since XiaoXuan application starts up almost instantly, it is suitable for
// using as 'function' in other applications or scripts.

// the data type of fields:
//
// - u8: data type, data section type, module share type
// - u16: memory store/load offset, data align, block break/recur skip depth, params count, results count
// - u32: section id, syscall number, env call number
//   module index, function type index, data index, local (variable list) index,
//   function index, dynamic function index, c function index
//
// on the host side, the data type of '*index' is usually represented as the 'usize'.

pub mod data_index_section;
pub mod data_name_section;
pub mod data_section;
pub mod external_func_index_section;
pub mod external_func_section;
pub mod external_library_section;
pub mod func_index_section;
pub mod func_name_section;
pub mod func_section;
pub mod local_variable_section;
pub mod type_section;
pub mod unified_external_func_section;
pub mod unified_external_library_section;

use ancvm_types::{
    IMAGE_MAGIC_NUMBER, IMAGE_MAJOR_VERSION, IMAGE_MINOR_VERSION, RUNTIME_MAJOR_VERSION,
    RUNTIME_MINOR_VERSION,
};

use crate::{
    module_image::data_index_section::DataIndexSection,
    module_image::func_index_section::FuncIndexSection,
    module_image::func_section::FuncSection,
    module_image::type_section::TypeSection,
    utils::{load_section_with_table_and_data_area, save_section_with_table_and_data_area},
    BinaryError,
};

use self::{
    data_name_section::DataNameSection,
    data_section::{ReadOnlyDataSection, ReadWriteDataSection, UninitDataSection},
    external_func_index_section::ExternalFuncIndexSection,
    external_func_section::ExternalFuncSection,
    external_library_section::ExternalLibrarySection,
    func_name_section::FuncNameSection,
    local_variable_section::LocalVariableSection,
    unified_external_func_section::UnifiedExternalFuncSection,
    unified_external_library_section::UnifiedExternalLibrarySection,
};

// the "module image file" binary layout:
//
//                 header
//              |---------------------------------------------------|
//              | magic number (u64)                                | 8 bytes
//              |---------------------------------------------------|
//              | image minor ver (u16)   | image major ver (u16)   | 4 bytes
//              | runtime minor ver (u16) | runtime major ver (u16) | 4 bytes
//              | name length (u16)       | padding 2 bytes         | 4 bytes
//              |---------------------------------------------------|
//              | module name (UTF-8) 256 bytes                     | 256 bytes
//              |---------------------------------------------------|
//
//                 header length = 276 bytes
//
//                 body
//              |------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                 | 8 bytes
//              |------------------------------------------------------|
//   item 0 --> | section id 0 (u32) | offset 0 (u32) | length 0 (u32) | <-- table
//   item 1 --> | section id 1       | offset 1       | length 1       |
//              | ...                                                  |
//              |------------------------------------------------------|
// offset 0 --> | section data 0                                       | <-- data
// offset 1 --> | section data 1                                       |
//              | ...                                                  |
//              |------------------------------------------------------|

#[derive(Debug, PartialEq)]
pub struct ModuleImage<'a> {
    pub name: &'a str,
    pub items: &'a [ModuleSection],
    pub sections_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct ModuleSection {
    pub id: ModuleSectionId, // u32
    pub offset: u32,
    pub length: u32,
}

impl ModuleSection {
    pub fn new(id: ModuleSectionId, offset: u32, length: u32) -> Self {
        Self { id, offset, length }
    }
}

#[repr(u32)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ModuleSectionId {
    // essential
    Type = 0x10,   // 0x10
    LocalVariable, // 0x11
    Func,          // 0x12

    // optional
    ReadOnlyData = 0x20, // 0x20
    ReadWriteData,       // 0x21
    UninitData,          // 0x22
    AutoFunc,            // 0x23

    // optional, for debuging and linking
    ImportFunc = 0x30, // 0x30
    FuncName,          // 0x31
    ImportData,        // 0x32
    DataName,          // 0x33
    ExternalLibrary,   // 0x34
    ExternalFunc,      // 0x35

    // essential indices
    FuncIndex = 0x40, // 0x40

    // optional indeces
    DataIndex = 0x50,       // 0x50
    UnifiedExternalLibrary, // 0x51
    UnifiedExternalFunc,    // 0x52
    ExternalFuncIndex,      // 0x53 (mapping ext-func to uni-ext-func)
}

// use for data index section and function index section
//
// one range item per module.
// for example, consider the following items:
//
// module 0 ----- index item 0
//            |-- index item 1
//            |-- index item 2
//
// module 1 ----- index item 3
//            |-- index item 4
//
// since there are 2 modules, so there will be
// 2 range items as the following:
//
// range 0 = {offset:0, count:3}
// range 1 = {offset:3, count:2}
//
// use the C style struct memory layout
// see also:
// https://doc.rust-lang.org/reference/type-layout.html#reprc-structs
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct RangeItem {
    pub offset: u32,
    pub count: u32,
}

impl RangeItem {
    pub fn new(offset: u32, count: u32) -> Self {
        Self { offset, count }
    }
}

pub trait SectionEntry<'a> {
    // there is a approach to 'downcast' a section entry to section object, e.g.
    //
    // ```rust
    // fn downcast_section_entry<'a, T>(entry: &'a dyn SectionEntry) -> &'a T {
    //     /* the 'entry' is a fat pointer, it contains (object_pointer, vtable) */
    //     let ptr = entry as *const dyn SectionEntry as *const T; /* get the first part of the fat pointer */
    //     unsafe { &*ptr }
    // }
    // ```

    fn id(&'a self) -> ModuleSectionId;
    fn load(section_data: &'a [u8]) -> Self
    where
        Self: Sized;
    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()>;
}

impl<'a> ModuleImage<'a> {
    pub fn load(image_data: &'a [u8]) -> Result<Self, BinaryError> {
        let magic_slice = &image_data[0..8];
        if magic_slice != IMAGE_MAGIC_NUMBER {
            return Err(BinaryError::new("Not a valid module image file."));
        }

        // there is another safe approach for obtaining the version number:
        //
        // ```rust
        //     let version_data: [u8;4] = (&image_data[4..8]).try_into().unwrap();
        //     let version = u32::from_le_bytes(version_data);
        // ```

        let ptr = image_data.as_ptr();
        let ptr_require_module_image_version = unsafe { ptr.offset(8) };
        let require_module_image_version =
            unsafe { std::ptr::read(ptr_require_module_image_version as *const u32) };

        let supported_module_image_version =
            ((IMAGE_MAJOR_VERSION as u32) << 16) | (IMAGE_MINOR_VERSION as u32); // supported version 1.0
        if require_module_image_version > supported_module_image_version {
            return Err(BinaryError::new(
                "The module image requires a newer version runtime.",
            ));
        }

        let ptr_require_runtime_version = unsafe { ptr.offset(12) };
        let require_runtime_version =
            unsafe { std::ptr::read(ptr_require_runtime_version as *const u32) };

        let supported_runtime_version =
            ((RUNTIME_MAJOR_VERSION as u32) << 16) | (RUNTIME_MINOR_VERSION as u32);

        // a module will only run if its required major and minor
        // versions match the current runtime version 100%.
        if require_runtime_version != supported_runtime_version {
            return Err(BinaryError::new(
                "The module requires a different version runtime to run.",
            ));
        }

        let ptr_name_length = unsafe { ptr.offset(16) };
        let name_length = unsafe { std::ptr::read(ptr_name_length as *const u16) };
        let name_data = &image_data[20..(20 + (name_length as usize))];
        let name = std::str::from_utf8(name_data).unwrap();

        let image_body = &image_data[(20 + 256)..];

        // since the structure of module image and a section are the same,
        // that is, the module image itself can be thought of
        // as a 'big' section that contains many child sections.
        // so we can load module by reusing function
        // `load_section_with_table_and_data_area` as well.
        let (items, sections_data) =
            load_section_with_table_and_data_area::<ModuleSection>(image_body);

        Ok(Self {
            name,
            items,
            sections_data,
        })
    }

    pub fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        // write header
        writer.write_all(IMAGE_MAGIC_NUMBER)?;
        writer.write_all(&IMAGE_MINOR_VERSION.to_le_bytes())?;
        writer.write_all(&IMAGE_MAJOR_VERSION.to_le_bytes())?;
        writer.write_all(&RUNTIME_MINOR_VERSION.to_le_bytes())?;
        writer.write_all(&RUNTIME_MAJOR_VERSION.to_le_bytes())?;
        writer.write_all(&(self.name.len() as u16).to_le_bytes())?;
        writer.write_all(&[0u8, 0])?; // padding, 2 bytes

        let name_bytes = self.name.as_bytes();
        let mut name_buffer = [0u8; 256];
        unsafe {
            std::ptr::copy(
                name_bytes.as_ptr(),
                name_buffer.as_mut_ptr(),
                self.name.len(),
            )
        };
        writer.write_all(&name_buffer)?;

        save_section_with_table_and_data_area(self.items, self.sections_data, writer)
    }

    pub fn convert_from_entries(
        entries: &[&'a dyn SectionEntry<'a>],
    ) -> (Vec<ModuleSection>, Vec<u8>) {
        let mut image_data: Vec<u8> = Vec::new();

        // len0, len0+1, len0+1+2..., len total
        let mut data_lengths: Vec<usize> = Vec::new();

        for entry in entries {
            entry.save(&mut image_data).unwrap();
            data_lengths.push(image_data.len());
        }

        let mut offsets: Vec<usize> = vec![0];
        offsets.extend(data_lengths.iter());
        offsets.pop();

        let lengths = data_lengths
            .iter()
            .zip(offsets.iter())
            .map(|(next, current)| next - current)
            .collect::<Vec<usize>>();

        let items = entries
            .iter()
            .zip(offsets.iter().zip(lengths.iter()))
            .map(|(entry, (offset, length))| {
                ModuleSection::new(entry.id(), *offset as u32, *length as u32)
            })
            .collect::<Vec<ModuleSection>>();

        (items, image_data)
    }

    pub fn get_section_index_by_id(&'a self, section_id: ModuleSectionId) -> Option<usize> {
        self.items.iter().enumerate().find_map(|(idx, item)| {
            if item.id == section_id {
                Some(idx)
            } else {
                None
            }
        })
    }

    fn get_section_data_by_id(&'a self, section_id: ModuleSectionId) -> Option<&'a [u8]> {
        self.items.iter().find_map(|item| {
            if item.id == section_id {
                let data =
                    &self.sections_data[item.offset as usize..(item.offset + item.length) as usize];
                Some(data)
            } else {
                None
            }
        })
    }

    // essential section
    pub fn get_type_section(&'a self) -> TypeSection<'a> {
        self.get_section_data_by_id(ModuleSectionId::Type)
            .map_or_else(
                || panic!("Can not find the type section."),
                TypeSection::load,
            )
    }

    // essential section
    pub fn get_local_variable_section(&'a self) -> LocalVariableSection<'a> {
        self.get_section_data_by_id(ModuleSectionId::LocalVariable)
            .map_or_else(
                || panic!("Can not find the local variable section."),
                LocalVariableSection::load,
            )
    }

    // essential section
    pub fn get_func_section(&'a self) -> FuncSection<'a> {
        self.get_section_data_by_id(ModuleSectionId::Func)
            .map_or_else(
                || panic!("Can not find the function section."),
                FuncSection::load,
            )
    }

    // essential section
    pub fn get_func_index_section(&'a self) -> FuncIndexSection<'a> {
        self.get_section_data_by_id(ModuleSectionId::FuncIndex)
            .map_or_else(
                || panic!("Can not find the function index section."),
                FuncIndexSection::load,
            )
    }

    // optional section
    pub fn get_optional_read_only_data_section(&'a self) -> Option<ReadOnlyDataSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::ReadOnlyData)
            .map(ReadOnlyDataSection::load)
    }

    // optional section
    pub fn get_optional_read_write_data_section(&'a self) -> Option<ReadWriteDataSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::ReadWriteData)
            .map(ReadWriteDataSection::load)
    }

    // optional section
    pub fn get_optional_uninit_data_section(&'a self) -> Option<UninitDataSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::UninitData)
            .map(UninitDataSection::load)
    }

    // todo get_optional_auto_func_section

    // todo get_optional_import_func_section

    // optional section
    pub fn get_optional_func_name_section(&'a self) -> Option<FuncNameSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::FuncName)
            .map(FuncNameSection::load)
    }

    // todo get_optional_import_data_section

    // optional section
    pub fn get_optional_data_name_section(&'a self) -> Option<DataNameSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::DataName)
            .map(DataNameSection::load)
    }

    // optional
    pub fn get_optional_external_library_section(&'a self) -> Option<ExternalLibrarySection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::ExternalLibrary)
            .map(ExternalLibrarySection::load)
    }

    // optional
    pub fn get_optional_external_func_section(&'a self) -> Option<ExternalFuncSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::ExternalFunc)
            .map(ExternalFuncSection::load)
    }

    // optional
    pub fn get_optional_data_index_section(&'a self) -> Option<DataIndexSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::DataIndex)
            .map(DataIndexSection::load)
    }

    // optional
    pub fn get_optional_unified_external_library_section(
        &'a self,
    ) -> Option<UnifiedExternalLibrarySection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::UnifiedExternalLibrary)
            .map(UnifiedExternalLibrarySection::load)
    }

    // optional
    pub fn get_optional_unified_external_func_section(
        &'a self,
    ) -> Option<UnifiedExternalFuncSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::UnifiedExternalFunc)
            .map(UnifiedExternalFuncSection::load)
    }

    // optional
    pub fn get_optional_external_func_index_section(
        &'a self,
    ) -> Option<ExternalFuncIndexSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::ExternalFuncIndex)
            .map(ExternalFuncIndexSection::load)
    }
}

#[cfg(test)]
mod tests {
    use ancvm_types::{
        entry::{
            FuncEntry, FuncIndexEntry, FuncIndexModuleEntry, LocalListEntry, LocalVariableEntry,
            TypeEntry,
        },
        DataType, MemoryDataType,
    };

    use crate::module_image::{
        func_index_section::{FuncIndexItem, FuncIndexSection},
        func_section::FuncSection,
        local_variable_section::{LocalVariableItem, LocalVariableSection},
        type_section::TypeSection,
        ModuleImage, RangeItem, SectionEntry, IMAGE_MAGIC_NUMBER,
    };

    #[test]
    fn test_save_module_image_and_load_module_image() {
        // build TypeSection instance
        // arbitrary data
        let type_entries = vec![
            TypeEntry {
                params: vec![DataType::I32, DataType::I64],
                results: vec![DataType::F32],
            },
            TypeEntry {
                params: vec![],
                results: vec![DataType::F64],
            },
        ];

        let (type_items, types_data) = TypeSection::convert_from_entries(&type_entries);
        let type_section = TypeSection {
            items: &type_items,
            types_data: &types_data,
        };

        // build FuncSection instance
        // arbitrary function entry and function code
        let func_entries = vec![
            FuncEntry {
                type_index: 2,
                local_list_index: 3,
                code: vec![1u8, 2, 3, 5, 7],
            },
            FuncEntry {
                type_index: 5,
                local_list_index: 7,
                code: vec![11u8, 13, 17, 19, 23, 29],
            },
        ];

        let (func_items, codes_data) = FuncSection::convert_from_entries(&func_entries);
        let func_section = FuncSection {
            items: &func_items,
            codes_data: &codes_data,
        };

        // build LocalVariableSection instance
        // arbitrary data
        let local_list_entries = vec![
            LocalListEntry::new(vec![
                LocalVariableEntry::from_i32(),
                LocalVariableEntry::from_i64(),
            ]),
            LocalListEntry::new(vec![LocalVariableEntry::from_bytes(12, 4)]),
        ];

        let (local_lists, local_list_data) =
            LocalVariableSection::convert_from_entries(&local_list_entries);
        let local_var_section = LocalVariableSection {
            lists: &local_lists,
            list_data: &local_list_data,
        };

        // build FuncIndexSection instance
        // arbitrary data
        let func_index_module_entries = vec![
            FuncIndexModuleEntry::new(vec![
                FuncIndexEntry::new(0, 1, 3),
                FuncIndexEntry::new(1, 5, 7),
            ]),
            FuncIndexModuleEntry::new(vec![FuncIndexEntry::new(0, 11, 13)]),
        ];

        let (func_index_ranges, func_index_items) =
            FuncIndexSection::convert_from_entries(&func_index_module_entries);

        let func_index_section = FuncIndexSection {
            ranges: &func_index_ranges,
            items: &func_index_items,
        };

        // build ModuleImage instance
        let section_entries: Vec<&dyn SectionEntry> = vec![
            &type_section,
            &func_section,
            &local_var_section,
            &func_index_section,
        ];
        let (section_items, sections_data) = ModuleImage::convert_from_entries(&section_entries);
        let module_image = ModuleImage {
            name: "main",
            items: &section_items,
            sections_data: &sections_data,
        };

        // save
        let mut image_data: Vec<u8> = Vec::new();
        module_image.save(&mut image_data).unwrap();

        assert_eq!(&image_data[0..8], IMAGE_MAGIC_NUMBER);
        assert_eq!(&image_data[8..10], &[0, 0]); // image minor version number, little endian
        assert_eq!(&image_data[10..12], &[1, 0]); // image major version number, little endian
        assert_eq!(&image_data[12..14], &[0, 0]); // runtime minor version number, little endian
        assert_eq!(&image_data[14..16], &[1, 0]); // runtime major version number, little endian

        // name
        assert_eq!(&image_data[16..18], &[4, 0]); // name length
        assert_eq!(&image_data[18..20], &[0, 0]); // padding
        assert_eq!(&image_data[20..24], &b"main".to_vec()); // name

        // header length = 276 bytes

        // section count
        assert_eq!(&image_data[276..280], &[4, 0, 0, 0]); // item count
        assert_eq!(&image_data[280..284], &[0, 0, 0, 0]); // padding

        let remains = &image_data[284..];

        // section table length = 12 (the record length) * 4
        let (section_table_data, remains) = remains.split_at(48);

        assert_eq!(
            section_table_data,
            &[
                0x10u8, 0, 0, 0, // section id, type section
                0, 0, 0, 0, // offset 0
                36, 0, 0, 0, // length 0
                //
                0x12u8, 0, 0, 0, // section id, func section
                36, 0, 0, 0, // offset 1
                52, 0, 0, 0, // length 1
                //
                0x11u8, 0, 0, 0, // section id, local variable section
                88, 0, 0, 0, // offset 2
                68, 0, 0, 0, // length 2
                //
                0x40u8, 0, 0, 0, // section id, func index section
                156, 0, 0, 0, // offset 3
                60, 0, 0, 0, // length 3
            ]
        );

        let (type_section_data, remains) = remains.split_at(36);
        assert_eq!(
            type_section_data,
            &[
                2u8, 0, 0, 0, // item count
                0, 0, 0, 0, // padding
                //
                2, 0, // param len 0
                1, 0, // result len 0
                0, 0, 0, 0, // param offset 0
                2, 0, 0, 0, // result offset 0
                //
                0, 0, // param len 1
                1, 0, // result len 1
                3, 0, 0, 0, // param offset 1
                3, 0, 0, 0, // result offset 1
                //
                0, // I32
                1, // I64
                2, // F32
                3, // F64
            ]
        );

        let (func_section_data, remains) = remains.split_at(52);
        assert_eq!(
            func_section_data,
            &[
                2, 0, 0, 0, // item count
                0, 0, 0, 0, // padding
                //
                0, 0, 0, 0, // code offset 0
                5, 0, 0, 0, // code len 0
                2, 0, 0, 0, // func type index 0
                3, 0, 0, 0, // local variable index 0
                //
                5, 0, 0, 0, // code offset 1
                6, 0, 0, 0, // code len 1
                5, 0, 0, 0, // func type index 1
                7, 0, 0, 0, // local variable index 1
                //
                1, 2, 3, 5, 7, // code 0
                11, 13, 17, 19, 23, 29, // code 1
                //
                0, // padding
            ]
        );

        let (local_var_section_data, remains) = remains.split_at(68);

        assert_eq!(
            local_var_section_data,
            &[
                // header
                2, 0, 0, 0, // item count
                0, 0, 0, 0, // 4 bytes padding
                // table
                0, 0, 0, 0, // offset
                2, 0, 0, 0, // count
                16, 0, 0, 0, // alloc bytes
                //
                24, 0, 0, 0, // offset (2 items * 12 bytes/item)
                1, 0, 0, 0, // count
                16, 0, 0, 0, // alloc bytes
                //
                // data
                //
                // list 0
                0, 0, 0, 0, // var offset (i32)
                4, 0, 0, 0, // var len
                0, // data type
                0, // padding
                4, 0, // align
                //
                8, 0, 0, 0, // var offset (i64)
                8, 0, 0, 0, // var len
                1, // data type
                0, // padding
                8, 0, // align
                //
                // list 1
                0, 0, 0, 0, // var offset
                12, 0, 0, 0, // var len
                4, // data type
                0, // padding
                4, 0, // align
            ]
        );

        assert_eq!(
            remains,
            &[
                /* table 0 */
                2, 0, 0, 0, // item count
                0, 0, 0, 0, // padding
                0, 0, 0, 0, // offset 0
                2, 0, 0, 0, // count 0
                2, 0, 0, 0, // offset 1
                1, 0, 0, 0, // count 1
                /* table 1 - module 0 */
                0, 0, 0, 0, // func idx 0
                1, 0, 0, 0, // target module idx 0
                3, 0, 0, 0, // target func idx 0
                1, 0, 0, 0, // func idx 1
                5, 0, 0, 0, // target module idx 1
                7, 0, 0, 0, // target func idx 1
                /* table 1 - module 0 */
                0, 0, 0, 0, // func idx 0
                11, 0, 0, 0, // target module idx 0
                13, 0, 0, 0, // target func idx 0
            ]
        );

        // load
        let module_image_restore = ModuleImage::load(&image_data).unwrap();
        assert_eq!(module_image_restore.items.len(), 4);

        // check type

        let type_section_restore = module_image_restore.get_type_section();
        assert_eq!(type_section_restore.items.len(), 2);

        assert_eq!(
            type_section_restore.get_item_params_and_results(0),
            (
                vec![DataType::I32, DataType::I64].as_ref(),
                vec![DataType::F32].as_ref(),
            )
        );

        assert_eq!(
            type_section_restore.get_item_params_and_results(1),
            ([].as_ref(), vec![DataType::F64].as_ref(),)
        );

        // check func

        let func_section_restore = module_image_restore.get_func_section();
        assert_eq!(func_section_restore.items.len(), 2);

        assert_eq!(
            func_section_restore.get_item_type_index_and_local_variable_index_and_code(0),
            (2, 3, vec![1u8, 2, 3, 5, 7].as_ref(),)
        );

        assert_eq!(
            func_section_restore.get_item_type_index_and_local_variable_index_and_code(1),
            (5, 7, vec![11u8, 13, 17, 19, 23, 29].as_ref(),)
        );

        // check local vars

        let local_var_section_restore = module_image_restore.get_local_variable_section();
        assert_eq!(local_var_section_restore.lists.len(), 2);

        assert_eq!(
            local_var_section_restore.get_local_list(0),
            &[
                LocalVariableItem::new(0, 4, MemoryDataType::I32, 4),
                LocalVariableItem::new(8, 8, MemoryDataType::I64, 8),
            ]
        );

        assert_eq!(
            local_var_section_restore.get_local_list(1),
            &[LocalVariableItem::new(0, 12, MemoryDataType::BYTES, 4),]
        );

        let func_index_section_restore = module_image_restore.get_func_index_section();

        assert_eq!(func_index_section_restore.ranges.len(), 2);
        assert_eq!(func_index_section_restore.items.len(), 3);

        assert_eq!(
            &func_index_section_restore.ranges[0],
            &RangeItem::new(0, 2,)
        );
        assert_eq!(
            &func_index_section_restore.ranges[1],
            &RangeItem::new(2, 1,)
        );

        assert_eq!(
            &func_index_section_restore.items[0],
            &FuncIndexItem::new(0, 1, 3)
        );
        assert_eq!(
            &func_index_section_restore.items[1],
            &FuncIndexItem::new(1, 5, 7)
        );
        assert_eq!(
            &func_index_section_restore.items[2],
            &FuncIndexItem::new(0, 11, 13)
        );
    }
}
