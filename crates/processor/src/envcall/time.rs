// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::thread_context::ThreadContext;
use libc::{clock_gettime, timespec, CLOCK_MONOTONIC};

// ref:
// https://linux.die.net/man/3/clock_gettime
pub fn time_now(thread_context: &mut ThreadContext) {
    // `fn () -> (seconds:u64, nano_seconds:u32)`

    let mut t = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    unsafe {
        clock_gettime(CLOCK_MONOTONIC, &mut t);
    }

    thread_context.stack.push_i64_u(t.tv_sec as u64);
    thread_context.stack.push_i32_u(t.tv_nsec as u32);
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use ancvm_binary::{
        bytecode_writer::BytecodeWriter, utils::helper_build_module_binary_with_single_function,
    };
    use ancvm_context::program_resource::ProgramResource;
    use ancvm_isa::{envcallcode::EnvCallCode, opcode::Opcode, OperandDataType};
    use libc::{clock_gettime, timespec, CLOCK_MONOTONIC};

    use crate::{in_memory_program_resource::InMemoryProgramResource, interpreter::process_function};

    #[test]
    fn test_envcall_time_now() {
        // () -> (i64, i32)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::envcall, EnvCallCode::time_now as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                             // params
            vec![OperandDataType::I64, OperandDataType::I32], // results
            vec![],                             // local vars
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        let results0 = result0.unwrap();

        let secs = results0[0].as_u64().unwrap();
        let nanos = results0[1].as_u32().unwrap();
        let dur_before = Duration::new(secs, nanos);

        let mut t: timespec = timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe {
            clock_gettime(CLOCK_MONOTONIC, &mut t);
        }
        let dur_after = Duration::new(t.tv_sec as u64, t.tv_nsec as u32);

        let duration = dur_after - dur_before;
        assert!(duration.as_millis() < 1000);
    }
}
