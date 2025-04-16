// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

// A simple stack implementation
// -----------------------------
//
// This stack is composed of three separate sub-stacks:
// 1. The frame information stack.
// 2. The local variables stack (used for both arguments and local variables).
// 3. The operand stack.
//
// ```diagram
//
//           /--------\      /--------------\     |            | <-- SP
//           |        | ---> | local vars 2 |     | operands 2 |
// frame 2   | info 2 |      | args 2       |     |            |
//           |        | --\  |              |     |            |
//           \--------/   |  \--------------/     |            |
//                        |                       |            |
//                        \-------locate--------> |------------| <-- FP
//                                                |            |
//           /--------\      /--------------\     |            |
//           |        | ---> | local vars 1 |     | operands 1 |
// frame 1   | info 1 |      | args 1       |     |            |
//           |        | --\  |              |     |            |
//           \--------/   |  \--------------/     |            |
//                        |                       |            |
//                        \-------locate--------> |------------|
//                                                |            |
//           /--------\      /--------------\     |            |
//           |        |      |              |     |            |
// frame 0   | info 0 | ---> | local vars 0 |     | operands 0 |
//           |        | --\  | args 0       |     |            |
// stack --> \--------/   |  \--------------/     |            |
// start                  |                       |            |
//                        \-------locate--------> \------------/ <-- stack
//                                                                   start
//           Info stack | Local variables stack | Operands stack
// ---------------------|-----------------------|----------------------------
//     Each frame has a |    Each frame has a   | Each frame has a
//          fixed size. |     variable size.    | variable size, and the last
//                      |                       | frame is growable.
// ```

// Constants defining the size of the operand stack in bytes.
const OPERAND_STACK_INITIAL_SIZE_IN_BYTES: usize = 16 * 1024; // The initial size of the operand stack: 16KB.
const OPERAND_STACK_FRAME_SIZE_IN_BYTES: usize = 4 * 1024; // The maximum size of a single operand stack frame: 4KB.
const OPERAND_STACK_MAX_SIZE_IN_BYTES: usize = 8 * 1024 * 1024; // The maximum size of the operand stack: 8MB (same as x86_64 Linux).

// The operand stack's capacity will be increased if the available space is less than
// `OPERAND_STACK_FRAME_SIZE_IN_BYTES` when creating a new stack frame.
const OPERAND_STACK_INCREMENT_SIZE_IN_BYTES: usize = 16 * 1024; // The size by which the operand stack grows: 16KB.
