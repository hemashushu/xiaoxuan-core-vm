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
// - local variable section
//   a function is consists of a type, a local variable list, and instructions
// - function section
// - data sections
//   there are 3 kinds of data sections: read-only, read-write, uninit(ialized)
//   all data are thread-local, so the read-write section will be cloned and the
//   uninitialized section will be allocated when a new thread is created.
// - start function index list section
//   aka constructor functions (one constructor per module), which are executed before the 'main' function
// - exit function index list section
//   aka destructor functions (one destructor per module), which are executed after the 'main' function
// - import module section
// - import function section
// - import data section
// - function name section
// - data name section
// - external library section
// - external function section
// - external function name section

// a base module only requires 3 sections:
//
// - function type section
// - local variable section
// - function section
//
// and there are optional sections:
// - read-only data section
// - read-write data section
// - uninitialized data section
//
// the following sections are not required during the runtime, they are generally used for debuging
// and linking.
//
// - import module section
// - import function section
// - import data section
// - function name section
// - data name section
// - external library section
// - external function section
// - external function name section

// in the stage of linking modules (which follows the stage compiling), all imports and exports
// are resolved and stored the indices in the following sections:
//
// - func index section
// - start func list section
// - exit func list section
// - data index section (optional)
// - unified external library section (optional)
// - unified external functon section (optional)
// - external function index section (optional)
//
// these sections help speeding up the program loading,
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
pub mod exit_function_list_section;
pub mod external_function_index_section;
pub mod external_function_section;
pub mod external_library_section;
pub mod function_index_section;
pub mod function_name_section;
pub mod function_section;
pub mod import_data_section;
pub mod import_function_section;
pub mod import_module_section;
pub mod local_variable_section;
pub mod property_section;
pub mod start_function_list_section;
pub mod type_section;
pub mod unified_external_function_section;
pub mod unified_external_library_section;

use ancvm_types::{
    IMAGE_FILE_MAGIC_NUMBER, IMAGE_FORMAT_MAJOR_VERSION, IMAGE_FORMAT_MINOR_VERSION,
    RUNTIME_MAJOR_VERSION, RUNTIME_MINOR_VERSION,
};

use crate::{
    module_image::data_index_section::DataIndexSection,
    module_image::function_index_section::FunctionIndexSection,
    module_image::function_section::FunctionSection,
    module_image::type_section::TypeSection,
    utils::{load_section_with_table_and_data_area, save_section_with_table_and_data_area},
    BinaryError,
};

use self::{
    data_name_section::DataNameSection,
    data_section::{ReadOnlyDataSection, ReadWriteDataSection, UninitDataSection},
    exit_function_list_section::ExitFunctionListSection,
    external_function_index_section::ExternalFunctionIndexSection,
    external_function_section::ExternalFunctionSection,
    external_library_section::ExternalLibrarySection,
    function_name_section::FunctionNameSection,
    import_data_section::ImportDataSection,
    import_function_section::ImportFunctionSection,
    import_module_section::ImportModuleSection,
    local_variable_section::LocalVariableSection,
    property_section::PropertySection,
    start_function_list_section::StartFunctionListSection,
    unified_external_function_section::UnifiedExternalFunctionSection,
    unified_external_library_section::UnifiedExternalLibrarySection,
};

// the "module image file" binary layout:
//
//                 header
//              |---------------------------------------------------|
//              | magic number (u64)                                | 8 bytes, off=0
//              |---------------------------------------------------|
//              | img fmt minor ver (u16) | img fmt major ver (u16) | 4 bytes, off=8
//              | runtime minor ver (u16) | runtime major ver (u16) | 4 bytes, off=12
//              | constructor func public index (u32)               | 4 bytes, off=16
//              | destructor func public index (u32)                | 4 bytes, off=20
//              | name length (u16)       | padding 2 bytes         | 4 bytes, off=24
//              |---------------------------------------------------|
//              | module name (UTF-8) 256 bytes                     | 256 bytes, off=28
//              |---------------------------------------------------|
//
//                 header length = 284ar bytes
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
    pub constructor_function_public_index: u32, // u32::max = none
    pub destructor_function_public_index: u32,  // u32::max = none
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
    Function,      // 0x12

    // optional
    ReadOnlyData = 0x20, // 0x20
    ReadWriteData,       // 0x21
    UninitData,          // 0x22

    // optional (for debug and link only)
    //
    // if the feature 'bridge function' (i.e., embed the XiaoXuan VM
    // in a C or Rust applicaton, and call VM functions) is required,
    // the section 'FunctionName' and 'DataName' are not optional.
    ImportModule = 0x30, // 0x30
    ImportFunction,      // 0x31
    ImportData,          // 0x32
    FunctionName,        // 0x33
    DataName,            // 0x34
    ExternalLibrary,     // 0x35
    ExternalFunction,    // 0x36

    // essential (application only)
    FunctionIndex = 0x40, // 0x40
    StartFunctionList,    // 0x41
    ExitFunctionList,     // 0x42
    Property,             // 0x43

    // optional (application only)
    DataIndex = 0x50,        // 0x50
    UnifiedExternalLibrary,  // 0x51
    UnifiedExternalFunction, // 0x52
    ExternalFunctionIndex,   // 0x53 (mapping 'external function' to 'unified external function')
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
        if magic_slice != IMAGE_FILE_MAGIC_NUMBER {
            return Err(BinaryError::new("Not a valid module image file."));
        }

        // there is another safe approach for obtaining the version number:
        //
        // ```rust
        //     let version_data: [u8;4] = (&image_data[4..8]).try_into().unwrap();
        //     let version = u32::from_le_bytes(version_data);
        // ```

        let ptr = image_data.as_ptr();
        let ptr_declared_module_format_image_version = unsafe { ptr.offset(8) };
        let declared_module_image_version =
            unsafe { std::ptr::read(ptr_declared_module_format_image_version as *const u32) };

        let supported_module_format_image_version =
            ((IMAGE_FORMAT_MAJOR_VERSION as u32) << 16) | (IMAGE_FORMAT_MINOR_VERSION as u32); // supported version 1.0
        if declared_module_image_version > supported_module_format_image_version {
            return Err(BinaryError::new(
                "The module image format requires a newer version runtime to read.",
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

        let constructor_function_public_index =
            unsafe { std::ptr::read(ptr.offset(16) as *const u32) };
        let destructor_function_public_index =
            unsafe { std::ptr::read(ptr.offset(20) as *const u32) };

        let name_length = unsafe { std::ptr::read(ptr.offset(24) as *const u16) };
        let name_bytes = &image_data[28..(28 + (name_length as usize))];
        let name = std::str::from_utf8(name_bytes).unwrap();

        const NAME_DATA_LENGTH: usize = 256;

        let image_body = &image_data[(28 + NAME_DATA_LENGTH)..];

        // since the structure of module image and a section are the same,
        // that is, the module image itself can be thought of
        // as a 'big' section that contains many child sections.
        // so we can load module by reusing function
        // `load_section_with_table_and_data_area` as well.
        let (items, sections_data) =
            load_section_with_table_and_data_area::<ModuleSection>(image_body);

        Ok(Self {
            name,
            constructor_function_public_index,
            destructor_function_public_index,
            items,
            sections_data,
        })
    }

    pub fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        // write header
        writer.write_all(IMAGE_FILE_MAGIC_NUMBER)?;
        writer.write_all(&IMAGE_FORMAT_MINOR_VERSION.to_le_bytes())?;
        writer.write_all(&IMAGE_FORMAT_MAJOR_VERSION.to_le_bytes())?;
        writer.write_all(&RUNTIME_MINOR_VERSION.to_le_bytes())?;
        writer.write_all(&RUNTIME_MAJOR_VERSION.to_le_bytes())?;
        //
        writer.write_all(&self.constructor_function_public_index.to_le_bytes())?;
        writer.write_all(&self.destructor_function_public_index.to_le_bytes())?;
        //
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
        let mut data_increment_lengths: Vec<usize> = Vec::new();

        for entry in entries {
            entry.save(&mut image_data).unwrap();
            data_increment_lengths.push(image_data.len());
        }

        let mut offsets: Vec<usize> = vec![0];
        offsets.extend(data_increment_lengths.iter());
        offsets.pop();

        let lengths = data_increment_lengths
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
    pub fn get_function_section(&'a self) -> FunctionSection<'a> {
        self.get_section_data_by_id(ModuleSectionId::Function)
            .map_or_else(
                || panic!("Can not find the function section."),
                FunctionSection::load,
            )
    }

    // essential section (application only)
    pub fn get_function_index_section(&'a self) -> FunctionIndexSection<'a> {
        self.get_section_data_by_id(ModuleSectionId::FunctionIndex)
            .map_or_else(
                || panic!("Can not find the function index section."),
                FunctionIndexSection::load,
            )
    }

    // essential section (application only)
    pub fn get_start_function_list_section(&'a self) -> StartFunctionListSection<'a> {
        self.get_section_data_by_id(ModuleSectionId::StartFunctionList)
            .map_or_else(
                || panic!("Can not find the start function list section."),
                StartFunctionListSection::load,
            )
    }

    // essential section (application only)
    pub fn get_exit_function_list_section(&'a self) -> ExitFunctionListSection<'a> {
        self.get_section_data_by_id(ModuleSectionId::ExitFunctionList)
            .map_or_else(
                || panic!("Can not find the exit function list section."),
                ExitFunctionListSection::load,
            )
    }

    // essential section (application only)
    pub fn get_property_section(&'a self) -> PropertySection {
        self.get_section_data_by_id(ModuleSectionId::Property)
            .map_or_else(
                || panic!("Can not find the property section."),
                PropertySection::load,
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

    // optional section (application only)
    pub fn get_optional_data_index_section(&'a self) -> Option<DataIndexSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::DataIndex)
            .map(DataIndexSection::load)
    }

    // optional section (application only)
    pub fn get_optional_unified_external_library_section(
        &'a self,
    ) -> Option<UnifiedExternalLibrarySection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::UnifiedExternalLibrary)
            .map(UnifiedExternalLibrarySection::load)
    }

    // optional section (application only)
    pub fn get_optional_unified_external_function_section(
        &'a self,
    ) -> Option<UnifiedExternalFunctionSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::UnifiedExternalFunction)
            .map(UnifiedExternalFunctionSection::load)
    }

    // optional section (application only)
    pub fn get_optional_external_function_index_section(
        &'a self,
    ) -> Option<ExternalFunctionIndexSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::ExternalFunctionIndex)
            .map(ExternalFunctionIndexSection::load)
    }

    // optional section (for debug and link only)
    pub fn get_optional_import_module_section(&'a self) -> Option<ImportModuleSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::ImportModule)
            .map(ImportModuleSection::load)
    }

    // optional section (for debug and link only)
    pub fn get_optional_import_function_section(&'a self) -> Option<ImportFunctionSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::ImportFunction)
            .map(ImportFunctionSection::load)
    }

    // optional section (for debug and link only)
    pub fn get_optional_import_data_section(&'a self) -> Option<ImportDataSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::ImportData)
            .map(ImportDataSection::load)
    }

    // optional section (for debug and link only)
    pub fn get_optional_function_name_section(&'a self) -> Option<FunctionNameSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::FunctionName)
            .map(FunctionNameSection::load)
    }

    // optional section (for debug and link only)
    pub fn get_optional_data_name_section(&'a self) -> Option<DataNameSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::DataName)
            .map(DataNameSection::load)
    }

    // optional section (for debug and link only)
    pub fn get_optional_external_library_section(&'a self) -> Option<ExternalLibrarySection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::ExternalLibrary)
            .map(ExternalLibrarySection::load)
    }

    // optional section (for debug and link only)
    pub fn get_optional_external_function_section(&'a self) -> Option<ExternalFunctionSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::ExternalFunction)
            .map(ExternalFunctionSection::load)
    }
}

#[cfg(test)]
mod tests {
    use ancvm_types::{
        entry::{
            FunctionEntry, FunctionIndexEntry, FunctionIndexModuleEntry, LocalListEntry,
            LocalVariableEntry, TypeEntry,
        },
        DataType, MemoryDataType,
    };

    use crate::module_image::{
        exit_function_list_section::ExitFunctionListSection,
        function_index_section::{FunctionIndexItem, FunctionIndexSection},
        function_section::FunctionSection,
        local_variable_section::{LocalVariableItem, LocalVariableSection},
        property_section::PropertySection,
        start_function_list_section::StartFunctionListSection,
        type_section::TypeSection,
        ModuleImage, RangeItem, SectionEntry, IMAGE_FILE_MAGIC_NUMBER,
    };

    #[test]
    fn test_save_module_image_and_load_module_image() {
        // build TypeSection instance
        // note: arbitrary types
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
        // note: arbitrary functions
        let function_entries = vec![
            FunctionEntry {
                type_index: 2,
                local_list_index: 3,
                code: vec![1u8, 2, 3, 5, 7],
            },
            FunctionEntry {
                type_index: 5,
                local_list_index: 7,
                code: vec![11u8, 13, 17, 19, 23, 29],
            },
        ];

        let (function_items, codes_data) = FunctionSection::convert_from_entries(&function_entries);
        let function_section = FunctionSection {
            items: &function_items,
            codes_data: &codes_data,
        };

        // build LocalVariableSection instance
        // note: arbitrary local variables
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
        // note: arbitrary indices
        let function_index_module_entries = vec![
            FunctionIndexModuleEntry::new(vec![
                FunctionIndexEntry::new(0, 1, 3),
                FunctionIndexEntry::new(1, 5, 7),
            ]),
            FunctionIndexModuleEntry::new(vec![FunctionIndexEntry::new(0, 11, 13)]),
        ];

        let (function_index_ranges, function_index_items) =
            FunctionIndexSection::convert_from_entries(&function_index_module_entries);

        let function_index_section = FunctionIndexSection {
            ranges: &function_index_ranges,
            items: &function_index_items,
        };

        // build start function list
        let start_indices = vec![31u32, 37, 41, 43];
        let start_function_list_section = StartFunctionListSection {
            items: &start_indices,
        };

        // build exit function list
        let exit_indices = vec![47u32, 53];
        let exit_function_list_section = ExitFunctionListSection {
            items: &exit_indices,
        };

        // build property section
        let property_section = PropertySection {
            entry_function_public_index: 17,
        };

        // build ModuleImage instance
        let section_entries: Vec<&dyn SectionEntry> = vec![
            &type_section,
            &function_section,
            &local_var_section,
            &function_index_section,
            &start_function_list_section,
            &exit_function_list_section,
            &property_section,
        ];

        let (section_items, sections_data) = ModuleImage::convert_from_entries(&section_entries);
        let module_image = ModuleImage {
            name: "main",
            constructor_function_public_index: 11,
            destructor_function_public_index: 13,
            items: &section_items,
            sections_data: &sections_data,
        };

        // save
        let mut image_data: Vec<u8> = Vec::new();
        module_image.save(&mut image_data).unwrap();

        assert_eq!(&image_data[0..8], IMAGE_FILE_MAGIC_NUMBER);
        assert_eq!(&image_data[8..10], &[0, 0]); // image format minor version number, little endian
        assert_eq!(&image_data[10..12], &[1, 0]); // image format major version number, little endian
        assert_eq!(&image_data[12..14], &[0, 0]); // runtime minor version number, little endian
        assert_eq!(&image_data[14..16], &[1, 0]); // runtime major version number, little endian

        // constructor and destructor
        assert_eq!(&image_data[16..20], &[11, 0, 0, 0]); // constructor
        assert_eq!(&image_data[20..24], &[13, 0, 0, 0]); // destructor

        // name
        assert_eq!(&image_data[24..26], &[4, 0]); // name length, 2 bytes
        assert_eq!(&image_data[26..28], &[0, 0]); // padding, 2 bytes
        assert_eq!(&image_data[28..32], &b"main".to_vec()); // name

        // header length = 284 bytes

        // section count
        assert_eq!(&image_data[284..288], &[7, 0, 0, 0]); // section item count
        assert_eq!(&image_data[288..292], &[0, 0, 0, 0]); // padding

        let remains = &image_data[292..];

        // section table length = 12 (the record length) * 7 = 84
        let (section_table_data, remains) = remains.split_at(84);

        assert_eq!(
            section_table_data,
            &[
                0x10u8, 0, 0, 0, // section id, type section
                0, 0, 0, 0, // offset 0
                36, 0, 0, 0, // length 0
                //
                0x12, 0, 0, 0, // section id, func section
                36, 0, 0, 0, // offset 1
                52, 0, 0, 0, // length 1
                //
                0x11, 0, 0, 0, // section id, local variable section
                88, 0, 0, 0, // offset 2
                68, 0, 0, 0, // length 2
                //
                0x40, 0, 0, 0, // section id, func index section
                156, 0, 0, 0, // offset 3
                60, 0, 0, 0, // length 3
                //
                0x41, 0, 0, 0, // section id, start func list
                216, 0, 0, 0, // offset 4
                24, 0, 0, 0, // length 4 (table header 8 bytes + 4 * 4)
                //
                0x42, 0, 0, 0, // section id, exit func list
                240, 0, 0, 0, // offset 5
                16, 0, 0, 0, // length 5 (table header 8 bytes + 2 * 4)
                //
                0x43, 0, 0, 0, // section id, property section
                0, 1, 0, 0, // offset 6, (0x01,00, int = 256)
                4, 0, 0, 0 // length 6
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

        let (function_section_data, remains) = remains.split_at(52);
        assert_eq!(
            function_section_data,
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

        let (function_index_section_data, remains) = remains.split_at(60);
        assert_eq!(
            function_index_section_data,
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

        let (start_function_list_section_data, remains) = remains.split_at(24);
        assert_eq!(
            start_function_list_section_data,
            &[
                4u8, 0, 0, 0, // item count
                0, 0, 0, 0, // padding
                //
                31, 0, 0, 0, //
                37, 0, 0, 0, //
                41, 0, 0, 0, //
                43, 0, 0, 0, //
            ]
        );

        let (exit_function_list_section_data, remains) = remains.split_at(16);
        assert_eq!(
            exit_function_list_section_data,
            &[
                2u8, 0, 0, 0, // item count
                0, 0, 0, 0, // padding
                //
                47, 0, 0, 0, //
                53, 0, 0, 0, //
            ]
        );

        assert_eq!(
            remains,
            &[
                17,0,0,0, // the public index of function 'entry'
            ]
        );

        // load
        let module_image_restore = ModuleImage::load(&image_data).unwrap();
        assert_eq!(module_image_restore.items.len(), 7);

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

        let function_section_restore = module_image_restore.get_function_section();
        assert_eq!(function_section_restore.items.len(), 2);

        assert_eq!(
            function_section_restore.get_item_type_index_and_local_variable_index_and_code(0),
            (2, 3, vec![1u8, 2, 3, 5, 7].as_ref(),)
        );

        assert_eq!(
            function_section_restore.get_item_type_index_and_local_variable_index_and_code(1),
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
            &[LocalVariableItem::new(0, 12, MemoryDataType::Bytes, 4),]
        );

        let function_index_section_restore = module_image_restore.get_function_index_section();

        assert_eq!(function_index_section_restore.ranges.len(), 2);
        assert_eq!(function_index_section_restore.items.len(), 3);

        assert_eq!(
            &function_index_section_restore.ranges[0],
            &RangeItem::new(0, 2,)
        );
        assert_eq!(
            &function_index_section_restore.ranges[1],
            &RangeItem::new(2, 1,)
        );

        assert_eq!(
            &function_index_section_restore.items[0],
            &FunctionIndexItem::new(0, 1, 3)
        );
        assert_eq!(
            &function_index_section_restore.items[1],
            &FunctionIndexItem::new(1, 5, 7)
        );
        assert_eq!(
            &function_index_section_restore.items[2],
            &FunctionIndexItem::new(0, 11, 13)
        );

        // check start function list
        let start_function_list_section_restore =
            module_image_restore.get_start_function_list_section();
        assert_eq!(
            start_function_list_section_restore.items,
            &[31, 37, 41, 43,]
        );

        // check exit function list
        let exit_function_list_section_restore =
            module_image_restore.get_exit_function_list_section();
        assert_eq!(exit_function_list_section_restore.items, &[47, 53]);

        // check property section
        let property_section_restore = module_image_restore.get_property_section();
        assert_eq!(property_section_restore.entry_function_public_index, 17);
    }
}
