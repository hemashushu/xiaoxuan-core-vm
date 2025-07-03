# Copyright (c) 2023 Hemashushu <hippospark@gmail.com>, All rights reserved.
#
# This Source Code Form is subject to the terms of
# the Mozilla Public License version 2.0 and additional exceptions,
# more details in file LICENSE and CONTRIBUTING.

#!/bin/bash
gcc -Wall -g -fpic -shared -Wl,-soname,libtest.so.1 -nostdlib -nodefaultlibs -o libtest.so.1.0.0 test.c

if [ ! -f libtest.so.1 ]
then
    ln -s libtest.so.1.0.0 libtest.so.1
fi

if [ ! -f libtest.so ]
then
    ln -s libtest.so.1.0.0 libtest.so
fi