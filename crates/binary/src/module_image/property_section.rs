// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use super::{ModuleSectionId, SectionEntry};

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PropertySection {
    pub entry_function_public_index: u32,
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
        let section = PropertySection {
            entry_function_public_index: 11,
        };

        let mut section_data: Vec<u8> = Vec::new();
        section.save(&mut section_data).unwrap();

        let expect_data = vec![
            11, 0, 0, 0, // the public index of function 'entry'
        ];

        assert_eq!(section_data, expect_data);
    }

    #[test]
    fn test_load_section() {
        let section_data = vec![
            11, 0, 0, 0, // the public index of function 'entry'
        ];

        let section = PropertySection::load(&section_data);
        assert_eq!(section.entry_function_public_index, 11);
    }
}
