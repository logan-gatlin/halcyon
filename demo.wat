(module
  (type (;0;) (func))
  (type $integer (;1;) (struct (field i64)))
  (table (;0;) 1 1 funcref)
  (global (;0;) (mut (ref null $integer)) ref.null $integer)
  (global (;1;) (mut (ref null $integer)) ref.null $integer)
  (export "A:a" (global 0))
  (export "B:a" (global 1))
  (start 0)
  (elem (;0;) (i32.const 0) func 0)
  (func (;0;) (type 0)
    i64.const 1
    struct.new $integer
    global.set 0
    global.get 0
    ref.as_non_null
    global.set 1
  )
)
