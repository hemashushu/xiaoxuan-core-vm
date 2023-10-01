// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// this crate is part of project XiaoXuan Core VM, it is
// not intended to be a standalone library.
// if you need a syscall library, please refer to:
// - https://github.com/jasonwhite/syscalls.git

pub mod arch;
pub mod errno;

#[cfg(test)]
mod tests {
    use crate::{
        arch::{
            syscall_with_1_arg, syscall_with_2_args, syscall_with_3_args, syscall_without_args,
            SysCallNum,
        },
        errno::Errno,
    };

    #[test]
    fn test_syscall_error() {
        let file_path0 = b"/this/file/should/not/exist\0";
        let file_path1 = b"/dev/zero\0";

        const O_RDONLY: u32 = 0x0;
        const O_CLOEXEC: u32 = 0x80000;

        let flags = O_RDONLY | O_CLOEXEC;

        // open file '/this/file/should/not/exist' with flag O_RDONLY and O_CLOEXEC
        // run `$ man 2 open` for manual about the syscall 'open'

        let result0 = unsafe {
            syscall_with_2_args(
                SysCallNum::open as usize,
                file_path0.as_ptr() as usize,
                flags as usize,
            )
        };

        assert!(matches!(result0, Err(errno) if errno == Errno::ENOENT as usize));

        // the equivalent C program
        //
        // ```c
        // #include <stdio.h>
        // #include <stdlib.h>
        // #include <unistd.h>
        // #include <fcntl.h>
        // #include <errno.h>
        //
        // int main(void)
        // {
        //     int fd = open("/this/file/should/not/exist", O_RDONLY | O_CLOEXEC);
        //     if (fd == -1)
        //     {
        //         printf("open file failed, errno: %d\n", errno);
        //     }
        //     else
        //     {
        //         printf("open file success, fd: %d", fd);
        //         close(fd);
        //     }
        //     return EXIT_SUCCESS;
        // }
        // ```

        let result1 = unsafe {
            syscall_with_2_args(
                SysCallNum::open as usize,
                file_path1.as_ptr() as usize,
                flags as usize,
            )
        };

        assert!(matches!(result1, Ok(fd) if fd > 0));
    }

    #[test]
    fn test_syscall_without_arg() {
        let result = unsafe { syscall_without_args(SysCallNum::getpid as usize) };
        assert!(matches!(result, Ok(pid) if pid > 0));
    }

    #[test]
    fn test_syscall_io_read_write() {
        let file_path0 = b"/dev/zero\0";

        let mut buffer = [2u8, 3, 5, 7, 11, 13, 17, 19];
        let result0 = unsafe {
            syscall_with_2_args(SysCallNum::open as usize, file_path0.as_ptr() as usize, 0)
        };
        let fd0 = result0.unwrap();

        // run `$ man 2 read` for manual of syscall 'read'
        let result1 = unsafe {
            syscall_with_3_args(
                SysCallNum::read as usize,
                fd0,
                buffer.as_mut_ptr() as usize,
                8,
            )
        };

        assert!(matches!(result1, Ok(read_bytes) if read_bytes == 8));
        assert_eq!(buffer, [0u8, 0, 0, 0, 0, 0, 0, 0]);

        let result2 = unsafe { syscall_with_1_arg(SysCallNum::close as usize, fd0) };
        assert!(matches!(result2, Ok(0)));
    }
}
