(module
  (type (;0;) (func))
  (type $integer (;1;) (struct (field i64)))
  (type $capture (;2;) (array (mut anyref)))
  (type $"(integer -> integer)" (;3;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (integer -> integer)" (;4;) (func (param (ref $integer) (ref $capture)) (result (ref $integer))))
  (type $real (;5;) (struct (field f64)))
  (type $"(real -> real)" (;6;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (real -> real)" (;7;) (func (param (ref $real) (ref $capture)) (result (ref $real))))
  (type $boolean (;8;) (struct (field i32)))
  (type $"(boolean -> boolean)" (;9;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (boolean -> boolean)" (;10;) (func (param (ref $boolean) (ref $capture)) (result (ref $boolean))))
  (type $"(integer -> (integer -> integer))" (;11;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (integer -> (integer -> integer))" (;12;) (func (param (ref $integer) (ref $capture)) (result (ref $"(integer -> integer)"))))
  (type $"(real -> (real -> real))" (;13;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (real -> (real -> real))" (;14;) (func (param (ref $real) (ref $capture)) (result (ref $"(real -> real)"))))
  (type $"(boolean -> (boolean -> boolean))" (;15;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (boolean -> (boolean -> boolean))" (;16;) (func (param (ref $boolean) (ref $capture)) (result (ref $"(boolean -> boolean)"))))
  (type $glyph (;17;) (struct (field i32)))
  (type $unit (;18;) (struct))
  (type $string (;19;) (array (mut i8)))
  (type $"('0 -> ('0 -> boolean))" (;20;) (struct (field i32) (field (ref $capture))))
  (type $"('0 -> boolean)" (;21;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('0 -> ('0 -> boolean))" (;22;) (func (param anyref (ref $capture)) (result (ref $"('0 -> boolean)"))))
  (type $"(raw) ('0 -> boolean)" (;23;) (func (param anyref (ref $capture)) (result (ref $boolean))))
  (type $"(unit -> '0)" (;24;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> '0)" (;25;) (func (param (ref $unit) (ref $capture)) (result anyref)))
  (type $"(string -> integer)" (;26;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (string -> integer)" (;27;) (func (param (ref $string) (ref $capture)) (result (ref $integer))))
  (type $"(string -> (string -> (string -> unit)))" (;28;) (struct (field i32) (field (ref $capture))))
  (type $"(string -> (string -> unit))" (;29;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (string -> (string -> (string -> unit)))" (;30;) (func (param (ref $string) (ref $capture)) (result (ref $"(string -> (string -> unit))"))))
  (type $"(string -> unit)" (;31;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (string -> (string -> unit))" (;32;) (func (param (ref $string) (ref $capture)) (result (ref $"(string -> unit)"))))
  (type $"(raw) (string -> unit)" (;33;) (func (param (ref $string) (ref $capture)) (result (ref $unit))))
  (type (;34;) (func (param i32 i32)))
  (type $"(integer -> string)" (;35;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (integer -> string)" (;36;) (func (param (ref $integer) (ref $capture)) (result (ref $string))))
  (type $"(string -> (string -> string))" (;37;) (struct (field i32) (field (ref $capture))))
  (type $"(string -> string)" (;38;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (string -> (string -> string))" (;39;) (func (param (ref $string) (ref $capture)) (result (ref $"(string -> string)"))))
  (type $"(raw) (string -> string)" (;40;) (func (param (ref $string) (ref $capture)) (result (ref $string))))
  (type $"(unit -> '3)" (;41;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> '3)" (;42;) (func (param (ref $unit) (ref $capture)) (result anyref)))
  (type $"(unit -> '1)" (;43;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> '1)" (;44;) (func (param (ref $unit) (ref $capture)) (result anyref)))
  (type $"(boolean -> unit)" (;45;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (boolean -> unit)" (;46;) (func (param (ref $boolean) (ref $capture)) (result (ref $unit))))
  (type $"({ specific_name: integer } -> integer)" (;47;) (struct (field i32) (field (ref $capture))))
  (type $"{ specific_name: integer }" (;48;) (struct (field (ref $integer))))
  (type $"(raw) ({ specific_name: integer } -> integer)" (;49;) (func (param (ref $"{ specific_name: integer }") (ref $capture)) (result (ref $integer))))
  (import "sys" "print_string" (func (;0;) (type 34)))
  (import "sys" "memory" (memory (;0;) 1))
  (table (;0;) 55 55 funcref)
  (global (;0;) (mut (ref null $"(integer -> integer)")) ref.null $"(integer -> integer)")
  (global (;1;) (mut (ref null $"(real -> real)")) ref.null $"(real -> real)")
  (global (;2;) (mut (ref null $"(boolean -> boolean)")) ref.null $"(boolean -> boolean)")
  (global (;3;) (mut (ref null $"(integer -> (integer -> integer))")) ref.null $"(integer -> (integer -> integer))")
  (global (;4;) (mut (ref null $"(integer -> (integer -> integer))")) ref.null $"(integer -> (integer -> integer))")
  (global (;5;) (mut (ref null $"(integer -> (integer -> integer))")) ref.null $"(integer -> (integer -> integer))")
  (global (;6;) (mut (ref null $"(integer -> (integer -> integer))")) ref.null $"(integer -> (integer -> integer))")
  (global (;7;) (mut (ref null $"(integer -> (integer -> integer))")) ref.null $"(integer -> (integer -> integer))")
  (global (;8;) (mut (ref null $"(real -> (real -> real))")) ref.null $"(real -> (real -> real))")
  (global (;9;) (mut (ref null $"(real -> (real -> real))")) ref.null $"(real -> (real -> real))")
  (global (;10;) (mut (ref null $"(real -> (real -> real))")) ref.null $"(real -> (real -> real))")
  (global (;11;) (mut (ref null $"(real -> (real -> real))")) ref.null $"(real -> (real -> real))")
  (global (;12;) (mut (ref null $"(boolean -> (boolean -> boolean))")) ref.null $"(boolean -> (boolean -> boolean))")
  (global (;13;) (mut (ref null $"(boolean -> (boolean -> boolean))")) ref.null $"(boolean -> (boolean -> boolean))")
  (global (;14;) (mut (ref null $"(boolean -> (boolean -> boolean))")) ref.null $"(boolean -> (boolean -> boolean))")
  (global (;15;) (mut (ref null $"('0 -> ('0 -> boolean))")) ref.null $"('0 -> ('0 -> boolean))")
  (global (;16;) (mut (ref null $"('0 -> ('0 -> boolean))")) ref.null $"('0 -> ('0 -> boolean))")
  (global (;17;) (mut (ref null $"('0 -> ('0 -> boolean))")) ref.null $"('0 -> ('0 -> boolean))")
  (global (;18;) (mut (ref null $"('0 -> ('0 -> boolean))")) ref.null $"('0 -> ('0 -> boolean))")
  (global (;19;) (mut (ref null $"('0 -> ('0 -> boolean))")) ref.null $"('0 -> ('0 -> boolean))")
  (global (;20;) (mut (ref null $"('0 -> ('0 -> boolean))")) ref.null $"('0 -> ('0 -> boolean))")
  (global (;21;) (mut (ref null $"(unit -> '0)")) ref.null $"(unit -> '0)")
  (global (;22;) (mut (ref null $"(string -> integer)")) ref.null $"(string -> integer)")
  (global (;23;) (mut (ref null $"(string -> (string -> (string -> unit)))")) ref.null $"(string -> (string -> (string -> unit)))")
  (global (;24;) (mut (ref null $"(string -> unit)")) ref.null $"(string -> unit)")
  (global (;25;) (mut (ref null $"(integer -> string)")) ref.null $"(integer -> string)")
  (global (;26;) (mut (ref null $"(string -> (string -> string))")) ref.null $"(string -> (string -> string))")
  (global (;27;) (mut (ref null $"(unit -> '3)")) ref.null $"(unit -> '3)")
  (global (;28;) (mut (ref null $"(boolean -> unit)")) ref.null $"(boolean -> unit)")
  (global (;29;) (mut (ref null $"(string -> integer)")) ref.null $"(string -> integer)")
  (global (;30;) (mut (ref null $"(string -> unit)")) ref.null $"(string -> unit)")
  (global (;31;) (mut (ref null $"({ specific_name: integer } -> integer)")) ref.null $"({ specific_name: integer } -> integer)")
  (global (;32;) (mut (ref null $unit)) ref.null $unit)
  (export "builtin:UnaryOp-" (global 0))
  (export "builtin:UnaryOp-." (global 1))
  (export "builtin:UnaryOpnot" (global 2))
  (export "builtin:BinaryOp+" (global 3))
  (export "builtin:BinaryOp-" (global 4))
  (export "builtin:BinaryOp*" (global 5))
  (export "builtin:BinaryOp/" (global 6))
  (export "builtin:BinaryOp%" (global 7))
  (export "builtin:BinaryOp+." (global 8))
  (export "builtin:BinaryOp-." (global 9))
  (export "builtin:BinaryOp*." (global 10))
  (export "builtin:BinaryOp/." (global 11))
  (export "builtin:BinaryOpand" (global 12))
  (export "builtin:BinaryOpor" (global 13))
  (export "builtin:BinaryOpxor" (global 14))
  (export "builtin:BinaryOp==" (global 15))
  (export "builtin:BinaryOp!=" (global 16))
  (export "builtin:BinaryOp<=" (global 17))
  (export "builtin:BinaryOp>=" (global 18))
  (export "builtin:BinaryOp<" (global 19))
  (export "builtin:BinaryOp>" (global 20))
  (export "builtin:panic" (global 21))
  (export "builtin:string_length" (global 22))
  (export "builtin:copy_string" (global 23))
  (export "builtin:print_string" (global 24))
  (export "builtin:zeroed_string" (global 25))
  (export "builtin:string_concatenate" (global 26))
  (export "std:panic" (global 27))
  (export "std:assert" (global 28))
  (export "std:string_length" (global 29))
  (export "std:print_string" (global 30))
  (export "TypeTest:tricky_inference" (global 31))
  (export "TypeTest:a" (global 32))
  (start 1)
  (elem (;0;) (i32.const 0) func 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 0 47 48 49 50 51 52 53 54)
  (func (;1;) (type 0)
    (local $0 (ref $"(boolean -> unit)")) (local (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)") (ref $"({ specific_name: integer } -> integer)"))
    i32.const 1
    array.new_fixed $capture 0
    struct.new $"(integer -> integer)"
    global.set 0
    i32.const 2
    array.new_fixed $capture 0
    struct.new $"(real -> real)"
    global.set 1
    i32.const 3
    array.new_fixed $capture 0
    struct.new $"(boolean -> boolean)"
    global.set 2
    i32.const 4
    array.new_fixed $capture 0
    struct.new $"(integer -> (integer -> integer))"
    global.set 3
    i32.const 6
    array.new_fixed $capture 0
    struct.new $"(integer -> (integer -> integer))"
    global.set 4
    i32.const 8
    array.new_fixed $capture 0
    struct.new $"(integer -> (integer -> integer))"
    global.set 5
    i32.const 10
    array.new_fixed $capture 0
    struct.new $"(integer -> (integer -> integer))"
    global.set 6
    i32.const 12
    array.new_fixed $capture 0
    struct.new $"(integer -> (integer -> integer))"
    global.set 7
    i32.const 14
    array.new_fixed $capture 0
    struct.new $"(real -> (real -> real))"
    global.set 8
    i32.const 16
    array.new_fixed $capture 0
    struct.new $"(real -> (real -> real))"
    global.set 9
    i32.const 18
    array.new_fixed $capture 0
    struct.new $"(real -> (real -> real))"
    global.set 10
    i32.const 20
    array.new_fixed $capture 0
    struct.new $"(real -> (real -> real))"
    global.set 11
    i32.const 22
    array.new_fixed $capture 0
    struct.new $"(boolean -> (boolean -> boolean))"
    global.set 12
    i32.const 24
    array.new_fixed $capture 0
    struct.new $"(boolean -> (boolean -> boolean))"
    global.set 13
    i32.const 26
    array.new_fixed $capture 0
    struct.new $"(boolean -> (boolean -> boolean))"
    global.set 14
    i32.const 28
    array.new_fixed $capture 0
    struct.new $"('0 -> ('0 -> boolean))"
    global.set 15
    i32.const 30
    array.new_fixed $capture 0
    struct.new $"('0 -> ('0 -> boolean))"
    global.set 16
    i32.const 32
    array.new_fixed $capture 0
    struct.new $"('0 -> ('0 -> boolean))"
    global.set 17
    i32.const 34
    array.new_fixed $capture 0
    struct.new $"('0 -> ('0 -> boolean))"
    global.set 18
    i32.const 36
    array.new_fixed $capture 0
    struct.new $"('0 -> ('0 -> boolean))"
    global.set 19
    i32.const 38
    array.new_fixed $capture 0
    struct.new $"('0 -> ('0 -> boolean))"
    global.set 20
    i32.const 40
    array.new_fixed $capture 0
    struct.new $"(unit -> '0)"
    global.set 21
    i32.const 41
    array.new_fixed $capture 0
    struct.new $"(string -> integer)"
    global.set 22
    i32.const 42
    array.new_fixed $capture 0
    struct.new $"(string -> (string -> (string -> unit)))"
    global.set 23
    i32.const 45
    array.new_fixed $capture 0
    struct.new $"(string -> unit)"
    global.set 24
    i32.const 47
    array.new_fixed $capture 0
    struct.new $"(integer -> string)"
    global.set 25
    i32.const 48
    array.new_fixed $capture 0
    struct.new $"(string -> (string -> string))"
    global.set 26
    i32.const 50
    array.new_fixed $capture 0
    struct.new $"(unit -> '3)"
    global.set 27
    i32.const 51
    array.new_fixed $capture 0
    struct.new $"(boolean -> unit)"
    global.set 28
    i32.const 52
    array.new_fixed $capture 0
    struct.new $"(string -> integer)"
    global.set 29
    i32.const 53
    array.new_fixed $capture 0
    struct.new $"(string -> unit)"
    global.set 30
    i32.const 54
    array.new_fixed $capture 0
    struct.new $"({ specific_name: integer } -> integer)"
    global.set 31
    global.get 28
    ref.as_non_null
    local.set $0
    global.get 31
    ref.as_non_null
    ref.cast (ref $"({ specific_name: integer } -> integer)")
    local.set 5
    i64.const 1
    struct.new $integer
    struct.new $"{ specific_name: integer }"
    local.get 5
    struct.get $"({ specific_name: integer } -> integer)" 1
    local.get 5
    struct.get $"({ specific_name: integer } -> integer)" 0
    call_indirect (type $"(raw) ({ specific_name: integer } -> integer)")
    ref.cast (ref $integer)
    global.get 3
    ref.as_non_null
    local.tee 3
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 3
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    local.set 4
    i64.const 1
    struct.new $integer
    local.get 4
    struct.get $"(integer -> integer)" 1
    local.get 4
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    global.get 15
    ref.as_non_null
    local.tee 1
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 1
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 2
    i64.const 2
    struct.new $integer
    local.get 2
    struct.get $"('0 -> boolean)" 1
    local.get 2
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get $0
    struct.get $"(boolean -> unit)" 1
    local.get $0
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    global.set 32
  )
  (func (;2;) (type $"(raw) (integer -> integer)") (param $0 (ref $integer)) (param (ref $capture)) (result (ref $integer))
    i64.const 0
    local.get $0
    struct.get $integer 0
    i64.sub
    struct.new $integer
  )
  (func (;3;) (type $"(raw) (real -> real)") (param $0 (ref $real)) (param (ref $capture)) (result (ref $real))
    f64.const 0x0p+0 (;=0;)
    local.get $0
    struct.get $real 0
    f64.sub
    struct.new $real
  )
  (func (;4;) (type $"(raw) (boolean -> boolean)") (param $0 (ref $boolean)) (param (ref $capture)) (result (ref $boolean))
    local.get $0
    struct.get $boolean 0
    i32.eqz
    struct.new $boolean
  )
  (func (;5;) (type $"(raw) (integer -> (integer -> integer))") (param $1 (ref $integer)) (param (ref $capture)) (result (ref $"(integer -> integer)"))
    i32.const 5
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(integer -> (integer -> integer))"
  )
  (func (;6;) (type $"(raw) (integer -> integer)") (param $0 (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 2
    local.get 2
    struct.get $integer 0
    local.get $0
    struct.get $integer 0
    i64.add
    struct.new $integer
  )
  (func (;7;) (type $"(raw) (integer -> (integer -> integer))") (param $1 (ref $integer)) (param (ref $capture)) (result (ref $"(integer -> integer)"))
    i32.const 7
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(integer -> (integer -> integer))"
  )
  (func (;8;) (type $"(raw) (integer -> integer)") (param $0 (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 2
    local.get 2
    struct.get $integer 0
    local.get $0
    struct.get $integer 0
    i64.sub
    struct.new $integer
  )
  (func (;9;) (type $"(raw) (integer -> (integer -> integer))") (param $1 (ref $integer)) (param (ref $capture)) (result (ref $"(integer -> integer)"))
    i32.const 9
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(integer -> (integer -> integer))"
  )
  (func (;10;) (type $"(raw) (integer -> integer)") (param $0 (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 2
    local.get 2
    struct.get $integer 0
    local.get $0
    struct.get $integer 0
    i64.mul
    struct.new $integer
  )
  (func (;11;) (type $"(raw) (integer -> (integer -> integer))") (param $1 (ref $integer)) (param (ref $capture)) (result (ref $"(integer -> integer)"))
    i32.const 11
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(integer -> (integer -> integer))"
  )
  (func (;12;) (type $"(raw) (integer -> integer)") (param $0 (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 2
    local.get 2
    struct.get $integer 0
    local.get $0
    struct.get $integer 0
    i64.div_s
    struct.new $integer
  )
  (func (;13;) (type $"(raw) (integer -> (integer -> integer))") (param $1 (ref $integer)) (param (ref $capture)) (result (ref $"(integer -> integer)"))
    i32.const 13
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(integer -> (integer -> integer))"
  )
  (func (;14;) (type $"(raw) (integer -> integer)") (param $0 (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 2
    local.get 2
    struct.get $integer 0
    local.get $0
    struct.get $integer 0
    i64.rem_s
    struct.new $integer
  )
  (func (;15;) (type $"(raw) (real -> (real -> real))") (param $1 (ref $real)) (param (ref $capture)) (result (ref $"(real -> real)"))
    i32.const 15
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(real -> (real -> real))"
  )
  (func (;16;) (type $"(raw) (real -> real)") (param $0 (ref $real)) (param (ref $capture)) (result (ref $real))
    (local (ref $real))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set 2
    local.get 2
    struct.get $real 0
    local.get $0
    struct.get $real 0
    f64.add
    struct.new $real
  )
  (func (;17;) (type $"(raw) (real -> (real -> real))") (param $1 (ref $real)) (param (ref $capture)) (result (ref $"(real -> real)"))
    i32.const 17
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(real -> (real -> real))"
  )
  (func (;18;) (type $"(raw) (real -> real)") (param $0 (ref $real)) (param (ref $capture)) (result (ref $real))
    (local (ref $real))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set 2
    local.get 2
    struct.get $real 0
    local.get $0
    struct.get $real 0
    f64.sub
    struct.new $real
  )
  (func (;19;) (type $"(raw) (real -> (real -> real))") (param $1 (ref $real)) (param (ref $capture)) (result (ref $"(real -> real)"))
    i32.const 19
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(real -> (real -> real))"
  )
  (func (;20;) (type $"(raw) (real -> real)") (param $0 (ref $real)) (param (ref $capture)) (result (ref $real))
    (local (ref $real))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set 2
    local.get 2
    struct.get $real 0
    local.get $0
    struct.get $real 0
    f64.mul
    struct.new $real
  )
  (func (;21;) (type $"(raw) (real -> (real -> real))") (param $1 (ref $real)) (param (ref $capture)) (result (ref $"(real -> real)"))
    i32.const 21
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(real -> (real -> real))"
  )
  (func (;22;) (type $"(raw) (real -> real)") (param $0 (ref $real)) (param (ref $capture)) (result (ref $real))
    (local (ref $real))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set 2
    local.get 2
    struct.get $real 0
    local.get $0
    struct.get $real 0
    f64.div
    struct.new $real
  )
  (func (;23;) (type $"(raw) (boolean -> (boolean -> boolean))") (param $1 (ref $boolean)) (param (ref $capture)) (result (ref $"(boolean -> boolean)"))
    i32.const 23
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(boolean -> (boolean -> boolean))"
  )
  (func (;24;) (type $"(raw) (boolean -> boolean)") (param $0 (ref $boolean)) (param (ref $capture)) (result (ref $boolean))
    (local (ref $boolean))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $boolean)
    local.set 2
    local.get 2
    struct.get $boolean 0
    local.get $0
    struct.get $boolean 0
    i32.and
    struct.new $boolean
  )
  (func (;25;) (type $"(raw) (boolean -> (boolean -> boolean))") (param $1 (ref $boolean)) (param (ref $capture)) (result (ref $"(boolean -> boolean)"))
    i32.const 25
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(boolean -> (boolean -> boolean))"
  )
  (func (;26;) (type $"(raw) (boolean -> boolean)") (param $0 (ref $boolean)) (param (ref $capture)) (result (ref $boolean))
    (local (ref $boolean))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $boolean)
    local.set 2
    local.get 2
    struct.get $boolean 0
    local.get $0
    struct.get $boolean 0
    i32.or
    struct.new $boolean
  )
  (func (;27;) (type $"(raw) (boolean -> (boolean -> boolean))") (param $1 (ref $boolean)) (param (ref $capture)) (result (ref $"(boolean -> boolean)"))
    i32.const 27
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(boolean -> (boolean -> boolean))"
  )
  (func (;28;) (type $"(raw) (boolean -> boolean)") (param $0 (ref $boolean)) (param (ref $capture)) (result (ref $boolean))
    (local (ref $boolean))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $boolean)
    local.set 2
    local.get 2
    struct.get $boolean 0
    local.get $0
    struct.get $boolean 0
    i32.xor
    struct.new $boolean
  )
  (func (;29;) (type $"(raw) ('0 -> ('0 -> boolean))") (param $1 anyref) (param (ref $capture)) (result (ref $"('0 -> boolean)"))
    i32.const 29
    local.get $1
    array.new_fixed $capture 1
    struct.new $"('0 -> ('0 -> boolean))"
  )
  (func (;30;) (type $"(raw) ('0 -> boolean)") (param $0 anyref) (param (ref $capture)) (result (ref $boolean))
    (local anyref i32 i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 2
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 2
                br_on_cast 0 (;@6;) anyref (ref $integer)
                br_on_cast 1 (;@5;) anyref (ref $real)
                br_on_cast 2 (;@4;) anyref (ref $boolean)
                br_on_cast 3 (;@3;) anyref (ref $glyph)
                br_on_cast 4 (;@2;) anyref (ref $unit)
                br_on_cast 5 (;@1;) anyref (ref $string)
                unreachable
              end
              ref.cast (ref $integer)
              struct.get $integer 0
              local.get $0
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.eq
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $0
            ref.cast (ref $real)
            struct.get $real 0
            f64.eq
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $0
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.eq
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $0
        ref.cast (ref $glyph)
        struct.get $glyph 0
        i32.eq
        struct.new $boolean
        return
      end
      i32.const 1
      struct.new $boolean
      return
    end
    ref.cast (ref $string)
    array.len
    local.get $0
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    local.get 2
    ref.cast (ref $string)
    array.len
    local.get $0
    ref.cast (ref $string)
    array.len
    i32.lt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    i32.const 0
    local.set 3
    local.get 2
    ref.cast (ref $string)
    array.len
    local.set 4
    loop ;; label = @1
      local.get 2
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $0
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      i32.ne
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 3
      i32.const 1
      i32.add
      local.tee 3
      local.get 4
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;31;) (type $"(raw) ('0 -> ('0 -> boolean))") (param $1 anyref) (param (ref $capture)) (result (ref $"('0 -> boolean)"))
    i32.const 31
    local.get $1
    array.new_fixed $capture 1
    struct.new $"('0 -> ('0 -> boolean))"
  )
  (func (;32;) (type $"(raw) ('0 -> boolean)") (param $0 anyref) (param (ref $capture)) (result (ref $boolean))
    (local anyref i32 i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 2
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 2
                br_on_cast 0 (;@6;) anyref (ref $integer)
                br_on_cast 1 (;@5;) anyref (ref $real)
                br_on_cast 2 (;@4;) anyref (ref $boolean)
                br_on_cast 3 (;@3;) anyref (ref $glyph)
                br_on_cast 4 (;@2;) anyref (ref $unit)
                br_on_cast 5 (;@1;) anyref (ref $string)
                unreachable
              end
              ref.cast (ref $integer)
              struct.get $integer 0
              local.get $0
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.ne
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $0
            ref.cast (ref $real)
            struct.get $real 0
            f64.ne
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $0
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.ne
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $0
        ref.cast (ref $glyph)
        struct.get $glyph 0
        i32.ne
        struct.new $boolean
        return
      end
      i32.const 0
      struct.new $boolean
      return
    end
    ref.cast (ref $string)
    array.len
    local.get $0
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    local.get 2
    ref.cast (ref $string)
    array.len
    local.get $0
    ref.cast (ref $string)
    array.len
    i32.lt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    i32.const 0
    local.set 3
    local.get 2
    ref.cast (ref $string)
    array.len
    local.set 4
    loop ;; label = @1
      local.get 2
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $0
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      i32.eq
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 3
      i32.const 1
      i32.add
      local.tee 3
      local.get 4
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;33;) (type $"(raw) ('0 -> ('0 -> boolean))") (param $1 anyref) (param (ref $capture)) (result (ref $"('0 -> boolean)"))
    i32.const 33
    local.get $1
    array.new_fixed $capture 1
    struct.new $"('0 -> ('0 -> boolean))"
  )
  (func (;34;) (type $"(raw) ('0 -> boolean)") (param $0 anyref) (param (ref $capture)) (result (ref $boolean))
    (local anyref i32 i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 2
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 2
                br_on_cast 0 (;@6;) anyref (ref $integer)
                br_on_cast 1 (;@5;) anyref (ref $real)
                br_on_cast 2 (;@4;) anyref (ref $boolean)
                br_on_cast 3 (;@3;) anyref (ref $glyph)
                br_on_cast 4 (;@2;) anyref (ref $unit)
                br_on_cast 5 (;@1;) anyref (ref $string)
                unreachable
              end
              ref.cast (ref $integer)
              struct.get $integer 0
              local.get $0
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.le_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $0
            ref.cast (ref $real)
            struct.get $real 0
            f64.le
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $0
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.le_s
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $0
        ref.cast (ref $glyph)
        struct.get $glyph 0
        i32.le_s
        struct.new $boolean
        return
      end
      i32.const 1
      struct.new $boolean
      return
    end
    ref.cast (ref $string)
    array.len
    local.get $0
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    local.get 2
    ref.cast (ref $string)
    array.len
    local.get $0
    ref.cast (ref $string)
    array.len
    i32.lt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    i32.const 0
    local.set 3
    local.get 2
    ref.cast (ref $string)
    array.len
    local.set 4
    loop ;; label = @1
      local.get 2
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $0
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      i32.gt_u
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 3
      i32.const 1
      i32.add
      local.tee 3
      local.get 4
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;35;) (type $"(raw) ('0 -> ('0 -> boolean))") (param $1 anyref) (param (ref $capture)) (result (ref $"('0 -> boolean)"))
    i32.const 35
    local.get $1
    array.new_fixed $capture 1
    struct.new $"('0 -> ('0 -> boolean))"
  )
  (func (;36;) (type $"(raw) ('0 -> boolean)") (param $0 anyref) (param (ref $capture)) (result (ref $boolean))
    (local anyref i32 i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 2
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 2
                br_on_cast 0 (;@6;) anyref (ref $integer)
                br_on_cast 1 (;@5;) anyref (ref $real)
                br_on_cast 2 (;@4;) anyref (ref $boolean)
                br_on_cast 3 (;@3;) anyref (ref $glyph)
                br_on_cast 4 (;@2;) anyref (ref $unit)
                br_on_cast 5 (;@1;) anyref (ref $string)
                unreachable
              end
              ref.cast (ref $integer)
              struct.get $integer 0
              local.get $0
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.ge_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $0
            ref.cast (ref $real)
            struct.get $real 0
            f64.ge
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $0
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.ge_s
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $0
        ref.cast (ref $glyph)
        struct.get $glyph 0
        i32.ge_s
        struct.new $boolean
        return
      end
      i32.const 1
      struct.new $boolean
      return
    end
    ref.cast (ref $string)
    array.len
    local.get $0
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    local.get 2
    ref.cast (ref $string)
    array.len
    local.get $0
    ref.cast (ref $string)
    array.len
    i32.lt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    i32.const 0
    local.set 3
    local.get 2
    ref.cast (ref $string)
    array.len
    local.set 4
    loop ;; label = @1
      local.get 2
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $0
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      i32.lt_u
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 3
      i32.const 1
      i32.add
      local.tee 3
      local.get 4
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;37;) (type $"(raw) ('0 -> ('0 -> boolean))") (param $1 anyref) (param (ref $capture)) (result (ref $"('0 -> boolean)"))
    i32.const 37
    local.get $1
    array.new_fixed $capture 1
    struct.new $"('0 -> ('0 -> boolean))"
  )
  (func (;38;) (type $"(raw) ('0 -> boolean)") (param $0 anyref) (param (ref $capture)) (result (ref $boolean))
    (local anyref i32 i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 2
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 2
                br_on_cast 0 (;@6;) anyref (ref $integer)
                br_on_cast 1 (;@5;) anyref (ref $real)
                br_on_cast 2 (;@4;) anyref (ref $boolean)
                br_on_cast 3 (;@3;) anyref (ref $glyph)
                br_on_cast 4 (;@2;) anyref (ref $unit)
                br_on_cast 5 (;@1;) anyref (ref $string)
                unreachable
              end
              ref.cast (ref $integer)
              struct.get $integer 0
              local.get $0
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.lt_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $0
            ref.cast (ref $real)
            struct.get $real 0
            f64.lt
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $0
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.lt_s
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $0
        ref.cast (ref $glyph)
        struct.get $glyph 0
        i32.lt_s
        struct.new $boolean
        return
      end
      i32.const 0
      struct.new $boolean
      return
    end
    ref.cast (ref $string)
    array.len
    local.get $0
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    local.get 2
    ref.cast (ref $string)
    array.len
    local.get $0
    ref.cast (ref $string)
    array.len
    i32.lt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    i32.const 0
    local.set 3
    local.get 2
    ref.cast (ref $string)
    array.len
    local.set 4
    loop ;; label = @1
      local.get 2
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $0
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      i32.ge_u
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 3
      i32.const 1
      i32.add
      local.tee 3
      local.get 4
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;39;) (type $"(raw) ('0 -> ('0 -> boolean))") (param $1 anyref) (param (ref $capture)) (result (ref $"('0 -> boolean)"))
    i32.const 39
    local.get $1
    array.new_fixed $capture 1
    struct.new $"('0 -> ('0 -> boolean))"
  )
  (func (;40;) (type $"(raw) ('0 -> boolean)") (param $0 anyref) (param (ref $capture)) (result (ref $boolean))
    (local anyref i32 i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 2
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 2
                br_on_cast 0 (;@6;) anyref (ref $integer)
                br_on_cast 1 (;@5;) anyref (ref $real)
                br_on_cast 2 (;@4;) anyref (ref $boolean)
                br_on_cast 3 (;@3;) anyref (ref $glyph)
                br_on_cast 4 (;@2;) anyref (ref $unit)
                br_on_cast 5 (;@1;) anyref (ref $string)
                unreachable
              end
              ref.cast (ref $integer)
              struct.get $integer 0
              local.get $0
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.gt_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $0
            ref.cast (ref $real)
            struct.get $real 0
            f64.gt
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $0
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.gt_s
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $0
        ref.cast (ref $glyph)
        struct.get $glyph 0
        i32.gt_s
        struct.new $boolean
        return
      end
      i32.const 0
      struct.new $boolean
      return
    end
    ref.cast (ref $string)
    array.len
    local.get $0
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    local.get 2
    ref.cast (ref $string)
    array.len
    local.get $0
    ref.cast (ref $string)
    array.len
    i32.lt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    i32.const 0
    local.set 3
    local.get 2
    ref.cast (ref $string)
    array.len
    local.set 4
    loop ;; label = @1
      local.get 2
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $0
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      i32.le_u
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 3
      i32.const 1
      i32.add
      local.tee 3
      local.get 4
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;41;) (type $"(raw) (unit -> '0)") (param $0 (ref $unit)) (param (ref $capture)) (result anyref)
    unreachable
  )
  (func (;42;) (type $"(raw) (string -> integer)") (param $0 (ref $string)) (param (ref $capture)) (result (ref $integer))
    local.get $0
    array.len
    i64.extend_i32_u
    struct.new $integer
  )
  (func (;43;) (type $"(raw) (string -> (string -> (string -> unit)))") (param $1 (ref $string)) (param (ref $capture)) (result (ref $"(string -> (string -> unit))"))
    i32.const 43
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(string -> (string -> (string -> unit)))"
  )
  (func (;44;) (type $"(raw) (string -> (string -> unit))") (param $2 (ref $string)) (param (ref $capture)) (result (ref $"(string -> unit)"))
    (local $0 (ref $string))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $string)
    local.set $0
    i32.const 44
    local.get $0
    local.get $2
    array.new_fixed $capture 2
    struct.new $"(string -> (string -> unit))"
  )
  (func (;45;) (type $"(raw) (string -> unit)") (param $0 (ref $string)) (param (ref $capture)) (result (ref $unit))
    (local $index (ref $string)) (local $length (ref $string))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $string)
    local.set $index
    local.get 1
    i32.const 1
    array.get $capture
    ref.cast (ref $string)
    local.set $length
    unreachable
  )
  (func (;46;) (type $"(raw) (string -> unit)") (param $0 (ref $string)) (param (ref $capture)) (result (ref $unit))
    (local i32 i32)
    i32.const 0
    local.set 2
    local.get $0
    array.len
    local.set 3
    loop ;; label = @1
      local.get 2
      local.get 3
      i32.lt_u
      if ;; label = @2
        local.get 2
        local.get $0
        local.get 2
        array.get_u $string
        i32.store8
        local.get 2
        i32.const 1
        i32.add
        local.set 2
        br 1 (;@1;)
      end
    end
    i32.const 0
    local.get 3
    i32.const 46
    call_indirect (type 34)
    struct.new $unit
  )
  (func (;47;) (type $"(raw) (integer -> string)") (param $0 (ref $integer)) (param (ref $capture)) (result (ref $string))
    local.get $0
    struct.get $integer 0
    i32.wrap_i64
    array.new_default $string
  )
  (func (;48;) (type $"(raw) (string -> (string -> string))") (param $1 (ref $string)) (param (ref $capture)) (result (ref $"(string -> string)"))
    i32.const 49
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(string -> (string -> string))"
  )
  (func (;49;) (type $"(raw) (string -> string)") (param $unit (ref $string)) (param (ref $capture)) (result (ref $string))
    (local (ref $string) i32 i32 i32 i32 (ref $string))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $string)
    local.set 2
    local.get 2
    array.len
    local.tee 4
    local.get $unit
    array.len
    local.tee 5
    i32.add
    local.tee 6
    array.new_default $string
    unreachable
  )
  (func (;50;) (type $"(raw) (unit -> '3)") (param $condition#0 (ref $unit)) (param (ref $capture)) (result anyref)
    (local (ref $"(unit -> '1)"))
    global.get 21
    ref.as_non_null
    local.set 2
    struct.new $unit
    local.get 2
    struct.get $"(unit -> '1)" 1
    local.get 2
    struct.get $"(unit -> '1)" 0
    call_indirect (type $"(raw) (unit -> '1)")
    ref.cast (ref any)
  )
  (func (;51;) (type $"(raw) (boolean -> unit)") (param $s#1 (ref $boolean)) (param (ref $capture)) (result (ref $unit))
    (local (ref $"(unit -> '3)"))
    local.get $s#1
    ref.cast (ref $boolean)
    struct.get $boolean 0
    if (result (ref $unit)) ;; label = @1
      struct.new $unit
    else
      global.get 27
      ref.as_non_null
      ref.cast (ref $"(unit -> '3)")
      local.set 2
      struct.new $unit
      local.get 2
      struct.get $"(unit -> '3)" 1
      local.get 2
      struct.get $"(unit -> '3)" 0
      call_indirect (type $"(raw) (unit -> '3)")
      ref.cast (ref $unit)
    end
  )
  (func (;52;) (type $"(raw) (string -> integer)") (param $s#2 (ref $string)) (param (ref $capture)) (result (ref $integer))
    (local (ref $"(string -> integer)"))
    global.get 22
    ref.as_non_null
    local.set 2
    local.get $s#2
    ref.cast (ref $string)
    local.get 2
    struct.get $"(string -> integer)" 1
    local.get 2
    struct.get $"(string -> integer)" 0
    call_indirect (type $"(raw) (string -> integer)")
    ref.cast (ref $integer)
  )
  (func (;53;) (type $"(raw) (string -> unit)") (param $a#0 (ref $string)) (param (ref $capture)) (result (ref $unit))
    (local (ref $"(string -> unit)"))
    global.get 24
    ref.as_non_null
    local.set 2
    local.get $a#0
    ref.cast (ref $string)
    local.get 2
    struct.get $"(string -> unit)" 1
    local.get 2
    struct.get $"(string -> unit)" 0
    call_indirect (type $"(raw) (string -> unit)")
    ref.cast (ref $unit)
  )
  (func (;54;) (type $"(raw) ({ specific_name: integer } -> integer)") (param (ref $"{ specific_name: integer }") (ref $capture)) (result (ref $integer))
    local.get 0
    ref.cast (ref $"{ specific_name: integer }")
    struct.get $"{ specific_name: integer }" 0
  )
)
