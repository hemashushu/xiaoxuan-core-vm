// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// a module consists of two parts, data and code (i.e., instructions), which
// are spread out in the following sections:
//
// - import data section (optional)
// - data sections (optional)
//   there are 3 kinds of data sections: read-only, read-write, uninit(ialized)
//   all data are thread-local, so the read-write section will be cloned and the
//   uninitialized section will be allocated when a new thread is created.
//
// - function type section
//   the signature of a function, the types are also applied to the code blocks.
// - import function section (optional)
// - function section
// - local variable section
//   a function is consists of a type, a local variable list, and instructions
//
// - (import) external C function section (optional)
//   a list of external C functions
// - export data section (optional)
// - export function section (optional)
// - auto function index list section (optional)
//   the index of functions that are executed before application start, after application exits and
//   the index of the entry (main) function.
//
// a minimal module only requires 3 sections:
//
// - function type section
// - function section
// - local variable section
//
// of these, the following sections are not required during the runtime, they are generally used for debuging
// and linking.
//
// - import data section
// - import function section
// - export data section
// - export function section
//
// because in the modules linking stage (which follows the compiling stage), all imports and exports
// are resolved and stored the indices in the following sections,
// this help speeding up the next time the program loading:
//
// - module index section
// - data index section (optional)
// - func index section
//
// note that only the application main module contains these sections.
//
//
// about the design of module:
//
// the loading and startup of XiaoXuan modules are extremely fast, because:
// - there is no parsing process, the loading process actually does only two things: maps
//   the module image file into memory, and locates the start and end positions of echo
//   sections.
// - instructions are executed directly on the binary (bytecode of the module)
//
// these allow the XiaoXuan applications to have almost no startup time, and are suitable
// for using as 'function' in scripts.

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
pub mod func_index_section;
pub mod func_section;
pub mod local_variable_section;
pub mod module_index_section;
pub mod type_section;

use crate::{
    module_image::data_index_section::DataIndexSection,
    module_image::func_index_section::FuncIndexSection,
    module_image::func_section::FuncSection,
    module_image::module_index_section::ModuleIndexSection,
    module_image::type_section::TypeSection,
    utils::{load_section_with_table_and_data_area, save_section_with_table_and_data_area},
    BinaryError,
};

use self::{
    data_section::{ReadOnlyDataSection, ReadWriteDataSection, UninitDataSection},
    local_variable_section::LocalVariableSection,
};

// the "module image file" binary layout:
//
//              |--------------------------------------------------------------------------|
//              | magic number (u64) | minor ver (u16) | major ver (u16) | padding 4 bytes |
//              |--------------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                                     |
//              |--------------------------------------------------------------------------|
//   item 0 --> | section id 0 (u32) | offset 0 (u32) | length 0 (u32)                     | <-- table
//   item 1 --> | section id 1       | offset 1       | length 1                           |
//              | ...                                                                      |
//              |--------------------------------------------------------------------------|
// offset 0 --> | section data 0                                                           | <-- data
// offset 1 --> | section data 1                                                           |
//              | ...                                                                      |
//              |--------------------------------------------------------------------------|

const IMAGE_MAGIC_NUMBER: &[u8; 8] = b"ancsmod\0"; // for "ANCS module"
const IMAGE_MAJOR_VERSION: u16 = 1;
const IMAGE_MINOR_VERSION: u16 = 0;

#[derive(Debug, PartialEq)]
pub struct ModuleImage<'a> {
    pub items: &'a [ModuleSection],
    pub sections_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct ModuleSection {
    pub id: SectionId, // u32
    pub offset: u32,
    pub length: u32,
}

impl ModuleSection {
    pub fn new(id: SectionId, offset: u32, length: u32) -> Self {
        Self { id, offset, length }
    }
}

#[repr(u32)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SectionId {
    ReadOnlyData = 0x10, // 0x10
    ReadWriteData,       // 0x11
    UninitData,          // 0x12
    //
    Type = 0x20,   // 0x20
    Func,          // 0x21
    LocalVariable, // 0x22
    //
    ImportData = 0x30, // 0x30
    ImportFunc,        // 0x31
    ExportData,        // 0x32
    ExportFunc,        // 0x33
    ExternalFunc,      // 0x34
    AutoFunc,          // 0x35
    //
    ModuleIndex = 0x40, // 0x40
    DataIndex,          // 0x41
    FuncIndex,          // 0x42
}

// impl From<u32> for SectionId {
//     fn from(value: u32) -> Self {
//         unsafe { std::mem::transmute::<u32, SectionId>(value) }
//     }
// }

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
    fn id(&'a self) -> SectionId;
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

        let image_body = &image_data[16..];

        // since the structure of module image and a section are the same,
        // that is, the module image itself can be thought of
        // as a 'big' section that contains many child sections.
        // so we can load module by reusing function
        // `load_section_with_table_and_data_area` as well.
        let (items, sections_data) =
            load_section_with_table_and_data_area::<ModuleSection>(image_body);

        Ok(Self {
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

    pub fn get_section_index_by_id(&'a self, section_id: SectionId) -> Option<usize> {
        self.items.iter().enumerate().find_map(|(idx, item)| {
            if item.id == section_id {
                Some(idx)
            } else {
                None
            }
        })
    }

    fn get_section_data_by_id(&'a self, section_id: SectionId) -> Option<&'a [u8]> {
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

    pub fn get_module_index_section(&'a self) -> ModuleIndexSection<'a> {
        let opt_section_data = self.get_section_data_by_id(SectionId::ModuleIndex);
        if let Some(section_data) = opt_section_data {
            ModuleIndexSection::load(section_data)
        } else {
            panic!("Can not found the module index section.")
        }
    }

    pub fn get_data_index_section(&'a self) -> DataIndexSection<'a> {
        let opt_section_data = self.get_section_data_by_id(SectionId::DataIndex);
        if let Some(section_data) = opt_section_data {
            DataIndexSection::load(section_data)
        } else {
            panic!("Can not found the data index section.")
        }
    }

    pub fn get_func_index_section(&'a self) -> FuncIndexSection<'a> {
        let opt_section_data = self.get_section_data_by_id(SectionId::FuncIndex);
        if let Some(section_data) = opt_section_data {
            FuncIndexSection::load(section_data)
        } else {
            panic!("Can not found the function index section.")
        }
    }

    pub fn get_read_only_data_section(&'a self) -> ReadOnlyDataSection<'a> {
        let opt_section_data = self.get_section_data_by_id(SectionId::ReadOnlyData);
        if let Some(section_data) = opt_section_data {
            ReadOnlyDataSection::load(section_data)
        } else {
            panic!("Can not found the read-only data section.")
        }
    }

    pub fn get_read_write_data_section(&'a self) -> ReadWriteDataSection<'a> {
        let opt_section_data = self.get_section_data_by_id(SectionId::ReadWriteData);
        if let Some(section_data) = opt_section_data {
            ReadWriteDataSection::load(section_data)
        } else {
            panic!("Can not found the read-write data section.")
        }
    }

    pub fn get_uninit_data_section(&'a self) -> UninitDataSection<'a> {
        let opt_section_data = self.get_section_data_by_id(SectionId::UninitData);
        if let Some(section_data) = opt_section_data {
            UninitDataSection::load(section_data)
        } else {
            panic!("Can not found the uninitialized data section.")
        }
    }

    pub fn get_type_section(&'a self) -> TypeSection<'a> {
        let opt_section_data = self.get_section_data_by_id(SectionId::Type);
        if let Some(section_data) = opt_section_data {
            TypeSection::load(section_data)
        } else {
            panic!("Can not found the type section.")
        }
    }

    pub fn get_func_section(&'a self) -> FuncSection<'a> {
        let opt_section_data = self.get_section_data_by_id(SectionId::Func);
        if let Some(section_data) = opt_section_data {
            FuncSection::load(section_data)
        } else {
            panic!("Can not found the function section.")
        }
    }

    pub fn get_local_variable_section(&'a self) -> LocalVariableSection<'a> {
        let opt_section_data = self.get_section_data_by_id(SectionId::LocalVariable);
        if let Some(section_data) = opt_section_data {
            LocalVariableSection::load(section_data)
        } else {
            panic!("Can not found the local variable section.")
        }
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
        module_index_section::{ModuleIndexEntry, ModuleIndexSection, ModuleShareType},
        type_section::{TypeEntry, TypeSection},
        ModuleImage, RangeItem, SectionEntry, IMAGE_MAGIC_NUMBER,
    };

    #[test]
    fn test_module_data_sections() {
        // TODO::
    }

    #[test]
    fn test_module_function_sections() {
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
        let code0: Vec<u8> = vec![1u8, 2, 3, 5, 7];
        let code1: Vec<u8> = vec![11u8, 13, 17, 19, 23, 29];

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

        assert_eq!(&image_data[16..20], &vec![3, 0, 0, 0]); // item count
        assert_eq!(&image_data[20..24], &vec![0, 0, 0, 0]); // padding

        // image header 24 bytes
        let remains = &image_data[24..];

        // section table length = 12 (the record length) * 3
        let (section_table_data, remains) = remains.split_at(36);

        assert_eq!(
            section_table_data,
            &vec![
                0x20u8, 0, 0, 0, // section id
                0, 0, 0, 0, // offset 0
                36, 0, 0, 0, // length 0
                //
                0x21u8, 0, 0, 0, // section id
                36, 0, 0, 0, // offset 1
                52, 0, 0, 0, // length 1
                //
                0x22u8, 0, 0, 0, // section id
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
            type_section_restore.get_entry(0),
            TypeEntry {
                params: type0,
                results: type1,
            }
        );

        assert_eq!(
            type_section_restore.get_entry(1),
            TypeEntry {
                params: type2,
                results: type3,
            }
        );

        // check func

        let func_section_restore = module_image_restore.get_func_section();
        assert_eq!(func_section_restore.items.len(), 2);

        assert_eq!(
            func_section_restore.get_entry(0),
            FuncEntry {
                type_index: 2,
                local_index: 3,
                code: code0,
            }
        );

        assert_eq!(
            func_section_restore.get_entry(1),
            FuncEntry {
                type_index: 5,
                local_index: 7,
                code: code1,
            }
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
    fn test_module_index_sections() {
        // build ModuleIndexSection instance
        let mut module_index_entries: Vec<ModuleIndexEntry> = Vec::new();
        module_index_entries.push(ModuleIndexEntry::new(
            ModuleShareType::Local,
            "main".to_string(),
        ));
        module_index_entries.push(ModuleIndexEntry::new(
            ModuleShareType::Shared,
            "httpclient".to_string(),
        ));

        let (module_index_items, names_data) =
            ModuleIndexSection::convert_from_entries(&module_index_entries);
        let module_index_section = ModuleIndexSection {
            items: &module_index_items,
            names_data: &names_data,
        };

        // build DataIndexSection instance
        let data_range0 = RangeItem::new(0, 3);

        let data_index_item0 = DataIndexItem::new(0, 1, DataSectionType::ReadOnly, 2);
        let data_index_item1 = DataIndexItem::new(3, 5, DataSectionType::ReadWrite, 7);
        let data_index_item2 = DataIndexItem::new(11, 13, DataSectionType::Uninit, 17);

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

        // build IndexMap instance

        let section_entries: Vec<&dyn SectionEntry> = vec![
            &module_index_section,
            &data_index_section,
            &func_index_section,
        ];
        let (section_items, sections_data) = ModuleImage::convert_from_entries(&section_entries);
        let module_image = ModuleImage {
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

        assert_eq!(&image_data[16..20], &vec![3, 0, 0, 0]); // item count
        assert_eq!(&image_data[20..24], &vec![0, 0, 0, 0]); // padding

        // image header 24 bytes
        let remains = &image_data[24..];

        // section table length = 12 (record length) * 3
        let (section_table_data, remains) = remains.split_at(36);

        assert_eq!(
            section_table_data,
            &vec![
                0x40u8, 0, 0, 0, // section id 0
                0, 0, 0, 0, // offset 0
                48, 0, 0, 0, // length 0
                //
                0x41u8, 0, 0, 0, // section id 1
                48, 0, 0, 0, // offset 1
                64, 0, 0, 0, // length 1
                //
                0x42u8, 0, 0, 0, // section id 2
                112, 0, 0, 0, // offset 2
                28, 0, 0, 0, // length 2
            ]
        );

        let (module_index_section_data, remains) = remains.split_at(48);

        assert_eq!(
            module_index_section_data,
            &vec![
                2, 0, 0, 0, // item count
                0, 0, 0, 0, // padding
                0, 0, 0, 0, // name offset 0
                4, 0, 0, 0, // name length 0
                0, // module type 0
                0, 0, 0, // padding
                //
                4, 0, 0, 0, // name offset 1
                10, 0, 0, 0, // name length 1
                1, // module type 1
                0, 0, 0, // padding
                //
                109, 97, 105, 110, // b"main"
                104, 116, 116, 112, 99, 108, 105, 101, 110, 116, // b"httpclient"
                0, 0, // padding for 4-byte alignment
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
                0, // target data section type 0
                0, 0, 0, // padding 0
                2, 0, 0, 0, // target data idx 0
                //
                3, 0, 0, 0, // data idx 1
                5, 0, 0, 0, // target module idx 1
                1, // target data section type 1
                0, 0, 0, // padding 1
                7, 0, 0, 0, // target data idx 1
                //
                11, 0, 0, 0, // data idx 2
                13, 0, 0, 0, // target module idx 2
                2, // target data section type 2
                0, 0, 0, // padding 2
                17, 0, 0, 0, // target data idx 2
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
        assert_eq!(module_image_restore.items.len(), 3);

        let module_index_section_restore = module_image_restore.get_module_index_section();

        assert_eq!(module_index_section_restore.items.len(), 2);

        assert_eq!(
            module_index_section_restore.get_entry(0),
            ModuleIndexEntry::new(ModuleShareType::Local, "main".to_string(),)
        );

        assert_eq!(
            module_index_section_restore.get_entry(1),
            ModuleIndexEntry::new(ModuleShareType::Shared, "httpclient".to_string(),)
        );

        let data_index_section_restore = module_image_restore.get_data_index_section();

        assert_eq!(data_index_section_restore.ranges.len(), 1);
        assert_eq!(data_index_section_restore.items.len(), 3);

        assert_eq!(&data_index_section_restore.ranges[0], &RangeItem::new(0, 3));

        assert_eq!(
            &data_index_section_restore.items[0],
            &DataIndexItem::new(0, 1, DataSectionType::ReadOnly, 2)
        );
        assert_eq!(
            &data_index_section_restore.items[1],
            &DataIndexItem::new(3, 5, DataSectionType::ReadWrite, 7)
        );
        assert_eq!(
            &data_index_section_restore.items[2],
            &DataIndexItem::new(11, 13, DataSectionType::Uninit, 17)
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
