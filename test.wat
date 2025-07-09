(module
  (type (;0;) (func))
  (type (;1;) (struct (field (mut i64))))
  (type (;2;) (func (param (ref 1)) (result (ref 1))))
  (table (;0;) 2 2 funcref)
  (start 0)
  (elem (;0;) (i32.const 0) func 0 1)
  (func (;0;) (type 0)
    ref.func 1
    drop
  )
  (func (;1;) (type 2) (param (ref 1)) (result (ref 1))
    local.get 0
    i64.const 1
    struct.new 1
    i64.add
  )
)
