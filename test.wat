(module
(import "js" "print_string" (func $_print_string
  (param i32)
  (param i32)
))
(import "js" "print_real" (func $_print_real
  (param f64)
))
(import "js" "print_glyph" (func $_print_glyph
  (param i32)
))
(import "js" "print_integer" (func $_print_integer
  (param i64)
))
(import "js" "print_boolean" (func $_print_boolean
  (param i32)
))
(import "js" "memory" (memory 10 100))
(data (i32.const 0) "")
(func $9function-1
  return
)
(start $9function-1)
)