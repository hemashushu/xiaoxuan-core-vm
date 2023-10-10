// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

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
// - export function section
// - import data section
// - export data section
// - external library section
// - external function section

// a minimal module only requires 3 sections:
//
// - function type section
// - function section
// - local variable section
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
// - export function section
// - import data section
// - export data section
// - external library section
// - external function section

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
pub mod data_section;
pub mod external_func_index_section;
pub mod external_func_section;
pub mod external_library_section;
pub mod func_index_section;
pub mod func_section;
pub mod local_variable_section;
pub mod type_section;
pub mod unified_external_func_section;
pub mod unified_external_library_section;

use crate::{
    module_image::data_index_section::DataIndexSection,
    module_image::func_index_section::FuncIndexSection,
    module_image::func_section::FuncSection,
    module_image::type_section::TypeSection,
    utils::{load_section_with_table_and_data_area, save_section_with_table_and_data_area},
    BinaryError,
};

use self::{
    data_section::{ReadOnlyDataSection, ReadWriteDataSection, UninitDataSection},
    external_func_index_section::ExternalFuncIndexSection,
    external_func_section::ExternalFuncSection,
    external_library_section::ExternalLibrarySection,
    local_variable_section::LocalVariableSection,
    unified_external_func_section::UnifiedExternalFuncSection,
    unified_external_library_section::UnifiedExternalLibrarySection,
};

// the "module image file" binary layout:
//
//                 header
//              |--------------------------------------------------------------------------|
//              | magic number (u64) | minor ver (u16) | major ver (u16) | padding 4 bytes | 16 bytes
//              |--------------------------------------------------------------------------|
//              | name length (u16) | module name (256 - 2) bytes (UTF-8)                  | 256 bytes
//              |--------------------------------------------------------------------------|
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

const IMAGE_MAGIC_NUMBER: &[u8; 8] = b"ancsmod\0"; // the abbr of "ANCS module"
const IMAGE_MAJOR_VERSION: u16 = 1;
const IMAGE_MINOR_VERSION: u16 = 0;

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
    Func,          // 0x11
    LocalVariable, // 0x12
    // optional
    ReadOnlyData = 0x20, // 0x20
    ReadWriteData,       // 0x21
    UninitData,          // 0x22
    AutoFunc,            // 0x23
    // for debuging and linking
    ImportFunc = 0x30, // 0x30
    ExportFunc,        // 0x31
    ImportData,        // 0x32
    ExportData,        // 0x33
    ExternalLibrary,   // 0x34
    ExternalFunc,      // 0x35
    // indices
    FuncIndex = 0x40,       // 0x40
    DataIndex,              // 0x41
    UnifiedExternalLibrary, // 0x42
    UnifiedExternalFunc,    // 0x43
    ExternalFuncIndex,      // 0x44 (mapping ext-func to uni-ext-func)
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
    // there is a way to 'downcast' a section entry to section object, e.g.
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
            return Err(BinaryError {
                message: "Not a module image file.".to_owned(),
            });
        }

        // the another safe way
        // ```rust
        //     let version_data: [u8;4] = (&image_data[4..8]).try_into().unwrap();
        //     let version = u32::from_le_bytes(version_data);
        // ```

        let ptr = image_data.as_ptr();
        let ptr_version = unsafe { ptr.offset(8) };
        let version = unsafe { std::ptr::read(ptr_version as *const u32) };

        let runtime_version = ((IMAGE_MAJOR_VERSION as u32) << 16) | (IMAGE_MINOR_VERSION as u32);
        if version > runtime_version {
            return Err(BinaryError {
                message: "The module requires a newer version of the runtime.".to_owned(),
            });
        }

        let name_area = &image_data[16..(16 + 256)];
        let name_length = unsafe { std::ptr::read(name_area.as_ptr() as *const u16) };
        let name_data = &image_data[(16 + 2)..(16 + 2 + (name_length as usize))];
        let name = std::str::from_utf8(name_data).unwrap();

        let image_body = &image_data[(16 + 256)..];

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
        writer.write_all(&[0u8, 0, 0, 0])?; // padding, 4 bytes

        let mut name_area = [0u8; 256];
        unsafe { std::ptr::write(name_area.as_mut_ptr() as *mut u16, self.name.len() as u16) };
        unsafe {
            std::ptr::copy(
                self.name.as_ptr(),
                name_area.as_mut_ptr().offset(2),
                self.name.len(),
            )
        };
        writer.write_all(&name_area)?;

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
                |section_data| TypeSection::load(section_data),
            )
    }

    // essential section
    pub fn get_func_section(&'a self) -> FuncSection<'a> {
        self.get_section_data_by_id(ModuleSectionId::Func)
            .map_or_else(
                || panic!("Can not find the function section."),
                |section_data| FuncSection::load(section_data),
            )
    }

    // essential section
    pub fn get_local_variable_section(&'a self) -> LocalVariableSection<'a> {
        self.get_section_data_by_id(ModuleSectionId::LocalVariable)
            .map_or_else(
                || panic!("Can not find the local variable section."),
                |section_data| LocalVariableSection::load(section_data),
            )
    }

    // essential section
    pub fn get_func_index_section(&'a self) -> FuncIndexSection<'a> {
        self.get_section_data_by_id(ModuleSectionId::FuncIndex)
            .map_or_else(
                || panic!("Can not find the function index section."),
                |section_data| FuncIndexSection::load(section_data),
            )
    }

    // optional section
    pub fn get_optional_read_only_data_section(&'a self) -> Option<ReadOnlyDataSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::ReadOnlyData)
            .map(|section_data| ReadOnlyDataSection::load(section_data))
    }

    // optional section
    pub fn get_optional_read_write_data_section(&'a self) -> Option<ReadWriteDataSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::ReadWriteData)
            .map(|section_data| ReadWriteDataSection::load(section_data))
    }

    // optional section
    pub fn get_optional_uninit_data_section(&'a self) -> Option<UninitDataSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::UninitData)
            .map(|section_data| UninitDataSection::load(section_data))
    }

    // todo get_optional_auto_func_section

    // todo get_optional_import_func_section

    // todo get_optional_export_func_section

    // todo get_optional_import_data_section

    // todo get_optional_export_data_section

    // optional
    pub fn get_optional_external_library_section(&'a self) -> Option<ExternalLibrarySection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::ExternalLibrary)
            .map(|section_data| ExternalLibrarySection::load(section_data))
    }

    // optional
    pub fn get_optional_external_func_section(&'a self) -> Option<ExternalFuncSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::ExternalFunc)
            .map(|section_data| ExternalFuncSection::load(section_data))
    }

    // optional
    pub fn get_optional_data_index_section(&'a self) -> Option<DataIndexSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::DataIndex)
            .map(|section_data| DataIndexSection::load(section_data))
    }

    // optional
    pub fn get_optional_unified_external_library_section(
        &'a self,
    ) -> Option<UnifiedExternalLibrarySection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::UnifiedExternalLibrary)
            .map(|section_data| UnifiedExternalLibrarySection::load(section_data))
    }

    // optional
    pub fn get_optional_unified_external_func_section(
        &'a self,
    ) -> Option<UnifiedExternalFuncSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::UnifiedExternalFunc)
            .map(|section_data| UnifiedExternalFuncSection::load(section_data))
    }

    // optional
    pub fn get_optional_external_func_index_section(
        &'a self,
    ) -> Option<ExternalFuncIndexSection<'a>> {
        self.get_section_data_by_id(ModuleSectionId::ExternalFuncIndex)
            .map(|section_data| ExternalFuncIndexSection::load(section_data))
    }
}

#[cfg(test)]
mod tests {
    use ancvm_types::{DataType, MemoryDataType};

    use crate::module_image::{
        data_index_section::{DataIndexItem, DataIndexSection},
        data_section::DataSectionType,
        func_index_section::{FuncIndexItem, FuncIndexSection},
        func_section::{FuncEntry, FuncSection},
        local_variable_section::{
            LocalVariableEntry, LocalVariableItem, LocalVariableListEntry, LocalVariableSection,
        },
        type_section::{TypeEntry, TypeSection},
        ModuleImage, RangeItem, SectionEntry, IMAGE_MAGIC_NUMBER,
    };

    #[test]
    fn test_module_common_sections_save_and_load() {
        // build TypeSection instance

        let mut type_entries: Vec<TypeEntry> = Vec::new();
        let type0 = vec![DataType::I32, DataType::I64];
        let type1 = vec![DataType::F32];
        let type2 = vec![];
        let type3 = vec![DataType::F64];

        type_entries.push(TypeEntry {
            params: type0.clone(),
            results: type1.clone(),
        });

        type_entries.push(TypeEntry {
            params: type2.clone(),
            results: type3.clone(),
        });

        let (type_items, types_data) = TypeSection::convert_from_entries(&type_entries);
        let type_section = TypeSection {
            items: &type_items,
            types_data: &types_data,
        };

        // build FuncSection instance

        let mut func_entries: Vec<FuncEntry> = Vec::new();
        let code0: Vec<u8> = vec![1u8, 2, 3, 5, 7]; // arbitrary code
        let code1: Vec<u8> = vec![11u8, 13, 17, 19, 23, 29]; // arbitrary code

        func_entries.push(FuncEntry {
            type_index: 2,
            local_index: 3,
            code: code0.clone(),
        });
        func_entries.push(FuncEntry {
            type_index: 5,
            local_index: 7,
            code: code1.clone(),
        });

        let (func_items, codes_data) = FuncSection::convert_from_entries(&func_entries);
        let func_section = FuncSection {
            items: &func_items,
            codes_data: &codes_data,
        };

        // build LocalVariableSection instance

        // note:
        // the local variable list should include the function arguments, but
        // it's ok in this unit test scenario.
        let mut local_var_list_entries: Vec<LocalVariableListEntry> = Vec::new();
        local_var_list_entries.push(LocalVariableListEntry::new(vec![
            LocalVariableEntry::from_i32(),
            LocalVariableEntry::from_i64(),
        ]));
        local_var_list_entries.push(LocalVariableListEntry::new(vec![
            LocalVariableEntry::from_bytes(12, 4),
        ]));

        let (local_var_lists, local_var_list_data) =
            LocalVariableSection::convert_from_entries(&local_var_list_entries);
        let local_var_section = LocalVariableSection {
            lists: &local_var_lists,
            list_data: &local_var_list_data,
        };

        // build ModuleImage instance
        let section_entries: Vec<&dyn SectionEntry> =
            vec![&type_section, &func_section, &local_var_section];
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
        assert_eq!(&image_data[8..10], &vec![0, 0]); // minor version number, little endian
        assert_eq!(&image_data[10..12], &vec![1, 0]); // major version number, little endian
        assert_eq!(&image_data[12..16], &vec![0, 0, 0, 0]);

        // name area
        assert_eq!(&image_data[16..(16 + 2)], &vec![4, 0]);
        assert_eq!(&image_data[(16 + 2)..(16 + 2 + 4)], &b"main".to_vec());

        // section count
        assert_eq!(&image_data[272..276], &vec![3, 0, 0, 0]); // item count
        assert_eq!(&image_data[276..280], &vec![0, 0, 0, 0]); // padding

        // image header 16 + 256 + section count 8 bytes
        let remains = &image_data[280..];

        // section table length = 12 (the record length) * 3
        let (section_table_data, remains) = remains.split_at(36);

        assert_eq!(
            section_table_data,
            &vec![
                0x10u8, 0, 0, 0, // section id, type section
                0, 0, 0, 0, // offset 0
                36, 0, 0, 0, // length 0
                //
                0x11u8, 0, 0, 0, // section id, func section
                36, 0, 0, 0, // offset 1
                52, 0, 0, 0, // length 1
                //
                0x12u8, 0, 0, 0, // section id, local variable section
                88, 0, 0, 0, // offset 1
                68, 0, 0, 0, // length 1
            ]
        );

        let (type_section_data, remains) = remains.split_at(36);
        assert_eq!(
            type_section_data,
            &vec![
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
            &vec![
                2, 0, 0, 0, // item count
                0, 0, 0, 0, // padding
                //
                0, 0, 0, 0, // code offset 0
                5, 0, 0, 0, // code len 0
                2, 0, 0, 0, // func type index 0
                3, 0, 0, 0, // local index 0
                //
                5, 0, 0, 0, // code offset 1
                6, 0, 0, 0, // code len 1
                5, 0, 0, 0, // func type index 1
                7, 0, 0, 0, // local index 1
                //
                1, 2, 3, 5, 7, // code 0
                11, 13, 17, 19, 23, 29, // code 1
                //
                0, // padding
            ]
        );

        assert_eq!(
            remains,
            &vec![
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

        // load
        let module_image_restore = ModuleImage::load(&image_data).unwrap();
        assert_eq!(module_image_restore.items.len(), 3);

        // check type

        let type_section_restore = module_image_restore.get_type_section();
        assert_eq!(type_section_restore.items.len(), 2);

        assert_eq!(
            type_section_restore.get_item_params_and_results(0),
            (type0.as_ref(), type1.as_ref(),)
        );

        assert_eq!(
            type_section_restore.get_item_params_and_results(1),
            (type2.as_ref(), type3.as_ref(),)
        );

        // check func

        let func_section_restore = module_image_restore.get_func_section();
        assert_eq!(func_section_restore.items.len(), 2);

        assert_eq!(
            func_section_restore.get_item_type_index_and_local_variable_index_and_code(0),
            (2, 3, code0.as_ref(),)
        );

        assert_eq!(
            func_section_restore.get_item_type_index_and_local_variable_index_and_code(1),
            (5, 7, code1.as_ref(),)
        );

        // check local vars

        let local_var_section_restore = module_image_restore.get_local_variable_section();
        assert_eq!(local_var_section_restore.lists.len(), 2);

        assert_eq!(
            local_var_section_restore.get_variable_list(0),
            &vec![
                LocalVariableItem::new(0, 4, MemoryDataType::I32, 4),
                LocalVariableItem::new(8, 8, MemoryDataType::I64, 8),
            ]
        );

        assert_eq!(
            local_var_section_restore.get_variable_list(1),
            &vec![LocalVariableItem::new(0, 12, MemoryDataType::BYTES, 4),]
        );
    }

    #[test]
    fn test_module_index_sections_save_and_load() {
        // build DataIndexSection instance
        let data_range0 = RangeItem::new(0, 3);

        let data_index_item0 = DataIndexItem::new(0, 1, 2, DataSectionType::ReadOnly);
        let data_index_item1 = DataIndexItem::new(3, 5, 7, DataSectionType::ReadWrite);
        let data_index_item2 = DataIndexItem::new(11, 13, 17, DataSectionType::Uninit);

        let data_index_section = DataIndexSection {
            ranges: &vec![data_range0],
            items: &vec![data_index_item0, data_index_item1, data_index_item2],
        };

        // build FuncIndexSection instance
        let func_range0 = RangeItem::new(0, 1);

        let func_index_item0 = FuncIndexItem::new(0, 1, 2);

        let func_index_section = FuncIndexSection {
            ranges: &vec![func_range0],
            items: &vec![func_index_item0],
        };

        // build module image
        let section_entries: Vec<&dyn SectionEntry> =
            vec![&data_index_section, &func_index_section];
        let (section_items, sections_data) = ModuleImage::convert_from_entries(&section_entries);
        let module_image = ModuleImage {
            name: "std",
            items: &section_items,
            sections_data: &sections_data,
        };

        // save
        let mut image_data: Vec<u8> = Vec::new();
        module_image.save(&mut image_data).unwrap();

        assert_eq!(&image_data[0..8], IMAGE_MAGIC_NUMBER);
        assert_eq!(&image_data[8..10], &vec![0, 0]); // minor version number, little endian
        assert_eq!(&image_data[10..12], &vec![1, 0]); // major version number, little endian
        assert_eq!(&image_data[12..16], &vec![0, 0, 0, 0]);

        // name area
        assert_eq!(&image_data[16..(16 + 2)], &vec![3, 0]);
        assert_eq!(&image_data[(16 + 2)..(16 + 2 + 3)], &b"std".to_vec());

        // section count
        assert_eq!(&image_data[272..276], &vec![2, 0, 0, 0]); // item count
        assert_eq!(&image_data[276..280], &vec![0, 0, 0, 0]); // padding

        // image header 16 + 256 + section count 8 bytes
        let remains = &image_data[280..];

        // section table length = 12 (record length) * 2
        let (section_table_data, remains) = remains.split_at(24);

        assert_eq!(
            section_table_data,
            &vec![
                0x41u8, 0, 0, 0, // section id 0, data index
                0, 0, 0, 0, // offset 0
                64, 0, 0, 0, // length 0
                //
                0x40u8, 0, 0, 0, // section id 1, func index
                64, 0, 0, 0, // offset 1
                28, 0, 0, 0, // length 1
            ]
        );

        let (data_index_section_data, remains) = remains.split_at(64);

        assert_eq!(
            data_index_section_data,
            &vec![
                /* table 0 */
                1, 0, 0, 0, // item count
                0, 0, 0, 0, // padding
                0, 0, 0, 0, // offset 0
                3, 0, 0, 0, // count 0
                /* table 1 */
                0, 0, 0, 0, // data idx 0
                1, 0, 0, 0, // target module idx 0
                2, 0, 0, 0, // target data idx 0
                0, // target data section type 0
                0, 0, 0, // padding 0
                //
                3, 0, 0, 0, // data idx 1
                5, 0, 0, 0, // target module idx 1
                7, 0, 0, 0, // target data idx 1
                1, // target data section type 1
                0, 0, 0, // padding 1
                //
                11, 0, 0, 0, // data idx 2
                13, 0, 0, 0, // target module idx 2
                17, 0, 0, 0, // target data idx 2
                2, // target data section type 2
                0, 0, 0, // padding 2
            ]
        );

        let (func_index_section_data, _remains) = remains.split_at(28);

        assert_eq!(
            func_index_section_data,
            &vec![
                /* table 0 */
                1, 0, 0, 0, // item count
                0, 0, 0, 0, // padding
                0, 0, 0, 0, // offset 0
                1, 0, 0, 0, // count 0
                /* table 1 */
                0, 0, 0, 0, // func idx 0
                1, 0, 0, 0, // target module idx 0
                2, 0, 0, 0, // target func idx 0
            ]
        );

        // load
        let module_image_restore = ModuleImage::load(&image_data).unwrap();
        assert_eq!(module_image_restore.items.len(), 2);

        let data_index_section_restore = module_image_restore
            .get_optional_data_index_section()
            .unwrap();

        assert_eq!(data_index_section_restore.ranges.len(), 1);
        assert_eq!(data_index_section_restore.items.len(), 3);

        assert_eq!(&data_index_section_restore.ranges[0], &RangeItem::new(0, 3));

        assert_eq!(
            &data_index_section_restore.items[0],
            &DataIndexItem::new(0, 1, 2, DataSectionType::ReadOnly,)
        );
        assert_eq!(
            &data_index_section_restore.items[1],
            &DataIndexItem::new(3, 5, 7, DataSectionType::ReadWrite,)
        );
        assert_eq!(
            &data_index_section_restore.items[2],
            &DataIndexItem::new(11, 13, 17, DataSectionType::Uninit,)
        );

        let func_index_section_restore = module_image_restore.get_func_index_section();

        assert_eq!(func_index_section_restore.ranges.len(), 1);
        assert_eq!(func_index_section_restore.items.len(), 1);

        assert_eq!(
            &func_index_section_restore.ranges[0],
            &RangeItem::new(0, 1,)
        );

        assert_eq!(
            &func_index_section_restore.items[0],
            &FuncIndexItem::new(0, 1, 2)
        );
    }
}
