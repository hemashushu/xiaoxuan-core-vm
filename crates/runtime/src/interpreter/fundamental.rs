// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use crate::thread::Thread;

use super::InterpretResult;

pub fn nop(_thread: &mut Thread) -> InterpretResult {
    InterpretResult::MoveOn(2)
}

pub fn break_(_thread: &mut Thread) -> InterpretResult {
    InterpretResult::Break
}

pub fn drop(thread: &mut Thread) -> InterpretResult {
    thread.stack.drop();
    InterpretResult::MoveOn(2)
}

pub fn duplicate(thread: &mut Thread) -> InterpretResult {
    thread.stack.duplicate();
    InterpretResult::MoveOn(2)
}

pub fn swap(thread: &mut Thread) -> InterpretResult {
    let a = thread.stack.pop_i64_u();
    let b = thread.stack.pop_i64_u();
    thread.stack.push_i64_u(a);
    thread.stack.push_i64_u(b);
    InterpretResult::MoveOn(2)
}

pub fn zero(thread: &mut Thread) -> InterpretResult {
    thread.stack.push_i64_u(0);
    InterpretResult::MoveOn(2)
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        load_modules_binary,
        utils::{build_module_binary_with_single_function, BytecodeReader, BytecodeWriter},
    };
    use ancvm_types::{opcode::Opcode, DataType, ForeignValue};

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

        // bytecodes
        //
        // 0x0000 swap
        // 0x0002 end
        //
        // (i32) -> (i32, i32)

        let code3 = BytecodeWriter::new()
            .write_opcode(Opcode::swap)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary3 = build_module_binary_with_single_function(
            vec![DataType::I32, DataType::I32], // params
            vec![DataType::I32, DataType::I32], // results
            code3,
            vec![], // local vars
        );

        let image3 = load_modules_binary(vec![&binary3]).unwrap();
        let mut thread3 = Thread::new(&image3);

        let result3 = process_function(
            &mut thread3,
            0,
            0,
            &vec![ForeignValue::UInt32(211), ForeignValue::UInt32(223)],
        );
        assert_eq!(
            result3.unwrap(),
            vec![ForeignValue::UInt32(223), ForeignValue::UInt32(211)]
        );

        // bytecodes
        //
        // 0x0000 zero
        // 0x0002 end
        //
        // (i32) -> (i32, i32)

        let code4 = BytecodeWriter::new()
            .write_opcode(Opcode::zero)
            .write_opcode(Opcode::end)
            .to_bytes();

        let binary4 = build_module_binary_with_single_function(
            vec![DataType::I32],                // params
            vec![DataType::I32, DataType::I32], // results
            code4,
            vec![], // local vars
        );

        let image4 = load_modules_binary(vec![&binary4]).unwrap();
        let mut thread4 = Thread::new(&image4);

        let result4 = process_function(&mut thread4, 0, 0, &vec![ForeignValue::UInt32(233)]);
        assert_eq!(
            result4.unwrap(),
            vec![ForeignValue::UInt32(233), ForeignValue::UInt32(0)]
        );
    }
}
