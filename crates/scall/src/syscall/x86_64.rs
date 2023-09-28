// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// x86-64 ABI/calling convention of syscall
//
// | register | usage    |
// |----------|----------|
// | rax      | call num | also use for store the return value.
// | rdi      | 1st      |
// | rsi      | 2nd      |
// | rdx      | 3rd      |
// | r10      | 4th      | 'rcx' for standard function calling
// | r8       | 5th      |
// | r9       | 6th      |
//
// arguments over 6 are passed through the stack, as well as the variable arguments.
//
// /------------\ <-- stack start
// |   ...      |
// | 8th arg    | rbp + 24
// | 7th arg    | rbp + 16
// | rip        | rbp + 8, also return addr
// | rbp        | <-- rbp
// | saved regs | e.g. r12-r15
// |------------| <-- rsp
// |   ...      |
// | 128 bytes  | reserved area (red zone) for such as local variables
// |------------|
// |            |

// r10, r11 for temporary, as well as the registers above are not preserved
// across a function call, i.e., any of these registers may be used in a function
// without have to save the original value.
// r12-15 are callee saved.
//
// rcx and r11 are used for store the rip and rflags before syscall, when syscall is finish,
// the old values of rcx and r11 will be restore, all these are done automatictly by 'syscall' instruction.
// because the values of rcx and r11 will be modified by the syscall, so to keep things simple,
// it is better not to use these registers in the user program.

// syscall example: print "hello world" to stdout
//
// ```yasm
// STDOUT           equ     1
// SYS_NUM_WRITE    equ     1
// message          db      "Hello, World!"
// message_length   dq      13
//
// mov              rax,    SYS_NUM_WRITE
// mov              rdi,    STDOUT
// mov              rsi,    message
// mov              rdx,    qword [message_length]
// syscall
// ```
//
// ref:
// - YASM
//   https://yasm.tortall.net/
// - Rust inline assembly
//   https://doc.rust-lang.org/stable/reference/inline-assembly.html
use std::arch::asm;

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn syscall_without_args(num: usize) -> Result<usize, usize> {
    let mut result: isize;
    asm!(
        "syscall",
        in("rax") num,
        out("rcx") _,
        out("r11") _,
        lateout("rax") result,
        options(nostack, preserves_flags)
    );
    convert_raw_return_code_from_rax(result)
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn syscall_with_1_arg(num: usize, arg1: usize) -> Result<usize, usize> {
    let mut result: isize;
    asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        out("rcx") _,
        out("r11") _,
        lateout("rax") result,
        options(nostack, preserves_flags)
    );
    convert_raw_return_code_from_rax(result)
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn syscall_with_2_args(num: usize, arg1: usize, arg2: usize) -> Result<usize, usize> {
    let mut result: isize;
    asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        out("rcx") _,
        out("r11") _,
        lateout("rax") result,
        options(nostack, preserves_flags)
    );
    convert_raw_return_code_from_rax(result)
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn syscall_with_3_args(
    num: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
) -> Result<usize, usize> {
    let mut result: isize;
    asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        out("rcx") _,
        out("r11") _,
        lateout("rax") result,
        options(nostack, preserves_flags)
    );
    convert_raw_return_code_from_rax(result)
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn syscall_with_4_args(
    num: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
) -> Result<usize, usize> {
    let mut result: isize;
    asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("r10") arg4,
        out("rcx") _,
        out("r11") _,
        lateout("rax") result,
        options(nostack, preserves_flags)
    );
    convert_raw_return_code_from_rax(result)
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn syscall_with_5_args(
    num: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
) -> Result<usize, usize> {
    let mut result: isize;
    asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("r10") arg4,
        in("r8") arg5,
        out("rcx") _,
        out("r11") _,
        lateout("rax") result,
        options(nostack, preserves_flags)
    );
    convert_raw_return_code_from_rax(result)
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn syscall_with_6_args(
    num: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> Result<usize, usize> {
    let mut result: isize;
    asm!(
        "syscall",
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("r10") arg4,
        in("r8") arg5,
        in("r9") arg6,
        out("rcx") _,
        out("r11") _,
        lateout("rax") result,
        options(nostack, preserves_flags)
    );
    convert_raw_return_code_from_rax(result)
}

#[inline(always)]
fn convert_raw_return_code_from_rax(raw_code: isize) -> Result<usize, usize> {
    // when a syscall complete, the return value is store into the 'rax' register,
    // if the operation fails, the value is a negative value (rax < 0).
    // in the C std lib, the negative value will be transformed to a positive number (-rax)
    // and is stored in the thread-local symbol '__errno_location' on linux (redox, fuchsia) or
    // '__error' on freebas (ios, macos), then the std lib return the caller value '-1'.
    //
    // in C, the error number can be obtained by external variable 'errno' in '<errno.h>',
    // in Rust, the os error number can be obtained by 'std::io::Error::last_os_error().raw_os_error()'
    //
    // note that there is no 'errno' if invoke syscall by assembly directly,
    // neither the 'std::io::Error::last_os_error()'.
    //
    // to make life easy, here just translate the raw return code into 'Result<T, E>', and will not
    // change the 'errno'.

    if raw_code < 0 {
        Err((-raw_code) as usize)
    } else {
        Ok(raw_code as usize)
    }
}
