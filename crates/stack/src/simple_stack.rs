// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

// A simple stack approach
// -----------------------
//
// A stack consists of three separate stacks:
// - The frame information stack.
// - The local variables stack (for both arguments and local variables).
// - The operands stack.
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

// The size of the operand stack in bytes.
const OPERAND_STACK_FRAME_SIZE_IN_BYTES: usize = 4 * 1024; // 4KB

