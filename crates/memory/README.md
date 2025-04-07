# XiaoXuan Core - Memory

The XiaoXuan Core VM manages various memory-like objects, including:

- **Local Variables**: These include parameters and variables defined within functions and blocks.
- **Data Sections**: Segments of memory containing read-only, read-write, or uninitialized data.
- **The Stack**: A region used for managing function calls, local variables, operands, and control flow.
- **Allocated Memory**: Dynamically allocated memory regions.

This module provides interfaces for memory operations, such as loading and storing data.

```diagram
|-----------------|
| Read-only Data  |
| Read-write Data | ----\
| Uninitialized   |     |
| Data            |     |
|-----------------|     |
                        |           |-------|         |--------|
|-----------------|     |   read    |       |   pop   |        |
| Allocated       |     | --------> |       | ------> |        |
| Memory          | ----|           | Stack |         | Thread |
|-----------------|     | <-------- |       | <------ |        |
                        |   write   |       |   push  |        |
|-----------------|     |           |-------|         |--------|
| Local Variables | ----/
|-----------------|
```
