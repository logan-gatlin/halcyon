(func $9function-1
  (local $2a-2$0 i64)
  (local $_0$0 i64)
  (local $_0$1 i64)
  (local $i$0 i64)
  (local $_3$0 i64)
  i64.const 0
  block $_1
  loop $_2
  i32.const 1
  if
  i64.const 1
  i64.const 2
  (local.set $_0$0)
  (local.set $_0$1)
  br $_1
  else
  i64.const 2
  i64.const 3
  (local.set $_0$0)
  (local.set $_0$1)
  br $_1
  end
  i64.const 1
  (local.set $i$0)
  br $_2
  end
  end
  (local.get $_0$1)
  (local.get $_0$0)
  (local.set $_3$0)
  drop
  (local.get $_3$0)
  i64.const 2
  i64.add
  (local.set $2a-2$0)
  return
)
