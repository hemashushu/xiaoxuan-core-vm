// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// processor

use ancvm_types::{opcode::Opcode, ForeignValue};

use crate::{thread::Thread, VMError};

type InterpretFunc = fn(&mut Thread) -> InterpretResult;

mod control_flow;
mod immediate;
mod operand;

pub struct Processor {
    pub interpreters: Vec<InterpretFunc>,
}

pub enum InterpretResult {
    MoveOn(usize),      // (increment_in_bytes: usize)
    Jump(usize, usize), // (module_index: usize, instruction_address: usize)
    End,
}

fn unreachable(_: &mut Thread) -> InterpretResult {
    unreachable!("Invalid instruction.")
}

impl Processor {
    pub fn new() -> Processor {
        let mut interpreters: Vec<InterpretFunc> = vec![unreachable; u16::MAX as usize];

        // operand
        interpreters[Opcode::nop as usize] = operand::nop;
        interpreters[Opcode::drop as usize] = operand::drop;
        interpreters[Opcode::duplicate as usize] = operand::duplicate;

        // immediate
        interpreters[Opcode::i32_imm as usize] = immediate::i32_imm;
        interpreters[Opcode::i64_imm as usize] = immediate::i64_imm;
        interpreters[Opcode::f32_imm as usize] = immediate::f32_imm;
        interpreters[Opcode::f64_imm as usize] = immediate::f64_imm;

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
                    thread.pc.addr += increment;
                }
                InterpretResult::Jump(module_index, instruction_address) => {
                    thread.pc.module_index = module_index;
                    thread.pc.addr = instruction_address;
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
        thread.stack.create_func_frame(
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
        thread.pc.addr = code_offset as usize;

        self.process_continuous_instructions(thread);

        // pop up results
        let results = thread.pop_values(&type_entry.results);

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use ancvm_binary::load_modules_binary;
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    use crate::{
        thread::Thread,
        utils::{test_helper::build_module_binary_with_single_function, BytecodeWriter},
    };

    use super::Processor;

    #[test]
    fn test_processor_operand() {
        let processor = Processor::new();

        // bytecodes
        //
        // 0d0000 nop
        // 0d0002 end
        //
        // (i32) -> (i32)

        let binary0 = build_module_binary_with_single_function(
            vec![DataType::I32], // params
            vec![DataType::I32], // results
            BytecodeWriter::new()
                .write_opcode(Opcode::nop)
                .write_opcode(Opcode::end)
                .to_bytes(),
            vec![], // local vars
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = processor.process_function(&mut thread0, 0, 0, &vec![ForeignValue::I32(11)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::I32(11)]);

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
            &vec![ForeignValue::I32(13), ForeignValue::I32(17)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::I32(13)]);

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

        let result2 = processor.process_function(&mut thread2, 0, 0, &vec![ForeignValue::I32(19)]);
        assert_eq!(
            result2.unwrap(),
            vec![ForeignValue::I32(19), ForeignValue::I32(19)]
        );
    }
}
