(module
(import "js" "memory" (memory 10 100))
(import "js" "print_string" (func $_print_string
  (param i32)
  (param i32)
))
(import "js" "print_integer" (func $_print_integer
  (param i64)
))
(import "js" "print_real" (func $_print_real
  (param i64)
))
(data (i32.const 0) "")
(func $9function-2
  (local $2b-3$0 i64)
  (local $2b-3$1 f64)
  (local $2b-3$2 i64)
  (local $2b-3$3 f64)
  i64.const 1
  f64.const 2
  i64.const 3
  f64.const 4
  (local.set $2b-3$3)
  (local.set $2b-3$2)
  (local.set $2b-3$1)
  (local.set $2b-3$0)
  return
)
(start $9function-2)
)