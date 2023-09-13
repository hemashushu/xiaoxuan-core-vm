// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

use ancvm_types::ecallcode::{ECallCode, MAX_ECALLCODE_NUMBER};

use crate::{processor::InterpretResult, thread::Thread};

pub mod heap;
pub mod info;

type EnvCallHandlerFunc = fn(&mut Thread) -> Result<(), usize>;

fn unreachable(_thread: &mut Thread) -> Result<(), usize> {
    unreachable!("Invalid environment call number.")
}

pub static mut handlers: Vec<EnvCallHandlerFunc> = vec![];

pub fn init_ecall_handlers() {
    let placehold: Vec<EnvCallHandlerFunc> = vec![unreachable; MAX_ECALLCODE_NUMBER];

    unsafe {
        handlers.extend(placehold.iter());

        //
        handlers[ECallCode::runtime_name as usize] = info::runtime_name;
        handlers[ECallCode::runtime_version as usize] = info::runtime_version;
    }
}

pub fn ecall(thread: &mut Thread) -> InterpretResult {
    // (param env_func_num:i32)

    let env_func_num = thread.get_param_i32();
    let func = unsafe { handlers[env_func_num as usize] };
    let result = func(thread);

    match result {
        Ok(_) => InterpretResult::MoveOn(8),
        Err(err_code) => InterpretResult::EnvError(err_code),
    }
}

#[cfg(test)]
mod tests {
    use ancvm_binary::load_modules_binary;
    use ancvm_types::{ecallcode::ECallCode, opcode::Opcode, DataType, ForeignValue};

    use crate::{
        processor::Processor,
        thread::Thread,
        utils::{test_helper::build_module_binary_with_single_function, BytecodeWriter, BytecodeReader},
        RUNTIME_MAJOR_VERSION, RUNTIME_MINOR_VERSION, RUNTIME_PATCH_VERSION,
    };

    #[test]
    fn test_ecall_runtime_info() {
        let processor = Processor::new();

        // bytecodes
        //
        // 0x0000 ecall                257
        // 0x0008 end
        //
        // () -> (i64)

        let code0 = BytecodeWriter::new()
            .write_opcode_i32(Opcode::ecall, ECallCode::runtime_version as u32)
            .write_opcode(Opcode::end)
            .to_bytes();

        // let text = BytecodeReader::new(&code0).to_text();
        // println!("{}", text);

        let binary0 = build_module_binary_with_single_function(
            vec![],              // params
            vec![DataType::I64], // results
            code0,
            vec![], // local varslist which
        );

        let image0 = load_modules_binary(vec![&binary0]).unwrap();
        let mut thread0 = Thread::new(&image0);

        let result0 = processor.process_function(&mut thread0, 0, 0, &vec![]);

        let expect_version_number = RUNTIME_PATCH_VERSION as u64
            | (RUNTIME_MINOR_VERSION as u64) << 16
            | (RUNTIME_MAJOR_VERSION as u64) << 32;

        assert_eq!(
            result0.unwrap(),
            vec![ForeignValue::UInt64(expect_version_number)]
        );
    }
}
