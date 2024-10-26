(module
(import "console" "log" (func $log (param i32)))
(global $a$0 (mut i32) (i32.const 0))
(func $0
i32.const 10
i32.const 5
i32.div_s
global.set $a$0
global.get $a$0
call $log
)

(start $0)
)
