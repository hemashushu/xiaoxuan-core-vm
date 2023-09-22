// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::thread::{ProgramCounter, Thread};

use super::InterpretResult;

pub fn end(thread: &mut Thread) -> InterpretResult {
    let opt_return_pc = thread.stack.remove_frames(0);

    if let Some(pc) = opt_return_pc {
        if pc.instruction_address == 0 {
            // the PC reaches the first function end, it means
            // the program reaches the ending.
            InterpretResult::End
        } else {
            // call another function or come back from another function
            InterpretResult::Jump(pc)
        }
    } else {
        // just move on
        InterpretResult::Move(2)
    }
}

pub fn block(thread: &mut Thread) -> InterpretResult {
    let type_index = thread.get_param_i32();
    do_block(thread, type_index);
    InterpretResult::Move(8)
}

pub fn block_nez(thread: &mut Thread) -> InterpretResult {
    let condition = thread.stack.pop_i32_u();
    let (type_index, alt_inst_offset) = thread.get_param_i32_i32();

    if condition == 0 {
        InterpretResult::Move(alt_inst_offset as isize)
    } else {
        do_block(thread, type_index);
        InterpretResult::Move(12)
    }
}

fn do_block(thread: &mut Thread, type_index: u32) {
    let ProgramCounter {
        instruction_address: _,
        function_internal_index: _,
        module_index,
    } = thread.pc;
    let module = &thread.context.modules[module_index];
    let type_item = &module.type_section.items[type_index as usize];

    thread
        .stack
        .create_block_frame(type_item.params_count, type_item.results_count);
}

pub fn return_(thread: &mut Thread) -> InterpretResult {
    let (skip_depth, next_inst_offset) = thread.get_param_i16_i32();
    do_return(thread, skip_depth, next_inst_offset)
}

pub fn return_nez(thread: &mut Thread) -> InterpretResult {
    let condition = thread.stack.pop_i32_u();
    let (skip_depth, next_inst_offset) = thread.get_param_i16_i32();

    if condition == 0 {
        InterpretResult::Move(8)
    } else {
        do_return(thread, skip_depth, next_inst_offset)
    }
}

fn do_return(thread: &mut Thread, skip_depth: u16, next_inst_offset: u32) -> InterpretResult {
    let opt_return_pc = thread.stack.remove_frames(skip_depth);

    if let Some(return_pc) = opt_return_pc {
        // the target frame is a function frame
        // the value of 'next_inst_offset' is ignored.
        if return_pc.instruction_address == 0 {
            // the PC reaches the first function end, it means
            // the program reaches the ending.
            InterpretResult::End
        } else {
            InterpretResult::Jump(return_pc)
        }
    } else {
        // the target frame is a block frame
        InterpretResult::Move(next_inst_offset as isize)
    }
}

pub fn recur(thread: &mut Thread) -> InterpretResult {
    let (skip_depth, start_inst_offset) = thread.get_param_i16_i32();
    do_recur(thread, skip_depth, start_inst_offset)
}

pub fn recur_nez(thread: &mut Thread) -> InterpretResult {
    let condition = thread.stack.pop_i32_u();
    let (skip_depth, start_inst_offset) = thread.get_param_i16_i32();

    if condition == 0 {
        InterpretResult::Move(8)
    } else {
        do_recur(thread, skip_depth, start_inst_offset)
    }
}

fn do_recur(thread: &mut Thread, skip_depth: u16, start_inst_offset: u32) -> InterpretResult {
    let is_func = thread.stack.reset_to_frame(skip_depth);
    if is_func {
        // the target frame is a function frame
        // the value of 'start_inst_offset' is ignored.
        let ProgramCounter {
            instruction_address,
            function_internal_index,
            module_index,
        } = thread.pc;
        let func_item =
            &thread.context.modules[module_index].func_section.items[function_internal_index];
        let relate_offset = func_item.code_offset as isize - instruction_address as isize;
        InterpretResult::Move(relate_offset)
    } else {
        // the target frame is a block frame
        InterpretResult::Move(-(start_inst_offset as isize))
    }
}

pub fn call(thread: &mut Thread) -> InterpretResult {
    let function_public_index = thread.get_param_i32();
    do_call(thread, function_public_index)
}

pub fn dcall(thread: &mut Thread) -> InterpretResult {
    let function_public_index = thread.stack.pop_i32_u();
    do_call(thread, function_public_index)
}

fn do_call(thread: &mut Thread, function_public_index: u32) -> InterpretResult {
    let ProgramCounter {
        instruction_address: return_instruction_address,
        function_internal_index: return_function_internal_index,
        module_index: return_module_index,
    } = thread.pc;

    let (target_module_index, target_function_internal_index) = thread
        .get_function_internal_index_and_module_index(
            return_module_index,
            function_public_index as usize,
        );
    let (type_index, code_offset, local_variables_allocate_bytes) = thread
        .get_function_type_and_code_offset_and_local_variables_allocate_bytes(
            target_module_index,
            target_function_internal_index,
        );

    let type_item = &thread.context.modules[target_module_index]
        .type_section
        .items[type_index];

    let return_pc = ProgramCounter {
        // the length of instruction 'call' is 8 bytes.
        // so when the target function is finish, the next instruction should be the
        // instruction after the instruction 'call'.
        instruction_address: return_instruction_address + 8,
        function_internal_index: return_function_internal_index,
        module_index: return_module_index,
    };
    thread.stack.create_function_frame(
        type_item.params_count,
        type_item.results_count,
        local_variables_allocate_bytes,
        return_pc,
    );

    let target_pc = ProgramCounter {
        instruction_address: code_offset,
        function_internal_index: target_function_internal_index,
        module_index: target_module_index,
    };

    InterpretResult::Jump(target_pc)
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        load_modules_binary,
        module_image::type_section::TypeEntry,
        utils::{
            build_module_binary_with_single_function_and_blocks, BytecodeReader, BytecodeWriter,
        },
    };
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    use crate::{init_runtime, interpreter::process_function, thread::Thread};

    #[test]
    fn test_process_control_block() {
        init_runtime();

        // func () -> (i32, i32, i32, i32)
        //     11
        //     13
        //     block () -> ()
        //         17
        //         19
        //     end
        //     23
        //     29
        // end
        //
        // expect (11, 13, 23, 29)

        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 11)
            .write_opcode_i32(Opcode::i32_imm, 13)
            .write_opcode_i32(Opcode::block, 1) // block type = 1
            .write_opcode_i32(Opcode::i32_imm, 17)
            .write_opcode_i32(Opcode::i32_imm, 19)
            .write_opcode(Opcode::end)
            .write_opcode_i32(Opcode::i32_imm, 23)
            .write_opcode_i32(Opcode::i32_imm, 29)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function_and_blocks(
            vec![],                                                           // params
            vec![DataType::I32, DataType::I32, DataType::I32, DataType::I32], // results
            code0,
            vec![], // local vars
            vec![TypeEntry {
                params: vec![],
                results: vec![],
            }],
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = process_function(&mut thread0, 0, 0, &vec![]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(11),
                ForeignValue::UInt32(13),
                ForeignValue::UInt32(23),
                ForeignValue::UInt32(29),
            ]
        );

        // func () -> (i32, i32, i32, i32)
        //     11
        //     13
        //     block (i32) -> (i32, i32)
        //         17
        //     end
        //     19
        // end
        //
        // expect (11, 13, 17, 19)

        let code1 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 11)
            .write_opcode_i32(Opcode::i32_imm, 13)
            .write_opcode_i32(Opcode::block, 1) // block type = 1
            .write_opcode_i32(Opcode::i32_imm, 17)
            .write_opcode(Opcode::end)
            .write_opcode_i32(Opcode::i32_imm, 19)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary1 = build_module_binary_with_single_function_and_blocks(
            vec![],                                                           // params
            vec![DataType::I32, DataType::I32, DataType::I32, DataType::I32], // results
            code1,
            vec![], // local vars
            vec![TypeEntry {
                params: vec![DataType::I32],
                results: vec![DataType::I32, DataType::I32],
            }],
        );

        let image1 = load_modules_binary(vec![&binary1]).unwrap();
        let mut thread1 = Thread::new(&image1);

        let result1 = process_function(&mut thread1, 0, 0, &vec![]);
        assert_eq!(
            result1.unwrap(),
            vec![
                ForeignValue::UInt32(11),
                ForeignValue::UInt32(13),
                ForeignValue::UInt32(17),
                ForeignValue::UInt32(19),
            ]
        );
    }

    #[test]
    fn test_process_control_return() {
        init_runtime();

        // func () -> (i32, i32)
        //     11
        //     13
        //     return 0 0
        //     17
        //     19
        // end
        //
        // expect (11, 13)

        // bytecode
        //
        // 0x0000 i32_imm              0xb
        // 0x0008 i32_imm              0xd
        // 0x0010 return               0 0
        // 0x0018 i32_imm              0x11
        // 0x0020 i32_imm              0x13
        // 0x0028 end

        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 11)
            .write_opcode_i32(Opcode::i32_imm, 13)
            .write_opcode_i16_i32(Opcode::return_, 0, 0)
            .write_opcode_i32(Opcode::i32_imm, 17)
            .write_opcode_i32(Opcode::i32_imm, 19)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function_and_blocks(
            vec![],                             // params
            vec![DataType::I32, DataType::I32], // results
            code0,
            vec![], // local vars
            vec![TypeEntry {
                params: vec![],
                results: vec![],
            }],
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = process_function(&mut thread0, 0, 0, &vec![]);
        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::UInt32(11), ForeignValue::UInt32(13),]
        );

        // func () -> (i32, i32, i32, i32)
        //     11
        //     13
        //     block () -> (i32, i32)
        //         17
        //         19
        //         return 0 x
        //         23
        //         29
        //     end
        //     31
        //     37
        // end
        //
        // expect (17, 19, 31, 37)

        // bytecode
        //
        // 0x0000 i32_imm              0xb
        // 0x0008 i32_imm              0xd
        // 0x0010 block                1
        // 0x0018 i32_imm              0x11
        // 0x0020 i32_imm              0x13
        // 0x0028 return               0 0x1a
        // 0x0030 i32_imm              0x17
        // 0x0038 i32_imm              0x1d
        // 0x0040 end
        // 0x0042 nop
        // 0x0044 i32_imm              0x1f
        // 0x004c i32_imm              0x25
        // 0x0054 end

        let code1 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 11)
            .write_opcode_i32(Opcode::i32_imm, 13)
            .write_opcode_i32(Opcode::block, 1) // block type = 1
            .write_opcode_i32(Opcode::i32_imm, 17)
            .write_opcode_i32(Opcode::i32_imm, 19)
            .write_opcode_i16_i32(Opcode::return_, 0, 0x1a)
            .write_opcode_i32(Opcode::i32_imm, 23)
            .write_opcode_i32(Opcode::i32_imm, 29)
            .write_opcode(Opcode::end)
            .write_opcode_i32(Opcode::i32_imm, 31)
            .write_opcode_i32(Opcode::i32_imm, 37)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary1 = build_module_binary_with_single_function_and_blocks(
            vec![],                                                           // params
            vec![DataType::I32, DataType::I32, DataType::I32, DataType::I32], // results
            code1,
            vec![], // local vars
            vec![TypeEntry {
                params: vec![],
                results: vec![DataType::I32, DataType::I32],
            }],
        );

        let image1 = load_modules_binary(vec![&binary1]).unwrap();
        let mut thread1 = Thread::new(&image1);

        let result1 = process_function(&mut thread1, 0, 0, &vec![]);
        assert_eq!(
            result1.unwrap(),
            vec![
                ForeignValue::UInt32(17),
                ForeignValue::UInt32(19),
                ForeignValue::UInt32(31),
                ForeignValue::UInt32(37),
            ]
        );

        // cross jump
        //
        // func () -> (i32, i32)
        //     11
        //     13
        //     block () -> ()
        //         17
        //         19
        //         return 1 0
        //         23
        //         29
        //     end
        //     31
        //     37
        // end
        //
        // expect (17, 19)

        // bytecode
        //
        // 0x0000 i32_imm              0xb
        // 0x0008 i32_imm              0xd
        // 0x0010 block                1
        // 0x0018 i32_imm              0x11
        // 0x0020 i32_imm              0x13
        // 0x0028 return               1 0x0
        // 0x0030 i32_imm              0x17
        // 0x0038 i32_imm              0x1d
        // 0x0040 end
        // 0x0042 nop
        // 0x0044 i32_imm              0x1f
        // 0x004c i32_imm              0x25
        // 0x0054 end

        let code2 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 11)
            .write_opcode_i32(Opcode::i32_imm, 13)
            .write_opcode_i32(Opcode::block, 1) // block type = 1
            .write_opcode_i32(Opcode::i32_imm, 17)
            .write_opcode_i32(Opcode::i32_imm, 19)
            .write_opcode_i16_i32(Opcode::return_, 1, 0)
            .write_opcode_i32(Opcode::i32_imm, 23)
            .write_opcode_i32(Opcode::i32_imm, 29)
            .write_opcode(Opcode::end)
            .write_opcode_i32(Opcode::i32_imm, 31)
            .write_opcode_i32(Opcode::i32_imm, 37)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary2 = build_module_binary_with_single_function_and_blocks(
            vec![],                             // params
            vec![DataType::I32, DataType::I32], // results
            code2,
            vec![], // local vars
            vec![TypeEntry {
                params: vec![],
                results: vec![],
            }],
        );

        let image2 = load_modules_binary(vec![&binary2]).unwrap();
        let mut thread2 = Thread::new(&image2);

        let result2 = process_function(&mut thread2, 0, 0, &vec![]);
        assert_eq!(
            result2.unwrap(),
            vec![ForeignValue::UInt32(17), ForeignValue::UInt32(19),]
        );
    }

    #[test]
    fn test_process_control_if() {
        init_runtime();

        // func $max (i32, i32) -> (i32)
        //     local_load32 0
        //     local_load32 0
        //     local_load32 1
        //     i32_lt
        //     blocl_nez ()->(i32)
        //         local_load32 1
        //     end
        // end
        //
        // assert (11, 13) -> (13)
        // assert (19, 17) -> (19)

        // bytecode
        //
        // 0x0000 local_load32         0 0
        // 0x0008 local_load32         0 0
        // 0x0010 local_load32         0 1
        // 0x0018 i32_lt_u
        // 0x001a nop
        // 0x001c block_nez            1 0x14
        // 0x0028 local_load32         0 1
        // 0x0030 end
        // 0x0032 end

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::i32_lt_u)
            .write_opcode_i32_i32(Opcode::block_nez, 1, 0x14)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::end)
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code0).to_text());

        let binary0 = build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32, DataType::I32], // params
            vec![DataType::I32],                // results
            code0,
            vec![], // local vars
            vec![TypeEntry {
                params: vec![],
                results: vec![DataType::I32],
            }],
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = process_function(
            &mut thread0,
            0,
            0,
            &vec![ForeignValue::UInt32(11), ForeignValue::UInt32(13)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(13)]);

        let result1 = process_function(
            &mut thread0,
            0,
            0,
            &vec![ForeignValue::UInt32(19), ForeignValue::UInt32(17)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(19)]);
    }
}
