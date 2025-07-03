/**
 * Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
 *
 * This Source Code Form is subject to the terms of
 * the Mozilla Public License version 2.0 and additional exceptions,
 * more details in file LICENSE and CONTRIBUTING.
 */

int add(int a, int b)
{
    return a + b;
}

int mul_add(int a, int b, int c)
{
    return a * b + c;
}

int do_something(int (*callback_func)(int), int a, int b)
{
    int s = (callback_func)(a);
    return s + b;
}

/**
 * Compile this file using the following command:
 *   gcc -Wall -g -fpic -shared -Wl,-soname,libtest.so.1 -o libtest.so.1.0.0 test.c
 *
 * or, if you want to avoid using the standard libraries:
 *   gcc -Wall -g -fpic -shared -Wl,-soname,libtest.so.1 -nostdlib -nodefaultlibs -o libtest.so.1.0.0 test.c
 *
 * It is recommended to create symbolic links to the shared library:
 *   ln -s libtest.so.1.0.0 libtest.so.1
 *   ln -s libtest.so.1.0.0 libtest.so
 */