// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// a module consists of two parts, data and code (i.e., instructions), which
// are spread out in the following sections:
//
// - type section
//   the signature of a function, the types are also applied to the code blocks and external functions.
// - local variable section
//   a function is consists of a type, a local variable list, and instructions
// - function section
// - data sections
//   there are 3 kinds of data sections:
//   - read-only
//   - read-write
//   - uninit(ialized)
//   all data are thread-local, so the read-write section will be cloned and the
//   uninitialized section will be allocated when a new thread is created.
// - import module section
// - import function section
// - import data section
// - function name section
// - data name section
// - external library section
// - external function section

// a simple module requires only 3 sections:
//
// - type section
// - local variable section
// - function section
//
// and there are optional sections:
// - read-only data section
// - read-write data section
// - uninitialized data section
//
// the following sections are not required during the runtime,
// they are used for debuging and linking.
//
// - import module section
// - import function section
// - import data section
// - function name section
// - data name section
// - external library section
// - external function section

// in the stage of linking modules (which follows the stage compiling), all imports and exports
// are resolved and stored the indices in the following sections:
//
// - function index section
// - data index section (optional)
// - unified external library section (optional)
// - unified external functon section (optional)
// - external function index section (optional)
//
// these sections help speeding up the program loading,
// note that only the application module contains these sections.

// about the design of module:
//
// the loading and startup of XiaoXuan modules are extremely fast, because:
// - there is no parsing process and zero-copy, the loading process actually does only two things: maps
//   the module image file into memory, and locates the start and end positions of echo
//   sections.
// - instructions are executed directly on the (binary) bytecode and all sores of indices of the module.
//
// these allow the XiaoXuan applications to have almost no startup delay time.
//
// since XiaoXuan application starts up almost instantly, it is suitable for
// using as 'function' in other applications or scripts.

// the data type of image section fields:
//
// - u8: data type, data section type, module share type
// - u16: 'local variables' and 'data' store/load offset, data align, block break/recur skip depth, params count, results count
// - u32: section id, syscall number, env call number
//   module index, function type index, data index, local (variable list) index,
//   function index, dynamic function index, c function index

use ancvm_isa::{IMAGE_FILE_MAGIC_NUMBER, IMAGE_FORMAT_MAJOR_VERSION, IMAGE_FORMAT_MINOR_VERSION};

use crate::{
    common_sections::{
        data_name_section::DataNameSection,
        data_section::{ReadOnlyDataSection, ReadWriteDataSection, UninitDataSection},
        external_function_section::ExternalFunctionSection,
        external_library_section::ExternalLibrarySection,
        function_name_section::FunctionNameSection,
        function_section::FunctionSection,
        import_data_section::ImportDataSection,
        import_function_section::ImportFunctionSection,
        import_module_section::ImportModuleSection,
        local_variable_section::LocalVariableSection,
        type_section::TypeSection,
    },
    index_sections::{
        data_index_section::DataIndexSection,
        external_function_index_section::ExternalFunctionIndexSection,
        function_index_section::FunctionIndexSection, property_section::PropertySection,
        unified_external_function_section::UnifiedExternalFunctionSection,
        unified_external_library_section::UnifiedExternalLibrarySection,
    },
    tableaccess::{load_section_with_table_and_data_area, save_section_with_table_and_data_area},
    BinaryError,
};

// the "module image file" binary layout:
//
//                 header
//              |---------------------------------------------------|
//              | magic number (u64)                                | 8 bytes, off=0
//              |---------------------------------------------------|
//              | img fmt minor ver (u16) | img fmt major ver (u16) | 4 bytes, off=8
//              | padding (4 bytes)                                 | 4 bytes, off=12
//              |---------------------------------------------------|
//                 header length = 16 bytes

//                 body
//              |------------------------------------------------------|
//              | section item count (u32) | (4 bytes padding)         | 8 bytes, off=16
//              |------------------------------------------------------|
//   item 0 --> | section id 0 (u32) | offset 0 (u32) | length 0 (u32) | <-- table
//   item 1 --> | section id 1       | offset 1       | length 1       |
//              | ...                                                  |
//              |------------------------------------------------------|
// offset 0 --> | section data 0                                       | <-- data
// offset 1 --> | section data 1                                       |
//              | ...                                                  |
//              |------------------------------------------------------|

pub const MODULE_HEADER_LENGTH: usize = 16;
pub const MODULE_NAME_BUFFER_LENGTH: usize = 256;
pub const DATA_ALIGN_BYTES: usize = 4;

#[derive(Debug, PartialEq)]
pub struct ModuleImage<'a> {
    pub items: &'a [ModuleSectionItem],
    pub sections_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct ModuleSectionItem {
    pub id: ModuleSectionId, // u32
    pub offset: u32,
    pub length: u32,
}

impl ModuleSectionItem {
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
    // if the feature 'bridge function' is required (i.e.,
    // embed the XiaoXuan Core VM in a C or Rust applicaton,
    // and then call XiaoXuan Core VM functions from application) ,
    // the section 'FunctionName' and 'DataName' are required also.
    ImportModule = 0x30, // 0x30
    ImportFunction,      // 0x31
    ImportData,          // 0x32
    FunctionName,        // 0x33
    DataName,            // 0x34
    ExternalLibrary,     // 0x35
    ExternalFunction,    // 0x36

    // essential (application only)
    FunctionIndex = 0x40, // 0x40
    Property,             // 0x41

    // optional (application only)
    DataIndex = 0x50,        // 0x50
    UnifiedExternalLibrary,  // 0x51
    UnifiedExternalFunction, // 0x52
    ExternalFunctionIndex,   // 0x53

                             // the section ExternalFunctionIndex is used for mapping
                             // 'external function' to 'unified external function')
}

// `RangeItem` is used for data index section and function index section
//
// note that one range item per module, e.g., consider the following items:
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

        let image_body = &image_data[MODULE_HEADER_LENGTH..];

        // since the structure of module image and a section are the same,
        // that is, the module image itself can be thought of
        // as a 'big' section that contains many child sections.
        // so we can load module by reusing function
        // `load_section_with_table_and_data_area` as well.
        let (items, sections_data) =
            load_section_with_table_and_data_area::<ModuleSectionItem>(image_body);

        Ok(Self {
            items,
            sections_data,
        })
    }

    pub fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        // write header
        writer.write_all(IMAGE_FILE_MAGIC_NUMBER)?;
        writer.write_all(&IMAGE_FORMAT_MINOR_VERSION.to_le_bytes())?;
        writer.write_all(&IMAGE_FORMAT_MAJOR_VERSION.to_le_bytes())?;

        // padding, 4 bytes
        writer.write_all(&[0u8, 0, 0, 0])?;

        save_section_with_table_and_data_area(self.items, self.sections_data, writer)
    }

    pub fn convert_from_entries(
        entries: &[&'a dyn SectionEntry<'a>],
    ) -> (Vec<ModuleSectionItem>, Vec<u8>) {
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
                ModuleSectionItem::new(entry.id(), *offset as u32, *length as u32)
            })
            .collect::<Vec<ModuleSectionItem>>();

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

    // optional section (for debug and link only), and for bridge function call
    pub fn get_optional_function_name_section(&'a self) -> Option<FunctionNameSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::FunctionName)
            .map(FunctionNameSection::load)
    }

    // optional section (for debug and link only), and for bridge function call
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
}

#[cfg(test)]
mod tests {
    use ancvm_isa::{
        entry::{
            FunctionEntry, FunctionIndexEntry, FunctionIndexListEntry, LocalListEntry,
            LocalVariableEntry, TypeEntry,
        },
        MemoryDataType, OperandDataType,
    };

    use crate::{
        common_sections::{
            function_section::FunctionSection,
            local_variable_section::{LocalVariableItem, LocalVariableSection},
            type_section::TypeSection,
        },
        index_sections::{
            function_index_section::{FunctionIndexItem, FunctionIndexSection},
            property_section::PropertySection,
        },
        module_image::{
            ModuleImage, RangeItem, SectionEntry, IMAGE_FILE_MAGIC_NUMBER, MODULE_HEADER_LENGTH,
            MODULE_NAME_BUFFER_LENGTH,
        },
    };

    #[test]
    fn test_save_module_image_and_load_module_image() {
        // build TypeSection instance
        // note: arbitrary types
        let type_entries = vec![
            TypeEntry {
                params: vec![OperandDataType::I32, OperandDataType::I64],
                results: vec![OperandDataType::F32],
            },
            TypeEntry {
                params: vec![],
                results: vec![OperandDataType::F64],
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
            FunctionIndexListEntry::new(vec![
                FunctionIndexEntry::new(0, 1, 3),
                FunctionIndexEntry::new(1, 5, 7),
            ]),
            FunctionIndexListEntry::new(vec![FunctionIndexEntry::new(0, 11, 13)]),
        ];

        let (function_index_ranges, function_index_items) =
            FunctionIndexSection::convert_from_entries(&function_index_module_entries);

        let function_index_section = FunctionIndexSection {
            ranges: &function_index_ranges,
            items: &function_index_items,
        };

        // build property section
        let mut module_name_buffer = [0u8; MODULE_NAME_BUFFER_LENGTH];
        module_name_buffer[0] = 29;
        module_name_buffer[1] = 31;
        module_name_buffer[2] = 37;

        let property_section = PropertySection {
            runtime_major_version: 11,
            runtime_minor_version: 13,
            entry_function_public_index: 17,
            constructor_function_public_index: 19,
            destructor_function_public_index: 23,
            module_name_length: 3,
            module_name_buffer,
        };

        // build ModuleImage instance
        let section_entries: Vec<&dyn SectionEntry> = vec![
            &type_section,
            &function_section,
            &local_var_section,
            &function_index_section,
            &property_section,
        ];

        let (section_items, sections_data) = ModuleImage::convert_from_entries(&section_entries);
        let module_image = ModuleImage {
            items: &section_items,
            sections_data: &sections_data,
        };

        // save
        let mut image_data: Vec<u8> = Vec::new();
        module_image.save(&mut image_data).unwrap();

        assert_eq!(&image_data[0..8], IMAGE_FILE_MAGIC_NUMBER);
        assert_eq!(&image_data[8..10], &[0, 0]); // image format minor version number, little endian
        assert_eq!(&image_data[10..12], &[1, 0]); // image format major version number, little endian
        assert_eq!(&image_data[12..16], &[0, 0, 0, 0]); // padding

        // body
        let remains = &image_data[MODULE_HEADER_LENGTH..];

        // section count
        let (section_count_data, remains) = remains.split_at(8);
        assert_eq!(&section_count_data[0..4], &[5, 0, 0, 0]); // section item count
        assert_eq!(&section_count_data[4..8], &[0, 0, 0, 0]); // padding

        // section table length = 12 (the record length) * 5= 60
        let (section_table_data, remains) = remains.split_at(60);

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
                0x41, 0, 0, 0, // section id, property section
                216, 0, 0, 0, // offset 6,
                20, 1, 0, 0 // length 256 + 20
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

        assert_eq!(
            &remains[..20],
            &[
                11, 0, // runtime major version
                13, 0, // runtime minor version
                17, 0, 0, 0, // entry function public index
                19, 0, 0, 0, // constructor function public index
                23, 0, 0, 0, // destructor function public index
                3, 0, 0, 0, // name length
            ]
        );

        // load
        let module_image_restore = ModuleImage::load(&image_data).unwrap();
        assert_eq!(module_image_restore.items.len(), 5);

        // check type

        let type_section_restore = module_image_restore.get_type_section();
        assert_eq!(type_section_restore.items.len(), 2);

        assert_eq!(
            type_section_restore.get_item_params_and_results(0),
            (
                vec![OperandDataType::I32, OperandDataType::I64].as_ref(),
                vec![OperandDataType::F32].as_ref(),
            )
        );

        assert_eq!(
            type_section_restore.get_item_params_and_results(1),
            ([].as_ref(), vec![OperandDataType::F64].as_ref(),)
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
            &[LocalVariableItem::new(0, 12, MemoryDataType::Raw, 4),]
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

        // check property section
        let property_section_restore = module_image_restore.get_property_section();
        assert_eq!(property_section_restore.runtime_major_version, 11);
        assert_eq!(property_section_restore.runtime_minor_version, 13);
        assert_eq!(property_section_restore.entry_function_public_index, 17);
        assert_eq!(
            property_section_restore.constructor_function_public_index,
            19
        );
        assert_eq!(
            property_section_restore.destructor_function_public_index,
            23
        );
        assert_eq!(property_section_restore.module_name_length, 3);
        assert_eq!(
            property_section_restore.module_name_buffer[..3],
            [29, 31, 37]
        );
    }
}
