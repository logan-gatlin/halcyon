(module
  (type (;0;) (struct (field (mut i64))))
  (type (;1;) (func (param (ref 0) (ref 0)) (result (ref 0))))
  (type (;2;) (struct (field (mut f64))))
  (type (;3;) (func (param (ref 2) (ref 2)) (result (ref 2))))
  (type (;4;) (struct (field (mut i32))))
  (type (;5;) (func (param (ref 4) (ref 4)) (result (ref 4))))
  (type (;6;) (func))
  (type (;7;) (struct (field (mut anyref)) (field (mut anyref)) (field (mut anyref))))
  (type (;8;) (func (param anyref) (result (ref 7))))
  (type (;9;) (struct (field (mut (ref 8)))))
  (type (;10;) (struct (field (mut (ref 0))) (field (mut (ref 0))) (field (mut (ref 0)))))
  (type (;11;) (func (param (ref 0)) (result (ref 10))))
  (type (;12;) (struct (field (mut (ref 11)))))
  (type (;13;) (struct (field (mut (ref 4))) (field (mut (ref 4))) (field (mut (ref 4)))))
  (type (;14;) (func (param (ref 4)) (result (ref 13))))
  (type (;15;) (struct (field (mut (ref 14)))))
  (type (;16;) (array (mut i8)))
  (type (;17;) (struct (field (mut (ref 16))) (field (mut (ref 16))) (field (mut (ref 16)))))
  (type (;18;) (func (param (ref 16)) (result (ref 17))))
  (type (;19;) (struct (field (mut (ref 18)))))
  (type (;20;) (struct (field (mut i32))))
  (type (;21;) (struct (field (mut (ref 20))) (field (mut (ref 20))) (field (mut (ref 20)))))
  (type (;22;) (func (param (ref 20)) (result (ref 21))))
  (type (;23;) (struct (field (mut (ref 22)))))
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
    (local anyref (ref 11) (ref 14) (ref 18) (ref 22))
    ref.func 12
    struct.new 9
    local.set 0
    local.get 0
    ref.cast (ref 12)
    struct.get 12 0
    local.set 1
    i64.const 1
    struct.new 0
    local.get 1
    call_ref 11
    drop
    local.get 0
    ref.cast (ref 15)
    struct.get 15 0
    local.set 2
    i32.const 1
    struct.new 4
    local.get 2
    call_ref 14
    drop
    local.get 0
    ref.cast (ref 19)
    struct.get 19 0
    local.set 3
    i32.const 97
    i32.const 115
    i32.const 100
    i32.const 102
    i32.const 32
    i32.const 97
    i32.const 115
    i32.const 100
    i32.const 102
    i32.const 32
    i32.const 97
    i32.const 115
    i32.const 100
    i32.const 102
    array.new_fixed 16 14
    local.get 3
    call_ref 18
    drop
    local.get 0
    ref.cast (ref 23)
    struct.get 23 0
    local.set 4
    i32.const 97
    struct.new 20
    local.get 4
    call_ref 22
    drop
  )
  (func (;12;) (type 8) (param anyref) (result (ref 7))
    local.get 0
    ref.cast (ref any)
    local.get 0
    ref.cast (ref any)
    local.get 0
    ref.cast (ref any)
    struct.new 7
  )
)
