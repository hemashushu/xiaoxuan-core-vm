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
    InterpretResult::Move(8)
}

pub fn block_alt(thread: &mut Thread) -> InterpretResult {
    let condition = thread.stack.pop_i32_u();
    let (type_index, alt_inst_offset) = thread.get_param_i32_i32();

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

    if condition == 0 {
        InterpretResult::Move(alt_inst_offset as isize)
    } else {
        InterpretResult::Move(12)
    }
}

pub fn block_nez(thread: &mut Thread) -> InterpretResult {
    let condition = thread.stack.pop_i32_u();
    let (type_index, alt_inst_offset) = thread.get_param_i32_i32();

    if condition == 0 {
        InterpretResult::Move(alt_inst_offset as isize)
    } else {
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

        InterpretResult::Move(12)
    }
}

pub fn break_(thread: &mut Thread) -> InterpretResult {
    let (skip_depth, next_inst_offset) = thread.get_param_i16_i32();
    do_break(thread, skip_depth, next_inst_offset)
}

pub fn break_nez(thread: &mut Thread) -> InterpretResult {
    let condition = thread.stack.pop_i32_u();
    let (skip_depth, next_inst_offset) = thread.get_param_i16_i32();

    if condition == 0 {
        InterpretResult::Move(8)
    } else {
        do_break(thread, skip_depth, next_inst_offset)
    }
}

fn do_break(thread: &mut Thread, skip_depth: u16, next_inst_offset: u32) -> InterpretResult {
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
        module_image::{local_variable_section::LocalVariableEntry, type_section::TypeEntry},
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
    }

    #[test]
    fn test_process_control_block_with_args_and_results() {
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
    fn test_process_control_break() {
        init_runtime();

        // func () -> (i32, i32)
        //     11
        //     13
        //     break 0 0
        //     17
        //     19
        // end
        //
        // expect (11, 13)

        // bytecode
        //
        // 0x0000 i32_imm              0xb
        // 0x0008 i32_imm              0xd
        // 0x0010 break                0 0
        // 0x0018 i32_imm              0x11
        // 0x0020 i32_imm              0x13
        // 0x0028 end

        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::i32_imm, 11)
            .write_opcode_i32(Opcode::i32_imm, 13)
            .write_opcode_i16_i32(Opcode::break_, 0, 0)
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
    }

    #[test]
    fn test_process_control_break_block() {
        init_runtime();
        // func () -> (i32, i32, i32, i32)
        //     11
        //     13
        //     block () -> (i32, i32)
        //         17
        //         19
        //         break 0 x
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
        // 0x0028 break                0 0x1a
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
            .write_opcode_i16_i32(Opcode::break_, 0, 0x1a)
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
    }

    #[test]
    fn test_process_control_break_cross() {
        init_runtime();
        // cross jump
        //
        // func () -> (i32, i32)
        //     11
        //     13
        //     block () -> ()
        //         17
        //         19
        //         break 1 0
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
        // 0x0028 break                1 0x0
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
            .write_opcode_i16_i32(Opcode::break_, 1, 0)
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
        //     block_nez ()->(i32)
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
        // 0x001c block_nez            1 0x16
        // 0x0028 local_load32         0 1
        // 0x0030 end
        // 0x0032 end

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::i32_lt_u)
            .write_opcode_i32_i32(Opcode::block_nez, 1, 0x16)
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

    #[test]
    fn test_process_control_if_else() {
        init_runtime();

        // func $max (i32, i32) -> (i32)
        //     local_load32 0
        //     local_load32 1
        //     i32_gt
        //     block_alt ()->(i32)
        //         local_load32 0
        //     break 0
        //         local_load32 1
        //     end
        // end
        //
        // assert (11, 13) -> (13)
        // assert (19, 17) -> (19)

        // bytecode
        //
        // 0x0000 local_load32         0 0
        // 0x0008 local_load32         0 1
        // 0x0010 i32_gt_u
        // 0x0012 nop
        // 0x0014 block_alt            1 0x1c
        // 0x0020 local_load32         0 0
        // 0x0028 break                0 0x12
        // 0x0030 local_load32         0 1
        // 0x0038 end
        // 0x003a end

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::i32_gt_u)
            .write_opcode_i32_i32(Opcode::block_alt, 1, 0x1c)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i16_i32(Opcode::break_, 0, 0x12)
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

    #[test]
    fn test_process_control_if_else_nest() {
        init_runtime();

        // func $level (i32) -> (i32)
        //     local_load32 0
        //     i32_imm 85
        //     i32_gt
        //     block_alt ()->(i32)
        //         i32_imm 65   ;; 'A' (85, 100]
        //     break 0
        //         local_load32 0
        //         i32_imm 70
        //         i32_gt
        //         block_alt ()->(i32)
        //             i32_imm 66   ;; 'B' (70,85]
        //         break 1
        //             local_load32 0
        //             i32_imm 55
        //             i32_gt
        //             block_alt ()->(i32)
        //                 i32_imm 67   ;; 'C' (55, 70]
        //             break 2
        //                 i32_imm 68   ;; 'D' [0, 55]
        //             end
        //         end
        //     end
        // end
        //
        // assert (90) -> (65) 'A'
        // assert (80) -> (66) 'B'
        // assert (70) -> (67) 'C'
        // assert (60) -> (67) 'C'
        // assert (50) -> (68) 'D'
        // assert (40) -> (68) 'D'

        // bytecode
        //
        // 0x0000 local_load32         0 0
        // 0x0008 i32_imm              0x55
        // 0x0010 i32_gt_u
        // 0x0012 nop
        // 0x0014 block_alt            1 0x1c
        // 0x0020 i32_imm              0x41
        // 0x0028 break                0 0x76
        // 0x0030 local_load32         0 0
        // 0x0038 i32_imm              0x46
        // 0x0040 i32_gt_u
        // 0x0042 nop
        // 0x0044 block_alt            1 0x1c
        // 0x0050 i32_imm              0x42
        // 0x0058 break                1 0x46
        // 0x0060 local_load32         0 0
        // 0x0068 i32_imm              0x37
        // 0x0070 i32_gt_u
        // 0x0072 nop
        // 0x0074 block_alt            1 0x1c
        // 0x0080 i32_imm              0x43
        // 0x0088 break                2 0x16
        // 0x0090 i32_imm              0x44
        // 0x0098 end
        // 0x009a end
        // 0x009c end
        // 0x009e end
        //
        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i32(Opcode::i32_imm, 85)
            .write_opcode(Opcode::i32_gt_u)
            .write_opcode_i32_i32(Opcode::block_alt, 1, 0x1c)
            .write_opcode_i32(Opcode::i32_imm, 65)
            .write_opcode_i16_i32(Opcode::break_, 0, 0x76)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i32(Opcode::i32_imm, 70)
            .write_opcode(Opcode::i32_gt_u)
            .write_opcode_i32_i32(Opcode::block_alt, 1, 0x1c)
            .write_opcode_i32(Opcode::i32_imm, 66)
            .write_opcode_i16_i32(Opcode::break_, 1, 0x46)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i32(Opcode::i32_imm, 55)
            .write_opcode(Opcode::i32_gt_u)
            .write_opcode_i32_i32(Opcode::block_alt, 1, 0x1c)
            .write_opcode_i32(Opcode::i32_imm, 67)
            .write_opcode_i16_i32(Opcode::break_, 2, 0x16)
            .write_opcode_i32(Opcode::i32_imm, 68)
            .write_opcode(Opcode::end)
            .write_opcode(Opcode::end)
            .write_opcode(Opcode::end)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32], // params
            vec![DataType::I32], // results
            code0,
            vec![], // local vars
            vec![TypeEntry {
                params: vec![],
                results: vec![DataType::I32],
            }],
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(90)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(65)]);

        let result1 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(80)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(66)]);

        let result2 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(70)]);
        assert_eq!(result2.unwrap(), vec![ForeignValue::UInt32(67)]);

        let result3 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(60)]);
        assert_eq!(result3.unwrap(), vec![ForeignValue::UInt32(67)]);

        let result4 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(50)]);
        assert_eq!(result4.unwrap(), vec![ForeignValue::UInt32(68)]);

        let result5 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(40)]);
        assert_eq!(result5.unwrap(), vec![ForeignValue::UInt32(68)]);
    }

    #[test]
    fn test_process_control_switch_case() {
        init_runtime();

        // func $level (i32) -> (i32)
        //     block ()->(i32)
        //         ;; case 1
        //         local_load32 0
        //         i32_imm 85
        //         i32_gt
        //         block_nez ()->()
        //             i32_imm 65   ;; 'A' (85, 100]
        //             break 1
        //         end
        //         ;; case 2
        //         local_load32 0
        //         i32_imm 70
        //         i32_gt
        //         block_nez ()->()
        //             i32_imm 66   ;; 'B' (70,85]
        //             break 1
        //         end
        //         ;; case 3
        //         local_load32 0
        //         i32_imm 55
        //         i32_gt
        //         block_nez ()->()
        //             i32_imm 67   ;; 'C' (55, 70]
        //             break 1
        //         end
        //         ;; default
        //         i32_imm 68   ;; 'D' [0, 55]
        //     end
        // end
        //
        // assert (90) -> (65) 'A'
        // assert (80) -> (66) 'B'
        // assert (70) -> (67) 'C'
        // assert (60) -> (67) 'C'
        // assert (50) -> (68) 'D'
        // assert (40) -> (68) 'D'

        // bytecode
        //
        // 0x0000 block                1
        // 0x0008 local_load32         0 0
        // 0x0010 i32_imm              0x55
        // 0x0018 i32_gt_u
        // 0x001a nop
        // 0x001c block_nez            2 0x1e
        // 0x0028 i32_imm              0x41
        // 0x0030 break                1 0x7e
        // 0x0038 end
        // 0x003a nop
        // 0x003c local_load32         0 0
        // 0x0044 i32_imm              0x46
        // 0x004c i32_gt_u
        // 0x004e nop
        // 0x0050 block_nez            2 0x1e
        // 0x005c i32_imm              0x42
        // 0x0064 break                1 0x4a
        // 0x006c end
        // 0x006e nop
        // 0x0070 local_load32         0 0
        // 0x0078 i32_imm              0x37
        // 0x0080 i32_gt_u
        // 0x0082 nop
        // 0x0084 block_nez            2 0x1e
        // 0x0090 i32_imm              0x43
        // 0x0098 break                1 0x16
        // 0x00a0 end
        // 0x00a2 nop
        // 0x00a4 i32_imm              0x44
        // 0x00ac end
        // 0x00ae end

        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::block, 1)
            // case 1
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i32(Opcode::i32_imm, 85)
            .write_opcode(Opcode::i32_gt_u)
            .write_opcode_i32_i32(Opcode::block_nez, 2, 0x1e)
            .write_opcode_i32(Opcode::i32_imm, 65)
            .write_opcode_i16_i32(Opcode::break_, 1, 0x7e)
            .write_opcode(Opcode::end)
            // case 2
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i32(Opcode::i32_imm, 70)
            .write_opcode(Opcode::i32_gt_u)
            .write_opcode_i32_i32(Opcode::block_nez, 2, 0x1e)
            .write_opcode_i32(Opcode::i32_imm, 66)
            .write_opcode_i16_i32(Opcode::break_, 1, 0x4a)
            .write_opcode(Opcode::end)
            // case 3
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i32(Opcode::i32_imm, 55)
            .write_opcode(Opcode::i32_gt_u)
            .write_opcode_i32_i32(Opcode::block_nez, 2, 0x1e)
            .write_opcode_i32(Opcode::i32_imm, 67)
            .write_opcode_i16_i32(Opcode::break_, 1, 0x16)
            .write_opcode(Opcode::end)
            // default
            .write_opcode_i32(Opcode::i32_imm, 68)
            .write_opcode(Opcode::end)
            // block end
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code0).to_text());

        let binary0 = build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32], // params
            vec![DataType::I32], // results
            code0,
            vec![], // local vars
            vec![
                TypeEntry {
                    params: vec![],
                    results: vec![DataType::I32],
                },
                TypeEntry {
                    params: vec![],
                    results: vec![],
                },
            ],
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(90)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(65)]);

        let result1 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(80)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(66)]);

        let result2 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(70)]);
        assert_eq!(result2.unwrap(), vec![ForeignValue::UInt32(67)]);

        let result3 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(60)]);
        assert_eq!(result3.unwrap(), vec![ForeignValue::UInt32(67)]);

        let result4 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(50)]);
        assert_eq!(result4.unwrap(), vec![ForeignValue::UInt32(68)]);

        let result5 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(40)]);
        assert_eq!(result5.unwrap(), vec![ForeignValue::UInt32(68)]);
    }

    #[test]
    fn test_process_control_while() {
        init_runtime();

        // func $accu (i32) -> (i32)
        //     (local (;0;) $sum i32)
        //     block ()->()
        //         (local_load32 1)
        //         (i32_eqz)
        //         (break_nez 0)
        //         ;; sum = sum + i
        //         (local_load32 0)
        //         (local_load32 1)
        //         (i32_add)
        //         (local_store32 0)
        //         ;; n = n - 1
        //         (local_load32 1)
        //         (i32_imm 1)
        //         (i32_sub)
        //         (local_store32 1)
        //         ;;
        //         (recur 0)
        //     end
        //     (local_load32 0)
        // end
        //
        // assert (10) -> (55)
        // assert (100) -> (5050)

        // bytecode
        //
        // 0x0000 block                1
        // 0x0008 local_load32         0 1
        // 0x0010 i32_eqz
        // 0x0012 nop
        // 0x0014 break_nez            0 0x4c
        // 0x001c local_load32         0 0
        // 0x0024 local_load32         0 1
        // 0x002c i32_add
        // 0x002e nop
        // 0x0030 local_store32        0 0
        // 0x0038 local_load32         0 1
        // 0x0040 i32_imm              0x1
        // 0x0048 i32_sub
        // 0x004a nop
        // 0x004c local_store32        0 1
        // 0x0054 recur                0 0x4c
        // 0x005c end
        // 0x005e nop
        // 0x0060 local_load32         0 0
        // 0x0068 end

        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::block, 1)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::i32_eqz)
            .write_opcode_i16_i32(Opcode::break_nez, 0, 0x4c)
            //
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::i32_add)
            .write_opcode_i16_i32(Opcode::local_store32, 0, 0)
            //
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode_i32(Opcode::i32_imm, 1)
            .write_opcode(Opcode::i32_sub)
            .write_opcode_i16_i32(Opcode::local_store32, 0, 1)
            //
            .write_opcode_i16_i32(Opcode::recur, 0, 0x4c)
            .write_opcode(Opcode::end)
            // block end
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code0).to_text());

        let binary0 = build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32], // params
            vec![DataType::I32], // results
            code0,
            vec![LocalVariableEntry::from_i32()], // local vars
            vec![TypeEntry {
                params: vec![],
                results: vec![],
            }],
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(10)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(55)]);

        let result1 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(100)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(5050)]);
    }

    #[test]
    fn test_process_control_while_opti() {
        init_runtime();
        // func $accu_optimized (i32) -> (i32)
        //     (zerp)          ;; sum
        //     block (i32)->(i32)
        //         (local_load32 0)
        //         (i32_eqz)
        //         (break_nez 0)
        //         ;; sum + i
        //         (local_load32 0)
        //         (i32_add)
        //         ;; n = n - 1
        //         (local_load32 0)
        //         (i32_dec 1)
        //         (local_store32 1)
        //         ;;
        //         (recur 0)
        //     end
        // end
        //
        // assert (10) -> (55)
        // assert (100) -> (5050)

        // bytecode
        //
        // 0x0000 zero
        // 0x0002 nop
        // 0x0004 block                0
        // 0x000c local_load32         0 0
        // 0x0014 i32_eqz
        // 0x0016 nop
        // 0x0018 break_nez            0 0x32
        // 0x0020 local_load32         0 0
        // 0x0028 i32_add
        // 0x002a nop
        // 0x002c local_load32         0 0
        // 0x0034 i32_dec              1
        // 0x0038 local_store32        0 1
        // 0x0040 recur                0 0x34
        // 0x0048 end
        // 0x004a end

        let code0 = BytecodeWriter::new()
            .write_opcode(Opcode::zero)
            //
            .write_opcode_i32(Opcode::block, 0)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode(Opcode::i32_eqz)
            .write_opcode_i16_i32(Opcode::break_nez, 0, 0x32)
            //
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode(Opcode::i32_add)
            //
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i16(Opcode::i32_dec, 1)
            .write_opcode_i16_i32(Opcode::local_store32, 0, 0)
            //
            .write_opcode_i16_i32(Opcode::recur, 0, 0x34)
            .write_opcode(Opcode::end)
            // block end
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code1).to_text());

        let binary0 = build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32], // params
            vec![DataType::I32], // results
            code0,
            vec![], // local vars
            vec![],
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(10)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(55)]);

        let result1 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(100)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(5050)]);
    }

    #[test]
    fn test_process_control_do_while() {
        init_runtime();

        // func $acc (i32) -> (i32)
        //     (local (;0;) $sum i32)
        //     block ()->()
        //         ;; sum = sum + i
        //         (local_load32 0)
        //         (local_load32 1)
        //         (i32_add)
        //         (local_store32 0)
        //         ;; n = n - 1
        //         (local_load32 1)
        //         (i32_dec 1)
        //         (local_store32 1)
        //         ;;
        //         (local_load32 1)
        //         (zero)
        //         (i32_gt)
        //         (recur_nez 0)
        //     end
        //     (local_load32 0)
        // end
        //
        // assert (10) -> (55)
        // assert (100) -> (5050)

        // bytecode
        //
        // 0x0000 block                1
        // 0x0008 local_load32         0 0
        // 0x0010 local_load32         0 1
        // 0x0018 i32_add
        // 0x001a nop
        // 0x001c local_store32        0 0
        // 0x0024 local_load32         0 1
        // 0x002c i32_dec              1
        // 0x0030 local_store32        0 1
        // 0x0038 local_load32         0 1
        // 0x0040 zero
        // 0x0042 i32_gt_u
        // 0x0044 recur_nez            0 0x3c
        // 0x004c end
        // 0x004e nop
        // 0x0050 local_load32         0 0
        // 0x0058 end

        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::block, 1)
            //
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::i32_add)
            .write_opcode_i16_i32(Opcode::local_store32, 0, 0)
            //
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode_i16(Opcode::i32_dec, 1)
            .write_opcode_i16_i32(Opcode::local_store32, 0, 1)
            //
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::zero)
            .write_opcode(Opcode::i32_gt_u)
            .write_opcode_i16_i32(Opcode::recur_nez, 0, 0x3c)
            .write_opcode(Opcode::end)
            // block end
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", BytecodeReader::new(&code0).to_text());

        let binary0 = build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32], // params
            vec![DataType::I32], // results
            code0,
            vec![LocalVariableEntry::from_i32()], // local vars
            vec![TypeEntry {
                params: vec![],
                results: vec![],
            }],
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(10)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(55)]);

        let result1 = process_function(&mut thread0, 0, 0, &vec![ForeignValue::UInt32(100)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(5050)]);
    }

    #[test]
    fn test_process_control_tco() {
        init_runtime();

        // func $accu (sum:i32, n:i32) -> (i32)
        //     ;; sum = sum + i
        //     (local_load32 0)
        //     (local_load32 1)
        //     (i32_add)
        //     (local_store32 0)
        //     ;; i = i - 1
        //     (local_load32 1)
        //     (i32_dec 1)
        //     (local_store32 1)
        //     ;;
        //     (local_load32 1)
        //     (zero)
        //     (i32_gt)
        //     (block_nez 0)
        //         (local_load32 0)
        //         (local_load32 1)
        //         (recur 1)
        //     end
        //     (local_load32 0)
        // end
        //
        // assert (0, 10) -> (55)
        // assert (0, 100) -> (5050)

        // bytecode
        //
        // 0x0000 local_load32         0 0
        // 0x0008 local_load32         0 1
        // 0x0010 i32_add
        // 0x0012 nop
        // 0x0014 local_store32        0 0
        // 0x001c local_load32         0 1
        // 0x0024 i32_dec              1
        // 0x0028 local_store32        0 1
        // 0x0030 local_load32         0 1
        // 0x0038 zero
        // 0x003a i32_gt_u
        // 0x003c block_nez            0 0x28
        // 0x0048 local_load32         0 0
        // 0x0050 local_load32         0 1
        // 0x0058 recur                1 0x0
        // 0x0060 end
        // 0x0062 nop
        // 0x0064 local_load32         0 0
        // 0x006c end

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::i32_add)
            .write_opcode_i16_i32(Opcode::local_store32, 0, 0)
            //
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode_i16(Opcode::i32_dec, 1)
            .write_opcode_i16_i32(Opcode::local_store32, 0, 1)
            //
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::zero)
            .write_opcode(Opcode::i32_gt_u)
            .write_opcode_i32_i32(Opcode::block_nez, 1, 0x28)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode_i16_i32(Opcode::recur, 1, 0)
            .write_opcode(Opcode::end)
            // block end
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
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
                results: vec![],
            }], // block types
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = process_function(
            &mut thread0,
            0,
            0,
            &vec![ForeignValue::UInt32(0), ForeignValue::UInt32(10)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(55)]);

        let result1 = process_function(
            &mut thread0,
            0,
            0,
            &vec![ForeignValue::UInt32(0), ForeignValue::UInt32(100)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(5050)]);
    }

    #[test]
    fn test_process_control_tco_opti() {
        init_runtime();

        // func $accu_opti (sum:i32, n:i32) -> (i32)
        //     ;; sum = sum + i
        //     (local_load32 0)
        //     (local_load32 1)
        //     (i32_add)
        //     ;; (local_store32 0)
        //     ;; i = i - 1
        //     (local_load32 1)
        //     (i32_dec 1)
        //     ;; (local_store32 1)
        //     ;;
        //     ;; (local_load32 0)
        //     ;; (local_load32 1)
        //     ;;
        //     ;; (local_load32 1)
        //     (duplicate)
        //     (zero)
        //     (i32_gt)
        //     (recur_nez 0)
        //     (drop)       ;; drop local 1
        // end
        //
        // assert (0, 10) -> (55)
        // assert (0, 100) -> (5050)

        let code0 = BytecodeWriter::new()
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::i32_add)
            //.write_opcode_i16_i32(Opcode::local_store32, 0, 0)
            //
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode_i16(Opcode::i32_dec, 1)
            //.write_opcode_i16_i32(Opcode::local_store32, 0, 1)
            //
            //.write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            //.write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            //
            //.write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::duplicate)
            .write_opcode(Opcode::zero)
            .write_opcode(Opcode::i32_gt_u)
            .write_opcode_i16_i32(Opcode::recur_nez, 0, 0)
            //
            .write_opcode(Opcode::drop)
            //
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
                results: vec![],
            }], // block types
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = process_function(
            &mut thread0,
            0,
            0,
            &vec![ForeignValue::UInt32(0), ForeignValue::UInt32(10)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(55)]);

        let result1 = process_function(
            &mut thread0,
            0,
            0,
            &vec![ForeignValue::UInt32(0), ForeignValue::UInt32(100)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(5050)]);
    }

    #[test]
    fn test_process_control_tco_branch() {
        init_runtime();

        // func $accu_opti (sum:i32, n:i32) -> (i32)
        //     ;;
        //     (local_load32 1)
        //     (i32_eqz)
        //     (block_alt 1)        ;; () -> (i32)
        //         (local_load32 0)
        //     (break 0)
        //         ;; sum = sum + i
        //         (local_load32 0)
        //         (local_load32 1)
        //         (i32_add)
        //         ;; i = i - 1
        //         (local_load32 1)
        //         (i32_dec 1)
        //         (recur 1)
        //     end
        // end
        //
        // assert (0, 10) -> (55)
        // assert (0, 100) -> (5050)

        // bytecode
        //
        // 0x0000 local_load32         0 1
        // 0x0008 i32_eqz
        // 0x000a nop
        // 0x000c block_alt            1 0x1c
        // 0x0018 local_load32         0 0
        // 0x0020 break                0 0x32
        // 0x0028 local_load32         0 0
        // 0x0030 local_load32         0 1
        // 0x0038 i32_add
        // 0x003a nop
        // 0x003c local_load32         0 1
        // 0x0044 i32_dec              1
        // 0x0048 recur                1 0x0
        // 0x0050 end
        // 0x0052 end

        let code0 = BytecodeWriter::new()
            //
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::i32_eqz)
            .write_opcode_i32_i32(Opcode::block_alt, 1, 0x1c)
            // then
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i16_i32(Opcode::break_, 0, 0x32)
            // else
            .write_opcode_i16_i32(Opcode::local_load32, 0, 0)
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode(Opcode::i32_add)
            //
            .write_opcode_i16_i32(Opcode::local_load32, 0, 1)
            .write_opcode_i16(Opcode::i32_dec, 1)
            //
            .write_opcode_i16_i32(Opcode::recur, 1, 0)
            .write_opcode(Opcode::end)
            // block end
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
            }], // block types
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = process_function(
            &mut thread0,
            0,
            0,
            &vec![ForeignValue::UInt32(0), ForeignValue::UInt32(10)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(55)]);

        let result1 = process_function(
            &mut thread0,
            0,
            0,
            &vec![ForeignValue::UInt32(0), ForeignValue::UInt32(100)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(5050)]);
    }
}
