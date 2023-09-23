// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_types::opcode::Opcode;
use ancvm_types::DataType;

use std::io::Write;
use std::{mem::size_of, ptr::slice_from_raw_parts};

use crate::module_image::{
    data_index_section::{DataIndexItem, DataIndexSection},
    data_section::{
        DataEntry, DataSectionType, ReadOnlyDataSection, ReadWriteDataSection, UninitDataEntry,
        UninitDataSection,
    },
    func_index_section::{FuncIndexItem, FuncIndexSection},
    func_section::{FuncEntry, FuncSection},
    local_variable_section::{LocalVariableEntry, LocalVariableSection},
    module_index_section::{ModuleIndexEntry, ModuleIndexSection, ModuleShareType},
    type_section::{TypeEntry, TypeSection},
    ModuleImage, RangeItem, SectionEntry,
};

const DATA_ALIGN_BYTES: usize = 4;

pub struct BytecodeWriter {
    buffer: Vec<u8>, // trait std::io::Write
}

pub struct BytecodeReader<'a> {
    codes: &'a [u8],
    offset: usize,
}

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

    // there is a "safe" way to read a number from pointer, e.g.
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

// pub fn downcast_section_entry<'a, T>(fat: &'a dyn SectionEntry) -> &'a T {
//     let ptr = fat as *const dyn SectionEntry as *const T;
//     unsafe { &*ptr }
// }

impl BytecodeWriter {
    pub fn new() -> Self {
        Self {
            buffer: Vec::<u8>::new(),
        }
    }

    fn start_opcode(mut self, opcode: Opcode) -> Self {
        let data = (opcode as u16).to_le_bytes();
        self.buffer.write_all(&data).unwrap();
        self
    }

    /// note that 'i32' in function name means a 32-bit integer, which is equivalent to
    /// the 'uint32_t' in C or 'u32' in Rust. do not confuse it with 'i32' in Rust.
    /// the same applies to the i8, i16 and i64.
    fn start_opcode_with_i16(self, opcode: Opcode, value: u16) -> Self {
        let mut new_self = self.start_opcode(opcode);
        let data = value.to_le_bytes();
        new_self.buffer.write_all(&data).unwrap();
        new_self
    }

    fn start_opcode_with_16bits_padding(self, opcode: Opcode) -> Self {
        self.start_opcode_with_i16(opcode, 0)
    }

    fn append_i32(mut self, value: u32) -> Self {
        let data = value.to_le_bytes();
        self.buffer.write_all(&data).unwrap();
        self
    }

    fn require_4bytes_padding(self) -> Self {
        if self.buffer.len() % 4 != 0 {
            // insert padding instruction
            self.start_opcode(Opcode::nop)
        } else {
            self
        }
    }

    /// 16-bit instruction
    pub fn write_opcode(self, opcode: Opcode) -> Self {
        self.start_opcode(opcode)
    }

    /// (16+16)-bit instruction
    pub fn write_opcode_i16(self, opcode: Opcode, value: u16) -> Self {
        self.start_opcode_with_i16(opcode, value)
    }

    /// 64-bit instruction
    pub fn write_opcode_i32(self, opcode: Opcode, value: u32) -> Self {
        self.require_4bytes_padding()
            .start_opcode_with_16bits_padding(opcode)
            .append_i32(value)
    }

    /// 64-bit instruction
    pub fn write_opcode_i16_i32(self, opcode: Opcode, param0: u16, param1: u32) -> Self {
        self.require_4bytes_padding()
            .start_opcode_with_i16(opcode, param0)
            .append_i32(param1)
    }

    /// 96-bit instruction
    pub fn write_opcode_i32_i32(self, opcode: Opcode, param0: u32, param1: u32) -> Self {
        self.require_4bytes_padding()
            .start_opcode_with_16bits_padding(opcode)
            .append_i32(param0)
            .append_i32(param1)
    }

    /// 96-bit instruction
    pub fn write_opcode_pesudo_i64(self, opcode: Opcode, value: u64) -> Self {
        let data = value.to_le_bytes();
        let mut new_self = self
            .require_4bytes_padding()
            .start_opcode_with_16bits_padding(opcode);
        new_self.buffer.write_all(&data).unwrap();
        new_self
    }

    /// 64-bit instruction
    pub fn write_opcode_pesudo_f32(self, opcode: Opcode, value: f32) -> Self {
        let data = value.to_le_bytes();
        let mut new_self = self
            .require_4bytes_padding()
            .start_opcode_with_16bits_padding(opcode);
        new_self.buffer.write_all(&data).unwrap();
        new_self
    }

    /// 96-bit instruction
    pub fn write_opcode_pesudo_f64(self, opcode: Opcode, value: f64) -> Self {
        let data = value.to_le_bytes();
        let mut new_self = self
            .require_4bytes_padding()
            .start_opcode_with_16bits_padding(opcode);
        new_self.buffer.write_all(&data).unwrap();
        new_self
    }

    pub fn to_bytes(self) -> Vec<u8> {
        self.buffer
    }

    pub fn save_bytecodes(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        writer.write_all(&self.buffer)
    }
}

impl Default for BytecodeWriter {
    fn default() -> Self {
        Self::new()
    }
}

pub fn print_bytecodes(codes: &[u8]) -> String {
    // display the bytecode as the following format:
    //
    // 0x0008  00 11 22 33  44 55 66 77
    // 0x0000  88 99 aa bb  cc dd ee ff
    //
    codes
        .iter()
        .enumerate()
        .map(|(idx, data)| {
            // Rust std format!()
            // https://doc.rust-lang.org/std/fmt/
            if idx % 8 == 0 {
                if idx == 0 {
                    // first line
                    format!("0x{:04x}  {:02x}", idx, data)
                } else {
                    // new line
                    format!("\n0x{:04x}  {:02x}", idx, data)
                }
            } else if idx % 4 == 0 {
                // middle
                format!("  {:02x}", data)
            } else {
                format!(" {:02x}", data)
            }
        })
        .collect::<Vec<String>>()
        .join("")
}

impl<'a> BytecodeReader<'a> {
    pub fn new(codes: &'a [u8]) -> Self {
        Self { codes, offset: 0 }
    }

    /// opcode, or
    /// 16 bits instruction
    /// [opcode]
    fn read_opcode(&mut self) -> Opcode {
        let opcode_data = &self.codes[self.offset..self.offset + 2];
        self.offset += 2;

        let opcode_u16 = u16::from_le_bytes(opcode_data.try_into().unwrap());
        unsafe { std::mem::transmute::<u16, Opcode>(opcode_u16) }
    }

    /// 32 bits instruction
    /// [opcode + i16]
    fn read_param_i16(&mut self) -> u16 {
        let param_data0 = &self.codes[self.offset..self.offset + 2];
        self.offset += 2;

        u16::from_le_bytes(param_data0.try_into().unwrap())
    }

    /// 64 bits instruction
    /// [opcode + padding + i32]
    ///
    /// note that 'i32' in function name means a 32-bit integer, which is equivalent to
    /// the 'uint32_t' in C or 'u32' in Rust. do not confuse it with 'i32' in Rust.
    /// the same applies to the i8, i16 and i64.
    fn read_param_i32(&mut self) -> u32 {
        let param_data0 = &self.codes[self.offset + 2..self.offset + 2 + 4];
        self.offset += 6;

        u32::from_le_bytes(param_data0.try_into().unwrap())
    }

    /// 64 bits instruction
    /// [opcode + i16 + i32]
    fn read_param_i16_i32(&mut self) -> (u16, u32) {
        let param_data0 = &self.codes[self.offset..self.offset + 2];
        let param_data1 = &self.codes[self.offset + 2..self.offset + 2 + 4];
        self.offset += 6;

        (
            u16::from_le_bytes(param_data0.try_into().unwrap()),
            u32::from_le_bytes(param_data1.try_into().unwrap()),
        )
    }

    /// 96 bits instruction
    /// [opcode + padding + i32 + i32]
    fn read_param_i32_i32(&mut self) -> (u32, u32) {
        let param_data0 = &self.codes[self.offset + 2..self.offset + 2 + 4];
        let param_data1 = &self.codes[self.offset + 2 + 4..self.offset + 2 + 4 + 4];
        self.offset += 10;

        (
            u32::from_le_bytes(param_data0.try_into().unwrap()),
            u32::from_le_bytes(param_data1.try_into().unwrap()),
        )
    }

    pub fn to_text(&mut self) -> String {
        let mut lines: Vec<String> = Vec::new();

        let code_len = self.codes.len();
        loop {
            let offset = self.offset;
            if offset == code_len {
                break;
            };

            let opcode = self.read_opcode();

            // format!(...)
            // https://doc.rust-lang.org/std/fmt/
            let mut line = format!("0x{:04x} {:20} ", offset, opcode.get_name());

            match opcode {
                // fundemental
                Opcode::zero | Opcode::drop | Opcode::duplicate | Opcode::swap => {}
                Opcode::i32_imm | Opcode::f32_imm => {
                    let v = self.read_param_i32();
                    line.push_str(&format!("0x{:x}", v));
                }
                Opcode::i64_imm | Opcode::f64_imm => {
                    let (v_low, v_high) = self.read_param_i32_i32();
                    line.push_str(&format!("0x{:x} 0x{:x}", v_low, v_high));
                }
                // local load/store
                Opcode::local_load
                | Opcode::local_load32
                | Opcode::local_load32_i16_s
                | Opcode::local_load32_i16_u
                | Opcode::local_load32_i8_s
                | Opcode::local_load32_i8_u
                | Opcode::local_load_f64
                | Opcode::local_load32_f32
                | Opcode::local_store
                | Opcode::local_store32
                | Opcode::local_store16
                | Opcode::local_store8 => {
                    let (offset, index) = self.read_param_i16_i32();
                    line.push_str(&format!("{} {}", offset, index));
                }
                //
                Opcode::local_long_load
                | Opcode::local_long_load32
                | Opcode::local_long_load32_i16_s
                | Opcode::local_long_load32_i16_u
                | Opcode::local_long_load32_i8_s
                | Opcode::local_long_load32_i8_u
                | Opcode::local_long_load_f64
                | Opcode::local_long_load32_f32
                | Opcode::local_long_store
                | Opcode::local_long_store32
                | Opcode::local_long_store16
                | Opcode::local_long_store8 => {
                    let index = self.read_param_i32();
                    line.push_str(&format!("{}", index));
                }
                // data load/store
                Opcode::data_load
                | Opcode::data_load32
                | Opcode::data_load32_i16_s
                | Opcode::data_load32_i16_u
                | Opcode::data_load32_i8_s
                | Opcode::data_load32_i8_u
                | Opcode::data_load_f64
                | Opcode::data_load32_f32
                | Opcode::data_store
                | Opcode::data_store32
                | Opcode::data_store16
                | Opcode::data_store8 => {
                    let (offset, index) = self.read_param_i16_i32();
                    line.push_str(&format!("{} {}", offset, index));
                }
                //
                Opcode::data_long_load
                | Opcode::data_long_load32
                | Opcode::data_long_load32_i16_s
                | Opcode::data_long_load32_i16_u
                | Opcode::data_long_load32_i8_s
                | Opcode::data_long_load32_i8_u
                | Opcode::data_long_load_f64
                | Opcode::data_long_load32_f32
                | Opcode::data_long_store
                | Opcode::data_long_store32
                | Opcode::data_long_store16
                | Opcode::data_long_store8 => {
                    let index = self.read_param_i32();
                    line.push_str(&format!("{}", index));
                }
                // heap load/store
                Opcode::heap_load
                | Opcode::heap_load32
                | Opcode::heap_load32_i16_s
                | Opcode::heap_load32_i16_u
                | Opcode::heap_load32_i8_s
                | Opcode::heap_load32_i8_u
                | Opcode::heap_load_f64
                | Opcode::heap_load32_f32
                | Opcode::heap_store
                | Opcode::heap_store32
                | Opcode::heap_store16
                | Opcode::heap_store8 => {
                    let offset = self.read_param_i16();
                    line.push_str(&format!("{}", offset));
                }
                // conversion
                Opcode::i32_demote_i64
                | Opcode::i64_promote_i32_s
                | Opcode::i64_promote_i32_u
                | Opcode::f32_demote_f64
                | Opcode::f64_promote_f32
                | Opcode::i32_trunc_f32_s
                | Opcode::i32_trunc_f32_u
                | Opcode::i32_trunc_f64_s
                | Opcode::i32_trunc_f64_u
                | Opcode::i64_trunc_f32_s
                | Opcode::i64_trunc_f32_u
                | Opcode::i64_trunc_f64_s
                | Opcode::i64_trunc_f64_u
                | Opcode::f32_convert_i32_s
                | Opcode::f32_convert_i32_u
                | Opcode::f32_convert_i64_s
                | Opcode::f32_convert_i64_u
                | Opcode::f64_convert_i32_s
                | Opcode::f64_convert_i32_u
                | Opcode::f64_convert_i64_s
                | Opcode::f64_convert_i64_u => {}
                // comparsion
                Opcode::i32_eqz
                | Opcode::i32_nez
                | Opcode::i32_eq
                | Opcode::i32_ne
                | Opcode::i32_lt_s
                | Opcode::i32_lt_u
                | Opcode::i32_gt_s
                | Opcode::i32_gt_u
                | Opcode::i32_le_s
                | Opcode::i32_le_u
                | Opcode::i32_ge_s
                | Opcode::i32_ge_u
                | Opcode::i64_eqz
                | Opcode::i64_nez
                | Opcode::i64_eq
                | Opcode::i64_ne
                | Opcode::i64_lt_s
                | Opcode::i64_lt_u
                | Opcode::i64_gt_s
                | Opcode::i64_gt_u
                | Opcode::i64_le_s
                | Opcode::i64_le_u
                | Opcode::i64_ge_s
                | Opcode::i64_ge_u
                | Opcode::f32_eq
                | Opcode::f32_ne
                | Opcode::f32_lt
                | Opcode::f32_gt
                | Opcode::f32_le
                | Opcode::f32_ge
                | Opcode::f64_eq
                | Opcode::f64_ne
                | Opcode::f64_lt
                | Opcode::f64_gt
                | Opcode::f64_le
                | Opcode::f64_ge => {}
                // arithmetic
                Opcode::i32_add
                | Opcode::i32_sub
                | Opcode::i32_mul
                | Opcode::i32_div_s
                | Opcode::i32_div_u
                | Opcode::i32_rem_s
                | Opcode::i32_rem_u => {}
                Opcode::i32_inc | Opcode::i32_dec => {
                    let amount = self.read_param_i16();
                    line.push_str(&format!("{}", amount));
                }
                Opcode::i64_add
                | Opcode::i64_sub
                | Opcode::i64_mul
                | Opcode::i64_div_s
                | Opcode::i64_div_u
                | Opcode::i64_rem_s
                | Opcode::i64_rem_u => {}
                Opcode::i64_inc | Opcode::i64_dec => {
                    let amount = self.read_param_i16();
                    line.push_str(&format!("{}", amount));
                }
                Opcode::f32_add
                | Opcode::f32_sub
                | Opcode::f32_mul
                | Opcode::f32_div
                | Opcode::f64_add
                | Opcode::f64_sub
                | Opcode::f64_mul
                | Opcode::f64_div => {}
                // bitwise
                Opcode::i32_and
                | Opcode::i32_or
                | Opcode::i32_xor
                | Opcode::i32_not
                | Opcode::i32_leading_zeros
                | Opcode::i32_trailing_zeros
                | Opcode::i32_count_ones
                | Opcode::i32_shift_left
                | Opcode::i32_shift_right_s
                | Opcode::i32_shift_right_u
                | Opcode::i32_rotate_left
                | Opcode::i32_rotate_right
                | Opcode::i64_and
                | Opcode::i64_or
                | Opcode::i64_xor
                | Opcode::i64_not
                | Opcode::i64_leading_zeros
                | Opcode::i64_trailing_zeros
                | Opcode::i64_count_ones
                | Opcode::i64_shift_left
                | Opcode::i64_shift_right_s
                | Opcode::i64_shift_right_u
                | Opcode::i64_rotate_left
                | Opcode::i64_rotate_right => {}
                // math
                Opcode::f32_abs
                | Opcode::f32_neg
                | Opcode::f32_ceil
                | Opcode::f32_floor
                | Opcode::f32_round_half_away_from_zero
                | Opcode::f32_trunc
                | Opcode::f32_fract
                | Opcode::f32_sqrt
                | Opcode::f32_cbrt
                | Opcode::f32_pow
                | Opcode::f32_exp
                | Opcode::f32_exp2
                | Opcode::f32_ln
                | Opcode::f32_log
                | Opcode::f32_log2
                | Opcode::f32_log10
                | Opcode::f32_sin
                | Opcode::f32_cos
                | Opcode::f32_tan
                | Opcode::f32_asin
                | Opcode::f32_acos
                | Opcode::f32_atan
                | Opcode::f64_abs
                | Opcode::f64_neg
                | Opcode::f64_ceil
                | Opcode::f64_floor
                | Opcode::f64_round_half_away_from_zero
                | Opcode::f64_trunc
                | Opcode::f64_fract
                | Opcode::f64_sqrt
                | Opcode::f64_cbrt
                | Opcode::f64_pow
                | Opcode::f64_exp
                | Opcode::f64_exp2
                | Opcode::f64_ln
                | Opcode::f64_log
                | Opcode::f64_log2
                | Opcode::f64_log10
                | Opcode::f64_sin
                | Opcode::f64_cos
                | Opcode::f64_tan
                | Opcode::f64_asin
                | Opcode::f64_acos
                | Opcode::f64_atan => {}
                // control flow
                Opcode::end => {}
                Opcode::block => {
                    let type_idx = self.read_param_i32();
                    line.push_str(&format!("{}", type_idx));
                }
                Opcode::block_alt | Opcode::block_nez => {
                    let (type_idx, offset) = self.read_param_i32_i32();
                    line.push_str(&format!("{} 0x{:x}", type_idx, offset));
                }
                Opcode::break_ | Opcode::break_nez | Opcode::recur | Opcode::recur_nez => {
                    let (deepth, offset) = self.read_param_i16_i32();
                    line.push_str(&format!("{} 0x{:x}", deepth, offset));
                }
                Opcode::call | Opcode::ecall | Opcode::scall | Opcode::ccall => {
                    let idx = self.read_param_i32();
                    line.push_str(&format!("{}", idx));
                }
                Opcode::dcall => {}
                // machine
                Opcode::nop | Opcode::debug => {}
                Opcode::host_addr_local | Opcode::host_addr_data => {
                    let (offset, idx) = self.read_param_i16_i32();
                    line.push_str(&format!("{} {}", offset, idx));
                }
                Opcode::host_addr_local_long | Opcode::host_addr_data_long => {
                    let idx = self.read_param_i32();
                    line.push_str(&format!("{}", idx));
                }
                Opcode::host_addr_heap => {
                    let offset = self.read_param_i16();
                    line.push_str(&format!("{}", offset));
                }
            }

            let line_clear = line.trim_end().to_string();
            lines.push(line_clear);
        }

        lines.join("\n")
    }
}

/// testing-helper
pub struct FuncWithLocalVariableItemsEntry {
    pub type_index: usize,
    pub code: Vec<u8>,
    pub local_variable_item_entries_without_args: Vec<LocalVariableEntry>,
}

/// testing-helper
pub fn build_module_binary_with_single_function(
    param_datatypes: Vec<DataType>,
    result_datatypes: Vec<DataType>,
    code: Vec<u8>,
    local_variable_item_entries_without_args: Vec<LocalVariableEntry>,
) -> Vec<u8> {
    build_module_binary_with_single_function_and_data_sections(
        vec![],
        vec![],
        vec![],
        param_datatypes,
        result_datatypes,
        code,
        local_variable_item_entries_without_args,
    )
}

/// testing-helper
pub fn build_module_binary_with_single_function_and_data_sections(
    read_only_data_entries: Vec<DataEntry>,
    read_write_data_entries: Vec<DataEntry>,
    uninit_uninit_data_entries: Vec<UninitDataEntry>,
    param_datatypes: Vec<DataType>,
    result_datatypes: Vec<DataType>,
    code: Vec<u8>,
    local_variable_item_entries_without_args: Vec<LocalVariableEntry>,
) -> Vec<u8> {
    let type_entries = vec![TypeEntry {
        params: param_datatypes,
        results: result_datatypes,
    }];

    let func_with_local_vars_entries = vec![FuncWithLocalVariableItemsEntry {
        type_index: 0,
        code,
        local_variable_item_entries_without_args,
    }];

    build_module_binary(
        read_only_data_entries,
        read_write_data_entries,
        uninit_uninit_data_entries,
        type_entries,
        func_with_local_vars_entries,
    )
}

/// testing-helper
pub fn build_module_binary_with_single_function_and_blocks(
    param_datatypes: Vec<DataType>,
    result_datatypes: Vec<DataType>,
    codes: Vec<u8>,
    local_variable_item_entries_without_args: Vec<LocalVariableEntry>,
    block_type_entries_without_function_type: Vec<TypeEntry>,
) -> Vec<u8> {
    let mut type_entries = vec![TypeEntry {
        params: param_datatypes,
        results: result_datatypes,
    }];
    type_entries.extend_from_slice(&block_type_entries_without_function_type);

    build_module_binary(
        vec![],
        vec![],
        vec![],
        type_entries,
        vec![FuncWithLocalVariableItemsEntry {
            type_index: 0,
            code: codes,
            local_variable_item_entries_without_args,
        }],
    )
}

/// testing-helper
pub fn build_module_binary(
    read_only_data_entries: Vec<DataEntry>,
    read_write_data_entries: Vec<DataEntry>,
    uninit_uninit_data_entries: Vec<UninitDataEntry>,
    type_entries: Vec<TypeEntry>,
    func_with_local_vars_entries: Vec<FuncWithLocalVariableItemsEntry>,
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

    // build local variable list section
    let local_var_entries = func_with_local_vars_entries
        .iter()
        .map(|fv| {
            let param_datatypes = &type_entries[fv.type_index].params;
            let params_as_local_variable_item_entries = param_datatypes
                .iter()
                .map(|dt| LocalVariableEntry::from_datatype(*dt))
                .collect::<Vec<_>>();

            let mut local_variable_item_entries: Vec<LocalVariableEntry> = Vec::new();

            local_variable_item_entries
                .extend_from_slice(&fv.local_variable_item_entries_without_args);
            local_variable_item_entries.extend_from_slice(&params_as_local_variable_item_entries);

            local_variable_item_entries
        })
        .collect::<Vec<Vec<LocalVariableEntry>>>();

    // build type section
    let (type_items, types_data) = TypeSection::convert_from_entries(&type_entries);
    let type_section = TypeSection {
        items: &type_items,
        types_data: &types_data,
    };

    // build function section
    let func_entries = func_with_local_vars_entries
        .iter()
        .map(|fv| FuncEntry {
            type_index: fv.type_index,
            code: fv.code.clone(),
        })
        .collect::<Vec<FuncEntry>>();

    let (func_items, codes_data) = FuncSection::convert_from_entries(&func_entries);
    let func_section = FuncSection {
        items: &func_items,
        codes_data: &codes_data,
    };

    // build local variable section
    let local_var_ref_entries = local_var_entries.iter().map(|e| &e[..]).collect::<Vec<_>>();
    let (local_var_lists, local_var_list_data) =
        LocalVariableSection::convert_from_entries(&local_var_ref_entries);
    let local_var_section = LocalVariableSection {
        lists: &local_var_lists,
        list_data: &local_var_list_data,
    };

    // build module index
    let mod_index_entries: Vec<ModuleIndexEntry> = vec![ModuleIndexEntry::new(
        ModuleShareType::Local,
        "main".to_string(),
    )];

    let (module_index_items, names_data) =
        ModuleIndexSection::convert_from_entries(&mod_index_entries);
    let mod_index_section = ModuleIndexSection {
        items: &module_index_items,
        names_data: &names_data,
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
            data_section_type,
            idx as u32,
        ));
    }

    let data_index_section = DataIndexSection {
        ranges: &data_ranges,
        items: &data_index_items,
    };

    // build function index
    let func_ranges: Vec<RangeItem> = vec![RangeItem {
        offset: 0,
        count: 1,
    }];

    let func_index_items: Vec<FuncIndexItem> = vec![FuncIndexItem::new(0, 0, 0)];
    let func_index_section = FuncIndexSection {
        ranges: &func_ranges,
        items: &func_index_items,
    };

    // build module image
    let section_entries: Vec<&dyn SectionEntry> = vec![
        &mod_index_section,
        &data_index_section,
        &func_index_section,
        &ro_data_section,
        &rw_data_section,
        &uninit_data_section,
        &type_section,
        &func_section,
        &local_var_section,
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

#[cfg(test)]
mod tests {
    use ancvm_types::{opcode::Opcode, DataType, MemoryDataType};

    use crate::{
        load_modules_binary,
        module_image::{
            data_index_section::DataIndexItem,
            data_section::{DataEntry, DataItem, DataSectionType, UninitDataEntry},
            func_index_section::FuncIndexItem,
            func_section::FuncEntry,
            local_variable_section::{LocalVariableEntry, LocalVariableItem},
            module_index_section::ModuleShareType,
            type_section::TypeEntry,
            RangeItem,
        },
        utils::{
            build_module_binary_with_single_function_and_data_sections, BytecodeReader,
            BytecodeWriter,
        },
    };

    use super::print_bytecodes;

    #[test]
    fn test_build_single_function_module_binary_and_data_sections() {
        let binary = build_module_binary_with_single_function_and_data_sections(
            vec![DataEntry::from_i32(0x11), DataEntry::from_i64(0x13)],
            vec![DataEntry::from_bytes(
                vec![0x17u8, 0x19, 0x23, 0x29, 0x31, 0x37],
                8,
            )],
            vec![
                UninitDataEntry::from_i32(),
                UninitDataEntry::from_i64(),
                UninitDataEntry::from_i32(),
            ],
            vec![DataType::I64, DataType::I64],
            vec![DataType::I32],
            vec![0u8],
            vec![LocalVariableEntry::from_i32()],
        );

        let module_images = load_modules_binary(vec![&binary]).unwrap();

        // check module image

        assert_eq!(module_images.len(), 1);

        let module_image = &module_images[0];

        // check module index section
        let module_index_section = module_image.get_module_index_section();
        assert_eq!(module_index_section.items.len(), 1);

        let module_index_entry = module_index_section.get_entry(0);
        assert_eq!(module_index_entry.name, "main".to_string());
        assert_eq!(module_index_entry.module_share_type, ModuleShareType::Local);

        // check data index section
        let data_index_section = module_image.get_data_index_section();
        assert_eq!(data_index_section.ranges.len(), 1);
        assert_eq!(data_index_section.items.len(), 6);

        assert_eq!(&data_index_section.ranges[0], &RangeItem::new(0, 6));

        assert_eq!(
            data_index_section.items,
            // 2,1,3
            &vec![
                //
                DataIndexItem::new(0, 0, DataSectionType::ReadOnly, 0),
                DataIndexItem::new(1, 0, DataSectionType::ReadOnly, 1),
                //
                DataIndexItem::new(2, 0, DataSectionType::ReadWrite, 0),
                //
                DataIndexItem::new(3, 0, DataSectionType::Uninit, 0),
                DataIndexItem::new(4, 0, DataSectionType::Uninit, 1),
                DataIndexItem::new(5, 0, DataSectionType::Uninit, 2),
            ]
        );

        // check function index section
        let func_index_section = module_image.get_func_index_section();
        assert_eq!(func_index_section.ranges.len(), 1);
        assert_eq!(func_index_section.items.len(), 1);

        assert_eq!(&func_index_section.ranges[0], &RangeItem::new(0, 1));

        assert_eq!(func_index_section.items, &vec![FuncIndexItem::new(0, 0, 0)]);

        // check data sections

        let ro_section = module_image.get_read_only_data_section();
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

        let rw_section = module_image.get_read_write_data_section();
        assert_eq!(
            &rw_section.items[0],
            &DataItem::new(0, 6, MemoryDataType::BYTES, 8)
        );
        assert_eq!(
            &rw_section.datas_data[rw_section.items[0].data_offset as usize..][0..6],
            &[0x17u8, 0x19, 0x23, 0x29, 0x31, 0x37]
        );

        let uninit_section = module_image.get_uninit_data_section();
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
            type_section.get_entry(0),
            TypeEntry {
                params: vec![DataType::I64, DataType::I64],
                results: vec![DataType::I32]
            }
        );

        // check func section
        let func_section = module_image.get_func_section();
        assert_eq!(func_section.items.len(), 1);
        assert_eq!(
            func_section.get_entry(0),
            FuncEntry {
                type_index: 0,
                code: vec![0u8]
            }
        );

        // check local variable section
        let local_variable_section = module_image.get_local_variable_section();
        assert_eq!(local_variable_section.lists.len(), 1);
        assert_eq!(
            local_variable_section.get_variable_list(0),
            &vec![
                LocalVariableItem::new(0, 4, MemoryDataType::I32, 4),
                LocalVariableItem::new(8, 8, MemoryDataType::I64, 8),
                LocalVariableItem::new(16, 8, MemoryDataType::I64, 8),
            ]
        );
    }

    #[test]
    fn test_bytecode_writer_schemes() {
        let code0 = BytecodeWriter::new()
            .write_opcode(Opcode::i32_add)
            .to_bytes();

        assert_eq!(code0, vec![0x00, 0x07]);

        let code1 = BytecodeWriter::new()
            .write_opcode_i16(Opcode::heap_load, 7)
            .to_bytes();

        assert_eq!(
            code1,
            vec![
                0x00, 0x04, // opcode
                07, 0, // param
            ]
        );

        let code2 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::block, 11)
            .to_bytes();

        assert_eq!(
            code2,
            vec![
                0x01, 0x0a, // opcode
                0, 0, // padding
                11, 0, 0, 0 // param
            ]
        );

        let code3 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::break_, 13, 17)
            .to_bytes();

        assert_eq!(
            code3,
            vec![
                0x02, 0x0a, // opcode
                13, 0, // param 0
                17, 0, 0, 0 // param 1
            ]
        );

        let code4 = BytecodeWriter::new()
            .write_opcode_i32_i32(Opcode::block_alt, 19, 23)
            .to_bytes();

        assert_eq!(
            code4,
            vec![
                0x04, 0x0a, // opcode
                0, 0, // padding
                19, 0, 0, 0, // param 0
                23, 0, 0, 0 // param 1
            ]
        );

        let code5 = BytecodeWriter::new()
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x1122334455667788u64)
            .to_bytes();

        assert_eq!(
            code5,
            vec![
                0x05, 0x01, // opcode
                0, 0, // padding
                0x88, 0x77, 0x66, 0x55, // param 0
                0x44, 0x33, 0x22, 0x11 // param 1
            ]
        );

        let code6 = BytecodeWriter::new()
            .write_opcode_pesudo_f64(Opcode::f64_imm, 6.62607015e-34f64)
            .to_bytes();

        // 6.62607015e-34f64 (dec) -> 0x390B860B DE023111 (hex)

        assert_eq!(
            code6,
            vec![
                0x07, 0x01, // opcode
                0, 0, // padding
                0x11, 0x31, 0x02, 0xde, // param 0
                0x0b, 0x86, 0x0b, 0x39, // param 1
            ]
        );
    }

    #[test]
    fn test_bytecode_writer_padding() {
        let code0 = BytecodeWriter::new()
            .write_opcode(Opcode::i32_add)
            .write_opcode_i16(Opcode::heap_load, 0x5)
            .write_opcode_i16(Opcode::heap_store, 0x7)
            // padding
            .write_opcode_i16_i32(Opcode::local_load, 0x11, 0x13)
            .write_opcode_i16_i32(Opcode::local_store, 0x17, 0x19)
            .write_opcode(Opcode::i32_sub)
            .write_opcode(Opcode::i32_mul)
            .write_opcode_i16_i32(Opcode::local_load, 0x23, 0x29)
            .write_opcode_i16_i32(Opcode::local_store, 0x31, 0x37)
            .write_opcode(Opcode::i32_div_s)
            // padding
            .write_opcode_i32(Opcode::block, 0x41)
            .write_opcode_i32(Opcode::call, 0x43)
            .write_opcode(Opcode::i32_div_u)
            // padding
            .write_opcode_i32_i32(Opcode::i64_imm, 0x47, 0x53)
            .write_opcode_i32_i32(Opcode::block_alt, 0x59, 0x61)
            .to_bytes();

        assert_eq!(
            code0,
            vec![
                // i32_add
                0x00, 0x07, //
                // heap load 0x5
                0x00, 0x04, //
                0x5, 0, //
                // heap store 0x7
                0x08, 0x04, //
                0x7, 0, //
                // padding nop
                0x00, 0x0b, //
                // local_load 0x11 0x13
                0x00, 0x02, //
                0x11, 0x00, //
                0x13, 0x00, 0x00, 0x00, //
                // local_store 0x17 0x19
                0x08, 0x02, //
                0x17, 0x00, //
                0x19, 0x00, 0x00, 0x00, //
                // i32_sub
                0x01, 0x07, //
                // i32_mul
                0x02, 0x07, //
                // local_load 0x23 0x29
                0x00, 0x02, //
                0x23, 0x00, //
                0x29, 0x00, 0x00, 0x00, //
                // local_store 0x31 0x37
                0x08, 0x02, //
                0x31, 0x00, //
                0x37, 0x00, 0x00, 0x00, //
                // i32_div_s
                0x03, 0x07, //
                // padding nop
                0x00, 0x0b, //
                // block 0x41
                0x01, 0x0a, //
                0x00, 0x00, //
                0x41, 0x00, 0x00, 0x00, //
                // call 0x43
                0x08, 0x0a, //
                0x00, 0x00, //
                0x43, 0x00, 0x00, 0x00, //
                // i32_div_u
                0x04, 0x07, //
                // padding nop
                0x00, 0x0b, //
                // i64_imm 0x47 0x53
                0x05, 0x01, //
                0x00, 0x00, //
                0x47, 0x00, 0x00, 0x00, //
                0x53, 0x00, 0x00, 0x00, //
                // block_alt 0x59 0x61
                0x04, 0x0a, //
                0x00, 0x00, //
                0x59, 0x00, 0x00, 0x00, //
                0x61, 0x00, 0x00, 0x00, //
            ]
        );
    }

    #[test]
    fn test_bytecode_print() {
        let code0 = BytecodeWriter::new()
            .write_opcode(Opcode::i32_add)
            .write_opcode_i16(Opcode::heap_load, 0x5)
            .write_opcode_i16(Opcode::heap_store, 0x7)
            // padding
            .write_opcode_i16_i32(Opcode::local_load, 0x11, 0x13)
            .write_opcode_i16_i32(Opcode::local_store, 0x17, 0x19)
            .write_opcode(Opcode::i32_sub)
            .write_opcode(Opcode::i32_mul)
            .write_opcode_i16_i32(Opcode::local_load, 0x23, 0x29)
            .write_opcode_i16_i32(Opcode::local_store, 0x31, 0x37)
            .write_opcode(Opcode::i32_div_s)
            // padding
            .write_opcode_i32(Opcode::block, 0x41)
            .write_opcode_i32(Opcode::call, 0x43)
            .write_opcode(Opcode::i32_div_u)
            // padding
            .write_opcode_i32_i32(Opcode::i64_imm, 0x47, 0x53)
            .write_opcode_i32_i32(Opcode::block_alt, 0x59, 0x61)
            .to_bytes();

        let text = print_bytecodes(&code0);

        assert_eq!(
            text,
            "
            0x0000  00 07 00 04  05 00 08 04
            0x0008  07 00 00 0b  00 02 11 00
            0x0010  13 00 00 00  08 02 17 00
            0x0018  19 00 00 00  01 07 02 07
            0x0020  00 02 23 00  29 00 00 00
            0x0028  08 02 31 00  37 00 00 00
            0x0030  03 07 00 0b  01 0a 00 00
            0x0038  41 00 00 00  08 0a 00 00
            0x0040  43 00 00 00  04 07 00 0b
            0x0048  05 01 00 00  47 00 00 00
            0x0050  53 00 00 00  04 0a 00 00
            0x0058  59 00 00 00  61 00 00 00"
                .split('\n')
                .map(|line| line.trim_start().to_string())
                .collect::<Vec<String>>()[1..]
                .join("\n")
        );
    }

    #[test]
    fn test_bytecode_reader() {
        let code0 = BytecodeWriter::new()
            .write_opcode(Opcode::i32_add)
            .write_opcode_i16(Opcode::heap_load, 5)
            .write_opcode_i16(Opcode::heap_store, 7)
            // padding
            .write_opcode_i16_i32(Opcode::local_load, 11, 13)
            .write_opcode_i16_i32(Opcode::local_store, 17, 19)
            .write_opcode(Opcode::i32_sub)
            .write_opcode(Opcode::i32_mul)
            .write_opcode_i16_i32(Opcode::local_load, 23, 29)
            .write_opcode_i16_i32(Opcode::local_store, 31, 37)
            .write_opcode(Opcode::i32_div_s)
            // padding
            .write_opcode_i32(Opcode::block, 41)
            .write_opcode_i32(Opcode::call, 43)
            .write_opcode(Opcode::i32_div_u)
            // padding
            .write_opcode_i32_i32(Opcode::i64_imm, 0x47, 0x53)
            .write_opcode_i32_i32(Opcode::block_alt, 59, 61)
            .to_bytes();

        let text = BytecodeReader::new(&code0).to_text();

        assert_eq!(
            text,
            "
            0x0000 i32_add
            0x0002 heap_load            5
            0x0006 heap_store           7
            0x000a nop
            0x000c local_load           11 13
            0x0014 local_store          17 19
            0x001c i32_sub
            0x001e i32_mul
            0x0020 local_load           23 29
            0x0028 local_store          31 37
            0x0030 i32_div_s
            0x0032 nop
            0x0034 block                41
            0x003c call                 43
            0x0044 i32_div_u
            0x0046 nop
            0x0048 i64_imm              0x47 0x53
            0x0054 block_alt            59 0x3d"
                .split('\n')
                .map(|line| line.trim_start().to_string())
                .collect::<Vec<String>>()[1..]
                .join("\n")
        )
    }
}
