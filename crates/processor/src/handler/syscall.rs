// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use ancvm_context::thread_context::ThreadContext;
use syscall_util::call::{
    syscall_with_1_arg, syscall_with_2_args, syscall_with_3_args, syscall_with_4_args,
    syscall_with_5_args, syscall_with_6_args, syscall_without_args,
};

use super::HandleResult;

type SysCallHandlerFunc = fn(&mut ThreadContext, usize) -> Result<usize, usize>;

const MAX_SYSCALL_TYPE_NUMBER: usize = 1 + 6; // 1 no args + 6 types with args
static mut HANDLERS: [SysCallHandlerFunc; MAX_SYSCALL_TYPE_NUMBER] =
    [unreachable; MAX_SYSCALL_TYPE_NUMBER];

pub fn syscall(thread_context: &mut ThreadContext) -> HandleResult {
    // (operand args..., syscall_num:i32 params_count: i32) -> (return_value:i64, error_no:i32)
    //
    // the syscall arguments should be pushed on the stack first, e.g.
    //
    // | params_count   |
    // | syscall_num    |
    // | arg6           |
    // | arg5           |
    // | arg4           |
    // | arg3           |
    // | arg2           |                  | error no       |
    // | arg1           |     return -->   | return value   |
    // | ...            |                  | ...            |
    // \----------------/ <-- stack start  \----------------/
    //
    // when a syscall complete, the return value is store into the 'rax' register,
    // if the operation fails, the value is a negative value (rax < 0).
    // there is no 'errno' if invoke syscall by assembly directly.

    let params_count = thread_context.stack.pop_i32_u();
    let syscall_num = thread_context.stack.pop_i32_u();

    let handler = unsafe { &HANDLERS[params_count as usize] };
    let result = handler(thread_context, syscall_num as usize);

    // push the result on the stack

    match result {
        Ok(value) => {
            thread_context.stack.push_i64_u(value as u64);
            thread_context.stack.push_i32_u(0);
        }
        Err(error_no) => {
            thread_context.stack.push_i64_u(0);
            thread_context.stack.push_i32_u(error_no as u32);
        }
    }

    HandleResult::Move(2)
}

// note:
//
// ensure this initialization is only called once
pub fn init_syscall_handlers() {
    let handlers = unsafe { &mut HANDLERS };

    handlers[0] = handle_syscall_without_args;
    handlers[1] = handle_syscall_with_1_arg;
    handlers[2] = handle_syscall_with_2_args;
    handlers[3] = handle_syscall_with_3_args;
    handlers[4] = handle_syscall_with_4_args;
    handlers[5] = handle_syscall_with_5_args;
    handlers[6] = handle_syscall_with_6_args;
}

fn unreachable(_thread_context: &mut ThreadContext, _number: usize) -> Result<usize, usize> {
    unreachable!()
}

fn handle_syscall_without_args(
    _thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    unsafe { syscall_without_args(number) }
}

fn handle_syscall_with_1_arg(
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 1;
    let args_u8 = thread_context.stack.pop_operands(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_u8.as_ptr() as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_1_arg(number, args[0]) }
}

fn handle_syscall_with_2_args(
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 2;
    let args_u8 = thread_context.stack.pop_operands(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_u8.as_ptr() as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_2_args(number, args[0], args[1]) }
}

fn handle_syscall_with_3_args(
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 3;
    let args_u8 = thread_context.stack.pop_operands(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_u8.as_ptr() as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_3_args(number, args[0], args[1], args[2]) }
}

fn handle_syscall_with_4_args(
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 4;
    let args_u8 = thread_context.stack.pop_operands(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_u8.as_ptr() as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_4_args(number, args[0], args[1], args[2], args[3]) }
}

fn handle_syscall_with_5_args(
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 5;
    let args_u8 = thread_context.stack.pop_operands(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_u8.as_ptr() as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_5_args(number, args[0], args[1], args[2], args[3], args[4]) }
}

fn handle_syscall_with_6_args(
    thread_context: &mut ThreadContext,
    number: usize,
) -> Result<usize, usize> {
    const ARGS_COUNT: usize = 6;
    let args_u8 = thread_context.stack.pop_operands(ARGS_COUNT);
    let args = unsafe { std::slice::from_raw_parts(args_u8.as_ptr() as *const usize, ARGS_COUNT) };
    unsafe { syscall_with_6_args(number, args[0], args[1], args[2], args[3], args[4], args[5]) }
}

#[cfg(test)]
mod tests {
    use ancvm_binary::{
        bytecode_writer::BytecodeWriter, utils::helper_build_module_binary_with_single_function,
    };
    use ancvm_context::resource::Resource;
    use syscall_util::{errno::Errno, number::SysCallNum};
    use ancvm_isa::{opcode::Opcode, OperandDataType, ForeignValue};

        use crate::{
        handler::Handler, in_memory_resource::InMemoryResource, process::process_function,
    };

    #[test]
    fn test_interpreter_syscall_without_args() {
        // fn $test () -> (result:i64 errno:i32)

        // syscall:
        // `pid_t getpid(void);`

        // bytecode:
        //
        // 0x0000  80 01 00 00  27 00 00 00    i32.imm           0x00000027
        // 0x0008  80 01 00 00  00 00 00 00    i32.imm           0x00000000
        // 0x0010  03 0b                       syscall
        // 0x0012  00 0a                       end

        let code0 = BytecodeWriterHelper::new()
            // push syscall args from 1 to 6
            // -none-
            // prepare syscall
            .append_opcode_i32(Opcode::imm_i32, SysCallNum::getpid as u32) // syscall num
            .append_opcode_i32(Opcode::imm_i32, 0) // the amount of syscall args
            //
            .append_opcode(Opcode::syscall)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", format_bytecode_as_text(&code0));

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
        let result_values0 = result0.unwrap();

        let pid = std::process::id();

        assert!(matches!(result_values0[0], ForeignValue::U64(value) if value == pid as u64));
        assert_eq!(result_values0[1], ForeignValue::U32(0));
    }

    #[test]
    fn test_interpreter_syscall_with_2_args() {
        // fn $test (buf_addr:i64, buf_len:i32) -> (result:i64 errno:i32)

        // syscall:
        // `char *getcwd(char buf[.size], size_t size);`

        // bytecode:
        //
        // 0x0000  00 02 00 00  00 00 00 00    local.load64_i64  rev:0   off:0x00  idx:0
        // 0x0008  02 02 00 00  00 00 01 00    local.load32_i32  rev:0   off:0x00  idx:1
        // 0x0010  80 01 00 00  4f 00 00 00    i32.imm           0x0000004f
        // 0x0018  80 01 00 00  02 00 00 00    i32.imm           0x00000002
        // 0x0020  03 0b                       syscall
        // 0x0022  00 0a                       end

        let code0 = BytecodeWriterHelper::new()
            // push syscall args from 1 to 6
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0)
            .append_opcode_i16_i16_i16(Opcode::local_load32_i32, 0, 0, 1)
            // prepare syscall
            .append_opcode_i32(Opcode::imm_i32, SysCallNum::getcwd as u32) // syscall num
            .append_opcode_i32(Opcode::imm_i32, 2) // the amount of syscall args
            //
            .append_opcode(Opcode::syscall)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", format_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function(
            vec![OperandDataType::I64, OperandDataType::I64], // params
            vec![OperandDataType::I64, OperandDataType::I32], // results
            vec![],                             // local vars
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        const BUF_LENGTH: u32 = 1024;
        let buf = [0u8; BUF_LENGTH as usize];
        let buf_addr = buf.as_ptr() as u64;

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[
                ForeignValue::U64(buf_addr),
                ForeignValue::U32(BUF_LENGTH),
            ],
        );

        let results0 = result0.unwrap();

        // note
        //
        // the syscall 'getcwd' in the libc returns the pointer to the buf, but the
        // raw syscall 'getcwd' returns the length of the path (includes the NULL terminated char)

        let null_pos = buf.iter().position(|u| *u == 0).unwrap();

        assert!(
            matches!(results0[0], ForeignValue::U64(value) if value == (null_pos + 1) as u64)
        );
        assert_eq!(results0[1], ForeignValue::U32(0));

        let path0 = String::from_utf8_lossy(&buf[0..null_pos]);
        let cwd = std::env::current_dir().unwrap();
        let cwd0 = cwd.as_os_str().to_string_lossy();
        assert_eq!(path0, cwd0);
    }

    #[test]
    fn test_interpreter_syscall_error_no() {
        // fn $test (file_path_buf_addr:i64) -> (result:i64 errno:i32)

        // syscall:
        // `int open(const char *pathname, int flags)`

        // bytecode:
        //
        // 0x0000  00 02 00 00  00 00 00 00    local.load64_i64  rev:0   off:0x00  idx:0
        // 0x0008  80 01 00 00  00 00 00 00    i32.imm           0x00000000
        // 0x0010  80 01 00 00  02 00 00 00    i32.imm           0x00000002
        // 0x0018  80 01 00 00  02 00 00 00    i32.imm           0x00000002
        // 0x0020  03 0b                       syscall
        // 0x0022  00 0a                       end

        let code0 = BytecodeWriterHelper::new()
            // push syscall args from 1 to 6
            .append_opcode_i16_i16_i16(Opcode::local_load_i64, 0, 0, 0) // file path addr
            .append_opcode_i32(Opcode::imm_i32, 0) // open flags
            // prepare syscall
            .append_opcode_i32(Opcode::imm_i32, SysCallNum::open as u32) // syscall num
            .append_opcode_i32(Opcode::imm_i32, 2) // the amount of syscall args
            //
            .append_opcode(Opcode::syscall)
            //
            .append_opcode(Opcode::end)
            .to_bytes();

        // println!("{}", format_bytecode_as_text(&code0));

        let binary0 = helper_build_module_binary_with_single_function(
            vec![OperandDataType::I64],                // params
            vec![OperandDataType::I64, OperandDataType::I32], // results
            vec![],                             // local vars
            code0,
        );

        let resource0 = InMemoryResource::new(vec![binary0]);
        let process_context0 = resource0.create_process_context().unwrap();
        let mut thread_context0 = process_context0.create_thread_context();

        let file_path0 = b"/this/file/should/not/exist\0";
        let file_path1 = b"/dev/zero\0";

        let file_path_addr0 = file_path0.as_ptr() as usize;
        let file_path_addr1 = file_path1.as_ptr() as usize;

        let result0 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U64(file_path_addr0 as u64)],
        );
        let results0 = result0.unwrap();

        assert_eq!(
            results0,
            vec![
                ForeignValue::U64(0),
                ForeignValue::U32(Errno::ENOENT as u32)
            ]
        );

        let result1 = process_function(
            &mut thread_context0,
            0,
            0,
            &[ForeignValue::U64(file_path_addr1 as u64)],
        );
        let results1 = result1.unwrap();

        assert!(matches!(results1[0], ForeignValue::U64(value) if value > 0));
        assert_eq!(results1[1], ForeignValue::U32(0));
    }
}
