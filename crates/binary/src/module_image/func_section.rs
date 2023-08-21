// Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE and CONTRIBUTING.

// "function section" binary layout
//
//                   |-------------------------------------------------------------------------------|
//                   | item count (u32) | (4 bytes padding)                                          |
//        item 0 --> | func type 0 (u16) | 16 bytes padding | code offset 0 (u32) | code len 0 (u32) |
//        item 1 --> | func type 1       | 16 bytes padding | code offset 1       | code len 1       |
//                   | ...                                                                           |
// code offset 0 --> | code 0                                                                        |
// code offset 1 --> | code 1                                                                        |
//                   | ...                                                                           |
//                   |-------------------------------------------------------------------------------|
