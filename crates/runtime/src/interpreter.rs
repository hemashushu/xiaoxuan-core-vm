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
