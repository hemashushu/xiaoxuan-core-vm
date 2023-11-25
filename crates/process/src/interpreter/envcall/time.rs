// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::{thread, time::Duration};

use ancvm_program::thread_context::ThreadContext;
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

// ref:
// https://linux.die.net/man/2/nanosleep
pub fn time_sleep(thread_context: &mut ThreadContext) {
    // `fn (milliseconds:u64) -> ()`

    let milliseconds = thread_context.stack.pop_i64_u();
    thread::sleep(Duration::from_millis(milliseconds));
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use ancvm_binary::{
        bytecode_writer::BytecodeWriter, utils::helper_build_module_binary_with_single_function,
    };
    use ancvm_program::program_source::ProgramSource;
    use ancvm_types::{envcallcode::EnvCallCode, opcode::Opcode, DataType};
    use libc::{clock_gettime, timespec, CLOCK_MONOTONIC};

    use crate::{in_memory_program_source::InMemoryProgramSource, interpreter::process_function};

    #[test]
    fn test_envcall_time_now() {
        // () -> (i64, i32)

        let code0 = BytecodeWriter::new()
            .append_opcode_i32(Opcode::envcall, EnvCallCode::time_now as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                             // params
            vec![DataType::I64, DataType::I32], // results
            vec![],                             // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        let results0 = result0.unwrap();

        let secs = results0[0].as_u64();
        let nanos = results0[1].as_u32();
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

    #[test]
    fn test_envcall_time_sleep() {
        // () -> ()

        let code0 = BytecodeWriter::new()
            .append_opcode_pesudo_i64(Opcode::i64_imm, 1000)
            .append_opcode_i32(Opcode::envcall, EnvCallCode::time_sleep as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![], // params
            vec![], // results
            vec![], // local vars
            code0,
        );

        let program_source0 = InMemoryProgramSource::new(vec![binary0]);
        let program0 = program_source0.build_program().unwrap();
        let mut thread_context0 = program0.create_thread_context();

        let before = Instant::now();
        let _ = process_function(&mut thread_context0, 0, 0, &[]);
        let after = Instant::now();

        let duration = after.duration_since(before);
        let ms = duration.as_millis() as u64;
        assert!(ms > 500);
    }
}
