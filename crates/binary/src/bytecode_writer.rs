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
    ///
    /// return the address of instruction
    pub fn write_opcode(&mut self, opcode: Opcode) -> usize {
        let addr = self.get_addr();
        self.write_i16(opcode as u16);
        addr
    }

    /// 32-bit instruction
    /// opcode 16 + param 16
    pub fn write_opcode_i16(&mut self, opcode: Opcode, value: u16) -> usize {
        let addr = self.write_opcode(opcode);
        self.write_i16(value);
        addr
    }

    fn insert_padding_if_necessary(&mut self) -> usize {
        let addr_of_next_inst = self.get_addr();

        // insert the padding instruction 'nop' if
        // the current position of byte stream is not 4-byte alignment.
        // all instructions contains 'u32' require this alignment.
        if self.buffer.len() % 4 != 0 {
            self.write_i16(Opcode::nop as u16);
            addr_of_next_inst + 2
        } else {
            addr_of_next_inst
        }
    }

    fn write_opcode_with_16bits_padding(&mut self, opcode: Opcode) -> usize {
        self.write_opcode_i16(opcode, 0)
    }

    /// 64-bit instruction
    /// opcode 16 + padding 16 + param 16
    pub fn write_opcode_i32(&mut self, opcode: Opcode, value: u32) -> usize {
        let addr = self.insert_padding_if_necessary();
        self.write_opcode_with_16bits_padding(opcode);
        self.write_i32(value);
        addr
    }

    /// 64-bit instruction
    /// opcode 16 + param0 16 + param1 32
    pub fn write_opcode_i16_i32(&mut self, opcode: Opcode, param0: u16, param1: u32) -> usize {
        let addr = self.insert_padding_if_necessary();
        self.write_opcode_i16(opcode, param0);
        self.write_i32(param1);
        addr
    }

    /// 64-bit instruction
    /// opcode 16 + param0 16 + param1 16 + param2 16
    pub fn write_opcode_i16_i16_i16(
        &mut self,
        opcode: Opcode,
        param0: u16,
        param1: u16,
        param2: u16,
    ) -> usize {
        let addr = self.write_opcode_i16(opcode, param0);
        self.write_i16(param1);
        self.write_i16(param2);
        addr
    }

    /// 96-bit instruction
    /// opcode 16 + padding 16 + param0 32 + param1 32
    pub fn write_opcode_i32_i32(&mut self, opcode: Opcode, param0: u32, param1: u32) -> usize {
        let addr = self.insert_padding_if_necessary();
        self.write_opcode_with_16bits_padding(opcode);
        self.write_i32(param0);
        self.write_i32(param1);
        addr
    }

    /// 128-bit instruction
    /// opcode 16 + padding 16 + param0 32 + param1 32 + param2 32
    pub fn write_opcode_i32_i32_i32(
        &mut self,
        opcode: Opcode,
        param0: u32,
        param1: u32,
        param2: u32,
    ) -> usize {
        let addr = self.insert_padding_if_necessary();
        self.write_opcode_with_16bits_padding(opcode);
        self.write_i32(param0);
        self.write_i32(param1);
        self.write_i32(param2);
        addr
    }

    /// 96-bit pesudo instruction
    /// opcode 16 + padding 16 + (param0 32 + param1 32)
    pub fn write_opcode_pesudo_i64(&mut self, opcode: Opcode, value: u64) -> usize {
        let data = value.to_le_bytes();

        let addr = self.insert_padding_if_necessary();
        self.write_opcode_with_16bits_padding(opcode);
        self.buffer.write_all(&data).unwrap();
        addr
    }

    /// 64-bit pesudo instruction
    /// opcode 16 + padding 16 + param0 32
    pub fn write_opcode_pesudo_f32(&mut self, opcode: Opcode, value: f32) -> usize {
        let data = value.to_le_bytes();

        let addr = self.insert_padding_if_necessary();
        self.write_opcode_with_16bits_padding(opcode);
        self.buffer.write_all(&data).unwrap();
        addr
    }

    /// 96-bit pesudo instruction
    /// opcode 16 + padding 16 + (param0 32 + param1 32)
    pub fn write_opcode_pesudo_f64(&mut self, opcode: Opcode, value: f64) -> usize {
        let data = value.to_le_bytes();

        let addr = self.insert_padding_if_necessary();
        self.write_opcode_with_16bits_padding(opcode);
        self.buffer.write_all(&data).unwrap();
        addr
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
        self.buffer[addr..(addr + 4)].copy_from_slice(value.to_le_bytes().as_ref());
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
        self.rewrite(addr + 8, next_inst_offset);
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

#[cfg(test)]
mod tests {
    use ancvm_types::opcode::Opcode;

    use crate::bytecode_writer::BytecodeWriter;

    #[test]
    fn test_bytecode_writer() {
        // 16 bits
        let code0 = BytecodeWriter::new()
            .append_opcode(Opcode::i32_add)
            .to_bytes();

        assert_eq!(code0, vec![0x00, 0x07]);

        // 32 bits
        let code1 = BytecodeWriter::new()
            .append_opcode_i16(Opcode::heap_load64_i64, 7)
            .to_bytes();

        assert_eq!(
            code1,
            vec![
                0x00, 0x04, // opcode
                7, 0, // param
            ]
        );

        // 64 bits - 1 param
        let code2 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 11)
            .to_bytes();

        assert_eq!(
            code2,
            vec![
                0x80, 0x01, // opcode
                0, 0, // padding
                11, 0, 0, 0 // param
            ]
        );

        // 64 bits - 2 params
        let code3 = BytecodeWriter::new()
            .append_opcode_i16_i32(Opcode::break_, 13, 17)
            .to_bytes();

        assert_eq!(
            code3,
            vec![
                0x02, 0x0a, // opcode
                13, 0, // param 0
                17, 0, 0, 0 // param 1
            ]
        );

        // 64 bits - 3 params
        let code4 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 19, 23, 29)
            .to_bytes();

        assert_eq!(
            code4,
            vec![
                0x00, 0x02, // opcode
                19, 0, // param 0
                23, 0, // param 1
                29, 0 // param 2
            ]
        );

        // 96 bits - 2 params
        let code5 = BytecodeWriter::new()
            .append_opcode_i32_i32(Opcode::block, 31, 37)
            .to_bytes();

        assert_eq!(
            code5,
            vec![
                0x01, 0x0a, // opcode
                0, 0, // padding
                31, 0, 0, 0, // param 0
                37, 0, 0, 0 // param 1
            ]
        );

        // 128 bits - 3 params
        let code6 = BytecodeWriter::new()
            .append_opcode_i32_i32_i32(Opcode::block_alt, 41, 73, 79)
            .to_bytes();

        assert_eq!(
            code6,
            vec![
                0x05, 0x0a, // opcode
                0, 0, // padding
                41, 0, 0, 0, // param 0
                73, 0, 0, 0, // param 1
                79, 0, 0, 0 // param 2
            ]
        );
    }

    #[test]
    fn test_bytecode_writer_with_pesudo_instructions() {
        // pesudo f32
        let code0 = BytecodeWriter::new()
            .append_opcode_pesudo_f32(Opcode::f32_imm, std::f32::consts::PI)
            .to_bytes();

        // 3.1415927 -> 0x40490FDB
        assert_eq!(
            code0,
            vec![
                0x82, 0x01, // opcode
                0, 0, // padding
                0xdb, 0x0f, 0x49, 0x40, // param 0
            ]
        );

        let code1 = BytecodeWriter::new()
            .append_opcode_pesudo_i64(Opcode::i64_imm, 0x1122334455667788u64)
            .to_bytes();

        assert_eq!(
            code1,
            vec![
                0x81, 0x01, // opcode
                0, 0, // padding
                0x88, 0x77, 0x66, 0x55, // param 0
                0x44, 0x33, 0x22, 0x11 // param 1
            ]
        );

        let code2 = BytecodeWriter::new()
            .append_opcode_pesudo_f64(Opcode::f64_imm, 6.62607015e-34f64)
            .to_bytes();

        // 6.62607015e-34f64 (dec) -> 0x390B860B DE023111 (hex)

        assert_eq!(
            code2,
            vec![
                0x83, 0x01, // opcode
                0, 0, // padding
                0x11, 0x31, 0x02, 0xde, // param 0
                0x0b, 0x86, 0x0b, 0x39, // param 1
            ]
        );
    }

    #[test]
    fn test_bytecode_writer_with_instructions_padding() {
        let code0 = BytecodeWriter::new()
            .append_opcode(Opcode::i32_add)
            .append_opcode_i16(Opcode::heap_load64_i64, 0x2)
            .append_opcode_i16(Opcode::heap_store64, 0x3)
            .append_opcode_i16_i16_i16(Opcode::local_load64_i64, 0x5, 0x7, 0x11)
            .append_opcode_i16_i16_i16(Opcode::local_store64, 0x13, 0x17, 0x19)
            // padding
            .append_opcode_i16_i32(Opcode::data_load64_i64, 0x23, 0x29)
            .append_opcode_i16_i32(Opcode::data_store64, 0x31, 0x37)
            .append_opcode(Opcode::i32_sub)
            .append_opcode(Opcode::i32_eqz)
            .append_opcode_i16_i32(Opcode::data_load64_i64, 0x41, 0x43)
            .append_opcode_i16_i32(Opcode::data_store64, 0x47, 0x53)
            .append_opcode(Opcode::i32_nez)
            // padding
            .append_opcode_i32(Opcode::i32_imm, 0x59)
            .append_opcode_i32(Opcode::call, 0x61)
            .append_opcode(Opcode::i32_eq)
            // padding
            .append_opcode_i32_i32(Opcode::i64_imm, 0x67, 0x71)
            .append_opcode_i32_i32(Opcode::block, 0x73, 0x79)
            .append_opcode(Opcode::zero)
            // padding
            .append_opcode_i32_i32_i32(Opcode::block_alt, 0x11, 0x13, 0x17)
            .append_opcode_i32_i32(Opcode::block_nez, 0x19, 0x23)
            // end
            .append_opcode(Opcode::end)
            .to_bytes();

        assert_eq!(
            code0,
            vec![
                0x00, 0x07, // i32_add
                0x00, 0x04, 0x02, 0x00, // heap_load 0x2
                0x08, 0x04, 0x03, 0x00, // heap_store 0x3
                0x00, 0x02, 0x05, 0x00, 0x07, 0x00, 0x11, 0x00, // local_load 0x5 0x7 0x11
                0x08, 0x02, 0x13, 0x00, 0x17, 0x00, 0x19, 0x00, // local_store 0x13 0x17 0x19
                0x00, 0x01, // padding nop
                0x00, 0x03, 0x23, 0x00, 0x29, 0x00, 0x00, 0x00, // data_load 0x23 0x29
                0x08, 0x03, 0x31, 0x00, 0x37, 0x00, 0x00, 0x00, // data_store 0x31 0x37
                0x01, 0x07, // i32_sub
                0x00, 0x06, // i32_eqz
                0x00, 0x03, 0x41, 0x00, 0x43, 0x00, 0x00, 0x00, // data_load 0x41 0x43
                0x08, 0x03, 0x47, 0x00, 0x53, 0x00, 0x00, 0x00, // data_store 0x47 0x53
                0x01, 0x06, // i32_nez
                0x00, 0x01, // padding nop
                0x80, 0x01, 0x00, 0x00, 0x59, 0x00, 0x00, 0x00, // i32_imm 0x59
                0x00, 0x0b, 0x00, 0x00, 0x61, 0x00, 0x00, 0x00, // call 0x61
                0x02, 0x06, // i32_eq
                0x00, 0x01, // padding nop
                0x81, 0x01, 0x00, 0x00, 0x67, 0x00, 0x00, 0x00, 0x71, 0x00, 0x00,
                0x00, // i64_imm
                0x01, 0x0a, 0x00, 0x00, 0x73, 0x00, 0x00, 0x00, 0x79, 0x00, 0x00,
                0x00, // block
                0x01, 0x01, // zero
                0x00, 0x01, // padding nop
                0x05, 0x0a, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00, 0x17, 0x00,
                0x00, 0x00, // block_alt
                0x04, 0x0a, 0x00, 0x00, 0x19, 0x00, 0x00, 0x00, 0x23, 0x00, 0x00,
                0x00, // block_nez
                0x00, 0x0a, // end
            ]
        );
    }
}
