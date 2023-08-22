// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_types::{SectionEntry, SectionId};

use crate::{
    utils::{load_section_with_table_and_data_area, save_section_with_table_and_data_area},
    BinaryError,
};

use self::{
    data_index_section::DataIndexSection, func_index_section::FuncIndexSection,
    module_index_section::ModuleIndexSection,
};

pub mod data_index_section;
pub mod func_index_section;
pub mod module_index_section;

// the "index map file" binary layout:
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

const MAGIC_NUMBER: &[u8; 8] = b"ancsim\0\0"; // for "ANCS index mapping"
const MAJOR_VERSION: u16 = 1;
const MINOR_VERSION: u16 = 0;

#[derive(Debug, PartialEq)]
pub struct IndexMap<'a> {
    pub items: &'a [IndexSection],
    pub sections_data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct IndexSection {
    pub id: SectionId,
    _padding0: u16,
    pub offset: u32,
    pub length: u32,
}

impl<'a> IndexMap<'a> {
    pub fn load(image_data: &'a [u8]) -> Result<Self, BinaryError> {
        let magic_slice = &image_data[0..8];
        if magic_slice != MAGIC_NUMBER {
            return Err(BinaryError {
                message: "Not a index mapping file.".to_owned(),
            });
        }

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
            load_section_with_table_and_data_area::<IndexSection>(image_body);

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
            SectionId::ModuleIndex => Box::new(ModuleIndexSection::load(section_data)),
            SectionId::DataIndex => Box::new(DataIndexSection::load(section_data)),
            SectionId::FuncIndex => Box::new(FuncIndexSection::load(section_data)),
            _ => unreachable!("Unknown index mapping section."),
        };

        entry
    }

    pub fn convert_from_entries(
        entries: &[&'a dyn SectionEntry<'a>],
    ) -> (Vec<IndexSection>, Vec<u8>) {
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
            .map(|(entry, (offset, length))| IndexSection {
                id: entry.id(),
                _padding0: 0,
                offset: *offset as u32,
                length: *length as u32,
            })
            .collect::<Vec<IndexSection>>();

        (items, image_data)
    }
}

#[cfg(test)]
mod tests {
    use ancvm_types::{DataSectionType, ModuleIndexEntry, ModuleShareType, SectionEntry};

    use crate::{
        downcast_section_entry,
        index_map::{
            data_index_section::{DataIndexItem, DataIndexOffset, DataIndexSection},
            func_index_section::{FuncIndexItem, FuncIndexOffset, FuncIndexSection},
            module_index_section::ModuleIndexSection,
            IndexMap, MAGIC_NUMBER,
        },
    };

    #[test]
    fn test_convert() {
        // build ModuleIndexSection instance
        let mut module_index_entries: Vec<ModuleIndexEntry> = Vec::new();
        module_index_entries.push(ModuleIndexEntry {
            module_share_type: ModuleShareType::Local,
            name: "main",
        });
        module_index_entries.push(ModuleIndexEntry {
            module_share_type: ModuleShareType::Shared,
            name: "httpclient",
        });

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
        let (section_items, sections_data) = IndexMap::convert_from_entries(&section_entries);
        let index_map = IndexMap {
            items: &section_items,
            sections_data: &sections_data,
        };

        // save
        let mut image_data: Vec<u8> = Vec::new();
        index_map.save(&mut image_data).unwrap();

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
                11u8, 0, // section id 0
                0, 0, // padding 0
                0, 0, 0, 0, // offset 0
                40, 0, 0, 0, // length 0
                //
                12, 0, // section id 1
                0, 0, // padding 1
                40, 0, 0, 0, // offset 1
                40, 0, 0, 0, // length 1
                //
                13, 0, // section id 2
                0, 0, // padding 2
                80, 0, 0, 0, // offset 2
                28, 0, 0, 0, // length 2
            ]
        );

        let (module_index_section_data, remains) = remains.split_at(40);

        assert_eq!(
            module_index_section_data,
            &vec![
                2, 0, 0, 0, // item count
                0, 0, 0, 0, // padding
                0, 0, 0, 0, // name offset 0
                4, 0, // name length 0
                0, 0, // module type 0
                4, 0, 0, 0, // name offset 1
                10, 0, // name length 1
                1, 0, // module type 1
                109, 97, 105, 110, // b"main"
                104, 116, 116, 112, 99, 108, 105, 101, 110, 116, // b"httpclient"
                0, 0, // padding for 4-byte alignment
            ]
        );

        let (data_index_section_data, remains) = remains.split_at(40);

        assert_eq!(
            data_index_section_data,
            &vec![
                /* table 0 */
                1, 0, 0, 0, // item count
                0, 0, 0, 0, // padding
                0, 0, 0, 0, // offset 0
                3, 0, 0, 0, // count 0
                /* table 1 */
                0, 0, // data idx 0
                1, 0, // target module idx 0
                0, // target data section type 0
                0, // padding 0
                2, 0, // target data idx 0
                3, 0, // data idx 1
                5, 0, // target module idx 1
                1, // target data section type 1
                0, // padding 1
                7, 0, // target data idx 1
                11, 0, // data idx 2
                13, 0, // target module idx 2
                2, // target data section type 2
                0, // padding 2
                17, 0, // target data idx 2
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
                1, 0, // target module idx 0
                0, 0, // padding
                2, 0, 0, 0, // target func idx 0
            ]
        );

        // load
        let index_map_restore = IndexMap::load(&image_data).unwrap();
        assert_eq!(index_map_restore.items.len(), 3);

        let module_index_section_box = index_map_restore.get_section_entry(0);
        let module_index_section_restore =
            downcast_section_entry::<ModuleIndexSection>(module_index_section_box.as_ref());

        assert_eq!(module_index_section_restore.items.len(), 2);

        assert_eq!(
            *module_index_section_restore.get_entry(0),
            ModuleIndexEntry {
                module_share_type: ModuleShareType::Local,
                name: "main",
            }
        );

        assert_eq!(
            *module_index_section_restore.get_entry(1),
            ModuleIndexEntry {
                module_share_type: ModuleShareType::Shared,
                name: "httpclient",
            }
        );

        let data_index_section_box = index_map_restore.get_section_entry(1);
        let data_index_section_restore =
            downcast_section_entry::<DataIndexSection>(data_index_section_box.as_ref());

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

        let func_index_section_box = index_map_restore.get_section_entry(2);
        let func_index_section_restore =
            downcast_section_entry::<FuncIndexSection>(func_index_section_box.as_ref());

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
