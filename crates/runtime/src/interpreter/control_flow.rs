// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_program::thread_context::{ProgramCounter, ThreadContext};

use super::InterpretResult;

pub fn end(thread_context: &mut ThreadContext) -> InterpretResult {
    // note that both instruction 'end' and 'break' can end a function.
    let opt_return_pc = thread_context.stack.remove_frames(0);

    if let Some(return_pc) = opt_return_pc {
        // current function end
        //
        // if the value of the MSB of 'return module index' is '1',
        // it indicates that it's the END of the current function call.
        if return_pc.module_index & 0x80000000 == 0x80000000 {
            // since the function call could be nested (e.g. a callback function call).
            // it's necessary to recover the original module index.
            let original_pc = ProgramCounter {
                instruction_address: return_pc.instruction_address,
                function_internal_index: return_pc.function_internal_index,

                // remove the value '1' of the MSB
                module_index: return_pc.module_index & 0x7fff_ffff,
            };

            InterpretResult::End(original_pc)
        } else {
            InterpretResult::Jump(return_pc)
        }
    } else {
        // just move on
        InterpretResult::Move(2)
    }
}

pub fn block(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param type_index:i32, local_list_index:i32)
    let (type_index, local_list_index) = thread_context.get_param_i32_i32();

    let ProgramCounter {
        instruction_address: _,
        function_internal_index: _,
        module_index,
    } = thread_context.pc;
    let module = &thread_context.program_context.program_modules[module_index];
    let type_item = &module.type_section.items[type_index as usize];
    let local_variables_allocate_bytes =
        module.local_variable_section.lists[local_list_index as usize].list_allocate_bytes;

    thread_context.stack.create_frame(
        type_item.params_count,
        type_item.results_count,
        local_list_index,
        local_variables_allocate_bytes,
        None,
    );
    InterpretResult::Move(12)
}

pub fn block_alt(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param type_index:i32, local_list_index:i32, alt_inst_offset:i32)
    let condition = thread_context.stack.pop_i32_u();
    let (type_index, local_list_index, alt_inst_offset) = thread_context.get_param_i32_i32_i32();

    let ProgramCounter {
        instruction_address: _,
        function_internal_index: _,
        module_index,
    } = thread_context.pc;
    let module = &thread_context.program_context.program_modules[module_index];
    let type_item = &module.type_section.items[type_index as usize];
    let local_variables_allocate_bytes =
        module.local_variable_section.lists[local_list_index as usize].list_allocate_bytes;

    thread_context.stack.create_frame(
        type_item.params_count,
        type_item.results_count,
        local_list_index,
        local_variables_allocate_bytes,
        None,
    );

    if condition == 0 {
        InterpretResult::Move(alt_inst_offset as isize)
    } else {
        InterpretResult::Move(16)
    }
}

pub fn block_nez(thread_context: &mut ThreadContext) -> InterpretResult {
    // (param type_index:i32, local_list_index:i32, next_inst_offset:i32)

    let condition = thread_context.stack.pop_i32_u();
    // let (type_index, local_list_index, alt_inst_offset) = thread_context.get_param_i32_i32_i32();
    let (local_list_index, alt_inst_offset) = thread_context.get_param_i32_i32();

    if condition == 0 {
        InterpretResult::Move(alt_inst_offset as isize)
    } else {
        let ProgramCounter {
            instruction_address: _,
            function_internal_index: _,
            module_index,
        } = thread_context.pc;
        let module = &thread_context.program_context.program_modules[module_index];
        // let type_item = &module.type_section.items[type_index as usize];
        let local_variables_allocate_bytes =
            module.local_variable_section.lists[local_list_index as usize].list_allocate_bytes;

        thread_context.stack.create_frame(
            0, // type_item.params_count,
            0, // type_item.results_count,
            local_list_index,
            local_variables_allocate_bytes,
            None,
        );

        InterpretResult::Move(12) // 96 bits instruction
    }
}

pub fn break_(thread_context: &mut ThreadContext) -> InterpretResult {
    let (reversed_index, next_inst_offset) = thread_context.get_param_i16_i32();
    do_break(thread_context, reversed_index, next_inst_offset)
}

pub fn break_nez(thread_context: &mut ThreadContext) -> InterpretResult {
    let condition = thread_context.stack.pop_i32_u();
    let (reversed_index, next_inst_offset) = thread_context.get_param_i16_i32();

    if condition == 0 {
        InterpretResult::Move(8)
    } else {
        do_break(thread_context, reversed_index, next_inst_offset)
    }
}

fn do_break(
    thread_context: &mut ThreadContext,
    reversed_index: u16,
    next_inst_offset: u32,
) -> InterpretResult {
    // note that both instruction 'end' and 'break' can end a function.
    let opt_return_pc = thread_context.stack.remove_frames(reversed_index);

    if let Some(return_pc) = opt_return_pc {
        // current function end
        //
        // if the value of the MSB of 'return module index' is '1',
        // it indicates that it's the END of the current function call.
        if return_pc.module_index & 0x80000000 == 0x80000000 {
            // since the function call could be nested (e.g. a callback function call).
            // it's necessary to recover the original module index.
            let original_pc = ProgramCounter {
                instruction_address: return_pc.instruction_address,
                function_internal_index: return_pc.function_internal_index,

                // remove the value '1' of the MSB
                module_index: return_pc.module_index & 0x7fff_ffff,
            };

            InterpretResult::End(original_pc)
        } else {
            InterpretResult::Jump(return_pc)
        }
    } else {
        // the target frame is a block frame
        InterpretResult::Move(next_inst_offset as isize)
    }
}

pub fn recur(thread_context: &mut ThreadContext) -> InterpretResult {
    let (reversed_index, start_inst_offset) = thread_context.get_param_i16_i32();
    do_recur(thread_context, reversed_index, start_inst_offset)
}

pub fn recur_nez(thread_context: &mut ThreadContext) -> InterpretResult {
    let condition = thread_context.stack.pop_i32_u();
    let (reversed_index, start_inst_offset) = thread_context.get_param_i16_i32();

    if condition == 0 {
        InterpretResult::Move(8)
    } else {
        do_recur(thread_context, reversed_index, start_inst_offset)
    }
}

fn do_recur(
    thread_context: &mut ThreadContext,
    reversed_index: u16,
    start_inst_offset: u32,
) -> InterpretResult {
    let is_func = thread_context.stack.reset_frames(reversed_index);
    if is_func {
        // the target frame is a function frame
        // the value of 'start_inst_offset' is ignored.
        let ProgramCounter {
            instruction_address,
            function_internal_index,
            module_index,
        } = thread_context.pc;
        let func_item = &thread_context.program_context.program_modules[module_index]
            .func_section
            .items[function_internal_index];
        let relate_offset = func_item.code_offset as isize - instruction_address as isize;
        InterpretResult::Move(relate_offset)
    } else {
        // the target frame is a block frame
        InterpretResult::Move(-(start_inst_offset as isize))
    }
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        bytecode_reader::print_bytecode_as_text,
        bytecode_writer::BytecodeWriter,
        module_image::local_variable_section::LocalVariableEntry,
        utils::{helper_build_module_binary_with_single_function_and_blocks, HelperBlockEntry},
    };

    use crate::{in_memory_program_source::InMemoryProgramSource, interpreter::process_function};
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

    #[test]
    fn test_interpreter_control_flow_block() {
        // fn () -> (i32, i32, i32, i32)
        //     (i32_imm 11)
        //     (i32_imm 13)
        //     (block 1 1) () -> ()
        //         (i32_imm 17)
        //         (i32_imm 19)
        //     end
        //     (i32_imm 23)
        //     (i32_imm 29)
        // end
        //
        // expect (11, 13, 23, 29)

        // bytecode:
        //
        // 0x0000  80 01 00 00  0b 00 00 00    i32.imm           0x0000000b
        // 0x0008  80 01 00 00  0d 00 00 00    i32.imm           0x0000000d
        // 0x0010  01 0a 00 00  01 00 00 00    block             type:1   local:1
        //         01 00 00 00
        // 0x001c  80 01 00 00  11 00 00 00    i32.imm           0x00000011
        // 0x0024  80 01 00 00  13 00 00 00    i32.imm           0x00000013
        // 0x002c  00 0a                       end
        // 0x002e  00 0c                       nop
        // 0x0030  80 01 00 00  17 00 00 00    i32.imm           0x00000017
        // 0x0038  80 01 00 00  1d 00 00 00    i32.imm           0x0000001d
        // 0x0040  00 0a                       end

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 11)
            .append_opcode_i32(Opcode::i32_imm, 13)
            .append_opcode_i32_i32(Opcode::block, 1, 1) // block type = 1, local variable index = 1
            .append_opcode_i32(Opcode::i32_imm, 17)
            .append_opcode_i32(Opcode::i32_imm, 19)
            .append_opcode(Opcode::end)
            .append_opcode_i32(Opcode::i32_imm, 23)
            .append_opcode_i32(Opcode::i32_imm, 29)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![],                                                           // params
            vec![DataType::I32, DataType::I32, DataType::I32, DataType::I32], // results
            vec![],                                                           // local vars
            code0,
            vec![HelperBlockEntry {
                params: vec![],
                results: vec![],
                local_variable_item_entries_without_args: vec![],
            }],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
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
    fn test_interpreter_control_flow_block_with_args_and_results() {
        // fn () -> (i32, i32, i32)
        //     (i32_imm 11)
        //     (i32_imm 13)
        //     (block 1 1) (i32) -> (i32)
        //         (local_load 0)
        //         (i32_imm 17)
        //         (i32_add)
        //     end
        //     (i32_imm 19)
        // end
        //
        // expect (11, 30, 19)

        // bytecode:
        //
        // 0x0000  80 01 00 00  0b 00 00 00    i32.imm           0x0000000b
        // 0x0008  80 01 00 00  0d 00 00 00    i32.imm           0x0000000d
        // 0x0010  01 0a 00 00  01 00 00 00    block             type:1   local:1
        //         01 00 00 00
        // 0x001c  02 02 00 00  00 00 00 00    local.load32_i32  off:0x00  rev:0   idx:0
        // 0x0024  80 01 00 00  11 00 00 00    i32.imm           0x00000011
        // 0x002c  00 07                       i32.add
        // 0x002e  00 0a                       end
        // 0x0030  80 01 00 00  13 00 00 00    i32.imm           0x00000013
        // 0x0038  00 0a                       end

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 11)
            .append_opcode_i32(Opcode::i32_imm, 13)
            .append_opcode_i32_i32(Opcode::block, 1, 1) // block type = 1, local list index = 1
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i32(Opcode::i32_imm, 17)
            .append_opcode(Opcode::i32_add)
            .append_opcode(Opcode::end)
            .append_opcode_i32(Opcode::i32_imm, 19)
            .append_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", print_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![],                                            // params
            vec![DataType::I32, DataType::I32, DataType::I32], // results
            vec![],                                            // local vars
            code0,
            vec![HelperBlockEntry {
                params: vec![DataType::I32],
                results: vec![DataType::I32],
                local_variable_item_entries_without_args: vec![],
            }],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(11),
                ForeignValue::UInt32(30),
                ForeignValue::UInt32(19),
            ]
        );
    }

    #[test]
    fn test_interpreter_control_flow_block_with_local_vars() {
        // func (a/0:i32, b/1:i32) -> (i32,i32,i32,i32,i32,i32,i32,i32)
        //     (local c/2:i32, d/3:i32)
        //     ;; c=a+1                     ;; 20
        //     ;; d=b+1                     ;; 12
        //     (block 1 1) () -> (i32, i32, i32,i32)
        //         (local p/0:i32, q/1:i32)
        //         ;; a=a-1                 ;; 18
        //         ;; b=b-1                 ;; 10
        //         ;; p=c+d                 ;; 32
        //         ;; q=c-d                 ;; 8
        //         ;; load c
        //         ;; load d
        //         (block 2 2) (x/0:i32, y/1:i32) -> (i32,i32)
        //             ;; d=d+1             ;; 13
        //             ;; q=q-1             ;; 7
        //             ;; x+q               ;; 27 (ret 0)
        //             ;; y+p               ;; 44 (ret 1)
        //         end
        //         ;; load p (ret 2)
        //         ;; load q (ret 3)
        //     end
        //     ;; load a (ret 4)
        //     ;; load b (ret 5)
        //     ;; load c (ret 6)
        //     ;; load d (ret 7)
        // end
        //
        // expect (19, 11) -> (27, 44, 32, 7, 18, 10, 20, 13)

        // bytecode:
        //
        // 0x0000  02 02 00 00  00 00 00 00    local.load32_i32  rev:0   off:0x00  idx:0
        // 0x0008  07 07 01 00                 i32.inc           1
        // 0x000c  09 02 00 00  00 00 02 00    local.store32     rev:0   off:0x00  idx:2
        // 0x0014  02 02 00 00  00 00 01 00    local.load32_i32  rev:0   off:0x00  idx:1
        // 0x001c  07 07 01 00                 i32.inc           1
        // 0x0020  09 02 00 00  00 00 03 00    local.store32     rev:0   off:0x00  idx:3
        // 0x0028  01 0a 00 00  01 00 00 00    block             type:1   local:1
        //         01 00 00 00
        // 0x0034  02 02 01 00  00 00 00 00    local.load32_i32  rev:1   off:0x00  idx:0
        // 0x003c  08 07 01 00                 i32.dec           1
        // 0x0040  09 02 01 00  00 00 00 00    local.store32     rev:1   off:0x00  idx:0
        // 0x0048  02 02 01 00  00 00 01 00    local.load32_i32  rev:1   off:0x00  idx:1
        // 0x0050  08 07 01 00                 i32.dec           1
        // 0x0054  09 02 01 00  00 00 01 00    local.store32     rev:1   off:0x00  idx:1
        // 0x005c  02 02 01 00  00 00 02 00    local.load32_i32  rev:1   off:0x00  idx:2
        // 0x0064  02 02 01 00  00 00 03 00    local.load32_i32  rev:1   off:0x00  idx:3
        // 0x006c  00 07                       i32.add
        // 0x006e  09 02 00 00  00 00 00 00    local.store32     rev:0   off:0x00  idx:0
        // 0x0076  02 02 01 00  00 00 02 00    local.load32_i32  rev:1   off:0x00  idx:2
        // 0x007e  02 02 01 00  00 00 03 00    local.load32_i32  rev:1   off:0x00  idx:3
        // 0x0086  01 07                       i32.sub
        // 0x0088  09 02 00 00  00 00 01 00    local.store32     rev:0   off:0x00  idx:1
        // 0x0090  02 02 01 00  00 00 02 00    local.load32_i32  rev:1   off:0x00  idx:2
        // 0x0098  02 02 01 00  00 00 03 00    local.load32_i32  rev:1   off:0x00  idx:3
        // 0x00a0  01 0a 00 00  02 00 00 00    block             type:2   local:2
        //         02 00 00 00
        // 0x00ac  02 02 02 00  00 00 03 00    local.load32_i32  rev:2   off:0x00  idx:3
        // 0x00b4  07 07 01 00                 i32.inc           1
        // 0x00b8  09 02 02 00  00 00 03 00    local.store32     rev:2   off:0x00  idx:3
        // 0x00c0  02 02 01 00  00 00 01 00    local.load32_i32  rev:1   off:0x00  idx:1
        // 0x00c8  08 07 01 00                 i32.dec           1
        // 0x00cc  09 02 01 00  00 00 01 00    local.store32     rev:1   off:0x00  idx:1
        // 0x00d4  02 02 00 00  00 00 00 00    local.load32_i32  rev:0   off:0x00  idx:0
        // 0x00dc  02 02 01 00  00 00 01 00    local.load32_i32  rev:1   off:0x00  idx:1
        // 0x00e4  00 07                       i32.add
        // 0x00e6  02 02 00 00  00 00 01 00    local.load32_i32  rev:0   off:0x00  idx:1
        // 0x00ee  02 02 01 00  00 00 00 00    local.load32_i32  rev:1   off:0x00  idx:0
        // 0x00f6  00 07                       i32.add
        // 0x00f8  00 0a                       end
        // 0x00fa  02 02 00 00  00 00 00 00    local.load32_i32  rev:0   off:0x00  idx:0
        // 0x0102  02 02 00 00  00 00 01 00    local.load32_i32  rev:0   off:0x00  idx:1
        // 0x010a  00 0a                       end
        // 0x010c  02 02 00 00  00 00 00 00    local.load32_i32  rev:0   off:0x00  idx:0
        // 0x0114  02 02 00 00  00 00 01 00    local.load32_i32  rev:0   off:0x00  idx:1
        // 0x011c  02 02 00 00  00 00 02 00    local.load32_i32  rev:0   off:0x00  idx:2
        // 0x0124  02 02 00 00  00 00 03 00    local.load32_i32  rev:0   off:0x00  idx:3
        // 0x012c  00 0a                       end

        let code0 = BytecodeWriter::new()
            // c=a+1
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16(Opcode::i32_inc, 1)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 0, 2)
            // d=b+1
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16(Opcode::i32_inc, 1)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 0, 3)
            // block 1
            .append_opcode_i32_i32(Opcode::block, 1, 1)
            // a=a-1
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 0)
            .append_opcode_i16(Opcode::i32_dec, 1)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 1, 0, 0)
            // b=b-1
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 1)
            .append_opcode_i16(Opcode::i32_dec, 1)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 1, 0, 1)
            // p=c+d
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 3)
            .append_opcode(Opcode::i32_add)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 0, 0)
            // q=c-d
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 3)
            .append_opcode(Opcode::i32_sub)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 0, 1)
            // load c, d
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 3)
            // block 2
            .append_opcode_i32_i32(Opcode::block, 2, 2)
            // d=d+1
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 2, 0, 3)
            .append_opcode_i16(Opcode::i32_inc, 1)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 2, 0, 3)
            // q=q-1
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 1)
            .append_opcode_i16(Opcode::i32_dec, 1)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 1, 0, 1)
            // x+q
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 1)
            .append_opcode(Opcode::i32_add)
            // y+p
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 0)
            .append_opcode(Opcode::i32_add)
            //
            .append_opcode(Opcode::end)
            // load p, q
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            //
            .append_opcode(Opcode::end)
            // load a, b, c, d
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 3)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", print_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32, DataType::I32], // params
            vec![
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
                DataType::I32,
            ], // results
            vec![
                LocalVariableEntry::from_i32(),
                LocalVariableEntry::from_i32(),
            ], // local vars
            code0,
            vec![
                HelperBlockEntry {
                    params: vec![],
                    results: vec![DataType::I32, DataType::I32, DataType::I32, DataType::I32],
                    local_variable_item_entries_without_args: vec![
                        LocalVariableEntry::from_i32(),
                        LocalVariableEntry::from_i32(),
                    ],
                },
                HelperBlockEntry {
                    params: vec![DataType::I32, DataType::I32],
                    results: vec![DataType::I32, DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
            ],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::UInt32(19), ForeignValue::UInt32(11)],
        );
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(27),
                ForeignValue::UInt32(44),
                ForeignValue::UInt32(32),
                ForeignValue::UInt32(7),
                ForeignValue::UInt32(18),
                ForeignValue::UInt32(10),
                ForeignValue::UInt32(20),
                ForeignValue::UInt32(13),
            ]
        );
    }

    #[test]
    fn test_interpreter_control_flow_break_function() {
        // fn () -> (i32, i32)
        //     (i32_imm 11)
        //     (i32_imm 13)
        //     (break 0)
        //     (i32_imm 17)
        //     (i32_imm 19)
        // end
        //
        // expect (11, 13)

        // bytecode:
        //
        // 0x0000  80 01 00 00  0b 00 00 00    i32.imm           0x0000000b
        // 0x0008  80 01 00 00  0d 00 00 00    i32.imm           0x0000000d
        // 0x0010  02 0a 00 00  00 00 00 00    break             rev:0   off:0x00
        // 0x0018  80 01 00 00  11 00 00 00    i32.imm           0x00000011
        // 0x0020  80 01 00 00  13 00 00 00    i32.imm           0x00000013
        // 0x0028  00 0a                       end

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 11)
            .append_opcode_i32(Opcode::i32_imm, 13)
            .append_opcode_i16_i32(Opcode::break_, 0, 0)
            .append_opcode_i32(Opcode::i32_imm, 17)
            .append_opcode_i32(Opcode::i32_imm, 19)
            .append_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", print_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![],                             // params
            vec![DataType::I32, DataType::I32], // results
            vec![],                             // local vars
            code0,
            vec![],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::UInt32(11), ForeignValue::UInt32(13),]
        );
    }

    #[test]
    fn test_interpreter_control_flow_break_block() {
        // fn () -> (i32, i32, i32, i32)
        //     (i32_imm 11)
        //     (i32_imm 13)
        //     (block 1 1) () -> (i32, i32)
        //         (i32_imm 17)
        //         (i32_imm 19)
        //         (break 0)
        //         (i32_imm 23)
        //         (i32_imm 29)
        //     end
        //     (i32_imm 31)
        //     (i32_imm 37)
        // end
        //
        // expect (17, 19, 31, 37)

        // bytecode:
        //
        // 0x0000  80 01 00 00  0b 00 00 00    i32.imm           0x0000000b
        // 0x0008  80 01 00 00  0d 00 00 00    i32.imm           0x0000000d
        // 0x0010  01 0a 00 00  01 00 00 00    block             type:1   local:1
        //         01 00 00 00
        // 0x001c  80 01 00 00  11 00 00 00    i32.imm           0x00000011
        // 0x0024  80 01 00 00  13 00 00 00    i32.imm           0x00000013
        // 0x002c  02 0a 00 00  1a 00 00 00    break             rev:0   off:0x1a
        // 0x0034  80 01 00 00  17 00 00 00    i32.imm           0x00000017
        // 0x003c  80 01 00 00  1d 00 00 00    i32.imm           0x0000001d
        // 0x0044  00 0a                       end
        // 0x0046  00 0c                       nop
        // 0x0048  80 01 00 00  1f 00 00 00    i32.imm           0x0000001f
        // 0x0050  80 01 00 00  25 00 00 00    i32.imm           0x00000025
        // 0x0058  00 0a                       end

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 11)
            .append_opcode_i32(Opcode::i32_imm, 13)
            .append_opcode_i32_i32(Opcode::block, 1, 1) // block type = 1
            .append_opcode_i32(Opcode::i32_imm, 17)
            .append_opcode_i32(Opcode::i32_imm, 19)
            .append_opcode_i16_i32(Opcode::break_, 0, 0x1a)
            .append_opcode_i32(Opcode::i32_imm, 23)
            .append_opcode_i32(Opcode::i32_imm, 29)
            .append_opcode(Opcode::end)
            .append_opcode_i32(Opcode::i32_imm, 31)
            .append_opcode_i32(Opcode::i32_imm, 37)
            .append_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", print_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![],                                                           // params
            vec![DataType::I32, DataType::I32, DataType::I32, DataType::I32], // results
            vec![],                                                           // local vars
            code0,
            vec![HelperBlockEntry {
                params: vec![],
                results: vec![DataType::I32, DataType::I32],
                local_variable_item_entries_without_args: vec![],
            }],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(17),
                ForeignValue::UInt32(19),
                ForeignValue::UInt32(31),
                ForeignValue::UInt32(37),
            ]
        );
    }

    #[test]
    fn test_interpreter_control_flow_break_block_to_function() {
        // fn () -> (i32, i32)
        //     (i32_imm 11)
        //     (i32_imm 13)
        //     (block 1 1) () -> (i32 i32)
        //         (i32_imm 17)
        //         (i32_imm 19)
        //         (break 1)
        //         (i32_imm 23)
        //         (i32_imm 29)
        //     end
        //     (i32_imm 31)
        //     (i32_imm 37)
        // end
        //
        // expect (17, 19)

        // bytecode:
        //
        // 0x0000  80 01 00 00  0b 00 00 00    i32.imm           0x0000000b
        // 0x0008  80 01 00 00  0d 00 00 00    i32.imm           0x0000000d
        // 0x0010  01 0a 00 00  01 00 00 00    block             type:1   local:1
        //         01 00 00 00
        // 0x001c  80 01 00 00  11 00 00 00    i32.imm           0x00000011
        // 0x0024  80 01 00 00  13 00 00 00    i32.imm           0x00000013
        // 0x002c  02 0a 01 00  00 00 00 00    break             rev:1   off:0x00
        // 0x0034  80 01 00 00  17 00 00 00    i32.imm           0x00000017
        // 0x003c  80 01 00 00  1d 00 00 00    i32.imm           0x0000001d
        // 0x0044  00 0a                       end
        // 0x0046  00 0c                       nop
        // 0x0048  80 01 00 00  1f 00 00 00    i32.imm           0x0000001f
        // 0x0050  80 01 00 00  25 00 00 00    i32.imm           0x00000025
        // 0x0058  00 0a                       end

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 11)
            .append_opcode_i32(Opcode::i32_imm, 13)
            .append_opcode_i32_i32(Opcode::block, 1, 1) // block type = 1, local variable index = 1
            .append_opcode_i32(Opcode::i32_imm, 17)
            .append_opcode_i32(Opcode::i32_imm, 19)
            .append_opcode_i16_i32(Opcode::break_, 1, 0)
            .append_opcode_i32(Opcode::i32_imm, 23)
            .append_opcode_i32(Opcode::i32_imm, 29)
            .append_opcode(Opcode::end)
            .append_opcode_i32(Opcode::i32_imm, 31)
            .append_opcode_i32(Opcode::i32_imm, 37)
            .append_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", print_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![],                             // params
            vec![DataType::I32, DataType::I32], // results
            vec![],                             // local vars
            code0,
            vec![HelperBlockEntry {
                params: vec![],
                results: vec![DataType::I32, DataType::I32],
                local_variable_item_entries_without_args: vec![],
            }],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::UInt32(17), ForeignValue::UInt32(19),]
        );
    }

    #[test]
    fn test_interpreter_control_flow_structure_when() {
        // func $max (i32, i32) -> (i32)
        //     (local $ret/2 i32)
        //
        //     (local_load32 0 0)
        //     (local_store32 0 2)
        //
        //     (local_load32 0 0)
        //     (local_load32 0 1)
        //     i32_lt_u
        //     (block_nez local_idx:1) ()->()
        //          (local_load32 1 1)
        //          (local_store32 1 2)
        //     end
        //     (local_load32 0 2)
        // end
        //
        // assert (11, 13) -> (13)
        // assert (19, 17) -> (19)

        // bytecode:
        //
        // 0x0000  02 02 00 00  00 00 00 00    local.load32_i32  rev:0   off:0x00  idx:0
        // 0x0008  09 02 00 00  00 00 02 00    local.store32     rev:0   off:0x00  idx:2
        // 0x0010  02 02 00 00  00 00 00 00    local.load32_i32  rev:0   off:0x00  idx:0
        // 0x0018  02 02 00 00  00 00 01 00    local.load32_i32  rev:0   off:0x00  idx:1
        // 0x0020  05 06                       i32.lt_u
        // 0x0022  00 0c                       nop
        // 0x0024  04 0a 00 00  01 00 00 00    block_nez         local:1   off:0x1e
        //         1e 00 00 00
        // 0x0030  02 02 01 00  00 00 01 00    local.load32_i32  rev:1   off:0x00  idx:1
        // 0x0038  09 02 01 00  00 00 02 00    local.store32     rev:1   off:0x00  idx:2
        // 0x0040  00 0a                       end
        // 0x0042  02 02 00 00  00 00 02 00    local.load32_i32  rev:0   off:0x00  idx:2
        // 0x004a  00 0a                       end

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 0, 2)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_lt_u)
            .append_opcode_i32_i32(Opcode::block_nez, 1, 0x1e)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 1)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 1, 0, 2)
            .append_opcode(Opcode::end)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", print_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32, DataType::I32],   // params
            vec![DataType::I32],                  // results
            vec![LocalVariableEntry::from_i32()], // local vars
            code0,
            vec![HelperBlockEntry {
                params: vec![],
                results: vec![],
                local_variable_item_entries_without_args: vec![],
            }],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::UInt32(11), ForeignValue::UInt32(13)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(13)]);

        let result1 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::UInt32(19), ForeignValue::UInt32(17)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(19)]);
    }

    #[test]
    fn test_interpreter_control_flow_break_block_crossing() {
        // cross block breaking
        //
        // fn (i32) -> (i32 i32 i32 i32)
        //     (i32_imm 11)
        //     (i32_imm 13)
        //     (block 1 1) () -> (i32 i32)
        //         (i32_imm 17)
        //         (i32_imm 19)
        //         (local_load32_i32 1 0)  ;; true
        //         (block_nez 2 2) () -> (i32 i32)
        //             (i32_imm 23)
        //             (i32_imm 29)
        //             (break 1)
        //             (i32_imm 31)
        //             (i32_imm 37)
        //         end
        //         (i32_imm 41)
        //         (i32_imm 43)
        //     end
        //     (i32_imm 51)
        //     (i32_imm 53)
        // end
        //
        // expect (1) -> (23, 29, 51, 53)
        // expect (0) -> (41, 43, 51, 53)

        // bytecode:
        //
        // 0x0000  80 01 00 00  0b 00 00 00    i32.imm           0x0000000b
        // 0x0008  80 01 00 00  0d 00 00 00    i32.imm           0x0000000d
        // 0x0010  01 0a 00 00  01 00 00 00    block             type:1   local:1
        //         01 00 00 00
        // 0x001c  80 01 00 00  11 00 00 00    i32.imm           0x00000011
        // 0x0024  80 01 00 00  13 00 00 00    i32.imm           0x00000013
        // 0x002c  02 02 01 00  00 00 00 00    local.load32_i32  rev:1   off:0x00  idx:0
        // 0x0034  04 0a 00 00  02 00 00 00    block_nez         local:2   off:0x36
        //         36 00 00 00
        // 0x0040  80 01 00 00  17 00 00 00    i32.imm           0x00000017
        // 0x0048  80 01 00 00  1d 00 00 00    i32.imm           0x0000001d
        // 0x0050  02 0a 01 00  2e 00 00 00    break             rev:1   off:0x2e
        // 0x0058  80 01 00 00  1f 00 00 00    i32.imm           0x0000001f
        // 0x0060  80 01 00 00  25 00 00 00    i32.imm           0x00000025
        // 0x0068  00 0a                       end
        // 0x006a  00 0c                       nop
        // 0x006c  80 01 00 00  29 00 00 00    i32.imm           0x00000029
        // 0x0074  80 01 00 00  2b 00 00 00    i32.imm           0x0000002b
        // 0x007c  00 0a                       end
        // 0x007e  00 0c                       nop
        // 0x0080  80 01 00 00  33 00 00 00    i32.imm           0x00000033
        // 0x0088  80 01 00 00  35 00 00 00    i32.imm           0x00000035
        // 0x0090  00 0a                       end

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::i32_imm, 11)
            .append_opcode_i32(Opcode::i32_imm, 13)
            .append_opcode_i32_i32(Opcode::block, 1, 1) // block type = 1
            .append_opcode_i32(Opcode::i32_imm, 17)
            .append_opcode_i32(Opcode::i32_imm, 19)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 0)
            .append_opcode_i32_i32(Opcode::block_nez, 2, 0x36) // block type = 2
            .append_opcode_i32(Opcode::i32_imm, 23)
            .append_opcode_i32(Opcode::i32_imm, 29)
            .append_opcode_i16_i32(Opcode::break_, 1, 0x2e)
            .append_opcode_i32(Opcode::i32_imm, 31)
            .append_opcode_i32(Opcode::i32_imm, 37)
            .append_opcode(Opcode::end)
            .append_opcode_i32(Opcode::i32_imm, 41)
            .append_opcode_i32(Opcode::i32_imm, 43)
            .append_opcode(Opcode::end)
            .append_opcode_i32(Opcode::i32_imm, 51)
            .append_opcode_i32(Opcode::i32_imm, 53)
            .append_opcode(Opcode::end)
            .to_bytes();

        println!("{}", print_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32],                                              // params
            vec![DataType::I32, DataType::I32, DataType::I32, DataType::I32], // results
            vec![],                                                           // local vars
            code0,
            vec![
                HelperBlockEntry {
                    params: vec![],
                    results: vec![DataType::I32, DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![],
                    results: vec![DataType::I32, DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
            ],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(1)]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(23),
                ForeignValue::UInt32(29),
                ForeignValue::UInt32(51),
                ForeignValue::UInt32(53),
            ]
        );

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(0)]);
        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt32(41),
                ForeignValue::UInt32(43),
                ForeignValue::UInt32(51),
                ForeignValue::UInt32(53),
            ]
        );
    }

    #[test]
    fn test_interpreter_control_flow_structure_if() {
        // func $max (i32, i32) -> (i32)
        //     (local_load32 0 0)
        //     (local_load32 0 1)
        //     i32_gt
        //     (block_alt 1 1) ()->(i32)
        //         (local_load32 1 0)
        //     (break 0)
        //         (local_load32 1 1)
        //     end
        // end
        //
        // assert (11, 13) -> (13)
        // assert (19, 17) -> (19)

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_gt_u)
            .append_opcode_i32_i32_i32(Opcode::block_alt, 1, 1, 0x20)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 0)
            .append_opcode_i16_i32(Opcode::break_, 0, 0x12)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 1)
            .append_opcode(Opcode::end)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32, DataType::I32], // params
            vec![DataType::I32],                // results
            vec![],                             // local vars
            code0,
            vec![HelperBlockEntry {
                params: vec![], // 'block_alt' has no PARAMS
                results: vec![DataType::I32],
                local_variable_item_entries_without_args: vec![],
            }],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::UInt32(11), ForeignValue::UInt32(13)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(13)]);

        let result1 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::UInt32(19), ForeignValue::UInt32(17)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(19)]);
    }

    #[test]
    fn test_interpreter_control_flow_structure_if_nested() {
        // func $level (i32) -> (i32)
        //     (local_load32 0 0)
        //     (i32_imm 85)
        //     i32_gt
        //     (block_alt 1 1) ()->(i32)            ;; block 1 1
        //         (i32_imm 65)                     ;; 'A' (85, 100]
        //     (break 0)
        //         (local_load32 1 0)
        //         (i32_imm 70)
        //         i32_gt_u
        //         (block_alt 2 2) ()->(i32)        ;; block 2 2
        //             (i32_imm 66)                 ;; 'B' (70,85]
        //         (break 1)
        //             (local_load32 2 0)
        //             (i32_imm 55)
        //             i32_gt_u
        //             (block_alt 3 3) ()->(i32)    ;; block 3 3
        //                 (i32_imm 67)             ;; 'C' (55, 70]
        //             (break 2)
        //                 (i32_imm 68)             ;; 'D' [0, 55]
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

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i32(Opcode::i32_imm, 85)
            .append_opcode(Opcode::i32_gt_u)
            .append_opcode_i32_i32_i32(Opcode::block_alt, 1, 1, 0x20)
            .append_opcode_i32(Opcode::i32_imm, 65)
            .append_opcode_i16_i32(Opcode::break_, 0, 0x7e)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 0)
            .append_opcode_i32(Opcode::i32_imm, 70)
            .append_opcode(Opcode::i32_gt_u)
            .append_opcode_i32_i32_i32(Opcode::block_alt, 2, 2, 0x20)
            .append_opcode_i32(Opcode::i32_imm, 66)
            .append_opcode_i16_i32(Opcode::break_, 1, 0x4a)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 2, 0, 0)
            .append_opcode_i32(Opcode::i32_imm, 55)
            .append_opcode(Opcode::i32_gt_u)
            .append_opcode_i32_i32_i32(Opcode::block_alt, 3, 3, 0x20)
            .append_opcode_i32(Opcode::i32_imm, 67)
            .append_opcode_i16_i32(Opcode::break_, 2, 0x16)
            .append_opcode_i32(Opcode::i32_imm, 68)
            .append_opcode(Opcode::end)
            .append_opcode(Opcode::end)
            .append_opcode(Opcode::end)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32], // params
            vec![DataType::I32], // results
            vec![],              // local vars
            code0,
            vec![
                HelperBlockEntry {
                    params: vec![], // 'block_alt' has no PARAMS
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![], // 'block_alt' has no PARAMS
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![], // 'block_alt' has no PARAMS
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
            ],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(90)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(65)]);

        let result1 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(80)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(66)]);

        let result2 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(70)]);
        assert_eq!(result2.unwrap(), vec![ForeignValue::UInt32(67)]);

        let result3 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(60)]);
        assert_eq!(result3.unwrap(), vec![ForeignValue::UInt32(67)]);

        let result4 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(50)]);
        assert_eq!(result4.unwrap(), vec![ForeignValue::UInt32(68)]);

        let result5 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(40)]);
        assert_eq!(result5.unwrap(), vec![ForeignValue::UInt32(68)]);
    }

    #[test]
    fn test_interpreter_control_flow_structure_branch() {
        // func $level (i32) -> (i32)
        //     (block 1 1) ()->(i32)        ;; block 1 1
        //                                  ;; case 1
        //         (local_load32 0 0)
        //         (i32_imm 85)
        //         i32_gt
        //         (block_nez 2) ()->()   ;; block 2 2
        //             (i32_imm 65)         ;; 'A' (85, 100]
        //             (break 1)
        //         end
        //                                  ;; case 2
        //         (local_load32 0 0)
        //         (i32_imm 70)
        //         i32_gt
        //         (block_nez 3) ()->()   ;; block 3 3
        //             (i32_imm 66)         ;; 'B' (70,85]
        //             (break 1)
        //         end
        //                                  ;; case 3
        //         (local_load32 0 0)
        //         (i32_imm 55)
        //         i32_gt
        //         (block_nez 4) ()->()   ;; block 4 4
        //             (i32_imm 67)         ;; 'C' (55, 70]
        //             (break 1)
        //         end
        //                                  ;; default
        //         (i32_imm 68)             ;; 'D' [0, 55]
        //     end
        // end
        //
        // assert (90) -> (65) 'A'
        // assert (80) -> (66) 'B'
        // assert (70) -> (67) 'C'
        // assert (60) -> (67) 'C'
        // assert (50) -> (68) 'D'
        // assert (40) -> (68) 'D'

        let code0 = BytecodeWriter::new()
            .append_opcode_i32_i32(Opcode::block, 1, 1)
            // case 1
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 0)
            .append_opcode_i32(Opcode::i32_imm, 85)
            .append_opcode(Opcode::i32_gt_u)
            .append_opcode_i32_i32(Opcode::block_nez, 2, 0x1e)
            .append_opcode_i32(Opcode::i32_imm, 65)
            .append_opcode_i16_i32(Opcode::break_, 1, 0x7c)
            .append_opcode(Opcode::end)
            // case 2
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 0)
            .append_opcode_i32(Opcode::i32_imm, 70)
            .append_opcode(Opcode::i32_gt_u)
            .append_opcode_i32_i32(Opcode::block_nez, 3, 0x1e)
            .append_opcode_i32(Opcode::i32_imm, 66)
            .append_opcode_i16_i32(Opcode::break_, 1, 0x48)
            .append_opcode(Opcode::end)
            // case 3
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 0)
            .append_opcode_i32(Opcode::i32_imm, 55)
            .append_opcode(Opcode::i32_gt_u)
            .append_opcode_i32_i32(Opcode::block_nez, 4, 0x1e)
            .append_opcode_i32(Opcode::i32_imm, 67)
            .append_opcode_i16_i32(Opcode::break_, 1, 0x14)
            .append_opcode(Opcode::end)
            // default
            .append_opcode_i32(Opcode::i32_imm, 68)
            // block end
            .append_opcode(Opcode::end)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32], // params
            vec![DataType::I32], // results
            vec![],              // local vars
            code0,
            vec![
                HelperBlockEntry {
                    params: vec![],
                    results: vec![DataType::I32],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![],
                    results: vec![],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![],
                    results: vec![],
                    local_variable_item_entries_without_args: vec![],
                },
                HelperBlockEntry {
                    params: vec![],
                    results: vec![],
                    local_variable_item_entries_without_args: vec![],
                },
            ],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(90)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(65)]);

        let result1 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(80)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(66)]);

        let result2 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(70)]);
        assert_eq!(result2.unwrap(), vec![ForeignValue::UInt32(67)]);

        let result3 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(60)]);
        assert_eq!(result3.unwrap(), vec![ForeignValue::UInt32(67)]);

        let result4 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(50)]);
        assert_eq!(result4.unwrap(), vec![ForeignValue::UInt32(68)]);

        let result5 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(40)]);
        assert_eq!(result5.unwrap(), vec![ForeignValue::UInt32(68)]);
    }

    #[test]
    fn test_interpreter_control_flow_structure_loop() {
        // func $accu (n/0:i32) -> (i32)
        //     (local sum/1:i32)
        //     (block 1 1) ()->()
        //                              ;; break if n==0
        //         (local_load32 1 0)
        //         i32_eqz
        //         (break_nez 0)
        //                              ;; sum = sum + n
        //         (local_load32 1 0)
        //         (local_load32 1 1)
        //         i32_add
        //         (local_store32 1 1)
        //                              ;; n = n - 1
        //         (local_load32 0)
        //         (i32_dec)
        //         (local_store32 0)
        //                              ;; recur
        //         (recur 0)
        //     end
        //     (local_load32 1)
        // end
        //
        // assert (10) -> (55)
        // assert (100) -> (5050)

        let code0 = BytecodeWriter::new()
            .append_opcode_i32_i32(Opcode::block, 1, 1)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 0)
            .append_opcode(Opcode::i32_eqz)
            .append_opcode_i16_i32(Opcode::break_nez, 0, 0x42)
            // sum = sum + n
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 1)
            .append_opcode(Opcode::i32_add)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 1, 0, 1)
            // n = n - 1
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 0)
            .append_opcode_i16(Opcode::i32_dec, 1)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 1, 0, 0)
            //
            .append_opcode_i16_i32(Opcode::recur, 0, 0x44)
            // block end
            .append_opcode(Opcode::end)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32],                  // params
            vec![DataType::I32],                  // results
            vec![LocalVariableEntry::from_i32()], // local vars
            code0,
            vec![HelperBlockEntry {
                params: vec![],
                results: vec![],
                local_variable_item_entries_without_args: vec![],
            }],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(10)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(55)]);

        let result1 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(100)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(5050)]);
    }

    #[test]
    fn test_interpreter_control_flow_structure_loop_with_parameters_and_local_vars() {
        // func $accu (i32) -> (i32)
        //     zero                     ;; sum
        //     (local_load32 0 0)       ;; n
        //     (block 1 1) (sum/0:i32, n/1:i32)->(i32)
        //         (local_load32 0 0)   ;; load sum
        //                              ;; break if n==0
        //         (local_load32 0 1)
        //         i32_eqz
        //         (break_nez 0)
        //                              ;; sum + n
        //         (local_load32 0 0)
        //         (local_load32 0 1)
        //         i32_add
        //                              ;; n - 1
        //         (local_load32 0 1)
        //         (i32_dec 1)
        //                              ;; recur
        //         (recur 0)
        //     end
        // end
        //
        // assert (10) -> (55)
        // assert (100) -> (5050)

        let code0 = BytecodeWriter::new()
            .append_opcode(Opcode::zero)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            //
            .append_opcode_i32_i32(Opcode::block, 1, 1)
            // load sum
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            // break if n==0
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_eqz)
            .append_opcode_i16_i32(Opcode::break_nez, 0, 0x32)
            // sum + n
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_add)
            // n - 1
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16(Opcode::i32_dec, 1)
            // recur
            .append_opcode_i16_i32(Opcode::recur, 0, 0x3c)
            // block end
            .append_opcode(Opcode::end)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32], // params
            vec![DataType::I32], // results
            vec![],              // local vars
            code0,
            vec![HelperBlockEntry {
                params: vec![DataType::I32, DataType::I32],
                results: vec![DataType::I32],
                local_variable_item_entries_without_args: vec![],
            }],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(10)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(55)]);

        let result1 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(100)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(5050)]);
    }

    #[test]
    fn test_interpreter_control_flow_structure_loop_with_optimized_inst_break_nez() {
        // func $accu_optimized (i32) -> (i32)
        //     zero                     ;; sum
        //     (local_load32 0 0)       ;; n
        //     (block 1 1) (sum/0:i32, n/1:i32)->(i32)
        //         (local_load32 0 0)   ;; load sum
        //                              ;; break if n==0
        //         (local_load32 0 1)
        //         i32_eqz
        //         (break_nez 0)
        //         drop                 ;; drop sum
        //                              ;; sum + n
        //         (local_load32 0 0)
        //         (local_load32 0 1)
        //         i32_add
        //                              ;; n - 1
        //         (local_load32 0 1)
        //         (i32_dec 1)
        //                              ;; recur
        //         (recur 0)
        //     end
        // end
        //
        // assert (10) -> (55)
        // assert (100) -> (5050)

        let code0 = BytecodeWriter::new()
            .append_opcode(Opcode::zero)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            //
            .append_opcode_i32_i32(Opcode::block, 1, 1)
            // load sum
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            // break if n==0
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_eqz)
            .append_opcode_i16_i32(Opcode::break_nez, 0, 0x32)
            // drop sum
            .append_opcode(Opcode::drop)
            // sum + n
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_add)
            // n - 1
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16(Opcode::i32_dec, 1)
            // recur
            .append_opcode_i16_i32(Opcode::recur, 0, 0x3c)
            // block end
            .append_opcode(Opcode::end)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32], // params
            vec![DataType::I32], // results
            vec![],              // local vars
            code0,
            vec![HelperBlockEntry {
                params: vec![DataType::I32, DataType::I32],
                results: vec![DataType::I32],
                local_variable_item_entries_without_args: vec![],
            }],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(10)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(55)]);

        let result1 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(100)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(5050)]);
    }

    #[test]
    fn test_interpreter_control_flow_structure_loop_variant() {
        // func $acc (n/0:i32) -> (i32)
        //     zero                     ;; sum
        //     (local_load32 0 0)       ;; n
        //     (block 1 1) (i32, i32)->(i32)
        //                              ;; sum = sum + n
        //         (local_load32 0 0)
        //         (local_load32 0 1)
        //         i32_add
        //         (local_store32 0 0)
        //                              ;; n = n - 1
        //         (local_load32 0 3)
        //         (i32_dec 1)
        //         (local_store32 0 1)
        //                              ;; load sum, n
        //         (local_load32 0 0)
        //         (local_load32 0 1)
        //                              ;; load n
        //                              ;; recur if n > 0
        //         (local_load32 0 1)
        //         zero
        //         i32_gt
        //         (recur_nez 0)
        //         drop               ;; drop n, keep sum
        //     end
        // end
        //
        // assert (10) -> (55)
        // assert (100) -> (5050)

        let code0 = BytecodeWriter::new()
            .append_opcode(Opcode::zero)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            //
            .append_opcode_i32_i32(Opcode::block, 1, 1)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_add)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 0, 0)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16(Opcode::i32_dec, 1)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 0, 1)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::zero)
            .append_opcode(Opcode::i32_gt_u)
            .append_opcode_i16_i32(Opcode::recur_nez, 0, 0x4c)
            //
            .append_opcode(Opcode::drop)
            // block end
            .append_opcode(Opcode::end)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32], // params
            vec![DataType::I32], // results
            vec![],              // local vars
            code0,
            vec![HelperBlockEntry {
                params: vec![DataType::I32, DataType::I32],
                results: vec![DataType::I32],
                local_variable_item_entries_without_args: vec![],
            }],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(10)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(55)]);

        let result1 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(100)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(5050)]);
    }

    #[test]
    fn test_interpreter_control_flow_structure_loop_with_local_vars() {
        // note:
        //
        // also test the block-level local variables

        // func $acc (n/0:i32) -> (i32)
        //     zero                     ;; sum
        //     (local_load32 0 0)       ;; n
        //     (block 1 1) (p_sum/0:i32, p_n/1:i32)->(i32)
        //         (local new_sum/2:i32 new_n/3:i32)
        //                              ;; new_sum = p_sum + p_n
        //         (local_load32 0 0)
        //         (local_load32 0 1)
        //         i32_add
        //         (local_store32 0 2)
        //                              ;; new_n = p_n - 1
        //         (local_load32 0 1)
        //         (i32_dec 1)
        //         (local_store32 0 3)
        //                              ;; load new_sum, new_n
        //         (local_load32 0 2)
        //         (local_load32 0 3)
        //                              ;; recur if new_n > 0
        //         (local_load32 0 3)   ;; load new_n
        //         zero
        //         i32_gt
        //         (recur_nez 0)
        //
        //         drop               ;; drop new_n, keep new_sum
        //     end
        // end
        //
        // assert (10) -> (55)
        // assert (100) -> (5050)

        let code0 = BytecodeWriter::new()
            .append_opcode(Opcode::zero)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            //
            .append_opcode_i32_i32(Opcode::block, 1, 1)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_add)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 0, 2)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16(Opcode::i32_dec, 1)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 0, 3)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 2)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 3)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 3)
            .append_opcode(Opcode::zero)
            .append_opcode(Opcode::i32_gt_u)
            .append_opcode_i16_i32(Opcode::recur_nez, 0, 0x4c)
            //
            .append_opcode(Opcode::drop)
            // block end
            .append_opcode(Opcode::end)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32], // params
            vec![DataType::I32], // results
            vec![],              // local vars
            code0,
            vec![HelperBlockEntry {
                params: vec![DataType::I32, DataType::I32],
                results: vec![DataType::I32],
                local_variable_item_entries_without_args: vec![
                    LocalVariableEntry::from_i32(),
                    LocalVariableEntry::from_i32(),
                ],
            }],
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(10)]);
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(55)]);

        let result1 = process_function(&mut thread_context0, 0, 0, &[ForeignValue::UInt32(100)]);
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(5050)]);
    }

    #[test]
    fn test_interpreter_control_flow_function_tco() {
        // func $accu (sum/0:i32, n/1:i32) -> (i32)
        //                              ;; sum = sum + n
        //     (local_load32 0 0)
        //     (local_load32 0 1)
        //     i32_add
        //     (local_store32 0 0)
        //                              ;; n = n - 1
        //     (local_load32 0 1)
        //     (i32_dec 1)
        //     (local_store32 0 1)
        //                              ;; if n > 0 recur (sum,n)
        //     (local_load32 0 1)
        //     zero
        //     i32_gt
        //     (block_nez 1) () -> ()
        //         (local_load32 0 0)
        //         (local_load32 0 1)
        //         (recur 1)
        //     end
        //     (local_load32 0 0)       ;; load sum
        // end
        //
        // assert (0, 10) -> (55)
        // assert (0, 100) -> (5050)

        let code0 = BytecodeWriter::new()
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_add)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 0, 0)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16(Opcode::i32_dec, 1)
            .append_opcode_i16_i16_i16(Opcode::local_store32, 0, 0, 1)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::zero)
            .append_opcode(Opcode::i32_gt_u)
            .append_opcode_i32_i32(Opcode::block_nez, 1, 0x26)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 1)
            .append_opcode_i16_i32(Opcode::recur, 1, 0)
            // block end
            .append_opcode(Opcode::end)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32, DataType::I32], // params
            vec![DataType::I32],                // results
            vec![],                             // local vars
            code0,
            vec![HelperBlockEntry {
                params: vec![],
                results: vec![],
                local_variable_item_entries_without_args: vec![],
            }], // blocks
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::UInt32(0), ForeignValue::UInt32(10)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(55)]);

        let result1 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::UInt32(0), ForeignValue::UInt32(100)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(5050)]);
    }

    #[test]
    fn test_interpreter_control_flow_function_tco_with_optimized_inst_recur_nez() {
        // func $accu_opti (sum:i32, n:i32) -> (i32)
        //                          ;; sum + n
        //     (local_load32 0)
        //     (local_load32 1)
        //     i32_add
        //                          ;; n - 1
        //     (local_load32 1)
        //     (i32_dec 1)
        //                          ;; recur if n>0
        //     duplicate
        //     zero
        //     i32_gt
        //     (recur_nez 0)
        //                          ;; drop n, keep sum
        //     drop
        // end
        //
        // assert (0, 10) -> (55)
        // assert (0, 100) -> (5050)

        let code0 = BytecodeWriter::new()
            // sum + n
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_add)
            // n - 1
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode_i16(Opcode::i32_dec, 1)
            //
            .append_opcode(Opcode::duplicate)
            .append_opcode(Opcode::zero)
            .append_opcode(Opcode::i32_gt_u)
            .append_opcode_i16_i32(Opcode::recur_nez, 0, 0)
            //
            .append_opcode(Opcode::drop)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32, DataType::I32], // params
            vec![DataType::I32],                // results
            vec![],                             // local vars
            code0,
            vec![], // blocks
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::UInt32(0), ForeignValue::UInt32(10)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(55)]);

        let result1 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::UInt32(0), ForeignValue::UInt32(100)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(5050)]);
    }

    #[test]
    fn test_interpreter_control_flow_function_tco_inside_if() {
        // func $accu_opti (sum:i32, n:i32) -> (i32)
        //     (local_load32 0 1)               ;; load n
        //     i32_eqz
        //     (block_alt 1 1) () -> (i32)      ;; if n == 0
        //         (local_load32 1 0)           ;; then sum
        //     (break 0)                        ;; else
        //                                      ;; sum + n
        //         (local_load32 1 0)
        //         (local_load32 1 1)
        //         i32_add
        //                                      ;; n - 1
        //         (local_load32 1 1)
        //         (i32_dec 1)
        //         (recur 1)                    ;; recur
        //     end
        // end
        //
        // assert (0, 10) -> (55)
        // assert (0, 100) -> (5050)

        let code0 = BytecodeWriter::new()
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            .append_opcode(Opcode::i32_eqz)
            .append_opcode_i32_i32_i32(Opcode::block_alt, 1, 1, 0x20)
            // then
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 0)
            .append_opcode_i16_i32(Opcode::break_, 0, 0x32)
            // else
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 1)
            .append_opcode(Opcode::i32_add)
            //
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 1, 0, 1)
            .append_opcode_i16(Opcode::i32_dec, 1)
            //
            .append_opcode_i16_i32(Opcode::recur, 1, 0)
            // block end
            .append_opcode(Opcode::end)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function_and_blocks(
            vec![DataType::I32, DataType::I32], // params
            vec![DataType::I32],                // results
            vec![],                             // local vars
            code0,
            vec![HelperBlockEntry {
                params: vec![], // 'block_alt' has no PARAMS
                results: vec![DataType::I32],
                local_variable_item_entries_without_args: vec![],
            }], // blocks
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::UInt32(0), ForeignValue::UInt32(10)],
        );
        assert_eq!(result0.unwrap(), vec![ForeignValue::UInt32(55)]);

        let result1 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::UInt32(0), ForeignValue::UInt32(100)],
        );
        assert_eq!(result1.unwrap(), vec![ForeignValue::UInt32(5050)]);
    }
}
