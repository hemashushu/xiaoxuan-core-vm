
## import XiaoXuan (script) application as a function (* move to the script language feature)

because XiaoXuan modules are loaded extremely fast, there is no parsing process,
there is almost no startup time, and instructions are executed directly on the binary
bytecode of the module, so the XiaoXuan applications can be used as functions.

consider an application called "digest" with the following options:

```text
digest
    -S, --stdin         data from stdin
    -a, --algorithm     the digist algorithm, possible values are "sha256" and "sha512"
    -f                  the input file
```

some command examples:

```bash
$ digest -f hello.txt
$ digest -f -a sha256
$ echo "hello" | digest -S
$ echo "world" | digest -S -a sha512
```

when treating this application as a function, then it has these parameters:

- "_S" or "stdin", type is "bool", which means this program option doesn't allow an argument.
- "_a" or "algorithm", type is "string"
- "_f", type is "string"

there is also an implicit parameter STDIN, but it's wrapped by the VM, the struct is:

```rust
Input<int, stream_writer>
```

the result is wrapped as a struct:

```rust
Result<int, stream_reader>
```

now that we have enough information, the import statement for the application is as follows:

```clojure
(module
    (appfunc $digest (file "digest.ancs")
        ;; node "option" means it does not map to a parameter of a function
        (option "_S" (type bool) (value true))

        ;; node "value" is the default value for the parameter
        (param "algorithm" (type string) (value "sha256"))

        ;; parameter "input" can be omitted if no data is read in from STDIN.
        (param "input" (type input))

        ;; note that do not specify parameter "_f", because we want to ignore this parameter.
        ;; the node "result" doesn't need to be specified because an application will definitely
        ;; have a return value and its type always is "Result<int, stream_writer>"

        ;; this node will generate a type as the following:
        ;; "(type (param i64) (param i64) (result i64))"
        ;; struct and string are pointers, so the data type is i64 (in 64-bit VM)
    )
)
```
