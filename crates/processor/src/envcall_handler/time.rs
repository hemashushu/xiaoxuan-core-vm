// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::time::SystemTime;

use anc_context::thread_context::ThreadContext;

// The current time can be obtained using the `libc::clock_gettime` function.
// Example:
//
// ```rust
// use libc::{clock_gettime, timespec, CLOCK_MONOTONIC};
//
// let mut t = timespec {
//     tv_sec: 0,
//     tv_nsec: 0,
// };
//
// unsafe {
//     clock_gettime(CLOCK_MONOTONIC, &mut t);
// }
//
// /*Seconds */ t.tv_sec
// /*Nanoseconds */ t.tv_nsec
// ```
//
// Reference:
// https://linux.die.net/man/3/clock_gettime
pub fn time_now(thread_context: &mut ThreadContext) {
    // `fn () -> (seconds: u64, nano_seconds: u64)`

    let total_nanos = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_nanos(),
        Err(_) => 0, // SystemTime before UNIX EPOCH
    };

    let secs = (total_nanos / 1_000_000_000_u128) as u64;
    let nanos = (total_nanos % 1_000_000_000_u128) as u64;

    thread_context.stack.push_i64_u(secs);
    thread_context.stack.push_i64_u(nanos);
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use anc_context::program_source::ProgramSource;
    use anc_image::{
        bytecode_writer::BytecodeWriterHelper,
        utils::helper_build_module_binary_with_single_function,
    };
    use anc_isa::{opcode::Opcode, OperandDataType};

    use crate::{
        envcall_num::EnvCallNum, in_memory_program_source::InMemoryProgramSource,
        process::process_function,
    };

    #[test]
    fn test_envcall_time_now() {
        // () -> (i64, i64)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::envcall, EnvCallNum::time_now as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            &[],                                           // params
            &[OperandDataType::I64, OperandDataType::I64], // results
            &[],                                           // local variables
            code0,
        );

        let resource0 = InMemoryProgramSource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&mut thread_context0, 0, 0, &[]);
        let results0 = result0.unwrap();

        let secs = results0[0].as_u64();
        let nanos = results0[1].as_u64();

        let duration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        assert_eq!(duration.as_secs(), secs);
        assert!(nanos > 0);
    }
}
