// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

// Environment Call (envcall) Number Encoding
// ------------------------------------------
//
// The number consists of two parts: categories and items, both of which are 8-bit numbers.
//
// MSB                             LSB
// 00000000 00000000 00000000 00000000 <-- bits
// -------- -------- -------- --------
// ^                 ^
// |                 | items
// |
// | categories

#[repr(u32)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum EnvCallNum {
    // Category: Runtime Information

    // Retrieve the VM runtime edition.
    //
    // `fn (module_index: i32, data_public_index: i32)`
    //
    // The data must be 8 bytes long.
    // The content is either a string with 8 characters or a null-terminated string.
    runtime_edition = 0x0001_0000,

    // Retrieve the VM runtime version.
    //
    // `fn () -> i64`
    //
    // The returned value is a 64-bit integer with the following structure:
    // 0x0000_0000_0000_0000
    //        |    |    |
    //        |    |    | patch version
    //        |    | minor version
    //        | major version
    runtime_version,

    // Category: Host Information

    // Retrieve the host's architecture.
    //
    // `fn (module_index: i32, data_public_index: i32) -> i32`
    //
    // The data must be 16 bytes long.
    // The content is either a string with 16 characters or a null-terminated string.
    // Possible values include: x86_64, aarch64, riscv64, etc.
    // Returns the actual number of characters.
    host_arch = 0x0002_0000,

    // Retrieve the host's operating system.
    //
    // `fn (module_index: i32, data_public_index: i32) -> i32`
    //
    // The data must be 16 bytes long.
    // The content is either a string with 16 characters or a null-terminated string.
    // Possible values include: linux, macos, windows, freebsd, android, ios, etc.
    // Returns the actual number of characters.
    host_os,

    // Retrieve the host's OS family.
    //
    // `fn (module_index: i32, data_public_index: i32) -> i32`
    //
    // The data must be 16 bytes long.
    // The content is either a string with 16 characters or a null-terminated string.
    // Possible values include: unix, windows, etc.
    // Returns the actual number of characters.
    host_family,

    // Retrieve the host's endianness.
    //
    // `fn () -> i32`
    //
    // Returns 0 for little-endian or 1 for big-endian.
    host_endian,

    // Retrieve the host's memory width.
    //
    // `fn () -> i32`
    //
    // Returns the width of the host's memory, which is also the size of a pointer.
    // Possible values include: 32, 64, etc.
    host_memory_width,

    // Reference:
    // https://doc.rust-lang.org/reference/conditional-compilation.html#target_arch

    // Category: Process environment

    // Retrieve the length of the program path in bytes.
    //
    // `fn () -> i32`
    //
    // The "program_path" is the file path of the currently running program.
    // It may be a directory path for an application module, a file path for a script or image file,
    // or an empty string if the program is built in memory.
    program_path_size = 0x0003_0000,

    // Retrieve the data of the "program_path".
    //
    // `fn (module_index: i32, data_public_index: i32) -> i32`
    //
    // Returns the actual length of the data read.
    // If the data length is less than the content length, the VM will panic.
    program_path_text,

    // Retrieve the program's source type.
    //
    // `fn () -> i32`
    //
    // Possible values:
    // - 0: module
    // - 1: script file
    // - 2: memory
    // - 3: package image
    program_source_type,

    // Retrieve the length of the program all arguments in bytes.
    //
    // `fn () -> i32`
    argument_size,

    // Retrieve the data of program all arguments.
    //
    // `fn (module_index: i32, data_public_index: i32) -> i32`
    //
    // Returns the actual length of the data read.
    // If the data length is less than the content length, the VM will panic.
    argument_text,

    // Retrieve the number of environment variables.
    //
    // `fn () -> i32`
    environment_variable_count,

    // Retrieve the length of the specified environment variable by index.
    //
    // `fn (environment_variable_index: i32) -> i32`
    environment_variable_item_size,

    // Retrieve the data of the specified environment variable by index.
    //
    // `fn (environment_variable_index: i32, module_index: i32, data_public_index: i32) -> i32`
    //
    // Returns the actual length of the data read.
    // If the data length is less than the content length, the VM will panic.
    environment_variable_item_text,

    // Set a specific environment variable.
    //
    // `fn (module_index: i32, data_public_index: i32, data_length_in_bytes: i32)`
    //
    // The data content is a string in the format "name=value", e.g., "EDITOR=vim".
    environment_variable_set,

    // Remove the environment variable with the specified name.
    //
    // `fn (module_index: i32, data_public_index: i32, data_length_in_bytes: i32)`
    //
    // The data content is a string representing the name of the environment variable.
    environment_variable_remove,

    // Category: Time

    // Retrieve the current time (elapsed since the epoch).
    //
    // `fn () -> (seconds: i64, nano_seconds: i64)`
    //
    // The value of "nano_seconds" is in the range [0, 999_999_999].
    time_now = 0x0004_0000,

    // Category: Random number generation

    // Retrieve a random number of type i32.
    //
    // `fn () -> i32`
    random_i32 = 0x0005_0000,

    // Retrieve a random number of type i64.
    //
    // `fn () -> i64`
    random_i64,

    // Retrieve a random number in the range [0, 1).
    //
    // `fn () -> f32`
    random_f32,

    // Retrieve a random number in the range [0, 1).
    //
    // `fn () -> f64`
    random_f64,

    // Retrieve a random number within a specific range.
    //
    // `fn (start_include: i32, end_exclude: i32) -> i32`
    random_range_i32,

    // Retrieve a random number within a specific range.
    //
    // `fn (start_include: i64, end_exclude: i64) -> i64`
    random_range_i64,

    // Retrieve a random number within a specific range.
    //
    // `fn (start_include: f32, end_exclude: f32) -> f32`
    random_range_f32,

    // Retrieve a random number within a specific range.
    //
    // `fn (start_include: f64, end_exclude: f64) -> f64`
    random_range_f64,

    // Category: I/O

    // Open or create a file.
    //
    // `fn (module_index: i32, data_public_index: i32, content_length_in_bytes:i32, open_options: i32, access_mode: i32) -> (file_index: i32, io_error_number: i32)`
    //
    // Parameters `module_index` and `data_public_index` specify a data object representing a file path.
    // The file path can be absolute (e.g., /home/yang/Documents/readme.txt) or relative (e.g., ../images/banner.png),
    // but cannot include shell-specific paths like `~/Downloads/123.dat` or `${HOME}/projects`.
    //
    // Returns `(file_index: i32, io_error_number: i32)`.
    file_open = 0x0006_0000,

    // Access mode (bit flags)
    // -----------------------
    // 1: read
    // 2: write
    // 4: append (implies `write` access)
    //    This mode means that writes will append to a file instead of overwriting previous contents.
    //    Append mode guarantees that writes will be positioned at the current end of file, even when
    //    there are other processes or threads appending to the same file. This is unlike
    //    seek(SeekFrom::End(0)) followed by write(), which has a race between seeking and writing
    //    during which another writer can write, with our write() overwriting their data.
    //
    // Bit flags can be combined, e.g.,
    // 1 + 2 = 3 = read + write

    // Open options (bit flags)
    // ------------------------
    // (not set) = normal open
    // 1: truncate
    //    Sets the option for truncating a previous file.
    //    If a file is successfully opened with this option set to true, it will truncate
    //    the file to 0 length if it already exists.
    //    The file must be opened with write access for truncate to work.
    //
    // 2: create_or_open
    //    Sets the option to create a new file, or open it if it already exists.
    //    In order for the file to be created, AccessMode::write or OpenOptions::append access must be used.
    //    This is a general option for creating a file with some given data.
    //
    // 4: create_only_non_exist
    //    Sets the option to create a new file, failing if it already exists.
    //    This option is useful because it is atomic. Otherwise, between checking whether
    //    a file exists and creating a new one, the file may have been created by another process.
    //    If this option is set, `create` and `truncate` are ignored.
    //    The file must be opened with AccessMode::write or OpenOptions::append in order
    //    to create a new file.

    // File Error Number
    // ---------------------
    // 1: NotFound
    //     - The specified file does not exist and neither create nor create_only_non_exist is set.
    //     - One of the directory components of the file path does not exist.
    // 2: PermissionDenied
    //     - The user lacks permission to get the specified access rights for the file.
    //     - The user lacks permission to open one of the directory components of the specified path.
    // 3: AlreadyExists
    //     create_only_non_exist was specified and the file already exists.
    // 4: InvalidInput
    //     Invalid combinations of open options (e.g., `truncate` without `write` access, etc.).

    // File index
    // ----------
    // There are three preset file indices for each program:
    // - 0: stdin
    //      Standard input of the current process.
    // - 1: stdout
    //      Standard output of the current process. By default, the handle is line-buffered
    //      when connected to a terminal, meaning it flushes automatically when
    //      a newline (\n) is encountered. For immediate output, you can manually call the
    //      `flush` method.
    // - 2: stderr
    //      Standard error of the current process. This file is not buffered.

    // Read data from a file and store it in a data object.
    //
    // `fn (file_index: i32, module_index: i32, data_public_index: i32, data_offset: i32, expected_bytes: i32) -> (actual_read_bytes: i32, io_error_number: i32)`
    //
    // The return value `actual_read_bytes` will be 0 if the file offset is at or beyond the end of the file.
    // If the available data space is less than the expected number of bytes, the VM will panic.
    file_read,

    // Write data to a file.
    //
    // `fn (file_index: i32, module_index: i32, data_public_index: i32, data_offset: i32, bytes_to_write: i32) -> (actual_write_bytes: i32, io_error_number: i32)`
    // If the available data is less than the `bytes_to_write`, the VM will panic.
    file_write,

    // Seek to an offset, in bytes, in a stream.
    //
    // A seek beyond the end of a stream is allowed, but behavior is defined by the implementation.
    //
    // `fn (file_index: i32, seek_from: i32, bytes: i64) -> (position: i64, io_error_number: i32)`
    //
    // The possible values for `seek_from` are:
    // - 0: start
    //   Sets the offset to the provided number of bytes.
    // - 1: end
    //   Sets the offset to the size of this object plus the specified number of bytes.
    // - 2: current
    //   Sets the offset to the current position plus the specified number of bytes.
    file_seek,

    // Flushes the output stream,
    // ensuring that all intermediately buffered contents reach their destination.
    //
    // `fn () -> io_error_number: i32`
    //
    // Returns the file error number.
    file_flush,

    // Close the specified file.
    //
    // `fn () -> io_error_number: i32`
    file_close,

    // Returns true if the file index refers to a terminal/tty.
    //
    // `fn (file_index: i32) -> i32`
    file_is_terminal,

    // Category: File System

    // Opens a directory stream corresponding to the directory name and returns an index
    // representing the directory stream.
    // The stream is positioned at the first entry in the directory.
    //
    // `fn (module_index: i32, data_public_index: i32, content_length_in_bytes:i32) -> (dir_index: i32, fs_error_number: i32)`
    fs_open_dir = 0x0007_0000,

    // Retrieve the next directory entry in the directory stream.
    //
    // `fn (dir_index: i32, module_index: i32, data_public_index: i32) -> (bytes_read:i32, fs_error_number: i32)`
    //
    // Retrieve the next entry in the directory stream.
    // The data buffer must be at least 256 bytes long. The exact size requirement may vary depending on the platform.
    //
    // If the last entry has been read, this function returns `(0, 0)`
    fs_read_dir,

    // Creates a new, empty directory at the provided path.
    //
    // `fn (module_index: i32, data_public_index: i32, content_length_in_bytes:i32) -> fs_error_number: i32`
    fs_create_dir,

    // Removes an empty directory.
    //
    // `fn (module_index: i32, data_public_index: i32, content_length_in_bytes:i32) -> fs_error_number: i32`
    fs_remove_dir,

    // Removes a file from the filesystem.
    //
    // `fn (module_index: i32, data_public_index: i32, content_length_in_bytes:i32) -> fs_error_number: i32`
    fs_remove_file,

    // Renames a file or directory to a new name, replacing the original file if the destination already exists.
    //
    // ```
    // fn (source_module_index: i32, source_data_public_index: i32, source_content_length_in_bytes:i32,
    // dest_module_index: i32, dest_data_public_index: i32, dest_content_length_in_bytes:i32) -> fs_error_number: i32`
    // ```
    fs_rename,

    // Checks if a file exists in the filesystem.
    //
    // `fn (module_index: i32, data_public_index: i32, content_length_in_bytes:i32) -> (i32, fs_error_number: i32)`
    //
    // This function will traverse symbolic links to query information about the destination file.
    // In case of broken symbolic links, this will return Ok(false).
    // This function will only return Ok(true) or Ok(false) if the path was verified to exist or not exist.
    // If its existence can neither be confirmed nor denied, an Error will be propagated instead.
    // This can be the case if, e.g., listing permission is denied on one of the parent directories.
    //
    // Note that while this avoids some pitfalls of the exists() method,
    // it still cannot prevent time-of-check to time-of-use (TOCTOU) bugs.
    // You should only use it in scenarios where those bugs are not an issue.
    fs_exists,

    // Category: Thread

    // Get the current thread ID.
    //
    // The value is 0 for the main thread, 1 for the first child thread,
    // 2 for the second child thread, and so on.
    //
    // Signature: `fn () -> i32`
    thread_index = 0x0008_0000,

    // XiaoXuan Core Thread Model
    // --------------------------
    //
    // The first thread created by the runtime is called the "main thread."
    // A thread can create one or more child threads, and the creator is referred to as the "parent thread."
    // When a parent thread exits, all its child threads are also terminated.
    //
    // A "channel" is created between the parent thread and each child thread. They can
    // communicate using the `thread_msg_receive`/`thread_msg_send` and
    // `thread_msg_receive_from`/`thread_msg_send_to` envcalls.
    // However, there is no direct channel between sibling child threads.
    //
    // Example thread tree:
    //
    // ```diagram
    // main thread
    //   |-- child thread 0
    //   |     |-- child thread 0-0
    //   |     |-- child thread 0-1
    //   |     |     |-- child thread 0-1-0
    //   |     |
    //   |     |-- child thread 0-2
    //   |
    //   |-- child thread 1
    // ```
    //
    // Communication between threads:
    //
    // ```diagram
    //     thread                                      channel     parent thread
    //   /--------------------------------\          /---------\    /----------\
    //   |                                | receive  | msg box |    |          |
    //   |                         o RX <------------\---------/<------ TX o   |
    //   |                         o TX ------------>/---------\------> RX o   |
    //   |                                | send     | msg box |    |          |
    //   |                                |          \---------/    \----------/
    //   |  /--------\  /-------\         |
    //   |  | memory |  | stack |         |          /---------\    /----------\
    //   |  \--------/  \-------/         | receive  | msg box |-\  |          |-\
    //   |                         o RX <------------\---------/<------ TX o   | |
    //   |  /------------\         o TX ------------>/---------\------> RX o   | |
    //   |  | SP, FP, PC |                | send     | msg box | |  |          | |
    //   |  \------------/                |          \---------/ |  \----------/ |
    //   |                                |            \--------/     \---------/
    //   |  /-----------------------\     |            channels      child threads
    //   |  | read-write data       |-\   |
    //   |  | uninit. data          | |   |
    //   |  |-----------------------| |   |
    //   |  | read-only data (ref)  | |   |
    //   |  | types (ref)           | |   |
    //   |  | functions (ref)       | |   |
    //   |  | ...                   | |   |
    //   |  \-----------------------/ |   |
    //   |    \----------------------/    |
    //   |       module images            |
    //   |                                |
    //   \--------------------------------/
    // ```
    //
    // Note:
    // - Memory, stack, and data sections are all thread-local.
    // - The XiaoXuan Core VM has no "global" data or variables.
    // - Threads can only communicate through channels. In the XiaoXuan Core VM,
    //   all "objects" are thread-safe.
    //
    // Message Channels
    // ----------------
    //
    // Threads communicate through message channels. Each channel has two pipes:
    // one for transmitting and one for receiving.
    // The raw type of a message is a `u8` array. Message types can include:
    // - Primitive data
    // - A struct
    // - An array
    // - The address (index) of data
    // - The address (index) of a function (including closures)
    //
    // When a thread finishes, its corresponding channel is closed. If a thread
    // is terminated by its parent, the channel is also destroyed.

    // Message Box
    // -----------
    //
    // Each pipe maintains a message box. When a thread sends a message,
    // it is sent to the message box, so the sender is not blocked.
    // This mechanism allows the sender to return immediately after sending a message.
    //
    // Similarly, when a thread receives a message, it reads from the message box.
    // If the message box is empty, the receiver is blocked until a message is available.

    // Create a new thread and execute the specified function.
    //
    // ```
    // fn (function_public_index:i32,
    //    thread_start_data_public_index:i32,
    //    thread_start_data_length:i32) -> i32
    // ```
    //
    // This envcall returns the child thread index/ID.
    //
    // The value of `thread_start_data_public_index` is the index of data.
    // The data will be copied to the new thread, and
    // the new thread can read this data using the `thread_start_data_read` envcall.
    //
    // The target function is called the "thread start function."
    // Its signature MUST be exactly:
    // `fn () -> i32`
    //
    // The "thread start function" has no parameters and returns an "exit code."
    // The meaning of the "exit code" is defined by the user.
    // You can simply return 0 if you do not need it.
    thread_create,

    // Get the length of the thread start data.
    //
    // `fn () -> i32`
    //
    // Returns the length.
    thread_start_data_length,

    // Read the "thread start data" to a writable data.
    //
    // `fn (module_index:i32, data_public_index:i32, offset_of_thread_start_data:i32, expected_length_in_bytes:i32) -> i32`
    //
    // Returns the length of data that was actually read.
    // If the available data space is less than `expected_length_in_bytes`, the VM will panic.
    thread_start_data_text,

    // Wait for the specified child thread to finish and collect resources from the child thread.
    //
    // `fn (child_thread_index:i32) -> (thread_exit_code:i32, thread_error_number:i32)`
    //
    // Returns:
    // - thread_exit_code: The value is identical to the "exit code" of the "thread start function."
    // - thread_error_number: 0 for success, 1 for thread not found.
    //
    // The caller will be blocked if the child thread is running. When the child thread finishes,
    // the function `thread_wait_and_collect` will return a tuple `(thread_exit_code, thread_error_number)`,
    // and the child thread will be removed from the "child thread collection," which is a collection
    // maintained by each parent thread.
    //
    // If the child thread finishes before the parent thread calls the
    // function `thread_wait_and_collect`, the resources of the child thread
    // will NOT be released, and they will be held in the "child thread collection" until
    // the parent thread calls `thread_wait_and_collect`.
    // In this case, the function `thread_wait_and_collect` will return immediately.
    //
    // The function `thread_wait_and_collect` is similar to the function `thread.join()`
    // in other programming languages.
    thread_wait_and_collect,

    // Check whether the specified (child) thread has finished.
    //
    // `fn (child_thread_index:i32) -> (running_status:i32, thread_error_number:i32)`
    //
    // Returns:
    // - running_status: 0=running, 1=finished
    // - thread_error_number: 0 for success, 1 for thread not found.
    thread_running_status,

    // Terminate the specified child thread and collect its resources.
    //
    // `fn (child_thread_index:i32) -> ()`
    thread_terminate,

    // Receive a message from the parent thread.
    //
    // `fn () -> i32`
    //
    // Returns the length (in bytes) of the new message.
    //
    // The thread is terminated normally if the parent thread is going to
    // terminate this thread, even if this thread is blocked by the function
    // `thread_receive_msg_from_parent`.
    //
    // Note:
    // - The message is copied to a runtime temporary memory. The function `thread_msg_read`
    //   should be called to read the message.
    //
    // ```diagram
    //
    // /---------\  thread_receive_msg  /-----------\
    // | message |--------------------->| temporary |
    // | box     |                      | memory    |
    // \---------/                      \-----------/
    //                                       |
    //                                       | thread_msg_read
    //                                       v
    //                                 writable data
    //
    // ```
    // - The runtime temporary memory will be replaced with a new message each time the function
    //   `thread_receive_msg_from_parent` or `thread_receive_msg` is called.
    // - This function will block the current thread if there is no data available.
    thread_receive_msg_from_parent,

    // Send a message to the parent thread.
    //
    // `fn (module_index:i32, data_public_index:i32, content_length_in_bytes:i32) -> ()`
    //
    // This function returns immediately and does not block the current thread.
    thread_send_msg_to_parent,

    // Receive a message from the specified child thread.
    //
    // `fn (child_thread_index:i32) -> (length:i32, thread_error_number:i32)`
    //
    // Returns:
    // - length: The length of the message
    // - thread_error_number: 0 for success, 1 for failure. The value 1 means the child thread has finished or
    //   the specified child thread does not exist.
    //
    // Note:
    // - The message is copied to a runtime temporary memory. The function `thread_msg_read`
    //   should be called to read the message.
    //
    // ```diagram
    //
    // /---------\  thread_receive_msg  /-----------\
    // | message |--------------------->| temporary |
    // | box     |                      | memory    |
    // \---------/                      \-----------/
    //                                       |
    //                                       | thread_msg_read
    //                                       v
    //                                 writable data
    //
    // ```
    // - The runtime temporary memory will be replaced with a new message each time the function
    //   `thread_receive_msg_from_parent` or `thread_receive_msg` is called.
    // - This function will block the current thread if there is no data available.
    thread_receive_msg,

    // Send a message to the specified child thread.
    //
    // `fn (child_thread_index:i32, module_index:i32, data_public_index:i32, content_length_in_bytes:i32) -> thread_error_number:i32`
    //
    // Returns the send result: 0 for success, 1 for failure. The value 1 means the
    // child thread has finished or the specified child thread does not exist.
    //
    // This function returns immediately and does not block the current thread.
    thread_send_msg,

    // Get the length of the last received message.
    //
    // `fn () -> i32`
    //
    // Returns the length of the message in bytes.
    thread_msg_length,

    // Read the last received message from the runtime temporary memory to writable data.
    //
    // `fn (data_public_index:i32, offset_of_message:i32, expected_size_in_bytes:i32) -> i32`
    //
    // Returns the actual length of the read data.
    // If the available data space is less than `expected_length_in_bytes`, the VM will panic.
    thread_msg_read,

    // Block the current thread for the specified milliseconds.
    //
    // `fn (milliseconds:i64)`
    thread_sleep,

    // Ref:
    // - https://doc.rust-lang.org/std/sync/mpsc/index.html
    // - https://doc.rust-lang.org/stable/rust-by-example/std_misc/channels.html
    // - https://smallcultfollowing.com/babysteps/blog/2015/12/18/rayon-data-parallelism-in-rust/

    // Category: Regular Expression

    // The regular expression implementation regex-anre is adopted.
    //
    // Ref:
    // - https://github.com/hemashushu/regex-anre
    // - https://github.com/rust-lang/regex

    // Compile the given regular expression.
    //
    // `fn (module_index:i32, data_public_index:i32, data_length_in_bytes:i32, flavour:i32) -> (regex_index:i32, regex_error_number:i32)`
    //
    // The content of data is the regular expression text.
    // Parameter `flavour` represents the syntax of the regular expression:
    // 0 for traditional, 1 for the "XiaoXuan Regular Expression (ANRE)."
    //
    // Returns `(regex_index:i32, regex_error_number:i32)`.
    regex_create = 0x0009_0000,

    // Get the number of capture groups.
    //
    // `fn (regex_index:i32) -> i32`
    regex_capture_group_count,

    // Get the length of all capture group names.
    //
    // Group names are concatenated with a '\0' character.
    //
    // `fn (regex_index:i32) -> i32`
    regex_capture_group_names_length,

    // Get all capture group names.
    //
    // Group names are concatenated with a '\0' character.
    //
    // `fn (regex_index:i32, module_index:i32, data_public_index:i32) -> i32`
    //
    // Returns the actual length of data that was read.
    // If the available data space is less than `expected_length_in_bytes`, the VM will panic.
    regex_capture_group_names_text,

    // Start a matching operation with the given text and offset.
    //
    // `fn (regex_index:i32, module_index:i32, data_public_index:i32, data_length_in_bytes:i32, offset_in_bytes:i32) -> (match_start:i32, match_length:i32)`
    //
    // Returns `(match_start:i32, match_length:i32)` if a match is found, or `(0, 0)` if no match is found.
    regex_match,

    // Get the result of the current capture groups.
    //
    // `fn (regex_index:i32, module_index:i32, data_public_index:i32, data_length_in_bytes:i32) -> i32`
    //
    // The result of capture groups is an `i32` array with the following scheme:
    //
    // `[group_0_start, group_0_end, group_1_start, group_1_end, ...]`
    //
    // The first capture group is the range of the whole text that matches the regular expression.
    // Returns the actual length of data that was read.
    // If the available data space is less than `expected_length_in_bytes`, the VM will panic.
    regex_capture_groups,

    // Remove the specified regex object.
    //
    // `fn (regex_index:i32)`
    regex_remove,
}
