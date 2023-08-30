// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// a module consists of a header and several sections:
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
// - (import) external C function section (optional)
//   a list of external C functions
// - auto function index list section (optional)
//   the index of functions that are executed before application start, after application exits and
//   the index of the entry (main) function.
//
// a minimal module can only contains 2 sections:
//
// - function type section
// - function section
//
// the following sections are not required during the runtime, they are generally used for debuging
// and linking.
//
// - import data section
// - import function section
// - export data section
// - export function section
// - import external C function section
//
// because once all modules (source file) have been compiled, all imports and exports are resolved
// and stored in the following sections, these sections speed up the next time the program loading:
// note that only the application main module contains these sections.
//
// - module index section
// - data index section (optional)
// - func index section
//
// there are also some in-memory tables, they are created at application startup:
//
// - external function index (optional)
// - library list (optional)

pub mod data_index_section;
pub mod data_section;
pub mod func_index_section;
pub mod func_section;
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

use self::data_section::{ReadOnlyDataSection, ReadWriteDataSection, UninitDataSection};

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

const MAGIC_NUMBER: &[u8; 8] = b"ancsmod\0"; // for "ANCS module"
const MAJOR_VERSION: u16 = 1;
const MINOR_VERSION: u16 = 0;

#[derive(Debug, PartialEq)]
pub struct ModuleImage<'a> {
    pub items: &'a [ModuleSection],
    pub sections_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct ModuleSection {
    pub id: SectionId, // u32
    // _padding0: u16,
    pub offset: u32,
    pub length: u32,
}

impl ModuleSection {
    pub fn new(id: SectionId, offset: u32, length: u32) -> Self {
        Self {
            id,
            // _padding0: 0,
            offset,
            length,
        }
    }
}

#[repr(u32)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SectionId {
    Type = 0x0,    // 0
    ImportData,    // 1
    ReadOnlyData,  // 2
    ReadWriteData, // 3
    UninitData,    // 4
    ImportFunc,    // 5
    Func,          // 6
    ExportData,    // 7
    ExportFunc,    // 8
    ExternalFunc,  // 9
    AutoFunc,      // 10

    ModuleIndex, // 11
    DataIndex,   // 12
    FuncIndex,   // 13
}

impl From<u32> for SectionId {
    fn from(value: u32) -> Self {
        unsafe { std::mem::transmute::<u32, SectionId>(value) }
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
        if magic_slice != MAGIC_NUMBER {
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

        let runtime_version = ((MAJOR_VERSION as u32) << 16) | (MINOR_VERSION as u32);
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
        writer.write_all(MAGIC_NUMBER)?;
        writer.write_all(&MINOR_VERSION.to_le_bytes())?;
        writer.write_all(&MAJOR_VERSION.to_le_bytes())?;
        writer.write_all(&vec![0u8, 0, 0, 0])?; // padding, 4 bytes

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
}

#[cfg(test)]
mod tests {
    use ancvm_types::DataType;

    use crate::module_image::{
        data_index_section::{DataIndexItem, DataIndexOffset, DataIndexSection},
        data_section::DataSectionType,
        func_index_section::{FuncIndexItem, FuncIndexOffset, FuncIndexSection},
        func_section::{FuncEntry, FuncSection},
        module_index_section::{ModuleIndexEntry, ModuleIndexSection, ModuleShareType},
        type_section::{TypeEntry, TypeSection},
        ModuleImage, SectionEntry, MAGIC_NUMBER,
    };

    #[test]
    fn test_module_general_sections() {
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
            func_type: 0,
            code: code0.clone(),
        });
        func_entries.push(FuncEntry {
            func_type: 1,
            code: code1.clone(),
        });

        let (func_items, codes_data) = FuncSection::convert_from_entries(&func_entries);
        let func_section = FuncSection {
            items: &func_items,
            codes_data: &codes_data,
        };

        // build ModuleImage instance

        let section_entries: Vec<&dyn SectionEntry> = vec![&type_section, &func_section];
        let (section_items, sections_data) = ModuleImage::convert_from_entries(&section_entries);
        let module_image = ModuleImage {
            items: &section_items,
            sections_data: &sections_data,
        };

        // save
        let mut image_data: Vec<u8> = Vec::new();
        module_image.save(&mut image_data).unwrap();

        assert_eq!(&image_data[0..8], MAGIC_NUMBER);
        assert_eq!(&image_data[8..10], &vec![0, 0]); // minor version number, little endian
        assert_eq!(&image_data[10..12], &vec![1, 0]); // major version number, little endian
        assert_eq!(&image_data[12..16], &vec![0, 0, 0, 0]);

        assert_eq!(&image_data[16..20], &vec![2, 0, 0, 0]); // item count
        assert_eq!(&image_data[20..24], &vec![0, 0, 0, 0]); // padding

        // image header 24 bytes
        let remains = &image_data[24..];

        // section table length = 12 (record length) * 2
        let (section_table_data, remains) = remains.split_at(24);

        assert_eq!(
            section_table_data,
            &vec![
                0u8, 0, 0, 0, // section id
                0, 0, 0, 0, // offset 0
                44, 0, 0, 0, // length 0
                //
                6, 0, 0, 0, // section id
                44, 0, 0, 0, // offset 1
                44, 0, 0, 0, // length 1
            ]
        );

        let (type_section_data, remains) = remains.split_at(44);
        assert_eq!(
            type_section_data,
            &vec![
                2u8, 0, 0, 0, // item count
                0, 0, 0, 0, // padding
                2, 0, 0, 0, // param len 0
                0, 0, 0, 0, // param offset 0
                1, 0, 0, 0, // result len 0
                2, 0, 0, 0, // result offset 0
                0, 0, 0, 0, // param len 1
                3, 0, 0, 0, // param offset 1
                1, 0, 0, 0, // result len 1
                3, 0, 0, 0, // result offset 1
                0, // I32
                1, // I64
                2, // F32
                3, // F64
            ]
        );

        let (func_section_data, _) = remains.split_at(44);
        assert_eq!(
            func_section_data,
            &vec![
                2, 0, 0, 0, // item count
                0, 0, 0, 0, // padding
                //
                0, 0, 0, 0, // code offset 0
                5, 0, 0, 0, // code len 0
                0, 0, 0, 0, // func type 0
                //
                5, 0, 0, 0, // code offset 1
                6, 0, 0, 0, // code len 1
                1, 0, 0, 0, // func type 1
                //
                1, 2, 3, 5, 7, // code 0
                11, 13, 17, 19, 23, 29, // code 1
                //
                0, // padding
            ]
        );

        // load
        let module_image_restore = ModuleImage::load(&image_data).unwrap();
        assert_eq!(module_image_restore.items.len(), 2);

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

        let func_section_restore = module_image_restore.get_func_section();

        assert_eq!(func_section_restore.items.len(), 2);

        assert_eq!(
            func_section_restore.get_entry(0),
            FuncEntry {
                func_type: 0,
                code: code0,
            }
        );

        assert_eq!(
            func_section_restore.get_entry(1),
            FuncEntry {
                func_type: 1,
                code: code1,
            }
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
        let data_index_offset0 = DataIndexOffset {
            count: 3,
            offset: 0,
        };

        let data_index_item0 = DataIndexItem::new(0, 1, DataSectionType::ReadOnly, 2);
        let data_index_item1 = DataIndexItem::new(3, 5, DataSectionType::ReadWrite, 7);
        let data_index_item2 = DataIndexItem::new(11, 13, DataSectionType::Uninit, 17);

        let data_index_section = DataIndexSection {
            offsets: &vec![data_index_offset0],
            items: &vec![data_index_item0, data_index_item1, data_index_item2],
        };

        // build FuncIndexSection instance
        let func_index_offset0 = FuncIndexOffset {
            offset: 0,
            count: 1,
        };

        let func_index_item0 = FuncIndexItem::new(0, 1, 2);

        let func_index_section = FuncIndexSection {
            offsets: &vec![func_index_offset0],
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

        assert_eq!(&image_data[0..8], MAGIC_NUMBER);
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
                11u8, 0, 0, 0, // section id 0
                0, 0, 0, 0, // offset 0
                48, 0, 0, 0, // length 0
                //
                12, 0, 0, 0, // section id 1
                48, 0, 0, 0, // offset 1
                64, 0, 0, 0, // length 1
                //
                13, 0, 0, 0, // section id 2
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

        let (func_index_section_data, _) = remains.split_at(28);

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

        assert_eq!(data_index_section_restore.offsets.len(), 1);
        assert_eq!(data_index_section_restore.items.len(), 3);

        assert_eq!(
            &data_index_section_restore.offsets[0],
            &DataIndexOffset {
                count: 3,
                offset: 0,
            }
        );

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

        assert_eq!(func_index_section_restore.offsets.len(), 1);
        assert_eq!(func_index_section_restore.items.len(), 1);

        assert_eq!(
            &func_index_section_restore.offsets[0],
            &FuncIndexOffset {
                offset: 0,
                count: 1,
            }
        );

        assert_eq!(
            &func_index_section_restore.items[0],
            &FuncIndexItem::new(0, 1, 2)
        );
    }
}
