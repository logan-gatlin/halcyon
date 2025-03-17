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
(data (i32.const 0) "fizzbuzzfizzbuzz")
(func $9function-5
  (param $7number-6$0 i64)
  (local $2i-7$0 i64)
  i64.const 0
  (local.set $2i-7$0)
  block $_0
  loop $_1
  (local.get $2i-7$0)
  i64.const 3
  i64.rem_s
  i64.const 0
  i64.eq
  (local.get $2i-7$0)
  i64.const 5
  i64.rem_s
  i64.const 0
  i64.eq
  i32.and
  if
  i32.const 0
  i32.const 8
  call $_print_string
  else
  (local.get $2i-7$0)
  i64.const 3
  i64.rem_s
  i64.const 0
  i64.eq
  if
  i32.const 8
  i32.const 4
  call $_print_string
  else
  (local.get $2i-7$0)
  i64.const 5
  i64.rem_s
  i64.const 0
  i64.eq
  if
  i32.const 12
  i32.const 4
  call $_print_string
  else
  (local.get $2i-7$0)
  call $_print_integer
  end
  end
  end
  (local.get $2i-7$0)
  (local.get $7number-6$0)
  i64.ge_s
  if
  br $_0
  end
  (local.get $2i-7$0)
  i64.const 1
  i64.add
  (local.set $2i-7$0)
  br $_1
  end
  end
  return
)
(func $9function-3
  i64.const 15
  call $9function-5
  return
)
(start $9function-3)
)