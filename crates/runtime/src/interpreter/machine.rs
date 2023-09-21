// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

//! it is currently assumed that the target architecture is 64-bit.

use crate::{memory::Memory, thread::Thread};

use super::InterpretResult;

pub fn nop(_thread: &mut Thread) -> InterpretResult {
    InterpretResult::Move(2)
}

pub fn break_(_thread: &mut Thread) -> InterpretResult {
    InterpretResult::Break
}

pub fn host_addr_local(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 local_variable_index:i32)
    let (offset_bytes, local_variable_index) = thread.get_param_i16_i32();
    do_host_addr_local(thread, local_variable_index as usize, offset_bytes as usize)
}

pub fn host_addr_local_long(thread: &mut Thread) -> InterpretResult {
    // (param local_variable_index:i32) (operand offset_bytes:i32)
    let local_variable_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_host_addr_local(thread, local_variable_index as usize, offset_bytes as usize)
}

fn do_host_addr_local(
    thread: &mut Thread,
    local_variable_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let final_offset = thread.get_current_function_local_variable_address_by_index_and_offset(
        local_variable_index,
        offset_bytes,
    );
    let ptr = thread.stack.get_ptr(final_offset);
    let address = ptr as u64;

    thread.stack.push_i64_u(address);

    InterpretResult::Move(8)
}

pub fn host_addr_data(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16 data_index:i32)
    let (offset_bytes, data_index) = thread.get_param_i16_i32();
    do_host_addr_data(thread, data_index as usize, offset_bytes as usize)
}

pub fn host_addr_data_long(thread: &mut Thread) -> InterpretResult {
    // (param data_index:i32) (operand offset_bytes:i32)
    let data_index = thread.get_param_i32();
    let offset_bytes = thread.stack.pop_i32_u();
    do_host_addr_data(thread, data_index as usize, offset_bytes as usize)
}

fn do_host_addr_data(
    thread: &mut Thread,
    data_index: usize,
    offset_bytes: usize,
) -> InterpretResult {
    let (datas, _target_module_index, internal_data_index) =
        thread.get_current_module_internal_data_index_and_datas_object(data_index);
    let final_offset = datas.get_idx_address(internal_data_index, offset_bytes);
    let ptr = datas.get_ptr(final_offset);
    let address = ptr as u64;

    thread.stack.push_i64_u(address);

    InterpretResult::Move(8)
}

pub fn host_addr_heap(thread: &mut Thread) -> InterpretResult {
    // (param offset_bytes:i16) (operand heap_addr:i64)
    let offset_bytes = thread.get_param_i16();
    let heap_address = thread.stack.pop_i64_u();

    let total_offset = heap_address as usize + offset_bytes as usize;
    let ptr = thread.heap.get_ptr(total_offset);

    let address = ptr as u64;

    thread.stack.push_i64_u(address);
    InterpretResult::Move(4)
}

#[cfg(test)]
mod tests {

    use ancvm_binary::{
        load_modules_binary,
        module_image::{
            data_section::{DataEntry, UninitDataEntry},
            local_variable_section::LocalVariableEntry,
        },
        utils::{
            build_module_binary_with_single_function,
            build_module_binary_with_single_function_and_data_sections, BytecodeReader,
            BytecodeWriter,
        },
    };
    use ancvm_types::{ecallcode::ECallCode, opcode::Opcode, DataType, ForeignValue};

    use crate::{init_runtime, interpreter::process_function, thread::Thread};

    #[test]
    fn test_process_machine() {
        // bytecodes
        //
        // 0x0000 nop
        // 0x0002 end
        //
        // (i32, i32) -> (i32, i32)

        let code0 = BytecodeWriter::new()
            .write_opcode(Opcode::nop)
            .write_opcode(Opcode::end)
            .to_bytes();

        assert_eq!(
            BytecodeReader::new(&code0).to_text(),
            "0x0000 nop\n0x0002 end"
        );

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32], // params
            vec![DataType::I32, DataType::I32], // results
            code0,
            vec![], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        init_runtime();
        let result0 = process_function(
            &mut thread0,
            0,
            0,
            &vec![ForeignValue::UInt32(7), ForeignValue::UInt32(11)],
        );
        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::UInt32(7), ForeignValue::UInt32(11)]
        );
    }

    #[test]
    fn test_process_host_address() {
        //        read-only data section
        //       |low address    high addr|
        //       |                        |
        // index |0              1        |
        //  type |i32------|    |i32------|
        //
        //  data 11 00 00 00    13 00 00 00
        //
        //        read write data section
        //       |low address             high address|
        //       |                                    |
        // index |2(0)                       3(1)     |
        //  type |i64------------------|    |i32------|
        //
        //  data 17 00 00 00 00 00 00 00    19 00 00 00
        //
        //        uninitialized data section
        //       |low address             high address|
        //       |                                    |
        // index |4(0)           5(1)                 |
        //  type |i32------|    |i64------------------|
        //
        //  data 23 00 00 00    29 00 00 00 00 00 00 00
        //
        //        local variable area
        //       |low address                                       high addr|
        //       |                                                           |
        // index |0       1                           2                      |
        //  type |bytes| |i32------|   |padding--|   |i32------|   |padding--|
        //
        //  data 0.....0 31 00 00 00   00 00 00 00   37 00 00 00   00 00 00 00
        //       ^
        //       | 64 bytes
        //       |because the results will overwrite the stack, so leave enough space for results
        //
        // () -> (i64,i64,i64,i64,  i64,i64,i64,i64)

        // bytecodes
        //
        // 0x0000 i64_imm              0x17 0x0
        // 0x000c data_store           0 2
        // 0x0014 i32_imm              0x19
        // 0x001c data_store32         0 3
        // 0x0024 i32_imm              0x23
        // 0x002c data_store32         0 4
        // 0x0034 i64_imm              0x29 0x0
        // 0x0040 data_store           0 5
        // 0x0048 i32_imm              0x31
        // 0x0050 local_store32        0 1
        // 0x0058 i32_imm              0x37
        // 0x0060 local_store32        0 2
        // 0x0068 host_addr_data       0 0
        // 0x0070 host_addr_data       0 1
        // 0x0078 host_addr_data       0 2
        // 0x0080 host_addr_data       0 3
        // 0x0088 host_addr_data       0 4
        // 0x0090 host_addr_data       0 5
        // 0x0098 host_addr_local      0 1
        // 0x00a0 host_addr_local      0 2
        // 0x00a8 end
        //
        // () -> (i64,i64,i64,i64,  i64,i64,i64,i64)

        let code0 = BytecodeWriter::new()
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x17)
            .write_opcode_i16_i32(Opcode::data_store, 0, 2)
            //
            .write_opcode_i32(Opcode::i32_imm, 0x19)
            .write_opcode_i16_i32(Opcode::data_store32, 0, 3)
            //
            .write_opcode_i32(Opcode::i32_imm, 0x23)
            .write_opcode_i16_i32(Opcode::data_store32, 0, 4)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x29)
            .write_opcode_i16_i32(Opcode::data_store, 0, 5)
            //
            .write_opcode_i32(Opcode::i32_imm, 0x31)
            .write_opcode_i16_i32(Opcode::local_store32, 0, 1)
            .write_opcode_i32(Opcode::i32_imm, 0x37)
            .write_opcode_i16_i32(Opcode::local_store32, 0, 2)
            //
            .write_opcode_i16_i32(Opcode::host_addr_data, 0, 0)
            .write_opcode_i16_i32(Opcode::host_addr_data, 0, 1)
            .write_opcode_i16_i32(Opcode::host_addr_data, 0, 2)
            .write_opcode_i16_i32(Opcode::host_addr_data, 0, 3)
            .write_opcode_i16_i32(Opcode::host_addr_data, 0, 4)
            .write_opcode_i16_i32(Opcode::host_addr_data, 0, 5)
            //
            .write_opcode_i16_i32(Opcode::host_addr_local, 0, 1)
            .write_opcode_i16_i32(Opcode::host_addr_local, 0, 2)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function_and_data_sections(
            vec![DataEntry::from_i32(0x11), DataEntry::from_i32(0x13)],
            vec![
                DataEntry::from_i64(0xee), // init data
                DataEntry::from_i32(0xff), // init data
            ],
            vec![UninitDataEntry::from_i32(), UninitDataEntry::from_i64()],
            vec![], // params
            vec![
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
            ], // results
            code0,
            vec![
                LocalVariableEntry::from_bytes(64, 8), // space
                LocalVariableEntry::from_i32(),
                LocalVariableEntry::from_i32(),
            ], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        init_runtime();
        let result0 = process_function(&mut thread0, 0, 0, &vec![]);
        let fvs = result0.unwrap();

        // it is currently assumed that the target architecture is 64-bit.

        let read_i64 = |fv: ForeignValue| -> u64 {
            if let ForeignValue::UInt64(addr) = fv {
                let ptr = addr as *const u64;
                unsafe { std::ptr::read(ptr) }
            } else {
                0
            }
        };

        let read_i32 = |fv: ForeignValue| -> u32 {
            if let ForeignValue::UInt64(addr) = fv {
                let ptr = addr as *const u32;
                unsafe { std::ptr::read(ptr) }
            } else {
                0
            }
        };

        assert_eq!(read_i32(fvs[0]), 0x11);
        assert_eq!(read_i32(fvs[1]), 0x13);
        assert_eq!(read_i64(fvs[2]), 0x17);
        assert_eq!(read_i32(fvs[3]), 0x19);
        assert_eq!(read_i32(fvs[4]), 0x23);
        assert_eq!(read_i64(fvs[5]), 0x29);

        // note:
        // depending on the implementation of the stack (the stack frame and local variables),
        // the following 'assert_eq' may fail,
        // because the local variables (as well as their host addresses) will no longer valid
        // when a function exits.

        assert_eq!(read_i32(fvs[6]), 0x31);
        assert_eq!(read_i32(fvs[7]), 0x37);
    }

    #[test]
    fn test_process_host_address_long() {
        //        read-only data section
        //       |low address  high addr|
        //       |                      |
        // index |0            1        |
        //  type |bytes----|  |byte-----|
        //
        //  data 02 03 05 07  11 13 17 19
        //       |     |            |  |
        //       |0    |1           |2 |3
        //
        //        local variable area
        //       |low address         high addr|
        //       |                             |
        // index |0       1                    |
        //  type |bytes| |bytes----------------|
        //
        //  data 0.....0 23 29 31 37 41 43 47 53
        //       ^       |        |        |  |
        //       |       |4       |5       |6 |7
        //       |
        //       | 64 bytes
        //       |because the results will overwrite the stack, so leave enough space for results
        //
        // () -> (i64,i64,i64,i64,  i64,i64,i64,i64)

        // bytecodes
        //
        // 0x0000 i64_imm              0x37312923 0x53474341
        // 0x000c local_store          0 1
        // 0x0014 i32_imm              0x0
        // 0x001c host_addr_data_long  0
        // 0x0024 i32_imm              0x2
        // 0x002c host_addr_data_long  0
        // 0x0034 i32_imm              0x2
        // 0x003c host_addr_data_long  1
        // 0x0044 i32_imm              0x3
        // 0x004c host_addr_data_long  1
        // 0x0054 i32_imm              0x0
        // 0x005c host_addr_local_long 1
        // 0x0064 i32_imm              0x3
        // 0x006c host_addr_local_long 1
        // 0x0074 i32_imm              0x6
        // 0x007c host_addr_local_long 1
        // 0x0084 i32_imm              0x7
        // 0x008c host_addr_local_long 1
        // 0x0094 end
        //
        // () -> (i64,i64,i64,i64,  i64,i64,i64,i64)

        let code0 = BytecodeWriter::new()
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x5347434137312923u64)
            .write_opcode_i16_i32(Opcode::local_store, 0, 1)
            //
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::host_addr_data_long, 0)
            .write_opcode_i32(Opcode::i32_imm, 2)
            .write_opcode_i32(Opcode::host_addr_data_long, 0)
            .write_opcode_i32(Opcode::i32_imm, 2)
            .write_opcode_i32(Opcode::host_addr_data_long, 1)
            .write_opcode_i32(Opcode::i32_imm, 3)
            .write_opcode_i32(Opcode::host_addr_data_long, 1)
            //
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::host_addr_local_long, 1)
            .write_opcode_i32(Opcode::i32_imm, 3)
            .write_opcode_i32(Opcode::host_addr_local_long, 1)
            .write_opcode_i32(Opcode::i32_imm, 6)
            .write_opcode_i32(Opcode::host_addr_local_long, 1)
            .write_opcode_i32(Opcode::i32_imm, 7)
            .write_opcode_i32(Opcode::host_addr_local_long, 1)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function_and_data_sections(
            vec![
                DataEntry::from_bytes(vec![0x02u8, 0x03, 0x05, 0x07], 4), // init data
                DataEntry::from_bytes(vec![0x11u8, 0x13, 0x17, 0x19], 4), // init data
            ], // init data
            vec![],
            vec![],
            vec![], // params
            vec![
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
            ], // results
            code0,
            vec![
                LocalVariableEntry::from_bytes(64, 8), // space
                LocalVariableEntry::from_bytes(8, 8),
            ], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        init_runtime();
        let result0 = process_function(&mut thread0, 0, 0, &vec![]);
        let fvs = result0.unwrap();

        // it is currently assumed that the target architecture is 64-bit.

        let read_i8 = |fv: ForeignValue| -> u8 {
            if let ForeignValue::UInt64(addr) = fv {
                let ptr = addr as *const u8;
                unsafe { std::ptr::read(ptr) }
            } else {
                0
            }
        };

        assert_eq!(read_i8(fvs[0]), 0x02);
        assert_eq!(read_i8(fvs[1]), 0x05);
        assert_eq!(read_i8(fvs[2]), 0x17);
        assert_eq!(read_i8(fvs[3]), 0x19);

        // note:
        // depending on the implementation of the stack (the stack frame and local variables),
        // the following 'assert_eq' may fail,
        // because the local variables (as well as their host addresses) will no longer valid
        // when a function exits.

        assert_eq!(read_i8(fvs[4]), 0x23);
        assert_eq!(read_i8(fvs[5]), 0x37);
        assert_eq!(read_i8(fvs[6]), 0x47);
        assert_eq!(read_i8(fvs[7]), 0x53);
    }

    #[test]
    fn test_process_host_address_heap() {
        //
        //        heap
        //       |low address                high addr|
        //       |                                    |
        //  addr |0x100         0x200                 |
        //  type |i32-------|   |i64------------------|
        //
        //  data  02 03 05 07   11 13 17 19 23 29 31 37
        //        ^     ^       ^           ^        ^
        //        |0    |1      |2          |3       |4
        // () -> (i64,i64,i64,i64,i64)

        // bytecodes
        //
        // 0x0000 i32_imm              0x1
        // 0x0008 ecall                262
        // 0x0010 drop
        // 0x0012 nop
        // 0x0014 i32_imm              0x7050302
        // 0x001c i64_imm              0x100 0x0
        // 0x0028 heap_store32         0
        // 0x002c i64_imm              0x19171311 0x37312923
        // 0x0038 i64_imm              0x200 0x0
        // 0x0044 heap_store           0
        // 0x0048 i64_imm              0x100 0x0
        // 0x0054 host_addr_heap       0
        // 0x0058 i64_imm              0x100 0x0
        // 0x0064 host_addr_heap       2
        // 0x0068 i64_imm              0x200 0x0
        // 0x0074 host_addr_heap       0
        // 0x0078 i64_imm              0x200 0x0
        // 0x0084 host_addr_heap       4
        // 0x0088 i64_imm              0x200 0x0
        // 0x0094 host_addr_heap       7
        // 0x0098 end
        //
        // () -> (i64,i64,i64,i64,i64)

        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 1)
            .write_opcode_i32(Opcode::ecall, ECallCode::heap_resize as u32)
            .write_opcode(Opcode::drop)
            //
            .write_opcode_i32(Opcode::i32_imm, 0x07050302)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_store32, 0)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x3731292319171311)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x200)
            .write_opcode_i16(Opcode::heap_store, 0)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::host_addr_heap, 0)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::host_addr_heap, 2)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x200)
            .write_opcode_i16(Opcode::host_addr_heap, 0)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x200)
            .write_opcode_i16(Opcode::host_addr_heap, 4)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x200)
            .write_opcode_i16(Opcode::host_addr_heap, 7)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code0).to_text());

        let binary0 = build_module_binary_with_single_function(
            vec![], // params
            vec![
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
            ], // results
            code0,
            vec![], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        init_runtime();
        let result0 = process_function(&mut thread0, 0, 0, &vec![]);
        let fvs = result0.unwrap();

        // it is currently assumed that the target architecture is 64-bit.

        let read_i64 = |fv: ForeignValue| -> u64 {
            if let ForeignValue::UInt64(addr) = fv {
                let ptr = addr as *const u64;
                unsafe { std::ptr::read(ptr) }
            } else {
                0
            }
        };

        let read_i32 = |fv: ForeignValue| -> u32 {
            if let ForeignValue::UInt64(addr) = fv {
                let ptr = addr as *const u32;
                unsafe { std::ptr::read(ptr) }
            } else {
                0
            }
        };

        let read_i16 = |fv: ForeignValue| -> u16 {
            if let ForeignValue::UInt64(addr) = fv {
                let ptr = addr as *const u16;
                unsafe { std::ptr::read(ptr) }
            } else {
                0
            }
        };

        let read_i8 = |fv: ForeignValue| -> u8 {
            if let ForeignValue::UInt64(addr) = fv {
                let ptr = addr as *const u8;
                unsafe { std::ptr::read(ptr) }
            } else {
                0
            }
        };

        assert_eq!(read_i32(fvs[0]), 0x07050302);
        assert_eq!(read_i16(fvs[1]), 0x0705);
        assert_eq!(read_i64(fvs[2]), 0x3731292319171311);
        assert_eq!(read_i32(fvs[3]), 0x37312923);
        assert_eq!(read_i8(fvs[4]), 0x37);
    }
}
