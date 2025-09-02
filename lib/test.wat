(module
  (type (;0;) (func))
  (type (;1;) (struct))
  (type (;2;) (struct (field i64)))
  (type (;3;) (struct (field f64)))
  (type (;4;) (struct (field i32)))
  (type (;5;) (array i8))
  (type (;6;) (struct (field i32)))
  (type (;7;) (array anyref))
  (type (;8;) (func (param anyref (ref 7)) (result anyref)))
  (type (;9;) (struct (field (ref 8)) (field (ref 7))))
  (type (;10;) (func (param (ref 9) (ref 7)) (result anyref)))
  (type (;11;) (struct (field (ref 10)) (field (ref 7))))
  (type (;12;) (func (param anyref (ref 7)) (result (ref 11))))
  (type (;13;) (struct (field (ref 12)) (field (ref 7))))
  (type (;14;) (func (param (ref 1) (ref 7)) (result anyref)))
  (type (;15;) (struct (field (ref 14)) (field (ref 7))))
  (type (;16;) (func (param i64) (result i64)))
  (import "sys" "println" (func (;0;) (type 16)))
  (import "sys" "memory" (memory (;0;) 1))
  (table (;0;) 6 6 funcref)
  (global (;0;) (mut (ref null 13)) ref.null 13)
  (global (;1;) (mut (ref null 15)) ref.null 15)
  (global (;2;) (mut (ref null 9)) ref.null 9)
  (elem (;0;) (i32.const 0) func 1 2 3 0 4 5 6)
  (func (;1;) (type 10) (param (ref 9) (ref 7)) (result anyref)
    (local anyref (ref 9) (ref 9))
    local.get 1
    i32.const 0
    array.get 7
    local.set 2
    local.get 0
    ref.cast (ref 9)
    ref.cast (ref 9)
    local.set 3
    local.get 2
    local.get 3
    local.tee 4
    struct.get 9 1
    local.get 4
    struct.get 9 0
    call_ref 8
  )
  (func (;2;) (type 12) (param anyref (ref 7)) (result (ref 11))
    ref.func 0
    local.get 0
    array.new_fixed 7 1
    struct.new 11
  )
  (func (;3;) (type 14) (param (ref 1) (ref 7)) (result anyref)
    unreachable
  )
  (func (;4;) (type 0)
    (local anyref anyref)
    ref.func 1
    array.new_fixed 7 0
    struct.new 13
    global.set 0
    ref.func 2
    array.new_fixed 7 0
    struct.new 15
    global.set 1
  )
  (func (;5;) (type 8) (param anyref (ref 7)) (result anyref)
    local.get 0
  )
  (func (;6;) (type 0)
    (local anyref anyref)
    ref.func 5
    array.new_fixed 7 0
    struct.new 9
    global.set 2
  )
)
