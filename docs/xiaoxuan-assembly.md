# Xiaoxuan Text Assembly

## base

```clojure
(module
    (name "main")       ;; the default module name is "main"
    (version 1 0 0)     ;; the default module version is "1.0.0"
)
```

## function

```clojure
(module
    (func $add (param $lhs i32) (param $rhs i32) (result i32)   ;; the `$add` is the node label/name
                                                                ;; and the child node means they are optional
        (local $sum i32)        ;; local variables
        (code                   ;; code node, there will be a series of instructions in this node.
            (local.get $lhs)    ;; one instruction per line
            (local.get $rhs)
            i32.add             ;; the parenthsis can be omitted if an instruction has no params
            (local.set $sum)
            (local.get $sum)
        )
    )
)
```

## block

```clojure
(module
    (func
        (code
            (block $blk0 (type $t0)
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
    (link $m0 (type module) (name "math") (version 1 0 0))  ;; from runtime share modules

    ;; `import "./phy.ancs"`
    (link $m1 (type module) (file "./phy.ancs"))            ;; from local file

    ;; external module can be also imported from the internet, e.g.
    ;;
    ;; `import "https://some.host/some/module.ancs"`                        // from website
    ;; `import "https://github.com/user/module" type="git" tag="v1.0.0"`    // from git repository
    ;;
    ;; the specified module file will be downloaded into a local file system directory,
    ;; so the corresponding assembly statement is the same as the above
    ;; importing from file.
)
```

## import

import thread local data/variables or functions from external modules.

```clojure
(module
    (import $m0 "add"                                       ;; target module index and the symbol name
        (func $add (param i32) (param i32) (result i32))    ;; import function
    )
    (import $m0 "message"                                   ;; target module index and the symbol name
        (data $message i32)                                 ;; import variable
    )
)
```

## multiple assembly source file

```clojure
(module
    (submodule "./path/to/source_file_one.ancs")
    (submodule "./path/to/source_file_two.ancs")
)
```

note:

multiple assembly source files will be compiled and combined into one (binary) bytecode file.

## type

```clojure
(module

)
```

## local variable

```clojure
(module
    (func
        (local i32 i32)     ;; two i32 local variables

        ;; note:
        ;; in the default XiaoXuan VM implement, each local variable takes up 8 bytes

        ;; note:
        ;; there is no "byte" type in local variable, allocates multiple i64 variables
        ;; if local "byte" data is required.
    )
)

```

## data

```clojure
(module

    (data $name0 (section read_only) (offset 0) (length 4) (type i32) 123)
    (data $name1 (section read_only) (offset 12) (length 24) (type byte) "hello world!\n\0")

    ;; the (offset) is the position followed the last data, so it can often be omitted.
    ;; the (type) will be "i32/f64/byte", which is determited by value if the (type ...) node is ommited.
    ;; the (length) will be determited by value too. for e.g.
    ;; the following (type), (offset) and (length) nodes will be set automatically
    (data $name2 (section read_only) 456)               ;; type is i32
    (data $name3 (section read_only) 3.14)              ;; type is f64
    (data $name4 (section read_only) "Hi! 你好吗\n\0")   ;; type is byte, and the string will be encoded with UTF-8

    ;; the byte data value can be unicode, hex number, dec number, escape characters.
    (data $name4 (section read_only) "\u{1234}\x10\10\n\t")

    ;; there are another two sections "read_write" and "uninit"
    (data $rw0 (section read_write) 789)

    ;; there is no value in the section "uninit", so the data type shouldn't be ommited.
    (data $bss1 (section uninit) (type i32))
    (data $bss0 (section uninit) (type byte) (length 128))

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
    (data $num (export "num") (type i64) (value 100))   ;; define and export data
    (func $sub (export "add")                           ;; define and export a function
        (param i32 i32)
        (result i32)
        (code
            (local.get 0)
            (local.get 1)
            (i32.sub)
        )
    )

    ;; export in an individual section
    (export "num" (data $num))
    (export "sub" (func $sub))
)
```
