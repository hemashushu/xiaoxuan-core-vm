// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_types::{
    opcode::{Opcode, MAX_OPCODE_NUMBER},
    ForeignValue,
};

use crate::{ecall, thread::Thread, VMError};

type InterpretFunc = fn(&mut Thread) -> InterpretResult;

mod control_flow;
mod conversion;
mod data;
mod fundamental;
mod heap;
mod host_address;
mod immediate;
mod local;

pub enum InterpretResult {
    MoveOn(usize),      // param (increment_in_bytes: usize)
    Break,              // VM debug
    Jump(usize, usize), // param (module_index: usize, instruction_address: usize)
    EnvError(usize),    // param (err_code: usize)
    End,
}

fn unreachable(_thread: &mut Thread) -> InterpretResult {
    unreachable!("Invalid opcode.")
}

static mut INTERPRETERS: [InterpretFunc; MAX_OPCODE_NUMBER] = [unreachable; MAX_OPCODE_NUMBER];

/// initilize the instruction interpreters
pub fn init_interpreters() {
    let interpreters = unsafe { &mut INTERPRETERS };

    if interpreters[Opcode::nop as usize] == fundamental::nop {
        // the initialization can only be called once
        return;
    }

    // operand
    interpreters[Opcode::nop as usize] = fundamental::nop;
    interpreters[Opcode::break_ as usize] = fundamental::break_;
    interpreters[Opcode::drop as usize] = fundamental::drop;
    interpreters[Opcode::duplicate as usize] = fundamental::duplicate;

    // immediate
    interpreters[Opcode::i32_imm as usize] = immediate::i32_imm;
    interpreters[Opcode::i64_imm as usize] = immediate::i64_imm;
    interpreters[Opcode::f32_imm as usize] = immediate::f32_imm;
    interpreters[Opcode::f64_imm as usize] = immediate::f64_imm;

    // local variables
    interpreters[Opcode::local_load as usize] = local::local_load;
    interpreters[Opcode::local_load32 as usize] = local::local_load32;
    interpreters[Opcode::local_load32_i16_s as usize] = local::local_load32_i16_s;
    interpreters[Opcode::local_load32_i16_u as usize] = local::local_load32_i16_u;
    interpreters[Opcode::local_load32_i8_s as usize] = local::local_load32_i8_s;
    interpreters[Opcode::local_load32_i8_u as usize] = local::local_load32_i8_u;
    interpreters[Opcode::local_load32_f32 as usize] = local::local_load32_f32;
    interpreters[Opcode::local_load_f64 as usize] = local::local_load_f64;
    interpreters[Opcode::local_store as usize] = local::local_store;
    interpreters[Opcode::local_store32 as usize] = local::local_store32;
    interpreters[Opcode::local_store16 as usize] = local::local_store16;
    interpreters[Opcode::local_store8 as usize] = local::local_store8;

    interpreters[Opcode::local_long_load as usize] = local::local_long_load;
    interpreters[Opcode::local_long_load32 as usize] = local::local_long_load32;
    interpreters[Opcode::local_long_load32_i16_s as usize] = local::local_long_load32_i16_s;
    interpreters[Opcode::local_long_load32_i16_u as usize] = local::local_long_load32_i16_u;
    interpreters[Opcode::local_long_load32_i8_s as usize] = local::local_long_load32_i8_s;
    interpreters[Opcode::local_long_load32_i8_u as usize] = local::local_long_load32_i8_u;
    interpreters[Opcode::local_long_load32_f32 as usize] = local::local_long_load32_f32;
    interpreters[Opcode::local_long_load_f64 as usize] = local::local_long_load_f64;
    interpreters[Opcode::local_long_store as usize] = local::local_long_store;
    interpreters[Opcode::local_long_store32 as usize] = local::local_long_store32;
    interpreters[Opcode::local_long_store16 as usize] = local::local_long_store16;
    interpreters[Opcode::local_long_store8 as usize] = local::local_long_store8;

    // data sections
    interpreters[Opcode::data_load as usize] = data::data_load;
    interpreters[Opcode::data_load32 as usize] = data::data_load32;
    interpreters[Opcode::data_load32_i16_s as usize] = data::data_load32_i16_s;
    interpreters[Opcode::data_load32_i16_u as usize] = data::data_load32_i16_u;
    interpreters[Opcode::data_load32_i8_s as usize] = data::data_load32_i8_s;
    interpreters[Opcode::data_load32_i8_u as usize] = data::data_load32_i8_u;
    interpreters[Opcode::data_load32_f32 as usize] = data::data_load32_f32;
    interpreters[Opcode::data_load_f64 as usize] = data::data_load_f64;
    interpreters[Opcode::data_store as usize] = data::data_store;
    interpreters[Opcode::data_store32 as usize] = data::data_store32;
    interpreters[Opcode::data_store16 as usize] = data::data_store16;
    interpreters[Opcode::data_store8 as usize] = data::data_store8;

    interpreters[Opcode::data_long_load as usize] = data::data_long_load;
    interpreters[Opcode::data_long_load32 as usize] = data::data_long_load32;
    interpreters[Opcode::data_long_load32_i16_s as usize] = data::data_long_load32_i16_s;
    interpreters[Opcode::data_long_load32_i16_u as usize] = data::data_long_load32_i16_u;
    interpreters[Opcode::data_long_load32_i8_s as usize] = data::data_long_load32_i8_s;
    interpreters[Opcode::data_long_load32_i8_u as usize] = data::data_long_load32_i8_u;
    interpreters[Opcode::data_long_load32_f32 as usize] = data::data_long_load32_f32;
    interpreters[Opcode::data_long_load_f64 as usize] = data::data_long_load_f64;
    interpreters[Opcode::data_long_store as usize] = data::data_long_store;
    interpreters[Opcode::data_long_store32 as usize] = data::data_long_store32;
    interpreters[Opcode::data_long_store16 as usize] = data::data_long_store16;
    interpreters[Opcode::data_long_store8 as usize] = data::data_long_store8;

    // heap
    interpreters[Opcode::heap_load as usize] = heap::heap_load;
    interpreters[Opcode::heap_load32 as usize] = heap::heap_load32;
    interpreters[Opcode::heap_load32_i16_s as usize] = heap::heap_load32_i16_s;
    interpreters[Opcode::heap_load32_i16_u as usize] = heap::heap_load32_i16_u;
    interpreters[Opcode::heap_load32_i8_s as usize] = heap::heap_load32_i8_s;
    interpreters[Opcode::heap_load32_i8_u as usize] = heap::heap_load32_i8_u;
    interpreters[Opcode::heap_load32_f32 as usize] = heap::heap_load32_f32;
    interpreters[Opcode::heap_load_f64 as usize] = heap::heap_load_f64;
    interpreters[Opcode::heap_store as usize] = heap::heap_store;
    interpreters[Opcode::heap_store32 as usize] = heap::heap_store32;
    interpreters[Opcode::heap_store16 as usize] = heap::heap_store16;
    interpreters[Opcode::heap_store8 as usize] = heap::heap_store8;

    // conversion
    interpreters[Opcode::i32_demote_i64 as usize] = conversion::i32_demote_i64;
    interpreters[Opcode::i64_promote_i32_s as usize] = conversion::i64_promote_i32_s;
    interpreters[Opcode::i64_promote_i32_u as usize] = conversion::i64_promote_i32_u;
    interpreters[Opcode::f32_demote_f64 as usize] = conversion::f32_demote_f64;
    interpreters[Opcode::f64_promote_f32 as usize] = conversion::f64_promote_f32;
    interpreters[Opcode::i32_trunc_f32_s as usize] = conversion::i32_trunc_f32_s;
    interpreters[Opcode::i32_trunc_f32_u as usize] = conversion::i32_trunc_f32_u;
    interpreters[Opcode::i32_trunc_f64_s as usize] = conversion::i32_trunc_f64_s;
    interpreters[Opcode::i32_trunc_f64_u as usize] = conversion::i32_trunc_f64_u;
    interpreters[Opcode::i64_trunc_f32_s as usize] = conversion::i64_trunc_f32_s;
    interpreters[Opcode::i64_trunc_f32_u as usize] = conversion::i64_trunc_f32_u;
    interpreters[Opcode::i64_trunc_f64_s as usize] = conversion::i64_trunc_f64_s;
    interpreters[Opcode::i64_trunc_f64_u as usize] = conversion::i64_trunc_f64_u;
    interpreters[Opcode::f32_convert_i32_s as usize] = conversion::f32_convert_i32_s;
    interpreters[Opcode::f32_convert_i32_u as usize] = conversion::f32_convert_i32_u;
    interpreters[Opcode::f32_convert_i64_s as usize] = conversion::f32_convert_i64_s;
    interpreters[Opcode::f32_convert_i64_u as usize] = conversion::f32_convert_i64_u;
    interpreters[Opcode::f64_convert_i32_s as usize] = conversion::f64_convert_i32_s;
    interpreters[Opcode::f64_convert_i32_u as usize] = conversion::f64_convert_i32_u;
    interpreters[Opcode::f64_convert_i64_s as usize] = conversion::f64_convert_i64_s;
    interpreters[Opcode::f64_convert_i64_u as usize] = conversion::f64_convert_i64_u;

    // control flow
    interpreters[Opcode::end as usize] = control_flow::end;

    // call
    interpreters[Opcode::ecall as usize] = ecall::ecall;

    // host address
    interpreters[Opcode::host_addr_local as usize] = host_address::host_addr_local;
    interpreters[Opcode::host_addr_local_long as usize] = host_address::host_addr_local_long;
    interpreters[Opcode::host_addr_data as usize] = host_address::host_addr_data;
    interpreters[Opcode::host_addr_data_long as usize] = host_address::host_addr_data_long;
    interpreters[Opcode::host_addr_heap as usize] = host_address::host_addr_heap;
}

pub fn process_next_instruction(thread: &mut Thread) -> InterpretResult {
    let opcode_num = thread.get_opcode_num();
    let func = unsafe { &INTERPRETERS[opcode_num as usize] };
    func(thread)
}

pub fn process_continuous_instructions(thread: &mut Thread) {
    loop {
        let result = //self.
                process_next_instruction(thread);
        match result {
            InterpretResult::MoveOn(increment) => {
                thread.pc.instruction_address += increment;
            }
            InterpretResult::Break => {
                thread.pc.instruction_address += 2;
            }
            InterpretResult::Jump(module_index, instruction_address) => {
                thread.pc.module_index = module_index;
                thread.pc.instruction_address = instruction_address;
            }
            InterpretResult::EnvError(code) => {
                panic!("Runtime error, code: {}", code)
            }
            InterpretResult::End => break,
        }
    }
}

pub fn process_function(
    thread: &mut Thread,
    module_index: u32,
    func_index: u32, // this index includes the imported functions
    arguments: &[ForeignValue],
) -> Result<Vec<ForeignValue>, VMError> {
    // find the code start address

    let (target_module_index, target_internal_function_index) =
        thread.get_target_function_module_index_and_internal_index(module_index, func_index);
    let (type_index, codeset, local_variables_allocate_bytes) = thread
        .get_internal_function_type_code_and_local_variables_allocate_bytes(
            target_module_index,
            target_internal_function_index,
        );

    let type_entry = thread.context.modules[target_module_index as usize]
        .type_section
        .get_entry(type_index);

    if type_entry.params.len() != arguments.len() {
        return Err(VMError::new(
            "The number of arguments does not match the specified funcion.",
        ));
    }

    // for simplicity, does not check the data type of arguments for now.

    // push arguments
    thread.push_values(arguments);

    // create function statck frame
    thread.stack.create_function_frame(
        local_variables_allocate_bytes,
        type_entry.params.len() as u16,
        type_entry.results.len() as u16,
        target_module_index,
        target_internal_function_index,
        0,
        // the '0' for 'return instruction address' is used to indicate that it's the END of the thread.
        //
        // the function stack frame is created only by 'call' instruction or
        // thread beginning, the 'call' instruction will set the 'return instruction address' to
        // the instruction next to 'call', which can't be '0'.
        // so when a stack frame exits and the 'return address' is zero, it can only
        // be the end of a thread.
        0,
    );

    // set new PC
    thread.pc.module_index = target_module_index as usize;
    thread.pc.instruction_address = codeset as usize;

    // self.
    process_continuous_instructions(thread);

    // pop results off the stack
    let results = thread.pop_values(&type_entry.results);

    Ok(results)
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        load_modules_binary,
        module_image::{
            data_section::{DataEntry, UninitDataEntry},
            local_variable_section::VariableItemEntry,
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
    fn test_process_fundamental() {
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

        // bytecodes
        //
        // 0x0000 drop
        // 0x0002 end
        //
        // (i32, i32) -> (i32)
        let code1 = BytecodeWriter::new()
            .write_opcode(Opcode::drop)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary1 = build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32], // params
            vec![DataType::I32],                // results
            code1,
            vec![], // local vars
        );

        let image1 = load_modules_binary(vec![&binary1]).unwrap();
        let mut thread1 = Thread::new(&image1);

        let result1 = process_function(
            &mut thread1,
            0,
            0,
            &vec![ForeignValue::UInt32(13), ForeignValue::UInt32(17)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(13)]);

        // bytecodes
        //
        // 0x0000 duplicate
        // 0x0002 end
        //
        // (i32) -> (i32, i32)
        let code2 = BytecodeWriter::new()
            .write_opcode(Opcode::duplicate)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary2 = build_module_binary_with_single_function(
            vec![DataType::I32],                // params
            vec![DataType::I32, DataType::I32], // results
            code2,
            vec![], // local vars
        );

        let image2 = load_modules_binary(vec![&binary2]).unwrap();
        let mut thread2 = Thread::new(&image2);

        let result2 = process_function(&mut thread2, 0, 0, &vec![ForeignValue::UInt32(19)]);
        assert_eq!(
            result2.unwrap(),
            vec![ForeignValue::UInt32(19), ForeignValue::UInt32(19)]
        );
    }

    #[test]
    fn test_process_immediate() {
        // bytecodes
        //
        // 0x0000 i32_imm              0x17
        // 0x0008 i64_imm              0x43475359 0x29313741    ;; 0x29313741_43475359
        // 0x0014 i32_imm              0xffffff21               ;; -223
        // 0x001c i64_imm              0xffffff1d 0xffffffff    ;; -227
        // 0x0028 end
        // () -> (i32, i64, i32, i64)
        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 23)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x29313741_43475359u64)
            .write_opcode_i32(Opcode::i32_imm, (0i32 - 223) as u32)
            .write_opcode_pesudo_i64(Opcode::i64_imm, (0i64 - 227) as u64)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![],                                                           // params
            vec![DataType::I32, DataType::I64, DataType::I32, DataType::I64], // results
            code0,
            vec![], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        init_runtime();
        let result0 = process_function(&mut thread0, 0, 0, &vec![]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(23),
                ForeignValue::UInt64(0x29313741_43475359u64),
                ForeignValue::UInt32((0i32 - 223) as u32),
                ForeignValue::UInt64((0i64 - 227) as u64)
            ]
        );

        // bytecodes
        //
        // 0x0000 f32_imm              0x40490fda               ;; 3.1415926
        // 0x0008 f64_imm              0xc5445f02 0x390b85f8    ;; 6.626e-34
        // 0x0014 f32_imm              0xc02df84d               ;; -2.71828
        // 0x001c f64_imm              0xb0000000 0xc1b1de6e    ;; -2.9979e8
        // 0x0028 end
        //
        // () -> (f32, f64, f32, f64)
        let code1 = BytecodeWriter::new()
            .write_opcode_pesudo_f32(Opcode::f32_imm, 3.1415926f32)
            .write_opcode_pesudo_f64(Opcode::f64_imm, 6.626e-34f64)
            .write_opcode_pesudo_f32(Opcode::f32_imm, -2.71828f32)
            .write_opcode_pesudo_f64(Opcode::f64_imm, -2.9979e8f64)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary1 = build_module_binary_with_single_function(
            vec![],                                                           // params
            vec![DataType::F32, DataType::F64, DataType::F32, DataType::F64], // results
            code1,
            vec![], // local vars
        );

        let image1 = load_modules_binary(vec![&binary1]).unwrap();
        let mut thread1 = Thread::new(&image1);

        let result1 = process_function(&mut thread1, 0, 0, &vec![]);
        assert_eq!(
            result1.unwrap(),
            vec![
                ForeignValue::Float32(3.1415926f32),
                ForeignValue::Float64(6.626e-34f64),
                ForeignValue::Float32(-2.71828f32),
                ForeignValue::Float64(-2.9979e8f64)
            ]
        );
    }

    #[test]
    fn test_process_local_load_store() {
        //       |low address                                                              high address|
        //       |                                                                                     |
        // index |0                                  1      2      3                         4         |
        //  type |bytes-------------------|         |f32|  |f64|  |i64------------------|   |i32-------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 12 17 19
        //       |           |        |  |          |      |      ^                          ^
        //       |store32    |store16 |  |          |sf32  |sf64  |                          |
        //        step0       step1   |  |          |step5 |step4 |                          |
        //                      store8|  |          |      |      |                          |
        //       |              step2    |store8    |      |      |store64                   |store32
        //       |                        step3     |      |      |                          |
        //       \----->--load64-->---------------------------->--/-->-------------------->--/
        //
        //       11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 12 17 19
        //       |           |        |  |load8u    |      |      |                          |
        //       |           |        |  |load8s  loadf32  |      |                          |
        //       |           |        |                  loadf64  |                          |
        //       |           |        |load16u                    |                          |
        //       |           |        |load16s                 load64                      load32
        //       |           |
        //       |load64     |load32
        //
        // (f32, f64) -> (i64,i32,i32,i32,i32,i32, f32,f64 ,i64,i32)

        // bytecodes
        //
        // 0x0000 i32_imm              0x19171311
        // 0x0008 local_store32        0 0          ;; store 0x19171311
        // 0x0010 i32_imm              0xd0c0
        // 0x0018 local_store16        4 0          ;; store 0xd0c0
        // 0x0020 i32_imm              0xe0
        // 0x0028 local_store8         6 0          ;; store 0xe0
        // 0x0030 i32_imm              0xf0
        // 0x0038 local_store8         7 0          ;; store 0xf0
        //
        // 0x0040 local_store          0 2          ;; store f64
        // 0x0048 local_store32        0 1          ;; store f32
        //
        // 0x0050 local_load           0 0
        // 0x0058 local_store          0 3          ;; store 0xf0e0d0c0_19171311
        // 0x0060 local_load           0 0
        // 0x0068 local_store32        0 4          ;; store 0x19171311
        //
        // 0x0070 local_load           0 0          ;; load 0xf0e0d0c0_19171311
        // 0x0078 local_load32         4 0          ;; load 0xf0e0d0c0
        // 0x0080 local_load32_i16_u   6 0          ;; load 0xf0e0
        // 0x0088 local_load32_i16_s   6 0          ;; load 0xf0e0
        // 0x0090 local_load32_i8_u    7 0          ;; load 0xf0
        // 0x0098 local_load32_i8_s    7 0          ;; load 0xf0
        //
        // 0x00a0 local_load32_f32     0 1          ;; load f32
        // 0x00a8 local_load_f64       0 2          ;; load f64
        // 0x00b0 local_load           0 3          ;; load 0xf0e0d0c0_19171311
        // 0x00b8 local_load32         0 4          ;; load 0x19171311
        // 0x00c0 end
        //
        // (f32, f64) -> (i64,i32,i32,i32,i32,i32, f32,f64 ,i64,i32)

        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 0x19171311)
            .write_opcode_i16_i32(Opcode::local_store32, 0, 0)
            .write_opcode_i32(Opcode::i32_imm, 0xd0c0)
            .write_opcode_i16_i32(Opcode::local_store16, 4, 0)
            .write_opcode_i32(Opcode::i32_imm, 0xe0)
            .write_opcode_i16_i32(Opcode::local_store8, 6, 0)
            .write_opcode_i32(Opcode::i32_imm, 0xf0)
            .write_opcode_i16_i32(Opcode::local_store8, 7, 0)
            //
            // here access arguments directly
            // note that the general method is using 'local_load' instruction
            .write_opcode_i16_i32(Opcode::local_store, 0, 2) // store f64
            .write_opcode_i16_i32(Opcode::local_store32, 0, 1) // store f32
            //
            .write_opcode_i16_i32(Opcode::local_load, 0, 0)
            .write_opcode_i16_i32(Opcode::local_store, 0, 3)
            //
            .write_opcode_i16_i32(Opcode::local_load, 0, 0)
            .write_opcode_i16_i32(Opcode::local_store32, 0, 4)
            //
            .write_opcode_i16_i32(Opcode::local_load, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load32, 4, 0)
            .write_opcode_i16_i32(Opcode::local_load32_i16_u, 6, 0)
            .write_opcode_i16_i32(Opcode::local_load32_i16_s, 6, 0)
            .write_opcode_i16_i32(Opcode::local_load32_i8_u, 7, 0)
            .write_opcode_i16_i32(Opcode::local_load32_i8_s, 7, 0)
            //
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 1)
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 2)
            //
            .write_opcode_i16_i32(Opcode::local_load, 0, 3)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 4)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::F32, DataType::F64], // params
            vec![
                DataType::I64,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::F32,
                DataType::F64,
                DataType::I64,
                DataType::I32,
            ], // results
            code0,
            vec![
                VariableItemEntry::from_bytes(8, 8),
                VariableItemEntry::from_f32(),
                VariableItemEntry::from_f64(),
                VariableItemEntry::from_i64(),
                VariableItemEntry::from_i32(),
            ], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        init_runtime();
        let result0 = process_function(
            &mut thread0,
            0,
            0,
            &vec![
                ForeignValue::Float32(3.1415926f32),
                ForeignValue::Float64(2.9979e8f64),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0xf0e0d0c0u32),
                ForeignValue::UInt32(0xf0e0u32),
                ForeignValue::UInt32(0xfffff0e0u32), // extend from i16 to i32
                ForeignValue::UInt32(0xf0u32),
                ForeignValue::UInt32(0xfffffff0u32), // extend from i8 to i32
                //
                ForeignValue::Float32(3.1415926f32),
                ForeignValue::Float64(2.9979e8f64),
                //
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0x19171311u32),
            ]
        );
    }

    #[test]
    fn test_process_local_long_load_store() {
        //       |low address                                 high address|
        //       |                                                        |
        // index |0                                  1                    |
        //  type |bytes-------------------|         |bytes----------------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0         11 13 17 19 c0 d0 e0 f0
        //       |           |        |  |          ^
        //       |store32    |store16 |  |          |
        //        step0       step1   |  |          |
        //                      store8|  |          |
        //       |              step2    |store8    |store64
        //       |                        step3     |
        //       \----->--load64-->-----------------/
        //
        //       11 13 17 19 c0 d0    e0 f0         11 13 17 19 c0 d0 e0 f0
        //       |           |        |  |load8u    |
        //       |           |        |  |load8s    |load64
        //       |           |        |             |load32
        //       |           |        |load16u      |load16u
        //       |           |        |load16s      |load8u
        //       |           |
        //       |load64     |load32
        //
        // () -> (i64,i32,i32,i32,i32,i32,  i64,i32,i32,i32)

        // bytecodes
        //
        // 0x0000 i32_imm              0x19171311
        // 0x0008 i32_imm              0x0
        // 0x0010 local_long_store32   0
        // 0x0018 i32_imm              0xd0c0
        // 0x0020 i32_imm              0x4
        // 0x0028 local_long_store16   0
        // 0x0030 i32_imm              0xe0
        // 0x0038 i32_imm              0x6
        // 0x0040 local_long_store8    0
        // 0x0048 i32_imm              0xf0
        // 0x0050 i32_imm              0x7
        // 0x0058 local_long_store8    0
        // 0x0060 i32_imm              0x0
        // 0x0068 local_long_load      0
        // 0x0070 i32_imm              0x0
        // 0x0078 local_long_store     1
        // 0x0080 i32_imm              0x0
        // 0x0088 local_long_load      0
        // 0x0090 i32_imm              0x4
        // 0x0098 local_long_load32    0
        // 0x00a0 i32_imm              0x6
        // 0x00a8 local_long_load32_i16_u 0
        // 0x00b0 i32_imm              0x6
        // 0x00b8 local_long_load32_i16_s 0
        // 0x00c0 i32_imm              0x7
        // 0x00c8 local_long_load32_i8_u 0
        // 0x00d0 i32_imm              0x7
        // 0x00d8 local_long_load32_i8_s 0
        // 0x00e0 i32_imm              0x0
        // 0x00e8 local_long_load      1
        // 0x00f0 i32_imm              0x0
        // 0x00f8 local_long_load32    1
        // 0x0100 i32_imm              0x0
        // 0x0108 local_long_load32_i16_u 1
        // 0x0110 i32_imm              0x0
        // 0x0118 local_long_load32_i8_u 1
        // 0x0120 end
        //
        // () -> (i64,i32,i32,i32,i32,i32,  i64,i32,i32,i32)

        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 0x19171311)
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::local_long_store32, 0) // store 32
            //
            .write_opcode_i32(Opcode::i32_imm, 0xd0c0)
            .write_opcode_i32(Opcode::i32_imm, 4)
            .write_opcode_i32(Opcode::local_long_store16, 0) // store 16
            //
            .write_opcode_i32(Opcode::i32_imm, 0xe0)
            .write_opcode_i32(Opcode::i32_imm, 6)
            .write_opcode_i32(Opcode::local_long_store8, 0) // store 8
            //
            .write_opcode_i32(Opcode::i32_imm, 0xf0)
            .write_opcode_i32(Opcode::i32_imm, 7)
            .write_opcode_i32(Opcode::local_long_store8, 0) // store 8
            //
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::local_long_load, 0) // load 64
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::local_long_store, 1) // store 64
            //
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::local_long_load, 0)
            .write_opcode_i32(Opcode::i32_imm, 4)
            .write_opcode_i32(Opcode::local_long_load32, 0)
            .write_opcode_i32(Opcode::i32_imm, 6)
            .write_opcode_i32(Opcode::local_long_load32_i16_u, 0)
            .write_opcode_i32(Opcode::i32_imm, 6)
            .write_opcode_i32(Opcode::local_long_load32_i16_s, 0)
            .write_opcode_i32(Opcode::i32_imm, 7)
            .write_opcode_i32(Opcode::local_long_load32_i8_u, 0)
            .write_opcode_i32(Opcode::i32_imm, 7)
            .write_opcode_i32(Opcode::local_long_load32_i8_s, 0)
            //
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::local_long_load, 1)
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::local_long_load32, 1)
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::local_long_load32_i16_u, 1)
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::local_long_load32_i8_u, 1)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![], // params
            vec![
                DataType::I64,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I64,
                DataType::I32,
                DataType::I32,
                DataType::I32,
            ], // results
            code0,
            vec![
                VariableItemEntry::from_bytes(8, 8),
                VariableItemEntry::from_bytes(8, 8),
            ], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        init_runtime();
        let result0 = process_function(&mut thread0, 0, 0, &vec![]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0xf0e0d0c0u32),
                ForeignValue::UInt32(0xf0e0u32),
                ForeignValue::UInt32(0xfffff0e0u32), // extend from i16 to i32
                ForeignValue::UInt32(0xf0u32),
                ForeignValue::UInt32(0xfffffff0u32), // extend from i8 to i32
                //
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0x19171311u32),
                ForeignValue::UInt32(0x00001311u32), // extend from i16 to i32
                ForeignValue::UInt32(0x00000011u32), // extend from i8 to i32
            ]
        );
    }

    #[test]
    fn test_process_data_load_store() {
        //        read-only data section
        //       |low address    high addr|
        //       |                        |
        // index |0           1           |
        //  type |i32------| |i32---------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0
        //       |           |        |  |
        //       |           |        |  |load8u (step 1)
        //       |           |        |load8u (step 2)
        //       |           |load16u (step 3)
        //       |load32 (step 4)
        //
        //        read-write data section
        //       |low address                                                              high address|
        //       |                                                                                     |
        // index |2(0)                               3(1)   4(2)   5(3)                      6(4)      |
        //  type |bytes-------------------|         |f32|  |f64|  |i64------------------|   |i32-------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 12 17 19
        //       |           |        |  |          |      |      ^                          ^
        //       |store32    |store16 |  |          |sf32  |sf64  |                          |
        //                            |  |          |stepN'|stepN |                          |
        //                      store8|  |          |      |      |                          |
        //       |                       |store8    |      |      |store64                   |store32
        //       |                                  |      |      |                          |
        //       \----->--load64-->---------------------------->--/-->-------------------->--/
        //
        //       11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 12 17 19
        //       |           |        |  |load8u    |      |      |                          |
        //       |           |        |  |load8s  loadf32  |      |                          |
        //       |           |        |                  loadf64  |                          |
        //       |           |        |load16u                    |                          |
        //       |           |        |load16s                 load64                      load32
        //       |           |
        //       |load64     |load32
        //
        // (f32, f64) -> (i64,i32,i32,i32,i32,i32, f32,f64 ,i64,i32)

        // bytecodes
        //
        // 0x0000 data_load32_i8_u     3 1
        // 0x0008 data_load32_i8_u     2 1
        // 0x0010 data_load32_i16_u    0 1
        // 0x0018 data_load32          0 0
        //
        // 0x0020 data_store32         0 2          ;; store 0x19171311
        // 0x0028 data_store16         4 2          ;; store 0xd0c0
        // 0x0030 data_store8          6 2          ;; store 0xe0
        // 0x0038 data_store8          7 2          ;; store 0xf0
        //
        // 0x0040 local_load_f64       0 3
        // 0x0048 data_store           0 4          ;; store f64
        // 0x0050 local_load32_f32     0 2
        // 0x0058 data_store32         0 3          ;; store f32
        //
        // 0x0060 data_load            0 2
        // 0x0068 data_store           0 5          ;; store 0xf0e0d0c0_19171311
        // 0x0070 data_load            0 2
        // 0x0078 data_store32         0 6          ;; store 0x19171311
        //
        // 0x0080 data_load            0 2          ;; load 0xf0e0d0c0_19171311
        // 0x0088 data_load32          4 2          ;; load 0xf0e0d0c0
        // 0x0090 data_load32_i16_u    6 2          ;; load 0xf0e0
        // 0x0098 data_load32_i16_s    6 2          ;; load 0xf0e0
        // 0x00a0 data_load32_i8_u     7 2          ;; load 0xf0
        // 0x00a8 data_load32_i8_s     7 2          ;; load 0xf0
        // 0x00b0 data_load32_f32      0 3          ;; load f32
        // 0x00b8 data_load_f64        0 4          ;; load f64
        // 0x00c0 data_load            0 5          ;; load 0xf0e0d0c0_19171311
        // 0x00c8 data_load32          0 6          ;; load 0x19171311
        // 0x00d0 end
        //
        // (f32, f64) -> (i64,i32,i32,i32,i32,i32, f32,f64 ,i64,i32)

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::data_load32_i8_u, 3, 1)
            .write_opcode_i16_i32(Opcode::data_load32_i8_u, 2, 1)
            .write_opcode_i16_i32(Opcode::data_load32_i16_u, 0, 1)
            .write_opcode_i16_i32(Opcode::data_load32, 0, 0)
            //
            .write_opcode_i16_i32(Opcode::data_store32, 0, 2)
            .write_opcode_i16_i32(Opcode::data_store16, 4, 2)
            .write_opcode_i16_i32(Opcode::data_store8, 6, 2)
            .write_opcode_i16_i32(Opcode::data_store8, 7, 2)
            //
            // the arguments f32 and f64 are at the top of stack, they can access directly
            // here test loading the arguments as local variables.
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 3)
            .write_opcode_i16_i32(Opcode::data_store, 0, 4) // store f64
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 2)
            .write_opcode_i16_i32(Opcode::data_store32, 0, 3) // store f32
            //
            .write_opcode_i16_i32(Opcode::data_load, 0, 2)
            .write_opcode_i16_i32(Opcode::data_store, 0, 5)
            //
            .write_opcode_i16_i32(Opcode::data_load, 0, 2)
            .write_opcode_i16_i32(Opcode::data_store32, 0, 6)
            //
            .write_opcode_i16_i32(Opcode::data_load, 0, 2)
            .write_opcode_i16_i32(Opcode::data_load32, 4, 2)
            .write_opcode_i16_i32(Opcode::data_load32_i16_u, 6, 2)
            .write_opcode_i16_i32(Opcode::data_load32_i16_s, 6, 2)
            .write_opcode_i16_i32(Opcode::data_load32_i8_u, 7, 2)
            .write_opcode_i16_i32(Opcode::data_load32_i8_s, 7, 2)
            //
            .write_opcode_i16_i32(Opcode::data_load32_f32, 0, 3)
            .write_opcode_i16_i32(Opcode::data_load_f64, 0, 4)
            //
            .write_opcode_i16_i32(Opcode::data_load, 0, 5)
            .write_opcode_i16_i32(Opcode::data_load32, 0, 6)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function_and_data_sections(
            vec![
                DataEntry::from_i32(0x19171311),
                DataEntry::from_i32(0xf0e0d0c0),
            ],
            vec![
                DataEntry::from_bytes(vec![0u8; 8], 8),
                DataEntry::from_f32(0.0f32),
                DataEntry::from_f64(0.0f64),
                DataEntry::from_i64(0),
                DataEntry::from_i32(0),
            ],
            vec![],
            vec![DataType::F32, DataType::F64], // params
            vec![
                DataType::I64,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::F32,
                DataType::F64,
                DataType::I64,
                DataType::I32,
            ], // results
            code0,
            vec![VariableItemEntry::from_i32(), VariableItemEntry::from_i64()], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        init_runtime();
        let result0 = process_function(
            &mut thread0,
            0,
            0,
            &vec![
                ForeignValue::Float32(3.1415926f32),
                ForeignValue::Float64(2.9979e8f64),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0xf0e0d0c0u32),
                ForeignValue::UInt32(0xf0e0u32),
                ForeignValue::UInt32(0xfffff0e0u32), // extend from i16 to i32
                ForeignValue::UInt32(0xf0u32),
                ForeignValue::UInt32(0xfffffff0u32), // extend from i8 to i32
                //
                ForeignValue::Float32(3.1415926f32),
                ForeignValue::Float64(2.9979e8f64),
                //
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0x19171311u32),
            ]
        );
    }

    #[test]
    fn test_process_data_load_store_uninitialized() {
        //        read-only data section
        //       |low address    high addr|
        //       |                        |
        // index |0           1           |
        //  type |i32------| |i32---------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0
        //       |           |        |  |
        //       |           |        |  |load8u (step 1)
        //       |           |        |load8u (step 2)
        //       |           |load16u (step 3)
        //       |load32 (step 4)
        //
        //        uninitialized data section
        //       |low address                                                              high address|
        //       |                                                                                     |
        // index |2(0)                               3(1)   4(2)   5(3)                      6(4)      |
        //  type |bytes-------------------|         |f32|  |f64|  |i64------------------|   |i32-------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 12 17 19
        //       |           |        |  |          |      |      ^                          ^
        //       |store32    |store16 |  |          |sf32  |sf64  |                          |
        //                            |  |          |stepN'|stepN |                          |
        //                      store8|  |          |      |      |                          |
        //       |                       |store8    |      |      |store64                   |store32
        //       |                                  |      |      |                          |
        //       \----->--load64-->---------------------------->--/-->-------------------->--/
        //
        //       11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 12 17 19
        //       |           |        |  |load8u    |      |      |                          |
        //       |           |        |  |load8s  loadf32  |      |                          |
        //       |           |        |                  loadf64  |                          |
        //       |           |        |load16u                    |                          |
        //       |           |        |load16s                 load64                      load32
        //       |           |
        //       |load64     |load32
        //
        // (f32, f64) -> (i64,i32,i32,i32,i32,i32, f32,f64 ,i64,i32)

        // bytecodes
        //
        // 0x0000 data_load32_i8_u     3 1
        // 0x0008 data_load32_i8_u     2 1
        // 0x0010 data_load32_i16_u    0 1
        // 0x0018 data_load32          0 0
        //
        // 0x0020 data_store32         0 2          ;; store 0x19171311
        // 0x0028 data_store16         4 2          ;; store 0xd0c0
        // 0x0030 data_store8          6 2          ;; store 0xe0
        // 0x0038 data_store8          7 2          ;; store 0xf0
        // 0x0040 data_store           0 4          ;; store f64
        // 0x0048 data_store32         0 3          ;; store f32
        //
        // 0x0050 data_load            0 2
        // 0x0058 data_store           0 5          ;; store 0xf0e0d0c0_19171311
        // 0x0060 data_load            0 2
        // 0x0068 data_store32         0 6          ;; store 0x19171311
        //
        // 0x0070 data_load            0 2          ;; load 0xf0e0d0c0_19171311
        // 0x0078 data_load32          4 2          ;; load 0xf0e0d0c0
        // 0x0080 data_load32_i16_u    6 2          ;; load 0xf0e0
        // 0x0088 data_load32_i16_s    6 2          ;; load 0xf0e0
        // 0x0090 data_load32_i8_u     7 2          ;; load 0xf0
        // 0x0098 data_load32_i8_s     7 2          ;; load 0xf0
        //
        // 0x00a0 data_load32_f32      0 3          ;; load f32
        // 0x00a8 data_load_f64        0 4          ;; load f64
        // 0x00b0 data_load            0 5          ;; load 0xf0e0d0c0_19171311
        // 0x00b8 data_load32          0 6          ;; load 0x19171311
        // 0x00c0 end
        //
        // (f32, f64) -> (i64,i32,i32,i32,i32,i32, f32,f64 ,i64,i32)

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::data_load32_i8_u, 3, 1)
            .write_opcode_i16_i32(Opcode::data_load32_i8_u, 2, 1)
            .write_opcode_i16_i32(Opcode::data_load32_i16_u, 0, 1)
            .write_opcode_i16_i32(Opcode::data_load32, 0, 0)
            //
            .write_opcode_i16_i32(Opcode::data_store32, 0, 2)
            .write_opcode_i16_i32(Opcode::data_store16, 4, 2)
            .write_opcode_i16_i32(Opcode::data_store8, 6, 2)
            .write_opcode_i16_i32(Opcode::data_store8, 7, 2)
            //
            .write_opcode_i16_i32(Opcode::data_store, 0, 4) // store f64
            .write_opcode_i16_i32(Opcode::data_store32, 0, 3) // store f32
            //
            .write_opcode_i16_i32(Opcode::data_load, 0, 2)
            .write_opcode_i16_i32(Opcode::data_store, 0, 5)
            //
            .write_opcode_i16_i32(Opcode::data_load, 0, 2)
            .write_opcode_i16_i32(Opcode::data_store32, 0, 6)
            //
            .write_opcode_i16_i32(Opcode::data_load, 0, 2)
            .write_opcode_i16_i32(Opcode::data_load32, 4, 2)
            .write_opcode_i16_i32(Opcode::data_load32_i16_u, 6, 2)
            .write_opcode_i16_i32(Opcode::data_load32_i16_s, 6, 2)
            .write_opcode_i16_i32(Opcode::data_load32_i8_u, 7, 2)
            .write_opcode_i16_i32(Opcode::data_load32_i8_s, 7, 2)
            //
            .write_opcode_i16_i32(Opcode::data_load32_f32, 0, 3)
            .write_opcode_i16_i32(Opcode::data_load_f64, 0, 4)
            //
            .write_opcode_i16_i32(Opcode::data_load, 0, 5)
            .write_opcode_i16_i32(Opcode::data_load32, 0, 6)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function_and_data_sections(
            vec![
                DataEntry::from_i32(0x19171311),
                DataEntry::from_i32(0xf0e0d0c0),
            ],
            vec![],
            vec![
                UninitDataEntry::from_bytes(8, 8),
                UninitDataEntry::from_f32(),
                UninitDataEntry::from_f64(),
                UninitDataEntry::from_i64(),
                UninitDataEntry::from_i32(),
            ],
            vec![DataType::F32, DataType::F64], // params
            vec![
                DataType::I64,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::F32,
                DataType::F64,
                DataType::I64,
                DataType::I32,
            ], // results
            code0,
            vec![
                VariableItemEntry::from_bytes(8, 8),
                VariableItemEntry::from_f32(),
                VariableItemEntry::from_f64(),
                VariableItemEntry::from_i64(),
                VariableItemEntry::from_i32(),
            ], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        init_runtime();
        let result0 = process_function(
            &mut thread0,
            0,
            0,
            &vec![
                ForeignValue::Float32(3.1415926f32),
                ForeignValue::Float64(2.9979e8f64),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0xf0e0d0c0u32),
                ForeignValue::UInt32(0xf0e0u32),
                ForeignValue::UInt32(0xfffff0e0u32), // extend from i16 to i32
                ForeignValue::UInt32(0xf0u32),
                ForeignValue::UInt32(0xfffffff0u32), // extend from i8 to i32
                //
                ForeignValue::Float32(3.1415926f32),
                ForeignValue::Float64(2.9979e8f64),
                //
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0x19171311u32),
            ]
        );
    }

    #[test]
    fn test_process_data_long_load_store() {
        //       |low address                                 high address|
        //       |                                                        |
        // index |0                                  1                    |
        //  type |bytes-------------------|         |bytes----------------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0         11 13 17 19 c0 d0 e0 f0
        //       |           |        |  |          ^
        //       |store32    |store16 |  |          |
        //        step0       step1   |  |          |
        //                      store8|  |          |
        //       |              step2    |store8    |store64
        //       |                        step3     |
        //       \----->--load64-->-----------------/
        //
        //       11 13 17 19 c0 d0    e0 f0         11 13 17 19 c0 d0 e0 f0
        //       |           |        |  |load8u    |
        //       |           |        |  |load8s    |load64
        //       |           |        |             |load32
        //       |           |        |load16u      |load16u
        //       |           |        |load16s      |load8u
        //       |           |
        //       |load64     |load32
        //
        // () -> (i64,i32,i32,i32,i32,i32,  i64,i32,i32,i32)

        // bytecodes
        //
        // 0x0000 i32_imm              0x19171311
        // 0x0008 i32_imm              0x0
        // 0x0010 data_long_store32    0
        // 0x0018 i32_imm              0xd0c0
        // 0x0020 i32_imm              0x4
        // 0x0028 data_long_store16    0
        // 0x0030 i32_imm              0xe0
        // 0x0038 i32_imm              0x6
        // 0x0040 data_long_store8     0
        // 0x0048 i32_imm              0xf0
        // 0x0050 i32_imm              0x7
        // 0x0058 data_long_store8     0
        // 0x0060 i32_imm              0x0
        // 0x0068 data_long_load       0
        // 0x0070 i32_imm              0x0
        // 0x0078 data_long_store      1
        // 0x0080 i32_imm              0x0
        // 0x0088 data_long_load       0
        // 0x0090 i32_imm              0x4
        // 0x0098 data_long_load32     0
        // 0x00a0 i32_imm              0x6
        // 0x00a8 data_long_load32_i16_u 0
        // 0x00b0 i32_imm              0x6
        // 0x00b8 data_long_load32_i16_s 0
        // 0x00c0 i32_imm              0x7
        // 0x00c8 data_long_load32_i8_u 0
        // 0x00d0 i32_imm              0x7
        // 0x00d8 data_long_load32_i8_s 0
        // 0x00e0 i32_imm              0x0
        // 0x00e8 data_long_load       1
        // 0x00f0 i32_imm              0x0
        // 0x00f8 data_long_load32     1
        // 0x0100 i32_imm              0x0
        // 0x0108 data_long_load32_i16_u 1
        // 0x0110 i32_imm              0x0
        // 0x0118 data_long_load32_i8_u 1
        // 0x0120 end
        //
        // () -> (i64,i32,i32,i32,i32,i32,  i64,i32,i32,i32)

        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 0x19171311)
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::data_long_store32, 0) // store 32
            //
            .write_opcode_i32(Opcode::i32_imm, 0xd0c0)
            .write_opcode_i32(Opcode::i32_imm, 4)
            .write_opcode_i32(Opcode::data_long_store16, 0) // store 16
            //
            .write_opcode_i32(Opcode::i32_imm, 0xe0)
            .write_opcode_i32(Opcode::i32_imm, 6)
            .write_opcode_i32(Opcode::data_long_store8, 0) // store 8
            //
            .write_opcode_i32(Opcode::i32_imm, 0xf0)
            .write_opcode_i32(Opcode::i32_imm, 7)
            .write_opcode_i32(Opcode::data_long_store8, 0) // store 8
            //
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::data_long_load, 0) // load 64
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::data_long_store, 1) // store 64
            //
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::data_long_load, 0)
            .write_opcode_i32(Opcode::i32_imm, 4)
            .write_opcode_i32(Opcode::data_long_load32, 0)
            .write_opcode_i32(Opcode::i32_imm, 6)
            .write_opcode_i32(Opcode::data_long_load32_i16_u, 0)
            .write_opcode_i32(Opcode::i32_imm, 6)
            .write_opcode_i32(Opcode::data_long_load32_i16_s, 0)
            .write_opcode_i32(Opcode::i32_imm, 7)
            .write_opcode_i32(Opcode::data_long_load32_i8_u, 0)
            .write_opcode_i32(Opcode::i32_imm, 7)
            .write_opcode_i32(Opcode::data_long_load32_i8_s, 0)
            //
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::data_long_load, 1)
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::data_long_load32, 1)
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::data_long_load32_i16_u, 1)
            .write_opcode_i32(Opcode::i32_imm, 0)
            .write_opcode_i32(Opcode::data_long_load32_i8_u, 1)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function_and_data_sections(
            vec![],
            vec![],
            vec![
                UninitDataEntry::from_bytes(8, 8),
                UninitDataEntry::from_bytes(8, 8),
            ],
            vec![], // params
            vec![
                DataType::I64,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I64,
                DataType::I32,
                DataType::I32,
                DataType::I32,
            ], // results
            code0,
            vec![], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        init_runtime();
        let result0 = process_function(&mut thread0, 0, 0, &vec![]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0xf0e0d0c0u32),
                ForeignValue::UInt32(0xf0e0u32),
                ForeignValue::UInt32(0xfffff0e0u32), // extend from i16 to i32
                ForeignValue::UInt32(0xf0u32),
                ForeignValue::UInt32(0xfffffff0u32), // extend from i8 to i32
                //
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0x19171311u32),
                ForeignValue::UInt32(0x00001311u32), // extend from i16 to i32
                ForeignValue::UInt32(0x00000011u32), // extend from i8 to i32
            ]
        );
    }

    #[test]
    fn test_process_heap_load_store() {
        //       |low address                                                              high address|
        //       |                                                                                     |
        // index |0x100                              0x200  0x300  0x400                     0x500     |
        //  type |bytes-------------------|         |f32|  |f64|  |i64------------------|   |i32-------|
        //
        //  data 11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 12 17 19
        //       |           |        |  |          |      |      ^                          ^
        //       |store32    |store16 |  |          |sf32  |sf64  |                          |
        //        step0       step1   |  |          |step5 |step4 |                          |
        //                      store8|  |          |      |      |                          |
        //       |              step2    |store8    |      |      |store64                   |store32
        //       |                        step3     |      |      |                          |
        //       \----->--load64-->---------------------------->--/-->-------------------->--/
        //
        //       11 13 17 19 c0 d0    e0 f0         f32    f64    11 13 17 19 c0 d0 e0 f0    11 12 17 19
        //       |           |        |  |load8u    |      |      |                          |
        //       |           |        |  |load8s  loadf32  |      |                          |
        //       |           |        |                  loadf64  |                          |
        //       |           |        |load16u                    |                          |
        //       |           |        |load16s                 load64                      load32
        //       |           |
        //       |load64     |load32
        //
        // (f32, f64) -> (i64,i32,i32,i32,i32,i32, f32,f64 ,i64,i32)

        // bytecodes
        //
        // 0x0000 i32_imm              0x1
        // 0x0008 ecall                262
        // 0x0010 drop
        // 0x0012 nop
        // 0x0014 i32_imm              0x19171311
        // 0x001c i64_imm              0x100 0x0
        // 0x0028 heap_store32         0
        // 0x002c i32_imm              0xd0c0
        // 0x0034 i64_imm              0x100 0x0
        // 0x0040 heap_store16         4
        // 0x0044 i32_imm              0xe0
        // 0x004c i64_imm              0x100 0x0
        // 0x0058 heap_store32         6
        // 0x005c i32_imm              0xf0
        // 0x0064 i64_imm              0x100 0x0
        // 0x0070 heap_store32         7
        // 0x0074 i64_imm              0x300 0x0
        // 0x0080 heap_store           0
        // 0x0084 i64_imm              0x200 0x0
        // 0x0090 heap_store32         0
        // 0x0094 i64_imm              0x100 0x0
        // 0x00a0 heap_load            0
        // 0x00a4 i64_imm              0x400 0x0
        // 0x00b0 heap_store           0
        // 0x00b4 i64_imm              0x100 0x0
        // 0x00c0 heap_load            0
        // 0x00c4 i64_imm              0x500 0x0
        // 0x00d0 heap_store32         0
        // 0x00d4 i64_imm              0x100 0x0
        // 0x00e0 heap_load            0
        // 0x00e4 i64_imm              0x100 0x0
        // 0x00f0 heap_load32          4
        // 0x00f4 i64_imm              0x100 0x0
        // 0x0100 heap_load32_i16_u    6
        // 0x0104 i64_imm              0x100 0x0
        // 0x0110 heap_load32_i16_s    6
        // 0x0114 i64_imm              0x100 0x0
        // 0x0120 heap_load32_i8_u     7
        // 0x0124 i64_imm              0x100 0x0
        // 0x0130 heap_load32_i8_s     7
        // 0x0134 i64_imm              0x200 0x0
        // 0x0140 heap_load32_f32      0
        // 0x0144 i64_imm              0x300 0x0
        // 0x0150 heap_load_f64        0
        // 0x0154 i64_imm              0x400 0x0
        // 0x0160 heap_load            0
        // 0x0164 i64_imm              0x500 0x0
        // 0x0170 heap_load32          0
        // 0x0174 end
        //
        // (f32, f64) -> (i64,i32,i32,i32,i32,i32, f32,f64 ,i64,i32)

        let code0 = BytecodeWriter::new()
            // note that the init size of heap is 0
            // change the capacity of heap before test
            .write_opcode_i32(Opcode::i32_imm, 1)
            .write_opcode_i32(Opcode::ecall, ECallCode::heap_resize as u32)
            .write_opcode(Opcode::drop)
            //
            .write_opcode_i32(Opcode::i32_imm, 0x19171311)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_store32, 0)
            //
            .write_opcode_i32(Opcode::i32_imm, 0xd0c0)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_store16, 4)
            //
            .write_opcode_i32(Opcode::i32_imm, 0xe0)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_store32, 6)
            //
            .write_opcode_i32(Opcode::i32_imm, 0xf0)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_store32, 7)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x300)
            .write_opcode_i16(Opcode::heap_store, 0) // store f64
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x200)
            .write_opcode_i16(Opcode::heap_store32, 0) // store f32
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_load, 0)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x400)
            .write_opcode_i16(Opcode::heap_store, 0)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_load, 0)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x500)
            .write_opcode_i16(Opcode::heap_store32, 0)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_load, 0)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_load32, 4)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_load32_i16_u, 6)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_load32_i16_s, 6)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_load32_i8_u, 7)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x100)
            .write_opcode_i16(Opcode::heap_load32_i8_s, 7)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x200)
            .write_opcode_i16(Opcode::heap_load32_f32, 0)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x300)
            .write_opcode_i16(Opcode::heap_load_f64, 0)
            //
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x400)
            .write_opcode_i16(Opcode::heap_load, 0)
            .write_opcode_pesudo_i64(Opcode::i64_imm, 0x500)
            .write_opcode_i16(Opcode::heap_load32, 0)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::F32, DataType::F64], // params
            vec![
                DataType::I64,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::F32,
                DataType::F64,
                DataType::I64,
                DataType::I32,
            ], // results
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
            &vec![
                // https://baseconvert.com/ieee-754-floating-point
                // https://www.binaryconvert.com/convert_float.html
                ForeignValue::Float32(3.1415926f32),
                // 0x40490FDA
                // 218,15,73,64
                ForeignValue::Float64(2.9979e8f64),
                // 0x41B1DE6EB0000000
                // 0,0,0,176,110,222,177,65
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0xf0e0d0c0u32),
                ForeignValue::UInt32(0xf0e0u32),
                ForeignValue::UInt32(0xfffff0e0u32), // extend from i16 to i32
                ForeignValue::UInt32(0xf0u32),
                ForeignValue::UInt32(0xfffffff0u32), // extend from i8 to i32
                //
                ForeignValue::Float32(3.1415926f32),
                ForeignValue::Float64(2.9979e8f64),
                //
                ForeignValue::UInt64(0xf0e0d0c0_19171311u64),
                ForeignValue::UInt32(0x19171311u32),
            ]
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
                VariableItemEntry::from_bytes(64, 8), // space
                VariableItemEntry::from_i32(),
                VariableItemEntry::from_i32(),
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
                VariableItemEntry::from_bytes(64, 8), // space
                VariableItemEntry::from_bytes(8, 8),
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

    #[test]
    fn test_process_conversion_demote_and_promote() {
        // bytecodes
        //
        // 0x0000 local_load32         0 1
        // 0x0008 i64_promote_i32_s
        // 0x000a nop
        // 0x000c local_load32         0 1
        // 0x0014 i64_promote_i32_u
        // 0x0016 nop
        // 0x0018 local_load           0 0
        // 0x0020 i32_demote_i64
        // 0x0022 end
        //
        // (i64, i32)  ->  (i64, i64, i32)
        //  |    |          ^    ^    ^
        //  |    | promote  |0   |1   |2
        //  |    \----------/----/    |
        //  \-------------------------/ demote

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::i64_promote_i32_s)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::i64_promote_i32_u)
            .write_opcode_i16_i32(Opcode::local_load, 0, 0)
            .write_opcode(Opcode::i32_demote_i64)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I64, DataType::I32],                // params
            vec![DataType::I64, DataType::I64, DataType::I32], // results
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
            &vec![
                ForeignValue::UInt64(0x19171311_07050302u64),
                ForeignValue::UInt32(0x80706050u32),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt64(0xffffffff_80706050u64),
                ForeignValue::UInt64(0x00000000_80706050u64),
                ForeignValue::UInt32(0x07050302u32),
            ]
        );

        // bytecodes
        //
        // 0x0000 local_load32_f32     0 1
        // 0x0008 f64_promote_f32
        // 0x000a nop
        // 0x000c local_load_f64       0 0
        // 0x0014 f32_demote_f64
        // 0x0016 end
        //
        // (f64, f32)  ->  (f64, f32)
        //  |    |          ^    ^
        //  |    | promote  |0   |2
        //  |    \----------/    |
        //  \--------------------/ demote

        let code1 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 1)
            .write_opcode(Opcode::f64_promote_f32)
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 0)
            .write_opcode(Opcode::f32_demote_f64)
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code1).to_text());

        let binary1 = build_module_binary_with_single_function(
            vec![DataType::F64, DataType::F32], // params
            vec![DataType::F64, DataType::F32], // results
            code1,
            vec![], // local vars
        );

        let image1 = load_modules_binary(vec![&binary1]).unwrap();
        let mut thread1 = Thread::new(&image1);

        let result1 = process_function(
            &mut thread1,
            0,
            0,
            &vec![
                ForeignValue::Float64(3.1415926535897931159979634685f64),
                // 0x400921FB54442D18 -> 0x40490FDB (3.1415927410125732421875)
                ForeignValue::Float32(2.71828f32),
                // 0x402DF84D -> 0x4005BF0995AAF790 (2.71828000000000002955857780762016773223876953125)
            ],
        );

        let exp0 = 2.71828f32 as f64;
        let exp1 = 3.1415926535897931159979634685f64 as f32;

        assert_eq!(
            result1.unwrap(),
            vec![ForeignValue::Float64(exp0), ForeignValue::Float32(exp1),]
        );
    }

    #[test]
    fn test_process_conversion_float_and_int() {
        // bytecodes
        //
        // 0x0000 local_load32_f32     0 0
        // 0x0008 i32_trunc_f32_s
        // 0x000a nop
        // 0x000c local_load32_f32     0 0
        // 0x0014 i32_trunc_f32_u
        // 0x0016 nop
        // 0x0018 local_load32_f32     0 0
        // 0x0020 i64_trunc_f32_s
        // 0x0022 nop
        // 0x0024 local_load32_f32     0 0
        // 0x002c i64_trunc_f32_u
        // 0x002e nop
        // 0x0030 local_load_f64       0 1
        // 0x0038 i32_trunc_f64_s
        // 0x003a nop
        // 0x003c local_load_f64       0 1
        // 0x0044 i32_trunc_f64_u
        // 0x0046 nop
        // 0x0048 local_load_f64       0 1
        // 0x0050 i64_trunc_f64_s
        // 0x0052 nop
        // 0x0054 local_load_f64       0 1
        // 0x005c i64_trunc_f64_u
        // 0x005e nop
        // 0x0060 local_load32_f32     0 2
        // 0x0068 i32_trunc_f32_s
        // 0x006a nop
        // 0x006c local_load32_f32     0 2
        // 0x0074 i32_trunc_f32_u
        // 0x0076 nop
        // 0x0078 local_load32_f32     0 2
        // 0x0080 i64_trunc_f32_s
        // 0x0082 nop
        // 0x0084 local_load32_f32     0 2
        // 0x008c i64_trunc_f32_u
        // 0x008e nop
        // 0x0090 local_load_f64       0 3
        // 0x0098 i32_trunc_f64_s
        // 0x009a nop
        // 0x009c local_load_f64       0 3
        // 0x00a4 i32_trunc_f64_u
        // 0x00a6 nop
        // 0x00a8 local_load_f64       0 3
        // 0x00b0 i64_trunc_f64_s
        // 0x00b2 nop
        // 0x00b4 local_load_f64       0 3
        // 0x00bc i64_trunc_f64_u
        // 0x00be end
        //
        // (f32,              f64,            -f32,             -f64)
        //  |                 |                |                 |
        //  |                 |                |                 |
        //  |                 |                |                 |
        //  |---\---\---\     |---\---\---\    |---\---\---\     |---\---\---\
        //  |   |   |   |     |   |   |   |    |   |   |   |     |   |   |   |
        //  v   v   v   v     v   v   v   v    v   v   v   v     v   v   v   v
        // (i32 i32 i64 i64   i32 i32 i64 i64  i32 i32 i64 i64   i32 i32 i64 i64)

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 0)
            .write_opcode(Opcode::i32_trunc_f32_s)
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 0)
            .write_opcode(Opcode::i32_trunc_f32_u)
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 0)
            .write_opcode(Opcode::i64_trunc_f32_s)
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 0)
            .write_opcode(Opcode::i64_trunc_f32_u)
            //
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 1)
            .write_opcode(Opcode::i32_trunc_f64_s)
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 1)
            .write_opcode(Opcode::i32_trunc_f64_u)
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 1)
            .write_opcode(Opcode::i64_trunc_f64_s)
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 1)
            .write_opcode(Opcode::i64_trunc_f64_u)
            //
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 2)
            .write_opcode(Opcode::i32_trunc_f32_s)
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 2)
            .write_opcode(Opcode::i32_trunc_f32_u)
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 2)
            .write_opcode(Opcode::i64_trunc_f32_s)
            .write_opcode_i16_i32(Opcode::local_load32_f32, 0, 2)
            .write_opcode(Opcode::i64_trunc_f32_u)
            //
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 3)
            .write_opcode(Opcode::i32_trunc_f64_s)
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 3)
            .write_opcode(Opcode::i32_trunc_f64_u)
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 3)
            .write_opcode(Opcode::i64_trunc_f64_s)
            .write_opcode_i16_i32(Opcode::local_load_f64, 0, 3)
            .write_opcode(Opcode::i64_trunc_f64_u)
            //
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::F32, DataType::F64, DataType::F32, DataType::F64], // params
            vec![
                DataType::I32,
                DataType::I32,
                DataType::I64,
                DataType::I64,
                //
                DataType::I32,
                DataType::I32,
                DataType::I64,
                DataType::I64,
                //
                DataType::I32,
                DataType::I32,
                DataType::I64,
                DataType::I64,
                //
                DataType::I32,
                DataType::I32,
                DataType::I64,
                DataType::I64,
                //
            ], // results
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
            &vec![
                ForeignValue::Float32(2.236),
                ForeignValue::Float64(3.162),
                ForeignValue::Float32(-5.099),
                ForeignValue::Float64(-7.071),
            ],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(2),
                ForeignValue::UInt32(2),
                ForeignValue::UInt64(2),
                ForeignValue::UInt64(2),
                //
                ForeignValue::UInt32(3),
                ForeignValue::UInt32(3),
                ForeignValue::UInt64(3),
                ForeignValue::UInt64(3),
                //
                ForeignValue::UInt32((0 - 5) as u32),
                ForeignValue::UInt32(0),
                ForeignValue::UInt64((0 - 5) as u64),
                ForeignValue::UInt64(0),
                //
                ForeignValue::UInt32((0 - 7) as u32),
                ForeignValue::UInt32(0),
                ForeignValue::UInt64((0 - 7) as u64),
                ForeignValue::UInt64(0),
                //
            ]
        );

        // bytecodes
        //
        // 0x0000 local_load32         0 0
        // 0x0008 f32_convert_i32_s
        // 0x000a nop
        // 0x000c local_load32         0 0
        // 0x0014 f32_convert_i32_u
        // 0x0016 nop
        // 0x0018 local_load32         0 0
        // 0x0020 f64_convert_i32_s
        // 0x0022 nop
        // 0x0024 local_load32         0 0
        // 0x002c f64_convert_i32_u
        // 0x002e nop
        // 0x0030 local_load           0 1
        // 0x0038 f32_convert_i64_s
        // 0x003a nop
        // 0x003c local_load           0 1
        // 0x0044 f32_convert_i64_u
        // 0x0046 nop
        // 0x0048 local_load           0 1
        // 0x0050 f64_convert_i64_s
        // 0x0052 nop
        // 0x0054 local_load           0 1
        // 0x005c f64_convert_i64_u
        // 0x005e nop
        // 0x0060 local_load32         0 2
        // 0x0068 f32_convert_i32_s
        // 0x006a nop
        // 0x006c local_load32         0 2
        // 0x0074 f32_convert_i32_u
        // 0x0076 nop
        // 0x0078 local_load32         0 2
        // 0x0080 f64_convert_i32_s
        // 0x0082 nop
        // 0x0084 local_load32         0 2
        // 0x008c f64_convert_i32_u
        // 0x008e nop
        // 0x0090 local_load           0 3
        // 0x0098 f32_convert_i64_s
        // 0x009a nop
        // 0x009c local_load           0 3
        // 0x00a4 f32_convert_i64_u
        // 0x00a6 nop
        // 0x00a8 local_load           0 3
        // 0x00b0 f64_convert_i64_s
        // 0x00b2 nop
        // 0x00b4 local_load           0 3
        // 0x00bc f64_convert_i64_u
        // 0x00be end
        //
        // (i32,              i64,            -i32,             -i64)
        //  |                 |                |                 |
        //  |                 |                |                 |
        //  |                 |                |                 |
        //  |---\---\---\     |---\---\---\    |---\---\---\     |---\---\---\
        //  |   |   |   |     |   |   |   |    |   |   |   |     |   |   |   |
        //  v   v   v   v     v   v   v   v    v   v   v   v     v   v   v   v
        // (f32 f32 f64 f64   f32 f32 f64 f64  f32 f32 f64 f64   f32 f32 f64 f64)

        let code1 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode(Opcode::f32_convert_i32_s)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode(Opcode::f32_convert_i32_u)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode(Opcode::f64_convert_i32_s)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode(Opcode::f64_convert_i32_u)
            //
            .write_opcode_i16_i32(Opcode::local_load, 0, 1)
            .write_opcode(Opcode::f32_convert_i64_s)
            .write_opcode_i16_i32(Opcode::local_load, 0, 1)
            .write_opcode(Opcode::f32_convert_i64_u)
            .write_opcode_i16_i32(Opcode::local_load, 0, 1)
            .write_opcode(Opcode::f64_convert_i64_s)
            .write_opcode_i16_i32(Opcode::local_load, 0, 1)
            .write_opcode(Opcode::f64_convert_i64_u)
            //
            .write_opcode_i16_i32(Opcode::local_load32, 0, 2)
            .write_opcode(Opcode::f32_convert_i32_s)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 2)
            .write_opcode(Opcode::f32_convert_i32_u)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 2)
            .write_opcode(Opcode::f64_convert_i32_s)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 2)
            .write_opcode(Opcode::f64_convert_i32_u)
            //
            .write_opcode_i16_i32(Opcode::local_load, 0, 3)
            .write_opcode(Opcode::f32_convert_i64_s)
            .write_opcode_i16_i32(Opcode::local_load, 0, 3)
            .write_opcode(Opcode::f32_convert_i64_u)
            .write_opcode_i16_i32(Opcode::local_load, 0, 3)
            .write_opcode(Opcode::f64_convert_i64_s)
            .write_opcode_i16_i32(Opcode::local_load, 0, 3)
            .write_opcode(Opcode::f64_convert_i64_u)
            //
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary1 = build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I64, DataType::I32, DataType::I64], // params
            vec![
                DataType::F32,
                DataType::F32,
                DataType::F64,
                DataType::F64,
                //
                DataType::F32,
                DataType::F32,
                DataType::F64,
                DataType::F64,
                //
                DataType::F32,
                DataType::F32,
                DataType::F64,
                DataType::F64,
                //
                DataType::F32,
                DataType::F32,
                DataType::F64,
                DataType::F64,
            ], // results
            code1,
            vec![], // local vars
        );

        let image1 = load_modules_binary(vec![&binary1]).unwrap();
        let mut thread1 = Thread::new(&image1);

        let result1 = process_function(
            &mut thread1,
            0,
            0,
            &vec![
                ForeignValue::UInt32(11),
                ForeignValue::UInt64(13),
                ForeignValue::UInt32((0 - 17) as u32),
                ForeignValue::UInt64((0 - 19) as u64),
            ],
        );

        println!(
            "{:x} {:x} {:x} {:x}",
            17,
            (0 - 17) as u32,
            19,
            (0 - 19) as u64
        );

        // -11 -> 0xffffffef (u32)
        // -19 -> 0xffffffffffffffed (u64)
        let exp0 = (0 - 17) as u32 as f32;
        let exp1 = (0 - 17) as u32 as f64;
        let exp2 = (0 - 19) as u64 as f32;
        let exp3 = (0 - 19) as u64 as f64;

        assert_eq!(
            result1.unwrap(),
            vec![
                ForeignValue::Float32(11.0),
                ForeignValue::Float32(11.0),
                ForeignValue::Float64(11.0),
                ForeignValue::Float64(11.0),
                //
                ForeignValue::Float32(13.0),
                ForeignValue::Float32(13.0),
                ForeignValue::Float64(13.0),
                ForeignValue::Float64(13.0),
                //
                ForeignValue::Float32(-17.0),
                ForeignValue::Float32(exp0),
                ForeignValue::Float64(-17.0),
                ForeignValue::Float64(exp1),
                //
                ForeignValue::Float32(-19.0),
                ForeignValue::Float32(exp2),
                ForeignValue::Float64(-19.0),
                ForeignValue::Float64(exp3),
                //
            ]
        );
    }
}
