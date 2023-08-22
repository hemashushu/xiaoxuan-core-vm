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
//
// - import data section
// - import function section
// - export data section
// - export function section
// - import external C function section
//
// because once all modules (source file) have been compiled, all imports and exports are resolved
// and stored in an _index map_ file, these sections speed up the next time the program loading:
//
// - module index section
// - data index section (optional)
// - func index section
//
// there are also some in-memory tables, they are created at application startup:
//
// - external function index (optional)
// - library list (optional)

use ancvm_types::{SectionEntry, SectionId};

use crate::{
    module_image::type_section::TypeSection,
    utils::{load_section_with_table_and_data_area, save_section_with_table_and_data_area},
    BinaryError,
};

use self::func_section::FuncSection;

pub mod func_section;
pub mod type_section;

// the "module image file" binary layout:
//
//              |--------------------------------------------------------------------------|
//              | magic number (u64) | minor ver (u16) | major ver (u16) | padding 4 bytes |
//              |--------------------------------------------------------------------------|
//              | item count (u32) | (4 bytes padding)                                     |
//              |--------------------------------------------------------------------------|
//   item 0 --> | section id 0 (u16) | padding 2 bytes | offset 0 (u32) | length 0 (u32)   | <-- table
//   item 1 --> | section id 1       | padding         | offset 1       | length 1         |
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
    pub id: SectionId,
    _padding0: u16,
    pub offset: u32,
    pub length: u32,
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

    pub fn save(
        // module_image: &ModuleImage,
        &'a self,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        // write header
        writer.write_all(MAGIC_NUMBER)?;
        writer.write_all(&MINOR_VERSION.to_le_bytes())?;
        writer.write_all(&MAJOR_VERSION.to_le_bytes())?;
        writer.write_all(&vec![0u8, 0, 0, 0])?; // padding, 4 bytes

        save_section_with_table_and_data_area(self.items, self.sections_data, writer)
    }

    pub fn get_section_entry(&'a self, idx: usize) -> Box<dyn SectionEntry + 'a> {
        let items = self.items;
        let sections_data = self.sections_data;

        let item = &items[idx];
        let section_data =
            &sections_data[item.offset as usize..(item.offset + item.length) as usize];

        let entry: Box<dyn SectionEntry + 'a> = match item.id {
            SectionId::Type => Box::new(TypeSection::load(section_data)),
            SectionId::Func => Box::new(FuncSection::load(section_data)),
            _ => todo!(),
        };

        entry
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
            .map(|(entry, (offset, length))| ModuleSection {
                id: entry.id(),
                _padding0: 0,
                offset: *offset as u32,
                length: *length as u32,
            })
            .collect::<Vec<ModuleSection>>();

        (items, image_data)
    }
}

#[cfg(test)]
mod tests {
    use ancvm_types::{DataType, FuncEntry, SectionEntry, TypeEntry};

    use crate::{downcast_section_entry, module_image::MAGIC_NUMBER};

    use super::{func_section::FuncSection, type_section::TypeSection, ModuleImage};

    #[test]
    fn test_convert() {
        // build TypeSection instance

        let mut type_entries: Vec<TypeEntry> = Vec::new();
        let type0 = vec![DataType::I32, DataType::I64];
        let type1 = vec![DataType::F32];
        let type2 = vec![];
        let type3 = vec![DataType::F64];

        type_entries.push(TypeEntry {
            params: &type0,
            results: &type1,
        });

        type_entries.push(TypeEntry {
            params: &type2,
            results: &type3,
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
            code: &code0,
        });
        func_entries.push(FuncEntry {
            func_type: 1,
            code: &code1,
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
                0u8, 0, // section id
                0, 0, // padding
                0, 0, 0, 0, // offset 0
                44, 0, 0, 0, // length 0
                //
                6, 0, // section id
                0, 0, // padding
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
                0, 0, // func type 0
                0, 0, // padding 0
                0, 0, 0, 0, // code offset 0
                5, 0, 0, 0, // code len 0
                //
                1, 0, // func type 1
                0, 0, // padding 1
                5, 0, 0, 0, // code offset 1
                6, 0, 0, 0, // code len 1
                //
                1, 2, 3, 5, 7, // code 0
                11, 13, 17, 19, 23, 29, // code 1
                //
                0,  // padding
            ]
        );

        // load
        let module_image_restore = ModuleImage::load(&image_data).unwrap();
        assert_eq!(module_image_restore.items.len(), 2);

        let type_section_box = module_image_restore.get_section_entry(0);
        let type_section_restore = downcast_section_entry::<TypeSection>(type_section_box.as_ref());

        assert_eq!(type_section_restore.items.len(), 2);

        assert_eq!(
            *type_section_restore.get_entry(0),
            TypeEntry {
                params: &type0,
                results: &type1,
            }
        );

        assert_eq!(
            *type_section_restore.get_entry(1),
            TypeEntry {
                params: &type2,
                results: &type3,
            }
        );

        let func_section_box = module_image_restore.get_section_entry(1);
        let func_section_restore = downcast_section_entry::<FuncSection>(func_section_box.as_ref());

        assert_eq!(func_section_restore.items.len(), 2);

        assert_eq!(
            *func_section_restore.get_entry(0),
            FuncEntry {
                func_type: 0,
                code: &code0,
            }
        );

        assert_eq!(
            *func_section_restore.get_entry(1),
            FuncEntry {
                func_type: 1,
                code: &code1,
            }
        );
    }
}
