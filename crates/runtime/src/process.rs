// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        load_modules_binary,
        module_image::{
            func_index_section::{FuncIndexItem, FuncIndexOffset, FuncIndexSection},
            func_section::{FuncEntry, FuncSection},
            module_index_section::{ModuleIndexEntry, ModuleIndexSection, ModuleShareType},
            type_section::{TypeEntry, TypeSection},
            ModuleImage, SectionEntry,
        },
    };
    use ancvm_types::DataType;

    use crate::{context::build_context, vm::ProgramCounter};

    fn build_module_binary_with_single_function(
        params: Vec<DataType>,
        results: Vec<DataType>,
        codes: Vec<u8>,
    ) -> Vec<u8> {
        // build type section
        let mut type_entries: Vec<TypeEntry> = Vec::new();
        type_entries.push(TypeEntry {
            params: params,
            results: results,
        });
        let (type_items, types_data) = TypeSection::convert_from_entries(&type_entries);
        let type_section = TypeSection {
            items: &type_items,
            types_data: &types_data,
        };

        // build function section
        let mut func_entries: Vec<FuncEntry> = Vec::new();
        func_entries.push(FuncEntry {
            func_type: 0,
            code: codes,
        });
        let (func_items, codes_data) = FuncSection::convert_from_entries(&func_entries);
        let func_section = FuncSection {
            items: &func_items,
            codes_data: &codes_data,
        };

        // build module index
        let mut mod_index_entries: Vec<ModuleIndexEntry> = Vec::new();
        mod_index_entries.push(ModuleIndexEntry::new(
            ModuleShareType::Local,
            "main".to_string(),
        ));
        let (module_index_items, names_data) =
            ModuleIndexSection::convert_from_entries(&mod_index_entries);
        let mod_index_section = ModuleIndexSection {
            items: &module_index_items,
            names_data: &names_data,
        };

        // build function index
        let mut func_index_offsets: Vec<FuncIndexOffset> = Vec::new();
        let mut func_index_items: Vec<FuncIndexItem> = Vec::new();

        func_index_offsets.push(FuncIndexOffset {
            offset: 0,
            count: 1,
        });
        func_index_items.push(FuncIndexItem::new(0, 0, 0));
        let func_index_section = FuncIndexSection {
            offsets: &func_index_offsets,
            items: &func_index_items,
        };

        // build module image
        let section_entries: Vec<&dyn SectionEntry> = vec![
            &type_section,
            &func_section,
            &mod_index_section,
            &func_index_section,
        ];
        let (section_items, sections_data) = ModuleImage::convert_from_entries(&section_entries);
        let module_image = ModuleImage {
            items: &section_items,
            sections_data: &sections_data,
        };

        // build module image binary
        let mut image_data: Vec<u8> = Vec::new();
        module_image.save(&mut image_data).unwrap();

        image_data
    }

    #[test]
    fn test_single_function_context() {
        let binary = build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32],
            vec![DataType::I64],
            vec![0u8],
        );

        let binaries = vec![&binary[..]];
        let module_images = load_modules_binary(binaries).expect("module binary error");
        let context = build_context(&module_images[..]);

        // check module index section
        assert_eq!(context.module_index_section.items.len(), 1);

        let module_index_entry = context.module_index_section.get_entry(0);
        assert_eq!(module_index_entry.name, "main".to_string());
        assert_eq!(module_index_entry.module_share_type, ModuleShareType::Local);

        // check data index section
        assert_eq!(context.data_index_section.items.len(), 0);

        // check function index section
        assert_eq!(context.func_index_section.offsets.len(), 1);
        assert_eq!(context.func_index_section.items.len(), 1);

        assert_eq!(
            &context.func_index_section.offsets[0],
            &FuncIndexOffset {
                offset: 0,
                count: 1
            }
        );

        assert_eq!(
            &context.func_index_section.items[0],
            &FuncIndexItem::new(0, 0, 0)
        );

        // check vm
        let vm = context.vm;
        // assert_eq!(vm.get_stack_capacity(), 32 * 1024);
        // assert_eq!(vm.heap.len(), 0);
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.fp, 0);
        assert_eq!(
            vm.pc,
            ProgramCounter {
                addr: 0,
                module_index: 0
            }
        );

        // check module
        assert_eq!(context.modules.len(), 1);

        let module = &context.modules[0];
        assert_eq!(module.data_items.len(), 3);

        assert_eq!(module.data_items[0].len(), 0);
        assert_eq!(module.data_items[1].len(), 0);
        assert_eq!(module.data_items[2].len(), 0);

        assert_eq!(module.read_only_datas.len(), 0);
        assert_eq!(module.read_write_datas.len(), 0);
        assert_eq!(module.uninit_datas.len(), 0);

        // check type section
        assert_eq!(module.type_section.items.len(), 1);
        assert_eq!(
            module.type_section.get_entry(0),
            TypeEntry {
                params: vec![DataType::I32, DataType::I32],
                results: vec![DataType::I64]
            }
        );

        // check func section
        assert_eq!(module.func_section.items.len(), 1);
        assert_eq!(
            module.func_section.get_entry(0),
            FuncEntry {
                func_type: 0,
                code: vec![0u8]
            }
        );
    }
}
