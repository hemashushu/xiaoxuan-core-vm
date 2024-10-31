// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use crate::module_image::{ModuleSectionId, SectionEntry};

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PropertySection {
    pub runtime_major_version: u16, // only application can specify runtime/compiler version
    pub runtime_minor_version: u16,
    pub entry_function_public_index: u32, // u32::max = none
    pub constructor_function_public_index: u32, // u32::max = none
    pub destructor_function_public_index: u32, // u32::max = none
    pub module_name_length: u32,
    pub module_name_buffer: [u8; 256],
}

impl<'a> SectionEntry<'a> for PropertySection {
    fn load(section_data: &'a [u8]) -> Self {
        let property_section_ptr = unsafe {
            std::mem::transmute::<*const u8, *const PropertySection>(section_data.as_ptr())
        };

        unsafe { *property_section_ptr }
    }

    fn save(&'a self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        let mut data = [0u8; std::mem::size_of::<PropertySection>()];
        let src = self as *const PropertySection as *const u8;
        let dst = data.as_mut_ptr();
        unsafe { std::ptr::copy(src, dst, data.len()) };

        writer.write_all(&data)
    }

    fn id(&'a self) -> ModuleSectionId {
        ModuleSectionId::Property
    }
}

#[cfg(test)]
mod tests {
    use crate::module_image::SectionEntry;

    use super::PropertySection;

    #[test]
    fn test_save_section() {
        let mut module_name_buffer = [0u8; 256];
        module_name_buffer[0] = 29;
        module_name_buffer[1] = 31;
        module_name_buffer[2] = 37;

        let section = PropertySection {
            runtime_major_version: 11,
            runtime_minor_version: 13,
            entry_function_public_index: 17,
            constructor_function_public_index: 19,
            destructor_function_public_index: 23,
            module_name_length: 3,
            module_name_buffer,
        };

        let mut section_data: Vec<u8> = Vec::new();
        section.save(&mut section_data).unwrap();

        let mut expect_data = vec![
            11, 0, // major runtime version
            13, 0, // minor runtime version
            17, 0, 0, 0, // entry function public index
            19, 0, 0, 0, // constructor function public index
            23, 0, 0, 0, // destructor function public index
            3, 0, 0, 0, // name length
            29, 31, 37, // name buffer
        ];

        expect_data.resize(std::mem::size_of::<PropertySection>(), 0);

        assert_eq!(section_data, expect_data);
    }

    #[test]
    fn test_load_section() {
        let mut section_data = vec![
            11, 0, // major runtime version
            13, 0, // minor runtime version
            17, 0, 0, 0, // entry function public index
            19, 0, 0, 0, // constructor function public index
            23, 0, 0, 0, // destructor function public index
            3, 0, 0, 0, // name length
            29, 31, 37, 0, // name buffer
        ];

        section_data.resize(std::mem::size_of::<PropertySection>(), 0);

        let section = PropertySection::load(&section_data);
        assert_eq!(section.runtime_major_version, 11);
        assert_eq!(section.runtime_minor_version, 13);
        assert_eq!(section.entry_function_public_index, 17);
        assert_eq!(section.constructor_function_public_index, 19);
        assert_eq!(section.destructor_function_public_index, 23);
        assert_eq!(section.module_name_length, 3);

        let mut module_name_buffer = [0u8; 256];
        module_name_buffer[0] = 29;
        module_name_buffer[1] = 31;
        module_name_buffer[2] = 37;

        assert_eq!(section.module_name_buffer, module_name_buffer);
    }
}
