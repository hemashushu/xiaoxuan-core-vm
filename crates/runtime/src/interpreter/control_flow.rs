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
    do_block(thread, type_index)
}

pub fn block_nez(thread: &mut Thread) -> InterpretResult {
    let condition = thread.stack.pop_i32_u();
    let (type_index, alt_inst_offset) = thread.get_param_i32_i32();

    if condition == 0 {
        InterpretResult::Move(alt_inst_offset as isize)
    } else {
        do_block(thread, type_index)
    }
}

fn do_block(thread: &mut Thread, type_index: u32) -> InterpretResult {
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
        InterpretResult::Jump(return_pc)
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
        // instruction which nexts to the instruction 'call'.
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
