# Xiaoxuan Text Assembly

## base

```clojure
(module
    (name "main")       ;; the default module name is "main"
    (version 1 0 0)     ;; the default module version is "1.0.0"
)
```

## function type/signature

```clojure
(module
    (type $binary_op (param $left i32) (param $right i32) (result i32))
    ;; note that the scheme "$foo" is the lable of the node
    ;; and the "(foo)" is the name of the node

    (type $noresult (param i32))
    (type $noparam (result i32))
    (type $multi_results (param i64) (result i32) (result i32))
    (type $void)
    ;; types are used as signature for functions and blocks,
    ;; a type can contains multiple parameters and results,
    ;; one node per parameter/result.
    ;; it also can has no parameter or result
)
```

## function

```clojure
(module
    (func $add (param $lhs i32) (param $rhs i32) (result i32)
        (local $sum i32)            ;; local variables
        (code                       ;; code node, there will be a series of instructions in this node.
            (local.load_index $lhs)        ;; one instruction per line
            (local.load_index $rhs)
            i32.add                 ;; the parenthsis can be omitted if an instruction has no params
            (local.store_index $sum)
            (local.store_index $sum)
        )
    )
    (func $sub (type $binary_op)    ;; the type of a function can be a lable of a defined type
        ...
    )
    (func $mul (type 1)             ;; the type of a function can also be an index of a defined type
        ...
    )
)
```

## fold

```clojure
(module
    (func
        (local.load 0)
        (local.load 1)
        (i32.lt)

        ;; the 3 lines above can be writen as fold
        (i32.lt (local.load 0) (local.load 1))

        ;; all instructions which contains operands can be writen as fold
        ;; e.g.
        (i32.add (i32.imm 11) (i32.imm 13))
        (dcall (data.load 0))
        (return (i32.imm 11) (i32.imm 13))

        ;; and fold can be nested, e.g.
        (i32.lt
            (i32.add (local.load 0) (local.load 1))
            (i32.add (local.load 2) (local.load 3))
        )
    )
)
```

## block

```clojure
(module
    (func
        (code
            (block $blk0
                ...
            )

            (block $blk1 (param i32) (param i32) (result i32) (result i32)
                ;; a block can also contains parameters and results just like a function, except
                ;; it wouldn't create local variable area, in fact it shares the local variables
                ;; with the function.
                ;;
                ;; if a block omits the type, it indicates that it has neigher parameters
                ;; nor return values.
                ...
            )

            (block_alt $blk2 (type $one)
                (then
                    ;; the "block_alt" node is also a block, but it has the two child nodes "then" and "else"
                    ;; when the operand before "if" is TRUE, the part "then" will be executed,
                    ;; otherwise the part "else" is executed.
                )
                (else
                    ;; the part "else"  may be omitted if not necessary.
                )
            )

            (block_alt $blk3 (type $two)
                ;; the condition node can be writen in the 'block_alt' node
                (i32.lt (local.load 0) (local.load 1))
                (then
                    ;; ...
                )
                (else
                    ;; ...
                )
            )

            (block $loop (param i32) (result i32)
                ...
                (block_alt
                    (...)
                    (then
                        (imm.i32 100)
                        (break 1)
                        ;; node "break" means jump out the block.
                        ;; note that blocks are nested, the node name is followed by a number,
                        ;; the number means how depth of the parant blocks.
                        ;; for example, 0 means just jump out the current block, 1 means jump out the
                        ;; current block and 1 parant block.
                    )
                )
                ...
                (imm.i32 99)
                (recur 0)
                ;; node "recur" means jump to the beginning of the specified block.
            )

            (block
                ...
                return  ;; node "return" means jump out the current function.
                ...
            )
        )
    )
)
```

## link

link/import the external modules.

```clojure
(module
    ;; `import "math" version="1.0.0"`
    (link $m0 (moduletype shared) (name "math") (version 1 0 0))  ;; from runtime share modules

    ;; `import "./phy.ancs"`
    (link $m1 (moduletype local) (file "./phy.ancs"))             ;; from local file

    ;; in the XiaoXuan script source code, the external modules can also be imported
    ;; from the internet, e.g.
    ;;
    ;; `import "https://some.host/some/module.ancs"`                        // from website
    ;; `import "https://github.com/user/module" type="git" tag="v1.0.0"`    // from git repository
    ;;
    ;; the specified module will be downloaded to a local file system directory,
    ;; so the corresponding module is actually imported as local file.
)
```

## import

import data, variables and functions from external modules.

```clojure
(module
    (import $m0 "add"                                       ;; target module index and the symbol name
        (func $add (param i32) (param i32) (result i32))    ;; import function
    )
    (import $m0 "add"
        (func $add (type $binary_op))                       ;; import function with speicified type
    )
    (import $m0 "message"                                   ;; target module index and the symbol name
        (data $message i32)                                 ;; import variable
    )
)
```

## local variable

```clojure
(module
    (func
        (local i32)         ;;
        (local i64)         ;; two local variables

        ;; note:
        ;; in the default XiaoXuan VM implement, each i32/i64/f32/f64 type local variable takes up 8 bytes

        (local $i i32)      ;; local variable with lable
        (local $j i32)

        ;; the actual length of local data with 'byte' type would be multiple by 8-byte
        ;; for example, a 12 bytes local variable takes up 16 (= 8 * 2) bytes.
        (local $msg (length 12) byte)
    )
)

```

## data

```clojure
(module

    (data $name0 (section read_only) (datatype i32) (length 4) (align 4) 123)
    (data $name1 (section read_only) (datatype byte) (length 24) (align 1) "hello world!\n\0")

    ;; the (datatype) can be determited by value as "i32/f64/byte" if the (type ...) node is ommited.
    ;; the (length) and (align) can be determited by the value.
    ;; e.g. the nodes (datatype), (length) and (align) at the following will be set automatically.
    (data $name2 (section read_only) 456)               ;; data type is i32, length is 4, align is 4
    (data $name3 (section read_only) 3.14)              ;; data type is f64, length is 8, align is 8
    (data $name4 (section read_only) "Hi! 你好吗\n\0")   ;; data type is byte, and the string will be encoded with UTF-8

    ;; the byte data value can be unicode, hex number, dec number, escape characters.
    (data $name4 (section read_only) "\u{1234}\x10\10\n")

    ;; if the data is a struct, the (align) node should be specified.
    (data $name5 (section read_only) (align 4) "\11\13\17\19\21\23\29\31")

    ;; there are another two sections called "read_write" and "uninit"
    (data $rw0 (section read_write) 789)

    ;; there is no value in the section "uninit", so the data type shouldn't be ommited.
    (data $bss1 (section uninit) (datatype i32))
    (data $bss0 (section uninit) (datatype byte) (length 128) (align 4))

    ;; the default section is "read_write"
    (data $rw1 100)
)
```

## comment

```clojure
(module
    ;; line comment
    ;; another line comment
    (name "math")   ;; comment at the ending of line
    (func
        (; inline comment ;)
        (;
            block comment
        ;)
        (; nested comment (; inner comment ;);)
        (i32.imm 123)
        (drop)
    )
)
```

## export

```clojure
(module
    (data $num (datatype i64) (value 100))      ;; define a data
    (func $sub                                  ;; define a function
        (param i32 i32)
        (result i32)
        (code
            (local.load_index 0)
            (local.load_index 1)
            (i32.sub)
        )
    )

    ;; export data and functions
    (export "num" (data $num))
    (export "sub" (func $sub))
)
```

## pesudo instruction

`return` = `break MAX`
`if` = `block_nez`
`if...else...` = `block_alt`
`break_if` = `break_nez`
`recur_if` = `recur_nez`
