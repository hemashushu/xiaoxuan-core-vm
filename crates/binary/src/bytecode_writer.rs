// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::io::Write;

use ancvm_types::opcode::Opcode;

pub struct BytecodeWriter {
    buffer: Vec<u8>, // trait std::io::Write
}

/// note that the word 'i32' in these function names indicate it's a 32-bit integer,
/// which is equivalent to the 'uint32_t' in C or 'u32' in Rust.
/// do not confuse it with 'i32' in Rust, the same applies to the words 'i8', 'i16' and 'i64'.
impl BytecodeWriter {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            buffer: Vec::<u8>::new(),
        }
    }

    fn write_i16(&mut self, value: u16) {
        let data = value.to_le_bytes();
        self.buffer.write_all(&data).unwrap();
    }

    fn write_i32(&mut self, value: u32) {
        let data = value.to_le_bytes();
        self.buffer.write_all(&data).unwrap();
    }

    /// 16-bit instruction
    pub fn write_opcode(&mut self, opcode: Opcode) {
        self.write_i16(opcode as u16);
    }

    /// 32-bit instruction
    /// opcode 16 + param 16
    pub fn write_opcode_i16(&mut self, opcode: Opcode, value: u16) {
        self.write_opcode(opcode);
        self.write_i16(value)
    }

    fn write_padding_if_necessary(&mut self) {
        // insert the padding instruction 'nop' if
        // the current position of byte stream is not 4-byte alignment.
        // all instructions contains 'u32' require this alignment.
        if self.buffer.len() % 4 != 0 {
            self.write_i16(Opcode::nop as u16);
        }
    }

    fn write_opcode_with_16bits_padding(&mut self, opcode: Opcode) {
        self.write_opcode_i16(opcode, 0);
    }

    /// 64-bit instruction
    /// opcode 16 + padding 16 + param 16
    pub fn write_opcode_i32(&mut self, opcode: Opcode, value: u32) {
        self.write_padding_if_necessary();
        self.write_opcode_with_16bits_padding(opcode);
        self.write_i32(value);
    }

    /// 64-bit instruction
    /// opcode 16 + param0 16 + param1 32
    pub fn write_opcode_i16_i32(&mut self, opcode: Opcode, param0: u16, param1: u32) {
        self.write_padding_if_necessary();
        self.write_opcode_i16(opcode, param0);
        self.write_i32(param1);
    }

    /// 64-bit instruction
    /// opcode 16 + param0 16 + param1 16 + param2 16
    pub fn write_opcode_i16_i16_i16(
        &mut self,
        opcode: Opcode,
        param0: u16,
        param1: u16,
        param2: u16,
    ) {
        self.write_opcode_i16(opcode, param0);
        self.write_i16(param1);
        self.write_i16(param2);
    }

    /// 96-bit instruction
    /// opcode 16 + padding 16 + param0 32 + param1 32
    pub fn write_opcode_i32_i32(&mut self, opcode: Opcode, param0: u32, param1: u32) {
        self.write_padding_if_necessary();
        self.write_opcode_with_16bits_padding(opcode);
        self.write_i32(param0);
        self.write_i32(param1);
    }

    /// 128-bit instruction
    /// opcode 16 + padding 16 + param0 32 + param1 32 + param2 32
    pub fn write_opcode_i32_i32_i32(
        &mut self,
        opcode: Opcode,
        param0: u32,
        param1: u32,
        param2: u32,
    ) {
        self.write_padding_if_necessary();
        self.write_opcode_with_16bits_padding(opcode);
        self.write_i32(param0);
        self.write_i32(param1);
        self.write_i32(param2);
    }

    /// 96-bit pesudo instruction
    /// opcode 16 + padding 16 + (param0 32 + param1 32)
    pub fn write_opcode_pesudo_i64(&mut self, opcode: Opcode, value: u64) {
        let data = value.to_le_bytes();

        self.write_padding_if_necessary();
        self.write_opcode_with_16bits_padding(opcode);
        self.buffer.write_all(&data).unwrap();
    }

    /// 64-bit pesudo instruction
    /// opcode 16 + padding 16 + param0 32
    pub fn write_opcode_pesudo_f32(&mut self, opcode: Opcode, value: f32) {
        let data = value.to_le_bytes();

        self.write_padding_if_necessary();
        self.write_opcode_with_16bits_padding(opcode);
        self.buffer.write_all(&data).unwrap();
    }

    /// 96-bit pesudo instruction
    /// opcode 16 + padding 16 + (param0 32 + param1 32)
    pub fn write_opcode_pesudo_f64(&mut self, opcode: Opcode, value: f64) {
        let data = value.to_le_bytes();

        self.write_padding_if_necessary();
        self.write_opcode_with_16bits_padding(opcode);
        self.buffer.write_all(&data).unwrap();
    }

    pub fn to_bytes(self) -> Vec<u8> {
        self.buffer
    }

    pub fn save_bytecodes(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        writer.write_all(&self.buffer)
    }
}

impl BytecodeWriter {
    pub fn get_addr(&self) -> usize {
        self.buffer.len()
    }

    fn rewrite(&mut self, addr: usize, value: u32) {
        self.buffer[addr..].copy_from_slice(&value.to_le_bytes().to_vec());
    }

    pub fn fill_break_stub(&mut self, addr: usize, next_inst_offset: u32) {
        // (opcode:i16 reversed_index:i16, next_inst_offset:i32)
        self.rewrite(addr + 4, next_inst_offset);
    }

    pub fn fill_recur_stub(&mut self, addr: usize, start_inst_offset: u32) {
        // (opcode:i16 reversed_index:i16, start_inst_offset:i32)
        self.rewrite(addr + 4, start_inst_offset);
    }

    pub fn fill_block_alt_stub(&mut self, addr: usize, alt_inst_offset: u32) {
        // (opcode:i16 padding:i16 type_index:i32 local_list_index:i32 alt_inst_offset:i32)
        self.rewrite(addr + 12, alt_inst_offset);
    }

    pub fn fill_block_nez_stub(&mut self, addr: usize, next_inst_offset: u32) {
        // (opcode:i16 padding:i16 local_list_index:i32 next_inst_offset:i32)
        self.rewrite(addr + 4, next_inst_offset);
    }

    pub fn fill_break_nez_stub(&mut self, addr: usize, next_inst_offset: u32) {
        // (opcode:i16 reversed_index:i16 next_inst_offset:i32)
        self.rewrite(addr + 4, next_inst_offset);
    }

    pub fn fill_recur_nez_stub(&mut self, addr: usize, start_inst_offset: u32) {
        // (opcode:i16 reversed_index:i16 start_inst_offset:i32)
        self.rewrite(addr + 4, start_inst_offset);
    }
}

/// chain calling style
impl BytecodeWriter {
    pub fn append_opcode(mut self, opcode: Opcode) -> Self {
        self.write_opcode(opcode);
        self
    }

    pub fn append_opcode_i16(mut self, opcode: Opcode, value: u16) -> Self {
        self.write_opcode_i16(opcode, value);
        self
    }

    pub fn append_opcode_i32(mut self, opcode: Opcode, value: u32) -> Self {
        self.write_opcode_i32(opcode, value);
        self
    }

    pub fn append_opcode_i16_i32(mut self, opcode: Opcode, param0: u16, param1: u32) -> Self {
        self.write_opcode_i16_i32(opcode, param0, param1);
        self
    }

    pub fn append_opcode_i16_i16_i16(
        mut self,
        opcode: Opcode,
        param0: u16,
        param1: u16,
        param2: u16,
    ) -> Self {
        self.write_opcode_i16_i16_i16(opcode, param0, param1, param2);
        self
    }

    pub fn append_opcode_i32_i32(mut self, opcode: Opcode, param0: u32, param1: u32) -> Self {
        self.write_opcode_i32_i32(opcode, param0, param1);
        self
    }

    pub fn append_opcode_i32_i32_i32(
        mut self,
        opcode: Opcode,
        param0: u32,
        param1: u32,
        param2: u32,
    ) -> Self {
        self.write_opcode_i32_i32_i32(opcode, param0, param1, param2);
        self
    }

    pub fn append_opcode_pesudo_i64(mut self, opcode: Opcode, value: u64) -> Self {
        self.write_opcode_pesudo_i64(opcode, value);
        self
    }

    pub fn append_opcode_pesudo_f32(mut self, opcode: Opcode, value: f32) -> Self {
        self.write_opcode_pesudo_f32(opcode, value);
        self
    }

    pub fn append_opcode_pesudo_f64(mut self, opcode: Opcode, value: f64) -> Self {
        self.write_opcode_pesudo_f64(opcode, value);
        self
    }
}
