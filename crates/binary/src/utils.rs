// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_types::entry::{
    ExternalFunctionEntry, ExternalLibraryEntry, FunctionEntry, InitedDataEntry, LocalListEntry,
    LocalVariableEntry, TypeEntry, UnifiedExternalFunctionEntry, UnifiedExternalLibraryEntry,
    UninitDataEntry,
};
use ancvm_types::{DataSectionType, DataType, ExternalLibraryType};

use std::{mem::size_of, ptr::slice_from_raw_parts};

use crate::module_image::external_function_index_section::{
    ExternalFunctionIndexItem, ExternalFunctionIndexSection,
};
use crate::module_image::external_function_section::ExternalFunctionSection;
use crate::module_image::external_library_section::ExternalLibrarySection;

use crate::module_image::unified_external_function_section::UnifiedExternalFunctionSection;
use crate::module_image::unified_external_library_section::UnifiedExternalLibrarySection;
use crate::module_image::{
    data_index_section::{DataIndexItem, DataIndexSection},
    data_section::{ReadOnlyDataSection, ReadWriteDataSection, UninitDataSection},
    function_index_section::{FunctionIndexItem, FunctionIndexSection},
    function_section::FunctionSection,
    local_variable_section::LocalVariableSection,
    type_section::TypeSection,
    ModuleImage, RangeItem, SectionEntry,
};

const DATA_ALIGN_BYTES: usize = 4;

/// load a section that contains two tables.
///
/// ```text
/// |----------------------------------------------|
/// | table 0 item count (u32) | padding (4 bytes) |
/// |----------------------------------------------|
/// | table 0 record 0                             | <-- record length must be a multiple of 0x4
/// | table 0 record 1                             |
/// | ...                                          |
/// |----------------------------------------------|
/// | table 1 record 0                             | <-- record length must be a multiple of 0x4
/// | table 1 record 1                             |
/// |----------------------------------------------|
/// ```
///
/// note that the items count of table 1 is calculated by:
/// (table 1 data length) / (one record length)
pub fn load_section_with_two_tables<T0, T1>(section_data: &[u8]) -> (&[T0], &[T1]) {
    let ptr = section_data.as_ptr();
    let item_count0 = unsafe { std::ptr::read(ptr as *const u32) } as usize;

    // there is a "safe" approach to read a number from pointer, e.g.
    //
    // ```rust
    //     let mut buf = [0u8; 4];
    //     let data = &binary[0..4];
    //     buf.clone_from_slice(data);
    //     let module_count =  u32::from_le_bytes(buf);
    // ```

    let one_record_length_in_bytes0 = size_of::<T0>();
    let total_length_in_bytes0 = one_record_length_in_bytes0 * item_count0;

    // 8 bytes is the length of header, i.e.
    // 4 bytes `item_count` + 4 bytes padding.
    let items0_data = &section_data[8..(8 + total_length_in_bytes0)];
    let items1_data = &section_data[(8 + total_length_in_bytes0)..];

    // there is another method to get the `items_data`, e.g.
    // ```rust
    //     let ptr_items = unsafe {
    //         ptr.offset(8)
    //     } as *const DataIndexOffset;
    // ```

    let one_record_length_in_bytes1 = size_of::<T1>();
    let item_count1 = items1_data.len() / one_record_length_in_bytes1;
    let items0 = load_items::<T0>(items0_data, item_count0);
    let items1 = load_items::<T1>(items1_data, item_count1);

    (items0, items1)
}

/// save a section that contains two tables.
///
/// ```text
/// |----------------------------------------------|
/// | table 0 item count (u32) | padding (4 bytes) |
/// |----------------------------------------------|
/// | table 0 record 0                             | <-- record length must be a multiple of 0x4
/// | table 0 record 1                             |
/// | ...                                          |
/// |----------------------------------------------|
/// | table 1 record 0                             | <-- record length must be a multiple of 0x4
/// | table 1 record 1                             |
/// |----------------------------------------------|
/// ```
pub fn save_section_with_two_tables<T0, T1>(
    items0: &[T0],
    items1: &[T1],
    writer: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    // write header
    let item_count0 = items0.len();
    writer.write_all(&(item_count0 as u32).to_le_bytes())?; // item count
    writer.write_all(&[0u8; 4])?; // 4 bytes padding

    save_items(items0, writer)?;
    save_items(items1, writer)?;
    // save_offsets(offsets, writer)?;
    // save_index_items(items, writer)?;

    Ok(())
}

/// load a section that contains a table and a variable-length data area.
///
/// ```text
/// |--------------------------------------|
/// | item count (u32) | padding (4 bytes) |
/// |--------------------------------------|
/// | record 0                             | <-- record length must be a multiple of 0x4
/// | record 1                             |
/// | ...                                  |
/// |--------------------------------------|
/// | variable length data area            | <-- data length must be a multiple of 0x4
/// | ...                                  |
/// |--------------------------------------|
/// ```
pub fn load_section_with_table_and_data_area<T>(section_data: &[u8]) -> (&[T], &[u8]) {
    let ptr = section_data.as_ptr();
    let item_count = unsafe { std::ptr::read(ptr as *const u32) } as usize;

    let one_record_length_in_bytes = size_of::<T>();
    let total_length_in_bytes = one_record_length_in_bytes * item_count as usize;

    // 8 bytes is the length of header,
    // 4 bytes `item_count` + 4 bytes padding.
    let items_data = &section_data[8..(8 + total_length_in_bytes)];
    let additional_data = &section_data[(8 + total_length_in_bytes)..];

    let items = load_items::<T>(items_data, item_count);

    (items, additional_data)
}

/// save a section that contains a table and a variable-length data area.
///
/// ```text
/// |--------------------------------------|
/// | item count (u32) | padding (4 bytes) |
/// |--------------------------------------|
/// | record 0                             | <-- record length must be a multiple of 0x4
/// | record 1                             |
/// | ...                                  |
/// |--------------------------------------|
/// | variable length data area            | <-- data length must be a multiple of 0x4
/// | ...                                  |     if the length is not 4x, byte '\0' will
/// |--------------------------------------|     be appended automatically by this function.
/// ```
pub fn save_section_with_table_and_data_area<T>(
    items: &[T],
    additional_data: &[u8],
    writer: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    // write header
    let item_count = items.len();
    writer.write_all(&(item_count as u32).to_le_bytes())?; // item count
    writer.write_all(&[0u8; 4])?; // 4 bytes padding

    save_items::<T>(items, writer)?;
    writer.write_all(additional_data)?;

    let remainder = additional_data.len() % DATA_ALIGN_BYTES; // remainder

    if remainder != 0 {
        let padding = DATA_ALIGN_BYTES - remainder;
        for _count in 0..padding {
            // writer.write(b"\0")?;
            writer.write_all(b"\0")?;
        }
    }

    Ok(())
}

/// load a section that contains only one table.
///
/// ```text
/// |--------------------------------------|
/// | item count (u32) | padding (4 bytes) |
/// |--------------------------------------|
/// | record 0                             | <-- record length must be a multiple of 0x4
/// | record 1                             |
/// | ...                                  |
/// |--------------------------------------|
/// ```
pub fn load_section_with_one_table<T>(section_data: &[u8]) -> &[T] {
    let ptr = section_data.as_ptr();
    let item_count = unsafe { std::ptr::read(ptr as *const u32) } as usize;

    let one_record_length_in_bytes = size_of::<T>();
    let total_length_in_bytes = one_record_length_in_bytes * item_count as usize;

    // 8 bytes is the length of header,
    // 4 bytes `item_count` + 4 bytes padding.
    let items_data = &section_data[8..(8 + total_length_in_bytes)];
    let items = load_items::<T>(items_data, item_count);

    items
}

/// save a section that contains only one table.
///
/// ```text
/// |--------------------------------------|
/// | item count (u32) | padding (4 bytes) |
/// |--------------------------------------|
/// | record 0                             | <-- record length must be a multiple of 0x4
/// | record 1                             |
/// | ...                                  |
/// |--------------------------------------|
/// ```
pub fn save_section_with_one_table<T>(
    items: &[T],
    writer: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    // write header
    let item_count = items.len();
    writer.write_all(&(item_count as u32).to_le_bytes())?; // item count
    writer.write_all(&[0u8; 4])?; // 4 bytes padding

    save_items::<T>(items, writer)?;
    Ok(())
}

/// load a table
/// note that record length must be a multiple of 0x4
pub fn load_items<T>(items_data: &[u8], item_count: usize) -> &[T] {
    let items_ptr = items_data.as_ptr() as *const T;
    // https://doc.rust-lang.org/std/ptr/fn.slice_from_raw_parts.html
    let items_slice = std::ptr::slice_from_raw_parts(items_ptr, item_count);
    unsafe { &*items_slice }
}

/// save a table
/// note that record length must be a multiple of 0x4
pub fn save_items<T>(items: &[T], writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    // let item_count = items.len();
    // let one_record_length_in_bytes = size_of::<T>();
    // let total_length_in_bytes = one_record_length_in_bytes * item_count;

    let total_length_in_bytes = std::mem::size_of_val(items);

    let ptr = items.as_ptr() as *const u8;
    let slice = slice_from_raw_parts(ptr, total_length_in_bytes);
    writer.write_all(unsafe { &*slice })?;

    // an example of writing a slice to Vec<u8>
    //
    // ```rust
    //     let one_record_length_in_bytes = size_of::<T>();
    //     let total_length_in_bytes = one_record_length_in_bytes * item_count;
    //
    //     let mut buf: Vec<u8> = Vec::with_capacity(total_length_in_bytes);
    //     let dst = buf.as_mut_ptr() as *mut u8;
    //     let src = items.as_ptr() as *const u8;
    //
    //     unsafe {
    //         std::ptr::copy(src, dst, total_length_in_bytes);
    //         buf.set_len(total_length_in_bytes);
    //     }
    // ```

    Ok(())
}

/// helper object for unit test
pub struct HelperFunctionEntryWithSignatureAndLocalVars {
    pub params: Vec<DataType>,
    pub results: Vec<DataType>,
    pub local_variable_item_entries_without_args: Vec<LocalVariableEntry>,
    pub code: Vec<u8>,
}

pub struct HelperFunctionEntryWithLocalVars {
    pub type_index: usize,
    pub local_variable_item_entries_without_args: Vec<LocalVariableEntry>,
    pub code: Vec<u8>,
}

/// helper object for unit test
pub struct HelperBlockEntry {
    pub params: Vec<DataType>,
    pub results: Vec<DataType>,
    pub local_variable_item_entries_without_args: Vec<LocalVariableEntry>,
}

/// helper object for unit test
pub struct HelperExternalFunctionEntry {
    pub external_library_type: ExternalLibraryType,
    pub library_name: String,
    pub function_name: String,
    pub type_index: usize,
}

/// helper function for unit test
pub fn helper_build_module_binary_with_single_function(
    param_datatypes: Vec<DataType>,
    result_datatypes: Vec<DataType>,
    local_variable_item_entries_without_args: Vec<LocalVariableEntry>,
    code: Vec<u8>,
) -> Vec<u8> {
    helper_build_module_binary_with_single_function_and_data_sections(
        param_datatypes,
        result_datatypes,
        local_variable_item_entries_without_args,
        code,
        vec![],
        vec![],
        vec![],
    )
}

/// helper function for unit test
pub fn helper_build_module_binary_with_single_function_and_data_sections(
    param_datatypes: Vec<DataType>,
    result_datatypes: Vec<DataType>,
    local_variable_item_entries_without_args: Vec<LocalVariableEntry>,
    code: Vec<u8>,
    read_only_data_entries: Vec<InitedDataEntry>,
    read_write_data_entries: Vec<InitedDataEntry>,
    uninit_uninit_data_entries: Vec<UninitDataEntry>,
) -> Vec<u8> {
    let type_entry = TypeEntry {
        params: param_datatypes.clone(),
        results: result_datatypes.clone(),
    };

    let params_as_local_variables = param_datatypes
        .iter()
        .map(|data_type| LocalVariableEntry::from_datatype(*data_type))
        .collect::<Vec<_>>();

    let mut local_variables = Vec::new();
    local_variables.extend_from_slice(&params_as_local_variables);
    local_variables.extend_from_slice(&local_variable_item_entries_without_args);

    let local_var_list_entry = LocalListEntry {
        variable_entries: local_variables,
    };

    let function_entry = FunctionEntry {
        type_index: 0,
        local_list_index: 0,
        code,
    };

    helper_build_module_binary(
        "main",
        read_only_data_entries,
        read_write_data_entries,
        uninit_uninit_data_entries,
        vec![type_entry],
        vec![local_var_list_entry],
        vec![function_entry],
        vec![],
    )
}

/// helper function for unit test
pub fn helper_build_module_binary_with_single_function_and_blocks(
    param_datatypes: Vec<DataType>,
    result_datatypes: Vec<DataType>,
    local_variable_item_entries_without_args: Vec<LocalVariableEntry>,
    code: Vec<u8>,

    // although there is no params and no results for the block_nez, but
    // it still is necessary create a 'HelperBlockEntry'.
    helper_block_entries: Vec<HelperBlockEntry>,
) -> Vec<u8> {
    let helper_function_entry = HelperFunctionEntryWithSignatureAndLocalVars {
        params: param_datatypes,
        results: result_datatypes,
        local_variable_item_entries_without_args,
        code,
    };

    helper_build_module_binary_with_functions_and_blocks(
        vec![helper_function_entry],
        helper_block_entries,
    )
}

/// helper function for unit test
pub fn helper_build_module_binary_with_functions_and_blocks(
    helper_function_entries: Vec<HelperFunctionEntryWithSignatureAndLocalVars>,
    helper_block_entries: Vec<HelperBlockEntry>,
) -> Vec<u8> {
    // build type entries

    // note:
    // for simplicity, duplicate items are not merged here.

    let function_type_entries = helper_function_entries
        .iter()
        .map(|entry| TypeEntry {
            params: entry.params.clone(),
            results: entry.results.clone(),
        })
        .collect::<Vec<_>>();

    let block_type_entries = helper_block_entries
        .iter()
        .map(|entry| TypeEntry {
            params: entry.params.clone(),
            results: entry.results.clone(),
        })
        .collect::<Vec<_>>();

    let mut type_entries = Vec::new();
    type_entries.extend_from_slice(&function_type_entries);
    type_entries.extend_from_slice(&block_type_entries);

    // build local vars list entries

    // note:
    // for simplicity, duplicate items are not merged here.

    let function_local_var_list_entries = helper_function_entries
        .iter()
        .map(|entry| {
            let params_as_local_variables = entry
                .params
                .iter()
                .map(|data_type| LocalVariableEntry::from_datatype(*data_type))
                .collect::<Vec<_>>();

            let mut local_variables = Vec::new();
            local_variables.extend_from_slice(&params_as_local_variables);
            local_variables.extend_from_slice(&entry.local_variable_item_entries_without_args);

            LocalListEntry {
                variable_entries: local_variables,
            }
        })
        .collect::<Vec<_>>();

    let block_local_var_list_entries = helper_block_entries
        .iter()
        .map(|entry| {
            let params_as_local_variables = entry
                .params
                .iter()
                .map(|data_type| LocalVariableEntry::from_datatype(*data_type))
                .collect::<Vec<_>>();

            let mut local_variables = Vec::new();
            local_variables.extend_from_slice(&params_as_local_variables);
            local_variables.extend_from_slice(&entry.local_variable_item_entries_without_args);

            LocalListEntry {
                variable_entries: local_variables,
            }
        })
        .collect::<Vec<_>>();

    let mut local_var_list_entries = Vec::new();
    local_var_list_entries.extend_from_slice(&function_local_var_list_entries);
    local_var_list_entries.extend_from_slice(&block_local_var_list_entries);

    // build func entries
    let function_entries = helper_function_entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| FunctionEntry {
            type_index: idx,
            local_list_index: idx,
            code: entry.code.clone(),
        })
        .collect::<Vec<_>>();

    helper_build_module_binary(
        "main",
        vec![],
        vec![],
        vec![],
        type_entries,
        local_var_list_entries,
        function_entries,
        vec![],
    )
}

/// helper function for unit test
#[allow(clippy::too_many_arguments)]
pub fn helper_build_module_binary_with_functions_and_external_functions(
    type_entries: Vec<TypeEntry>,
    helper_function_entries: Vec<HelperFunctionEntryWithLocalVars>,
    read_only_data_entries: Vec<InitedDataEntry>,
    read_write_data_entries: Vec<InitedDataEntry>,
    uninit_uninit_data_entries: Vec<UninitDataEntry>,
    helper_external_function_entries: Vec<HelperExternalFunctionEntry>,
) -> Vec<u8> {
    let mut function_entries = vec![];
    let mut local_var_list_entries = vec![];

    helper_function_entries
        .iter()
        .enumerate()
        .for_each(|(idx, entry)| {
            let params_as_local_variables = type_entries[entry.type_index]
                .params
                .iter()
                .map(|data_type| LocalVariableEntry::from_datatype(*data_type))
                .collect::<Vec<_>>();

            let mut local_variables = Vec::new();
            local_variables.extend_from_slice(&params_as_local_variables);
            local_variables.extend_from_slice(&entry.local_variable_item_entries_without_args);

            let local_var_list_entry = LocalListEntry {
                variable_entries: local_variables,
            };

            let function_entry = FunctionEntry {
                type_index: entry.type_index,
                local_list_index: idx,
                code: entry.code.clone(),
            };

            function_entries.push(function_entry);
            local_var_list_entries.push(local_var_list_entry);
        });

    helper_build_module_binary(
        "main",
        read_only_data_entries,
        read_write_data_entries,
        uninit_uninit_data_entries,
        type_entries,
        local_var_list_entries,
        function_entries,
        helper_external_function_entries,
    )
}

/// helper function for unit test
#[allow(clippy::too_many_arguments)]
pub fn helper_build_module_binary(
    name: &str,
    read_only_data_entries: Vec<InitedDataEntry>,
    read_write_data_entries: Vec<InitedDataEntry>,
    uninit_uninit_data_entries: Vec<UninitDataEntry>,
    type_entries: Vec<TypeEntry>,
    local_var_list_entries: Vec<LocalListEntry>, // this local list includes args
    function_entries: Vec<FunctionEntry>,
    helper_external_function_entries: Vec<HelperExternalFunctionEntry>,
) -> Vec<u8> {
    // build read-only data section
    let (ro_items, ro_data) = ReadOnlyDataSection::convert_from_entries(&read_only_data_entries);
    let ro_data_section = ReadOnlyDataSection {
        items: &ro_items,
        datas_data: &ro_data,
    };

    // build read-write data section
    let (rw_items, rw_data) = ReadWriteDataSection::convert_from_entries(&read_write_data_entries);
    let rw_data_section = ReadWriteDataSection {
        items: &rw_items,
        datas_data: &rw_data,
    };

    // build uninitilized data section
    let uninit_items = UninitDataSection::convert_from_entries(&uninit_uninit_data_entries);
    let uninit_data_section = UninitDataSection {
        items: &uninit_items,
    };

    // build type section
    let (type_items, types_data) = TypeSection::convert_from_entries(&type_entries);
    let type_section = TypeSection {
        items: &type_items,
        types_data: &types_data,
    };

    // build function section
    let (function_items, codes_data) = FunctionSection::convert_from_entries(&function_entries);
    let function_section = FunctionSection {
        items: &function_items,
        codes_data: &codes_data,
    };

    // build local variable section
    let (local_var_lists, local_var_list_data) =
        LocalVariableSection::convert_from_entries(&local_var_list_entries);
    let local_var_section = LocalVariableSection {
        lists: &local_var_lists,
        list_data: &local_var_list_data,
    };

    // build external library section
    let mut external_library_entries = helper_external_function_entries
        .iter()
        .map(|e| ExternalLibraryEntry::new(e.library_name.clone(), e.external_library_type))
        .collect::<Vec<_>>();
    external_library_entries.sort_by(|left, right| left.name.cmp(&right.name));
    external_library_entries.dedup_by(|left, right| left.name == right.name);
    let (external_library_items, external_library_data) =
        ExternalLibrarySection::convert_from_entries(&external_library_entries);
    let external_library_section = ExternalLibrarySection {
        items: &external_library_items,
        names_data: &external_library_data,
    };

    // build external function section
    let external_function_entries = helper_external_function_entries
        .iter()
        .map(|library_and_function_entry| {
            let library_index = external_library_entries
                .iter()
                .position(|library_entry| library_entry.name == library_and_function_entry.library_name)
                .unwrap();
            ExternalFunctionEntry::new(
                library_and_function_entry.function_name.clone(),
                library_index,
                library_and_function_entry.type_index,
            )
        })
        .collect::<Vec<_>>();
    let (external_function_items, external_function_data) =
        ExternalFunctionSection::convert_from_entries(&external_function_entries);
    let external_function_section = ExternalFunctionSection {
        items: &external_function_items,
        names_data: &external_function_data,
    };

    // build data index

    // the data index is ordered by:
    // 1. imported ro data
    // 2. ro data
    // 3. imported rw data
    // 4. rw data
    // 5. imported uninit data
    // 6. uninit data
    let data_ranges: Vec<RangeItem> = vec![RangeItem {
        offset: 0,
        count: (ro_items.len() + rw_items.len() + uninit_items.len()) as u32,
    }];

    let mut data_index_items: Vec<DataIndexItem> = Vec::new();

    let ro_iter = ro_items
        .iter()
        .enumerate()
        .map(|(idx, _item)| (idx, DataSectionType::ReadOnly));
    let rw_iter = rw_items
        .iter()
        .enumerate()
        .map(|(idx, _item)| (idx, DataSectionType::ReadWrite));
    let uninit_iter = uninit_items
        .iter()
        .enumerate()
        .map(|(idx, _item)| (idx, DataSectionType::Uninit));
    for (total_data_index, (idx, data_section_type)) in
        ro_iter.chain(rw_iter).chain(uninit_iter).enumerate()
    {
        data_index_items.push(DataIndexItem::new(
            total_data_index as u32,
            0,
            idx as u32,
            data_section_type,
        ));
    }

    let data_index_section = DataIndexSection {
        ranges: &data_ranges,
        items: &data_index_items,
    };

    // build function index
    let function_ranges: Vec<RangeItem> = vec![RangeItem {
        offset: 0,
        count: function_entries.len() as u32,
    }];

    let function_index_items: Vec<FunctionIndexItem> = (0..function_entries.len())
        .map(|idx| {
            let idx_u32 = idx as u32;
            FunctionIndexItem::new(idx_u32, 0, idx_u32)
        })
        .collect::<Vec<_>>();

    let function_index_section = FunctionIndexSection {
        ranges: &function_ranges,
        items: &function_index_items,
    };

    // build unified external library section
    // it's 1:1 to the external_library_entries
    let unified_external_library_entries = external_library_entries
        .iter()
        .map(|e| UnifiedExternalLibraryEntry {
            name: e.name.clone(),
            external_library_type: e.external_library_type,
        })
        .collect::<Vec<_>>();

    let (unified_external_library_items, unified_external_library_data) =
        UnifiedExternalLibrarySection::convert_from_entries(&unified_external_library_entries);
    let unified_external_library_section = UnifiedExternalLibrarySection {
        items: &unified_external_library_items,
        names_data: &unified_external_library_data,
    };

    // build unified external function section
    // it's 1:1 to external_function_entries
    let unified_external_function_entries = external_function_entries
        .iter()
        .map(|e| UnifiedExternalFunctionEntry {
            name: e.name.clone(),
            unified_external_library_index: e.external_library_index,
        })
        .collect::<Vec<_>>();

    let (unified_external_function_items, unified_external_function_data) =
        UnifiedExternalFunctionSection::convert_from_entries(&unified_external_function_entries);
    let unified_external_function_section = UnifiedExternalFunctionSection {
        items: &unified_external_function_items,
        names_data: &unified_external_function_data,
    };

    // external function index section
    let external_function_ranges: Vec<RangeItem> = vec![RangeItem {
        offset: 0,
        count: unified_external_function_entries.len() as u32,
    }];

    let external_function_index_items: Vec<ExternalFunctionIndexItem> = external_function_entries
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            ExternalFunctionIndexItem::new(idx as u32, idx as u32, item.type_index as u32)
        })
        .collect::<Vec<_>>();

    let external_function_index_section = ExternalFunctionIndexSection {
        ranges: &external_function_ranges,
        items: &external_function_index_items,
    };

    // build module image
    let section_entries: Vec<&dyn SectionEntry> = vec![
        // index sections
        &data_index_section,
        &function_index_section,
        &unified_external_library_section,
        &unified_external_function_section,
        &external_function_index_section,
        // common sections
        &ro_data_section,
        &rw_data_section,
        &uninit_data_section,
        &type_section,
        &function_section,
        &local_var_section,
        &external_library_section,
        &external_function_section,
    ];

    let (section_items, sections_data) = ModuleImage::convert_from_entries(&section_entries);
    let module_image = ModuleImage {
        name,
        constructor_function_public_index: u32::MAX,
        destructor_function_public_index: u32::MAX,
        items: &section_items,
        sections_data: &sections_data,
    };

    // build module image binary
    let mut image_data: Vec<u8> = Vec::new();
    module_image.save(&mut image_data).unwrap();

    image_data
}

#[cfg(test)]
mod tests {
    use ancvm_types::{
        entry::{InitedDataEntry, LocalVariableEntry, TypeEntry, UninitDataEntry},
        DataSectionType, DataType, ExternalLibraryType, MemoryDataType,
    };

    use crate::{
        load_modules_from_binaries,
        module_image::{
            data_index_section::DataIndexItem, data_section::DataItem,
            external_function_index_section::ExternalFunctionIndexItem, function_index_section::FunctionIndexItem,
            local_variable_section::LocalVariableItem, RangeItem,
        },
        utils::{
            helper_build_module_binary_with_functions_and_external_functions,
            helper_build_module_binary_with_single_function_and_data_sections,
            HelperExternalFunctionEntry, HelperFunctionEntryWithLocalVars,
        },
    };

    #[test]
    fn test_build_module_binary_with_single_function_and_data_sections() {
        let binary = helper_build_module_binary_with_single_function_and_data_sections(
            vec![DataType::I64, DataType::I64],
            vec![DataType::I32],
            vec![LocalVariableEntry::from_i32()],
            vec![0u8],
            vec![
                InitedDataEntry::from_i32(0x11),
                InitedDataEntry::from_i64(0x13),
            ],
            vec![InitedDataEntry::from_bytes(
                vec![0x17u8, 0x19, 0x23, 0x29, 0x31, 0x37],
                8,
            )],
            vec![
                UninitDataEntry::from_i32(),
                UninitDataEntry::from_i64(),
                UninitDataEntry::from_i32(),
            ],
        );

        // load module
        let module_images = load_modules_from_binaries(vec![&binary]).unwrap();
        assert_eq!(module_images.len(), 1);

        // check module image
        let module_image = &module_images[0];
        assert_eq!(module_image.name, "main");

        // check data index section
        let data_index_section = module_image.get_optional_data_index_section().unwrap();
        assert_eq!(data_index_section.ranges.len(), 1);
        assert_eq!(data_index_section.items.len(), 6);

        assert_eq!(&data_index_section.ranges[0], &RangeItem::new(0, 6));

        assert_eq!(
            data_index_section.items,
            // 2,1,3
            &[
                //
                DataIndexItem::new(0, 0, 0, DataSectionType::ReadOnly,),
                DataIndexItem::new(1, 0, 1, DataSectionType::ReadOnly,),
                //
                DataIndexItem::new(2, 0, 0, DataSectionType::ReadWrite,),
                //
                DataIndexItem::new(3, 0, 0, DataSectionType::Uninit,),
                DataIndexItem::new(4, 0, 1, DataSectionType::Uninit,),
                DataIndexItem::new(5, 0, 2, DataSectionType::Uninit,),
            ]
        );

        // check function index section
        let function_index_section = module_image.get_function_index_section();
        assert_eq!(function_index_section.ranges.len(), 1);
        assert_eq!(function_index_section.items.len(), 1);

        assert_eq!(&function_index_section.ranges[0], &RangeItem::new(0, 1));

        assert_eq!(function_index_section.items, &[FunctionIndexItem::new(0, 0, 0)]);

        // check data sections
        let ro_section = module_image.get_optional_read_only_data_section().unwrap();
        assert_eq!(
            &ro_section.items[0],
            &DataItem::new(0, 4, MemoryDataType::I32, 4)
        );
        assert_eq!(
            &ro_section.items[1],
            &DataItem::new(8, 8, MemoryDataType::I64, 8)
        );
        assert_eq!(
            &ro_section.datas_data[ro_section.items[0].data_offset as usize..][0..4],
            [0x11, 0, 0, 0]
        );
        assert_eq!(
            &ro_section.datas_data[ro_section.items[1].data_offset as usize..][0..8],
            [0x13, 0, 0, 0, 0, 0, 0, 0]
        );

        let rw_section = module_image.get_optional_read_write_data_section().unwrap();
        assert_eq!(
            &rw_section.items[0],
            &DataItem::new(0, 6, MemoryDataType::BYTES, 8)
        );
        assert_eq!(
            &rw_section.datas_data[rw_section.items[0].data_offset as usize..][0..6],
            &[0x17u8, 0x19, 0x23, 0x29, 0x31, 0x37]
        );

        let uninit_section = module_image.get_optional_uninit_data_section().unwrap();
        assert_eq!(
            &uninit_section.items[0],
            &DataItem::new(0, 4, MemoryDataType::I32, 4)
        );
        assert_eq!(
            &uninit_section.items[1],
            &DataItem::new(8, 8, MemoryDataType::I64, 8)
        );
        assert_eq!(
            &uninit_section.items[2],
            &DataItem::new(16, 4, MemoryDataType::I32, 4)
        );

        // check type section
        let type_section = module_image.get_type_section();
        assert_eq!(type_section.items.len(), 1);
        assert_eq!(
            type_section.get_item_params_and_results(0),
            (
                vec![DataType::I64, DataType::I64].as_ref(),
                vec![DataType::I32].as_ref()
            )
        );

        // check func section
        let function_section = module_image.get_function_section();
        assert_eq!(function_section.items.len(), 1);

        assert_eq!(
            function_section.get_item_type_index_and_local_variable_index_and_code(0),
            (0, 0, vec![0u8].as_ref())
        );

        // check local variable section
        let local_variable_section = module_image.get_local_variable_section();
        assert_eq!(local_variable_section.lists.len(), 1);
        assert_eq!(
            local_variable_section.get_local_list(0),
            &[
                LocalVariableItem::new(0, 8, MemoryDataType::I64, 8),
                LocalVariableItem::new(8, 8, MemoryDataType::I64, 8),
                LocalVariableItem::new(16, 4, MemoryDataType::I32, 4),
            ]
        );
    }

    #[test]
    fn test_build_module_binary_with_functions_and_blocks() {
        // TODO
    }

    #[test]
    fn test_build_module_binary_with_single_function_and_external_functions() {
        let binary = helper_build_module_binary_with_functions_and_external_functions(
            vec![
                TypeEntry {
                    params: vec![],
                    results: vec![],
                },
                TypeEntry {
                    params: vec![DataType::I32],
                    results: vec![],
                },
                TypeEntry {
                    params: vec![DataType::I32, DataType::I32],
                    results: vec![DataType::I32],
                },
            ],
            vec![HelperFunctionEntryWithLocalVars {
                type_index: 0,
                local_variable_item_entries_without_args: vec![],
                code: vec![0u8],
            }],
            vec![],
            vec![],
            vec![],
            vec![
                HelperExternalFunctionEntry {
                    external_library_type: ExternalLibraryType::System,
                    library_name: "libc.so".to_string(),
                    function_name: "getuid".to_string(),
                    type_index: 1,
                },
                HelperExternalFunctionEntry {
                    external_library_type: ExternalLibraryType::System,
                    library_name: "libc.so".to_string(),
                    function_name: "getenv".to_string(),
                    type_index: 2,
                },
                HelperExternalFunctionEntry {
                    external_library_type: ExternalLibraryType::Share,
                    library_name: "libmagic.so".to_string(),
                    function_name: "magic_open".to_string(), // magic_load
                    type_index: 2,
                },
                HelperExternalFunctionEntry {
                    external_library_type: ExternalLibraryType::User,
                    library_name: "libz.so".to_string(),
                    function_name: "inflate".to_string(), // inflateInit/inflateEnd
                    type_index: 1,
                },
                HelperExternalFunctionEntry {
                    external_library_type: ExternalLibraryType::System,
                    library_name: "libc.so".to_string(),
                    function_name: "fopen".to_string(),
                    type_index: 0,
                },
                HelperExternalFunctionEntry {
                    external_library_type: ExternalLibraryType::Share,
                    library_name: "libmagic.so".to_string(),
                    function_name: "magic_file".to_string(), // magic_close
                    type_index: 2,
                },
            ],
        );

        // load module
        let module_images = load_modules_from_binaries(vec![&binary]).unwrap();
        assert_eq!(module_images.len(), 1);

        let module_image = &module_images[0];

        // check unified external library section
        let unified_external_library_section = module_image
            .get_optional_unified_external_library_section()
            .unwrap();
        assert_eq!(
            unified_external_library_section.get_item_name_and_external_library_type(0),
            ("libc.so", ExternalLibraryType::System)
        );
        assert_eq!(
            unified_external_library_section.get_item_name_and_external_library_type(1),
            ("libmagic.so", ExternalLibraryType::Share)
        );
        assert_eq!(
            unified_external_library_section.get_item_name_and_external_library_type(2),
            ("libz.so", ExternalLibraryType::User)
        );

        // check unified external function section
        let unified_external_function_section = module_image
            .get_optional_unified_external_function_section()
            .unwrap();
        assert_eq!(
            unified_external_function_section.get_item_name_and_unified_external_library_index(0),
            ("getuid", 0)
        );
        assert_eq!(
            unified_external_function_section.get_item_name_and_unified_external_library_index(1),
            ("getenv", 0)
        );
        assert_eq!(
            unified_external_function_section.get_item_name_and_unified_external_library_index(2),
            ("magic_open", 1)
        );
        assert_eq!(
            unified_external_function_section.get_item_name_and_unified_external_library_index(3),
            ("inflate", 2)
        );
        assert_eq!(
            unified_external_function_section.get_item_name_and_unified_external_library_index(4),
            ("fopen", 0)
        );
        assert_eq!(
            unified_external_function_section.get_item_name_and_unified_external_library_index(5),
            ("magic_file", 1)
        );

        // check external function index section
        let external_function_index_section = module_image
            .get_optional_external_function_index_section()
            .unwrap();
        assert_eq!(external_function_index_section.ranges.len(), 1);
        assert_eq!(external_function_index_section.items.len(), 6);

        assert_eq!(
            &external_function_index_section.ranges[0],
            &RangeItem::new(0, 6)
        );

        assert_eq!(
            external_function_index_section.items,
            &[
                ExternalFunctionIndexItem::new(0, 0, 1),
                ExternalFunctionIndexItem::new(1, 1, 2),
                ExternalFunctionIndexItem::new(2, 2, 2),
                ExternalFunctionIndexItem::new(3, 3, 1),
                ExternalFunctionIndexItem::new(4, 4, 0),
                ExternalFunctionIndexItem::new(5, 5, 2),
            ]
        );

        // check external library sections
        let external_library_section = module_image
            .get_optional_external_library_section()
            .unwrap();
        assert_eq!(
            external_library_section.get_item_name_and_external_library_type(0),
            ("libc.so", ExternalLibraryType::System)
        );
        assert_eq!(
            external_library_section.get_item_name_and_external_library_type(1),
            ("libmagic.so", ExternalLibraryType::Share)
        );
        assert_eq!(
            external_library_section.get_item_name_and_external_library_type(2),
            ("libz.so", ExternalLibraryType::User)
        );

        // check external function section
        let external_function_section = module_image.get_optional_external_function_section().unwrap();
        assert_eq!(
            external_function_section.get_item_name_and_external_library_index_and_type_index(0),
            ("getuid", 0, 1)
        );
        assert_eq!(
            external_function_section.get_item_name_and_external_library_index_and_type_index(1),
            ("getenv", 0, 2)
        );
        assert_eq!(
            external_function_section.get_item_name_and_external_library_index_and_type_index(2),
            ("magic_open", 1, 2)
        );
        assert_eq!(
            external_function_section.get_item_name_and_external_library_index_and_type_index(3),
            ("inflate", 2, 1)
        );
        assert_eq!(
            external_function_section.get_item_name_and_external_library_index_and_type_index(4),
            ("fopen", 0, 0)
        );
        assert_eq!(
            external_function_section.get_item_name_and_external_library_index_and_type_index(5),
            ("magic_file", 1, 2)
        );
    }
}
