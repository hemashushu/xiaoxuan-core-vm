// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use anc_context::thread_context::ThreadContext;
use libc::{clock_gettime, timespec, CLOCK_MONOTONIC};

use crate::handler::Handler;

// ref:
// https://linux.die.net/man/3/clock_gettime
pub fn time_now(_handler: &Handler, thread_context: &mut ThreadContext) {
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

    use anc_context::resource::Resource;
    use anc_image::{
        bytecode_writer::BytecodeWriterHelper,
        utils::helper_build_module_binary_with_single_function,
    };
    use anc_isa::{opcode::Opcode, OperandDataType};
    use libc::{clock_gettime, timespec, CLOCK_MONOTONIC};

    use crate::{
        envcall_num::EnvCallNum, handler::Handler, in_memory_resource::InMemoryResource,
        process::process_function,
    };

    #[test]
    fn test_envcall_time_now() {
        // () -> (i64, i32)

        let code0 = BytecodeWriterHelper::new()
            .append_opcode_i32(Opcode::envcall, EnvCallNum::time_now as u32)
            .append_opcode(Opcode::end)
            .to_bytes();

        let binary0 = helper_build_module_binary_with_single_function(
            vec![],                                           // params
            vec![OperandDataType::I64, OperandDataType::I32], // results
            vec![],                                           // local variables
            code0,
        );

        let handler = Handler::new();
        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let result0 = process_function(&handler, &mut thread_context0, 0, 0, &[]);
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
}
