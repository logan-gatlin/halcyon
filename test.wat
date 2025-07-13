(module
  (type (;0;) (struct (field (mut i64))))
  (type (;1;) (func (param (ref 0) (ref 0)) (result (ref 0))))
  (type (;2;) (struct (field (mut f64))))
  (type (;3;) (func (param (ref 2) (ref 2)) (result (ref 2))))
  (type (;4;) (struct (field (mut i32))))
  (type (;5;) (func (param (ref 4) (ref 4)) (result (ref 4))))
  (type (;6;) (func))
  (type (;7;) (func (param (ref 0)) (result (ref 0))))
  (type (;8;) (struct (field (mut (ref 7)))))
  (table (;0;) 13 13 funcref)
  (start 11)
  (elem (;0;) (i32.const 0) func 0 1 2 3 4 5 6 7 8 9 10 11 12)
  (func (;0;) (type 1) (param (ref 0) (ref 0)) (result (ref 0))
    local.get 0
    struct.get 0 0
    local.get 1
    struct.get 0 0
    i64.add
    struct.new 0
  )
  (func (;1;) (type 1) (param (ref 0) (ref 0)) (result (ref 0))
    local.get 0
    struct.get 0 0
    local.get 1
    struct.get 0 0
    i64.sub
    struct.new 0
  )
  (func (;2;) (type 1) (param (ref 0) (ref 0)) (result (ref 0))
    local.get 0
    struct.get 0 0
    local.get 1
    struct.get 0 0
    i64.mul
    struct.new 0
  )
  (func (;3;) (type 1) (param (ref 0) (ref 0)) (result (ref 0))
    local.get 0
    struct.get 0 0
    local.get 1
    struct.get 0 0
    i64.div_s
    struct.new 0
  )
  (func (;4;) (type 3) (param (ref 2) (ref 2)) (result (ref 2))
    local.get 0
    struct.get 2 0
    local.get 1
    struct.get 2 0
    f64.add
    struct.new 2
  )
  (func (;5;) (type 3) (param (ref 2) (ref 2)) (result (ref 2))
    local.get 0
    struct.get 2 0
    local.get 1
    struct.get 2 0
    f64.sub
    struct.new 2
  )
  (func (;6;) (type 3) (param (ref 2) (ref 2)) (result (ref 2))
    local.get 0
    struct.get 2 0
    local.get 1
    struct.get 2 0
    f64.mul
    struct.new 2
  )
  (func (;7;) (type 3) (param (ref 2) (ref 2)) (result (ref 2))
    local.get 0
    struct.get 2 0
    local.get 1
    struct.get 2 0
    f64.div
    struct.new 2
  )
  (func (;8;) (type 5) (param (ref 4) (ref 4)) (result (ref 4))
    local.get 0
    struct.get 4 0
    local.get 1
    struct.get 4 0
    i32.and
    struct.new 4
  )
  (func (;9;) (type 5) (param (ref 4) (ref 4)) (result (ref 4))
    local.get 0
    struct.get 4 0
    local.get 1
    struct.get 4 0
    i32.or
    struct.new 4
  )
  (func (;10;) (type 5) (param (ref 4) (ref 4)) (result (ref 4))
    local.get 0
    struct.get 4 0
    local.get 1
    struct.get 4 0
    i32.xor
    struct.new 4
  )
  (func (;11;) (type 6)
    (local anyref (ref 7))
    ref.func 12
    struct.new 8
    local.set 0
    local.get 0
    ref.cast (ref 8)
    struct.get 8 0
    local.set 1
    i64.const 1
    struct.new 0
    local.get 1
    call_ref 7
    drop
  )
  (func (;12;) (type 7) (param (ref 0)) (result (ref 0))
    local.get 0
    ref.cast (ref 0)
    i64.const 1
    struct.new 0
    call 0
  )
)
