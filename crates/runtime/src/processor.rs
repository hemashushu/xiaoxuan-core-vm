// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// processor

use ancvm_types::{
    opcode::{Opcode, MAX_OPCODE_NUMBER},
    ForeignValue,
};

use crate::{thread::Thread, VMError};

type InterpretFunc = fn(&mut Thread) -> InterpretResult;

mod control_flow;
mod immediate;
mod local;
mod operand;

pub struct Processor {
    pub interpreters: Vec<InterpretFunc>,
}

pub enum InterpretResult {
    MoveOn(usize),      // PARAM (increment_in_bytes: usize)
    Jump(usize, usize), // PARAM (module_index: usize, instruction_address: usize)
    End,
}

fn unreachable(_: &mut Thread) -> InterpretResult {
    unreachable!("Invalid instruction.")
}

impl Processor {
    pub fn new() -> Processor {
        let mut interpreters: Vec<InterpretFunc> = vec![unreachable; MAX_OPCODE_NUMBER];

        // operand
        interpreters[Opcode::nop as usize] = operand::nop;
        interpreters[Opcode::drop as usize] = operand::drop;
        interpreters[Opcode::duplicate as usize] = operand::duplicate;

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

        // control flow
        interpreters[Opcode::end as usize] = control_flow::end;

        Self { interpreters }
    }

    pub fn process_next_instruction(&self, thread: &mut Thread) -> InterpretResult {
        let opcode = thread.get_opcode();
        self.interpreters[opcode as usize](thread)
    }

    pub fn process_continuous_instructions(&self, thread: &mut Thread) {
        loop {
            let result = self.process_next_instruction(thread);
            match result {
                InterpretResult::MoveOn(increment) => {
                    thread.pc.instruction_address += increment;
                }
                InterpretResult::Jump(module_index, instruction_address) => {
                    thread.pc.module_index = module_index;
                    thread.pc.instruction_address = instruction_address;
                }
                InterpretResult::End => break,
            }
        }
    }

    pub fn process_function(
        &self,
        thread: &mut Thread,
        module_index: u32,
        func_index: u32, // this index includes the imported functions
        arguments: &[ForeignValue],
    ) -> Result<Vec<ForeignValue>, VMError> {
        // find the code start address

        let (target_module_index, target_internal_function_index) =
            thread.get_target_function_module_index_and_internal_index(module_index, func_index);
        let (type_index, code_offset, local_variables_allocate_bytes) = thread
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
        thread.pc.instruction_address = code_offset as usize;

        self.process_continuous_instructions(thread);

        // pop results off the stack
        let results = thread.pop_values(&type_entry.results);

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        load_modules_binary, module_image::local_variable_section::VariableItemEntry,
    };
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    use crate::{
        thread::Thread,
        utils::{test_helper::build_module_binary_with_single_function, BytecodeWriter},
    };

    use super::Processor;

    #[test]
    fn test_process_operand() {
        let processor = Processor::new();

        // bytecodes
        //
        // 0d0000 nop
        // 0d0002 end
        //
        // (i32, i32) -> (i32, i32)

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32], // params
            vec![DataType::I32, DataType::I32], // results
            BytecodeWriter::new()
                .write_opcode(Opcode::nop)
                .write_opcode(Opcode::end)
                .to_bytes(),
            vec![], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = processor.process_function(
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
        // 0d0000 drop
        // 0d0002 end
        //
        // (i32, i32) -> (i32)

        let binary1 = build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32], // params
            vec![DataType::I32],                // results
            BytecodeWriter::new()
                .write_opcode(Opcode::drop)
                .write_opcode(Opcode::end)
                .to_bytes(),
            vec![], // local vars
        );

        let image1 = load_modules_binary(vec![&binary1]).unwrap();
        let mut thread1 = Thread::new(&image1);

        let result1 = processor.process_function(
            &mut thread1,
            0,
            0,
            &vec![ForeignValue::UInt32(13), ForeignValue::UInt32(17)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(13)]);

        // bytecodes
        //
        // 0d0000 duplicate
        // 0d0002 end
        //
        // (i32) -> (i32, i32)

        let binary2 = build_module_binary_with_single_function(
            vec![DataType::I32],                // params
            vec![DataType::I32, DataType::I32], // results
            BytecodeWriter::new()
                .write_opcode(Opcode::duplicate)
                .write_opcode(Opcode::end)
                .to_bytes(),
            vec![], // local vars
        );

        let image2 = load_modules_binary(vec![&binary2]).unwrap();
        let mut thread2 = Thread::new(&image2);

        let result2 =
            processor.process_function(&mut thread2, 0, 0, &vec![ForeignValue::UInt32(19)]);
        assert_eq!(
            result2.unwrap(),
            vec![ForeignValue::UInt32(19), ForeignValue::UInt32(19)]
        );
    }

    #[test]
    fn test_process_immediate() {
        let processor = Processor::new();

        // bytecodes
        //
        // 0d0000 i32_imm 23
        // 0d0008 i64_imm 0x29313741_43475359
        // 0d0020 i32_imm -223
        // 0d0028 i64_imm -227
        // 0d0040 end
        //
        // () -> (i32, i64, i32, i64)

        let binary0 = build_module_binary_with_single_function(
            vec![],                                                           // params
            vec![DataType::I32, DataType::I64, DataType::I32, DataType::I64], // results
            BytecodeWriter::new()
                .write_opcode_i32(Opcode::i32_imm, 23)
                .write_opcode_pesudo_i64(Opcode::i64_imm, 0x29313741_43475359u64)
                .write_opcode_i32(Opcode::i32_imm, (0i32 - 223) as u32)
                .write_opcode_pesudo_i64(Opcode::i64_imm, (0i64 - 227) as u64)
                .write_opcode(Opcode::end)
                .to_bytes(),
            vec![], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = processor.process_function(&mut thread0, 0, 0, &vec![]);
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
        // 0d0000 f32_imm 3.1415926
        // 0d0008 f64_imm 6.626e-34
        // 0d0020 f32_imm -2.71828
        // 0d0028 f64_imm -2.9979e8
        // 0d0040 end
        //
        // () -> (f32, f64, f32, f64)

        let binary1 = build_module_binary_with_single_function(
            vec![],                                                           // params
            vec![DataType::F32, DataType::F64, DataType::F32, DataType::F64], // results
            BytecodeWriter::new()
                .write_opcode_pesudo_f32(Opcode::f32_imm, 3.1415926f32)
                .write_opcode_pesudo_f64(Opcode::f64_imm, 6.626e-34f64)
                .write_opcode_pesudo_f32(Opcode::f32_imm, -2.71828f32)
                .write_opcode_pesudo_f64(Opcode::f64_imm, -2.9979e8f64)
                .write_opcode(Opcode::end)
                .to_bytes(),
            vec![], // local vars
        );

        let image1 = load_modules_binary(vec![&binary1]).unwrap();
        let mut thread1 = Thread::new(&image1);

        let result1 = processor.process_function(&mut thread1, 0, 0, &vec![]);
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
        //       |store32    |        |  |          |sf32  |sf64  |                          |
        //                   |store16 |  |store8    |      |      |                          |
        //                            |             |      |      |                          |
        //       |                    |store8       |      |      |store64                   |store32
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

        let processor = Processor::new();

        // bytecodes
        //
        // 0d0000 i32_imm 0x19171311
        // 0d0008 local_store32       0 0     ;; store 0x19171311
        // 0d0016 i32_imm 0xd0c0
        // 0d0024 local_store16       4 0     ;; store 0xd0c0
        // 0d0032 i32_imm 0xe0
        // 0d0040 local_store8        6 0     ;; store 0xe0
        // 0d0048 i32_imm 0xf0
        // 0d0056 local_store8        7 0     ;; store 0xf0
        //
        // 0d0064 local_store         0 2     ;; store f64
        // 0d0072 local_store32       0 1     ;; store f32
        //
        // 0d0080 local_load          0 0
        // 0d0088 local_store         0 3     ;; store 0xf0e0d0c0_19171311
        //
        // 0d0096 local_load          0 0
        // 0d0104 local_store32       0 4     ;; store 0x19171311
        //
        // 0d0112 local_load          0 0     ;; load 0xf0e0d0c0_19171311
        // 0d0120 local_load32        4 0     ;; load 0xf0e0d0c0
        // 0d0128 local_load32_i16_u  6 0     ;; load 0xf0e0
        // 0d0136 local_load32_i16_s  6 0     ;; load 0xf0e0
        // 0d0144 local_load32_i8_u   7 0     ;; load 0xf0
        // 0d0152 local_load32_i8_s   7 0     ;; load 0xf0
        //
        // 0d0160 local_load32_f32    0 1     ;; load f32
        // 0d0168 local_load_f64      0 2     ;; load f64
        //
        // 0d0176 local_load          0 3     ;; load 0xf0e0d0c0_19171311
        // 0d0184 local_load32        0 4     ;; load 0x19171311
        //
        // (f32, f64) -> (i64,i32,i32,i32,i32,i32, f32,f64 ,i64,i32)

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
            BytecodeWriter::new()
                .write_opcode_i32(Opcode::i32_imm, 0x19171311)
                .write_opcode_i16_i32(Opcode::local_store32, 0, 0)
                .write_opcode_i32(Opcode::i32_imm, 0xd0c0)
                .write_opcode_i16_i32(Opcode::local_store16, 4, 0)
                .write_opcode_i32(Opcode::i32_imm, 0xe0)
                .write_opcode_i16_i32(Opcode::local_store8, 6, 0)
                .write_opcode_i32(Opcode::i32_imm, 0xf0)
                .write_opcode_i16_i32(Opcode::local_store8, 7, 0)
                //
                .write_opcode_i16_i32(Opcode::local_store, 0, 2)
                .write_opcode_i16_i32(Opcode::local_store32, 0, 1)
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
                .to_bytes(),
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

        let result0 = processor.process_function(
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
}
