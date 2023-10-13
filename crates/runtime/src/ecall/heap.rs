// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_program::{resizeable_memory::ResizeableMemory, thread_context::ThreadContext};

pub fn heap_capacity(thread_context: &mut ThreadContext) {
    // `fn () -> pages:i64`
    let pages = thread_context.heap.get_capacity_in_pages();
    thread_context.stack.push_i64_u(pages as u64);
}

pub fn heap_resize(thread_context: &mut ThreadContext) {
    // `fn (pages:i64) -> new_pages:i64`
    let pages = thread_context.stack.pop_i64_u();
    let new_pages = thread_context.heap.resize(pages as usize);
    thread_context.stack.push_i64_u(new_pages as u64);
}

pub fn heap_fill(thread_context: &mut ThreadContext) {
    // `fn (address:i64, value:i8, count:i64)`
    let count = thread_context.stack.pop_i64_u() as usize;
    let value = thread_context.stack.pop_i32_u() as u8;
    let address = thread_context.stack.pop_i64_u() as usize;

    thread_context.heap.fill(address, value, count);
}

pub fn heap_copy(thread_context: &mut ThreadContext) {
    // `fn (dst_address:i64, src_address:i64, length_in_bytes:i64)`

    let length_in_bytes = thread_context.stack.pop_i64_u() as usize;
    let src_address = thread_context.stack.pop_i64_u() as usize;
    let dst_address = thread_context.stack.pop_i64_u() as usize;

    thread_context
        .heap
        .copy(dst_address, src_address, length_in_bytes);
}

#[cfg(test)]
mod tests {
    use ancvm_binary::utils::{build_module_binary_with_single_function, BytecodeWriter};
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{ecallcode::ECallCode, opcode::Opcode, DataType, ForeignValue};

    use crate::{in_memory_program_source::InMemoryProgramSource, interpreter::process_function};

    #[test]
    fn test_ecall_heap_capacity() {
        // bytecodes
        //
        // 0x0000 ecall                261
        // 0x0008 i32_imm              0x2
        // 0x0010 ecall                262
        // 0x0018 i32_imm              0x4
        // 0x0020 ecall                262
        // 0x0028 i32_imm              0x1
        // 0x0030 ecall                262
        // 0x0038 ecall                261
        // 0x0040 end
        //
        // () -> (i64, i64, i64, i64, i64)

        let code0 = BytecodeWriter::new()
            // get the capacity
            .write_opcode_i32(Opcode::ecall, ECallCode::heap_capacity as u32)
            // resize - increase
            .write_opcode_i32(Opcode::i32_imm, 2)
            .write_opcode_i32(Opcode::ecall, ECallCode::heap_resize as u32)
            // resize - increase
            .write_opcode_i32(Opcode::i32_imm, 4)
            .write_opcode_i32(Opcode::ecall, ECallCode::heap_resize as u32)
            // resize - decrease
            .write_opcode_i32(Opcode::i32_imm, 1)
            .write_opcode_i32(Opcode::ecall, ECallCode::heap_resize as u32)
            // get the capcity
            .write_opcode_i32(Opcode::ecall, ECallCode::heap_capacity as u32)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary0 = build_module_binary_with_single_function(
            vec![], // params
            vec![
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
                DataType::I64,
            ], // results
            vec![], // local varslist which
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &vec![]);

        assert_eq!(
            result0.unwrap(),
            vec![
                ForeignValue::UInt64(0),
                ForeignValue::UInt64(2),
                ForeignValue::UInt64(4),
                ForeignValue::UInt64(1),
                ForeignValue::UInt64(1),
            ]
        );
    }
}
