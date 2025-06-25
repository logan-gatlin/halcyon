(module
    (import "host" "host_func" (func $host_hello (param i32)))

    (func (export "hello")
        i32.const 3
        call $host_hello
    )
)
