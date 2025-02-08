(module
(import "js" "print_string" (func $print_string
  (param  i32)
  (param  i32)
))
(import "js" "memory" (memory 10 100))
(data (i32.const 0) "fizzbuzz\0afizz\0abuzz\0a")

(type $tup (struct i64))

(func $5anon-3
  (param $7number-2$0 i64)
  (local $2i-4$0 i64)
  (local $_1$0 i32)
  (local $_1$1 i32)
  (local $_2$0 i32)
  (local $_2$1 i32)
  (local $_3$0 i32)
  (local $_3$1 i32)
  i64.const 0
  (block $_0
    (loop $_4
      local.get $2i-4$0
      i64.const 3
      i64.rem_s
      i64.const 0
      i64.eq
      local.get $2i-4$0
      i64.const 5
      i64.rem_s
      i64.const 0
      i64.eq
      i32.and
      i32.const 1
      (if (then
        i32.const 0
        i32.const 9
        (local.set $_1$0)
        (local.set $_1$1)
      )
      (else
        local.get $2i-4$0
        i64.const 3
        i64.rem_s
        i64.const 0
        i64.eq
        (if (then
          i32.const 9
          i32.const 5
          (local.set $_2$0)
          (local.set $_2$1)
        )
        (else
          local.get $2i-4$0
          i64.const 5
          i64.rem_s
          i64.const 0
          i64.eq
          (if (then
            i32.const 14
            i32.const 5
            (local.set $_3$0)
            (local.set $_3$1)
          )
          (else
            i32.const 19
            i32.const 0
            (local.set $_3$0)
            (local.set $_3$1)
          ))
          local.get $_3$1
          local.get $_3$0
          (local.set $_2$0)
          (local.set $_2$1)
        ))
        local.get $_2$1
        local.get $_2$0
        (local.set $_1$0)
        (local.set $_1$1)
      ))
      local.get $_1$1
      local.get $_1$0
      call $print_string
      local.get $2i-4$0
      local.get $7number-2$0
      i64.eq
      (if (then
        br $_0
      )
      (else
      ))
      local.get $2i-4$0
      i64.const 1
      i64.add
      (local.set $2i-4$0)
      br $_4
    )
  )
  return
)
(func $main
  i64.const 15
  call $5anon-3
  return
)
(start $main)
)
