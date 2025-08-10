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
  (type $"(string -> unit)" (;28;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (string -> unit)" (;29;) (func (param (ref $string) (ref $capture)) (result (ref $unit))))
  (type (;30;) (func (param i32 i32)))
  (type $"(string -> (string -> string))" (;31;) (struct (field i32) (field (ref $capture))))
  (type $"(string -> string)" (;32;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (string -> (string -> string))" (;33;) (func (param (ref $string) (ref $capture)) (result (ref $"(string -> string)"))))
  (type $"(raw) (string -> string)" (;34;) (func (param (ref $string) (ref $capture)) (result (ref $string))))
  (type $"(unit -> '1)" (;35;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> '1)" (;36;) (func (param (ref $unit) (ref $capture)) (result anyref)))
  (type $"(unit -> '2)" (;37;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> '2)" (;38;) (func (param (ref $unit) (ref $capture)) (result anyref)))
  (type $"(boolean -> unit)" (;39;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (boolean -> unit)" (;40;) (func (param (ref $boolean) (ref $capture)) (result (ref $unit))))
  (import "sys" "print_string" (func (;0;) (type 30)))
  (import "sys" "memory" (memory (;0;) 1))
  (table (;0;) 52 52 funcref)
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
  (global (;23;) (mut (ref null $"(string -> unit)")) ref.null $"(string -> unit)")
  (global (;24;) (mut (ref null $"(string -> (string -> string))")) ref.null $"(string -> (string -> string))")
  (global (;25;) (mut (ref null $"(unit -> '1)")) ref.null $"(unit -> '1)")
  (global (;26;) (mut (ref null $"(boolean -> unit)")) ref.null $"(boolean -> unit)")
  (global (;27;) (mut (ref null $"(string -> integer)")) ref.null $"(string -> integer)")
  (global (;28;) (mut (ref null $"(string -> unit)")) ref.null $"(string -> unit)")
  (global (;29;) (mut (ref null $"(string -> (string -> string))")) ref.null $"(string -> (string -> string))")
  (global (;30;) (mut (ref null $unit)) ref.null $unit)
  (global (;31;) (mut (ref null $unit)) ref.null $unit)
  (global (;32;) (mut (ref null $unit)) ref.null $unit)
  (global (;33;) (mut (ref null $unit)) ref.null $unit)
  (global (;34;) (mut (ref null $unit)) ref.null $unit)
  (global (;35;) (mut (ref null $unit)) ref.null $unit)
  (global (;36;) (mut (ref null $unit)) ref.null $unit)
  (global (;37;) (mut (ref null $unit)) ref.null $unit)
  (global (;38;) (mut (ref null $unit)) ref.null $unit)
  (global (;39;) (mut (ref null $unit)) ref.null $unit)
  (global (;40;) (mut (ref null $unit)) ref.null $unit)
  (global (;41;) (mut (ref null $unit)) ref.null $unit)
  (global (;42;) (mut (ref null $unit)) ref.null $unit)
  (global (;43;) (mut (ref null $unit)) ref.null $unit)
  (global (;44;) (mut (ref null $unit)) ref.null $unit)
  (global (;45;) (mut (ref null $unit)) ref.null $unit)
  (global (;46;) (mut (ref null $unit)) ref.null $unit)
  (global (;47;) (mut (ref null $unit)) ref.null $unit)
  (global (;48;) (mut (ref null $unit)) ref.null $unit)
  (global (;49;) (mut (ref null $unit)) ref.null $unit)
  (global (;50;) (mut (ref null $unit)) ref.null $unit)
  (global (;51;) (mut (ref null $unit)) ref.null $unit)
  (global (;52;) (mut (ref null $unit)) ref.null $unit)
  (global (;53;) (mut (ref null $unit)) ref.null $unit)
  (global (;54;) (mut (ref null $unit)) ref.null $unit)
  (global (;55;) (mut (ref null $unit)) ref.null $unit)
  (global (;56;) (mut (ref null $unit)) ref.null $unit)
  (global (;57;) (mut (ref null $unit)) ref.null $unit)
  (global (;58;) (mut (ref null $unit)) ref.null $unit)
  (global (;59;) (mut (ref null $unit)) ref.null $unit)
  (global (;60;) (mut (ref null $unit)) ref.null $unit)
  (global (;61;) (mut (ref null $unit)) ref.null $unit)
  (global (;62;) (mut (ref null $unit)) ref.null $unit)
  (global (;63;) (mut (ref null $unit)) ref.null $unit)
  (global (;64;) (mut (ref null $unit)) ref.null $unit)
  (global (;65;) (mut (ref null $unit)) ref.null $unit)
  (global (;66;) (mut (ref null $unit)) ref.null $unit)
  (global (;67;) (mut (ref null $unit)) ref.null $unit)
  (global (;68;) (mut (ref null $unit)) ref.null $unit)
  (global (;69;) (mut (ref null $unit)) ref.null $unit)
  (global (;70;) (mut (ref null $unit)) ref.null $unit)
  (global (;71;) (mut (ref null $unit)) ref.null $unit)
  (global (;72;) (mut (ref null $unit)) ref.null $unit)
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
  (export "builtin:print_string" (global 23))
  (export "builtin:string_concatenate" (global 24))
  (export "std:panic" (global 25))
  (export "std:assert" (global 26))
  (export "std:string_length" (global 27))
  (export "std:print_string" (global 28))
  (export "std:string_concatenate" (global 29))
  (export "OpTest:_#0" (global 30))
  (export "OpTest:_#1" (global 31))
  (export "OpTest:_#2" (global 32))
  (export "OpTest:_#3" (global 33))
  (export "OpTest:_#4" (global 34))
  (export "OpTest:_#5" (global 35))
  (export "OpTest:_#6" (global 36))
  (export "OpTest:_#7" (global 37))
  (export "OpTest:_#8" (global 38))
  (export "OpTest:_#9" (global 39))
  (export "OpTest:_#10" (global 40))
  (export "OpTest:_#11" (global 41))
  (export "OpTest:_#12" (global 42))
  (export "OpTest:_#13" (global 43))
  (export "OpTest:_#14" (global 44))
  (export "OpTest:_#15" (global 45))
  (export "OpTest:_#16" (global 46))
  (export "OpTest:_#17" (global 47))
  (export "OpTest:_#18" (global 48))
  (export "OpTest:_#19" (global 49))
  (export "OpTest:_#20" (global 50))
  (export "OpTest:_#21" (global 51))
  (export "OpTest:_#22" (global 52))
  (export "OpTest:_#23" (global 53))
  (export "OpTest:_#24" (global 54))
  (export "OpTest:_#25" (global 55))
  (export "OpTest:_#26" (global 56))
  (export "OpTest:_#27" (global 57))
  (export "OpTest:_#28" (global 58))
  (export "OpTest:_#29" (global 59))
  (export "OpTest:_#30" (global 60))
  (export "OpTest:_#31" (global 61))
  (export "OpTest:_#32" (global 62))
  (export "OpTest:_#33" (global 63))
  (export "OpTest:_#34" (global 64))
  (export "OpTest:_#35" (global 65))
  (export "OpTest:_#36" (global 66))
  (export "OpTest:_#37" (global 67))
  (export "OpTest:_#38" (global 68))
  (export "OpTest:_#39" (global 69))
  (export "OpTest:_#40" (global 70))
  (export "OpTest:_#41" (global 71))
  (export "OpTest:_#42" (global 72))
  (start 1)
  (elem (;0;) (i32.const 0) func 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 0 44 45 46 47 48 49 50 51)
  (func (;1;) (type 0)
    (local $0 (ref $"(unit -> '1)")) (local (ref $"(boolean -> unit)") (ref $"(string -> integer)") (ref $"(string -> unit)") (ref $"(string -> (string -> string))") (ref $unit) (ref $"(boolean -> unit)") (ref $"(boolean -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"(boolean -> boolean)") (ref $"(boolean -> boolean)") (ref $"(boolean -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"(integer -> integer)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)") (ref $unit) (ref $"(boolean -> unit)") (ref $"(real -> real)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"(real -> (real -> real))") (ref $"(real -> real)") (ref $unit) (ref $"(boolean -> unit)") (ref $"(real -> real)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"(real -> (real -> real))") (ref $"(real -> real)") (ref $unit) (ref $"(boolean -> unit)") (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"(integer -> integer)") (ref $unit) (ref $"(boolean -> unit)") (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"(real -> (real -> real))") (ref $"(real -> real)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"(real -> (real -> real))") (ref $"(real -> real)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"(real -> (real -> real))") (ref $"(real -> real)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"(real -> (real -> real))") (ref $"(real -> real)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"(boolean -> (boolean -> boolean))") (ref $"(boolean -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"(boolean -> (boolean -> boolean))") (ref $"(boolean -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"(boolean -> (boolean -> boolean))") (ref $"(boolean -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"(boolean -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)"))
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
    struct.new $"(string -> unit)"
    global.set 23
    i32.const 44
    array.new_fixed $capture 0
    struct.new $"(string -> (string -> string))"
    global.set 24
    i32.const 46
    array.new_fixed $capture 0
    struct.new $"(unit -> '1)"
    local.set $0
    local.get $0
    global.set 25
    i32.const 47
    array.new_fixed $capture 0
    struct.new $"(boolean -> unit)"
    local.set 1
    local.get 1
    global.set 26
    i32.const 48
    array.new_fixed $capture 0
    struct.new $"(string -> integer)"
    local.set 2
    local.get 2
    global.set 27
    i32.const 49
    array.new_fixed $capture 0
    struct.new $"(string -> unit)"
    local.set 3
    local.get 3
    global.set 28
    i32.const 50
    array.new_fixed $capture 0
    struct.new $"(string -> (string -> string))"
    local.set 4
    local.get 4
    global.set 29
    global.get 26
    ref.as_non_null
    local.set 6
    i32.const 0
    struct.new $boolean
    global.get 2
    ref.as_non_null
    local.tee 7
    struct.get $"(boolean -> boolean)" 1
    local.get 7
    struct.get $"(boolean -> boolean)" 0
    call_indirect (type $"(raw) (boolean -> boolean)")
    local.get 6
    struct.get $"(boolean -> unit)" 1
    local.get 6
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 5
    local.get 5
    global.set 30
    global.get 26
    ref.as_non_null
    local.set 9
    i32.const 0
    struct.new $boolean
    global.get 2
    ref.as_non_null
    local.tee 10
    struct.get $"(boolean -> boolean)" 1
    local.get 10
    struct.get $"(boolean -> boolean)" 0
    call_indirect (type $"(raw) (boolean -> boolean)")
    global.get 2
    ref.as_non_null
    local.tee 11
    struct.get $"(boolean -> boolean)" 1
    local.get 11
    struct.get $"(boolean -> boolean)" 0
    call_indirect (type $"(raw) (boolean -> boolean)")
    global.get 2
    ref.as_non_null
    local.tee 12
    struct.get $"(boolean -> boolean)" 1
    local.get 12
    struct.get $"(boolean -> boolean)" 0
    call_indirect (type $"(raw) (boolean -> boolean)")
    local.get 9
    struct.get $"(boolean -> unit)" 1
    local.get 9
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 8
    local.get 8
    global.set 31
    global.get 26
    ref.as_non_null
    local.set 14
    i64.const 1
    struct.new $integer
    global.get 0
    ref.as_non_null
    local.tee 15
    struct.get $"(integer -> integer)" 1
    local.get 15
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    global.get 15
    ref.as_non_null
    local.tee 16
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 16
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 17
    i64.const 0
    struct.new $integer
    global.get 4
    ref.as_non_null
    local.tee 18
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 18
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    local.set 19
    i64.const 1
    struct.new $integer
    local.get 19
    struct.get $"(integer -> integer)" 1
    local.get 19
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    local.get 17
    struct.get $"('0 -> boolean)" 1
    local.get 17
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 14
    struct.get $"(boolean -> unit)" 1
    local.get 14
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 13
    local.get 13
    global.set 32
    global.get 26
    ref.as_non_null
    local.set 21
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    global.get 1
    ref.as_non_null
    local.tee 22
    struct.get $"(real -> real)" 1
    local.get 22
    struct.get $"(real -> real)" 0
    call_indirect (type $"(raw) (real -> real)")
    global.get 15
    ref.as_non_null
    local.tee 23
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 23
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 24
    f64.const 0x0p+0 (;=0;)
    struct.new $real
    global.get 9
    ref.as_non_null
    local.tee 25
    struct.get $"(real -> (real -> real))" 1
    local.get 25
    struct.get $"(real -> (real -> real))" 0
    call_indirect (type $"(raw) (real -> (real -> real))")
    local.set 26
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    local.get 26
    struct.get $"(real -> real)" 1
    local.get 26
    struct.get $"(real -> real)" 0
    call_indirect (type $"(raw) (real -> real)")
    local.get 24
    struct.get $"('0 -> boolean)" 1
    local.get 24
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 21
    struct.get $"(boolean -> unit)" 1
    local.get 21
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 20
    local.get 20
    global.set 33
    global.get 26
    ref.as_non_null
    local.set 28
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    global.get 1
    ref.as_non_null
    local.tee 29
    struct.get $"(real -> real)" 1
    local.get 29
    struct.get $"(real -> real)" 0
    call_indirect (type $"(raw) (real -> real)")
    global.get 15
    ref.as_non_null
    local.tee 30
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 30
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 31
    f64.const 0x0p+0 (;=0;)
    struct.new $real
    global.get 9
    ref.as_non_null
    local.tee 32
    struct.get $"(real -> (real -> real))" 1
    local.get 32
    struct.get $"(real -> (real -> real))" 0
    call_indirect (type $"(raw) (real -> (real -> real))")
    local.set 33
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    local.get 33
    struct.get $"(real -> real)" 1
    local.get 33
    struct.get $"(real -> real)" 0
    call_indirect (type $"(raw) (real -> real)")
    local.get 31
    struct.get $"('0 -> boolean)" 1
    local.get 31
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 28
    struct.get $"(boolean -> unit)" 1
    local.get 28
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 27
    local.get 27
    global.set 34
    global.get 26
    ref.as_non_null
    local.set 35
    i64.const 1
    struct.new $integer
    global.get 3
    ref.as_non_null
    local.tee 36
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 36
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    local.set 37
    i64.const 2
    struct.new $integer
    local.get 37
    struct.get $"(integer -> integer)" 1
    local.get 37
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    global.get 15
    ref.as_non_null
    local.tee 38
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 38
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 39
    i64.const 3
    struct.new $integer
    local.get 39
    struct.get $"('0 -> boolean)" 1
    local.get 39
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 35
    struct.get $"(boolean -> unit)" 1
    local.get 35
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 34
    local.get 34
    global.set 35
    global.get 26
    ref.as_non_null
    local.set 41
    i64.const 1
    struct.new $integer
    global.get 4
    ref.as_non_null
    local.tee 42
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 42
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    local.set 43
    i64.const 2
    struct.new $integer
    local.get 43
    struct.get $"(integer -> integer)" 1
    local.get 43
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    global.get 15
    ref.as_non_null
    local.tee 44
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 44
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 45
    i64.const 1
    struct.new $integer
    global.get 0
    ref.as_non_null
    local.tee 46
    struct.get $"(integer -> integer)" 1
    local.get 46
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    local.get 45
    struct.get $"('0 -> boolean)" 1
    local.get 45
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 41
    struct.get $"(boolean -> unit)" 1
    local.get 41
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 40
    local.get 40
    global.set 36
    global.get 26
    ref.as_non_null
    local.set 48
    i64.const 1
    struct.new $integer
    global.get 5
    ref.as_non_null
    local.tee 49
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 49
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    local.set 50
    i64.const 2
    struct.new $integer
    local.get 50
    struct.get $"(integer -> integer)" 1
    local.get 50
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    global.get 15
    ref.as_non_null
    local.tee 51
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 51
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 52
    i64.const 2
    struct.new $integer
    local.get 52
    struct.get $"('0 -> boolean)" 1
    local.get 52
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 48
    struct.get $"(boolean -> unit)" 1
    local.get 48
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 47
    local.get 47
    global.set 37
    global.get 26
    ref.as_non_null
    local.set 54
    i64.const 1
    struct.new $integer
    global.get 6
    ref.as_non_null
    local.tee 55
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 55
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    local.set 56
    i64.const 2
    struct.new $integer
    local.get 56
    struct.get $"(integer -> integer)" 1
    local.get 56
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    global.get 15
    ref.as_non_null
    local.tee 57
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 57
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 58
    i64.const 0
    struct.new $integer
    local.get 58
    struct.get $"('0 -> boolean)" 1
    local.get 58
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 54
    struct.get $"(boolean -> unit)" 1
    local.get 54
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 53
    local.get 53
    global.set 38
    global.get 26
    ref.as_non_null
    local.set 60
    i64.const 1
    struct.new $integer
    global.get 7
    ref.as_non_null
    local.tee 61
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 61
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    local.set 62
    i64.const 2
    struct.new $integer
    local.get 62
    struct.get $"(integer -> integer)" 1
    local.get 62
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    global.get 15
    ref.as_non_null
    local.tee 63
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 63
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 64
    i64.const 1
    struct.new $integer
    local.get 64
    struct.get $"('0 -> boolean)" 1
    local.get 64
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 60
    struct.get $"(boolean -> unit)" 1
    local.get 60
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 59
    local.get 59
    global.set 39
    global.get 26
    ref.as_non_null
    local.set 66
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    global.get 8
    ref.as_non_null
    local.tee 67
    struct.get $"(real -> (real -> real))" 1
    local.get 67
    struct.get $"(real -> (real -> real))" 0
    call_indirect (type $"(raw) (real -> (real -> real))")
    local.set 68
    f64.const 0x1p+1 (;=2;)
    struct.new $real
    local.get 68
    struct.get $"(real -> real)" 1
    local.get 68
    struct.get $"(real -> real)" 0
    call_indirect (type $"(raw) (real -> real)")
    global.get 15
    ref.as_non_null
    local.tee 69
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 69
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 70
    f64.const 0x1.8p+1 (;=3;)
    struct.new $real
    local.get 70
    struct.get $"('0 -> boolean)" 1
    local.get 70
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 66
    struct.get $"(boolean -> unit)" 1
    local.get 66
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 65
    local.get 65
    global.set 40
    global.get 26
    ref.as_non_null
    local.set 72
    f64.const 0x1p+1 (;=2;)
    struct.new $real
    global.get 9
    ref.as_non_null
    local.tee 73
    struct.get $"(real -> (real -> real))" 1
    local.get 73
    struct.get $"(real -> (real -> real))" 0
    call_indirect (type $"(raw) (real -> (real -> real))")
    local.set 74
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    local.get 74
    struct.get $"(real -> real)" 1
    local.get 74
    struct.get $"(real -> real)" 0
    call_indirect (type $"(raw) (real -> real)")
    global.get 15
    ref.as_non_null
    local.tee 75
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 75
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 76
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    local.get 76
    struct.get $"('0 -> boolean)" 1
    local.get 76
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 72
    struct.get $"(boolean -> unit)" 1
    local.get 72
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 71
    local.get 71
    global.set 41
    global.get 26
    ref.as_non_null
    local.set 78
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    global.get 10
    ref.as_non_null
    local.tee 79
    struct.get $"(real -> (real -> real))" 1
    local.get 79
    struct.get $"(real -> (real -> real))" 0
    call_indirect (type $"(raw) (real -> (real -> real))")
    local.set 80
    f64.const 0x1p+1 (;=2;)
    struct.new $real
    local.get 80
    struct.get $"(real -> real)" 1
    local.get 80
    struct.get $"(real -> real)" 0
    call_indirect (type $"(raw) (real -> real)")
    global.get 15
    ref.as_non_null
    local.tee 81
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 81
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 82
    f64.const 0x1p+1 (;=2;)
    struct.new $real
    local.get 82
    struct.get $"('0 -> boolean)" 1
    local.get 82
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 78
    struct.get $"(boolean -> unit)" 1
    local.get 78
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 77
    local.get 77
    global.set 42
    global.get 26
    ref.as_non_null
    local.set 84
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    global.get 11
    ref.as_non_null
    local.tee 85
    struct.get $"(real -> (real -> real))" 1
    local.get 85
    struct.get $"(real -> (real -> real))" 0
    call_indirect (type $"(raw) (real -> (real -> real))")
    local.set 86
    f64.const 0x1p+1 (;=2;)
    struct.new $real
    local.get 86
    struct.get $"(real -> real)" 1
    local.get 86
    struct.get $"(real -> real)" 0
    call_indirect (type $"(raw) (real -> real)")
    global.get 15
    ref.as_non_null
    local.tee 87
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 87
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 88
    f64.const 0x1p-1 (;=0.5;)
    struct.new $real
    local.get 88
    struct.get $"('0 -> boolean)" 1
    local.get 88
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 84
    struct.get $"(boolean -> unit)" 1
    local.get 84
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 83
    local.get 83
    global.set 43
    global.get 26
    ref.as_non_null
    local.set 90
    i32.const 1
    struct.new $boolean
    global.get 12
    ref.as_non_null
    local.tee 91
    struct.get $"(boolean -> (boolean -> boolean))" 1
    local.get 91
    struct.get $"(boolean -> (boolean -> boolean))" 0
    call_indirect (type $"(raw) (boolean -> (boolean -> boolean))")
    local.set 92
    i32.const 1
    struct.new $boolean
    local.get 92
    struct.get $"(boolean -> boolean)" 1
    local.get 92
    struct.get $"(boolean -> boolean)" 0
    call_indirect (type $"(raw) (boolean -> boolean)")
    local.get 90
    struct.get $"(boolean -> unit)" 1
    local.get 90
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 89
    local.get 89
    global.set 44
    global.get 26
    ref.as_non_null
    local.set 94
    i32.const 1
    struct.new $boolean
    global.get 13
    ref.as_non_null
    local.tee 95
    struct.get $"(boolean -> (boolean -> boolean))" 1
    local.get 95
    struct.get $"(boolean -> (boolean -> boolean))" 0
    call_indirect (type $"(raw) (boolean -> (boolean -> boolean))")
    local.set 96
    i32.const 0
    struct.new $boolean
    local.get 96
    struct.get $"(boolean -> boolean)" 1
    local.get 96
    struct.get $"(boolean -> boolean)" 0
    call_indirect (type $"(raw) (boolean -> boolean)")
    local.get 94
    struct.get $"(boolean -> unit)" 1
    local.get 94
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 93
    local.get 93
    global.set 45
    global.get 26
    ref.as_non_null
    local.set 98
    i32.const 1
    struct.new $boolean
    global.get 14
    ref.as_non_null
    local.tee 99
    struct.get $"(boolean -> (boolean -> boolean))" 1
    local.get 99
    struct.get $"(boolean -> (boolean -> boolean))" 0
    call_indirect (type $"(raw) (boolean -> (boolean -> boolean))")
    local.set 100
    i32.const 0
    struct.new $boolean
    local.get 100
    struct.get $"(boolean -> boolean)" 1
    local.get 100
    struct.get $"(boolean -> boolean)" 0
    call_indirect (type $"(raw) (boolean -> boolean)")
    local.get 98
    struct.get $"(boolean -> unit)" 1
    local.get 98
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 97
    local.get 97
    global.set 46
    global.get 26
    ref.as_non_null
    local.set 102
    struct.new $unit
    global.get 15
    ref.as_non_null
    local.tee 103
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 103
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 104
    struct.new $unit
    local.get 104
    struct.get $"('0 -> boolean)" 1
    local.get 104
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    global.get 15
    ref.as_non_null
    local.tee 105
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 105
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 106
    i32.const 1
    struct.new $boolean
    local.get 106
    struct.get $"('0 -> boolean)" 1
    local.get 106
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 102
    struct.get $"(boolean -> unit)" 1
    local.get 102
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 101
    local.get 101
    global.set 47
    global.get 26
    ref.as_non_null
    local.set 108
    struct.new $unit
    global.get 16
    ref.as_non_null
    local.tee 109
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 109
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 110
    struct.new $unit
    local.get 110
    struct.get $"('0 -> boolean)" 1
    local.get 110
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    global.get 15
    ref.as_non_null
    local.tee 111
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 111
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 112
    i32.const 0
    struct.new $boolean
    local.get 112
    struct.get $"('0 -> boolean)" 1
    local.get 112
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 108
    struct.get $"(boolean -> unit)" 1
    local.get 108
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 107
    local.get 107
    global.set 48
    global.get 26
    ref.as_non_null
    local.set 114
    struct.new $unit
    global.get 17
    ref.as_non_null
    local.tee 115
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 115
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 116
    struct.new $unit
    local.get 116
    struct.get $"('0 -> boolean)" 1
    local.get 116
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    global.get 15
    ref.as_non_null
    local.tee 117
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 117
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 118
    i32.const 1
    struct.new $boolean
    local.get 118
    struct.get $"('0 -> boolean)" 1
    local.get 118
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 114
    struct.get $"(boolean -> unit)" 1
    local.get 114
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 113
    local.get 113
    global.set 49
    global.get 26
    ref.as_non_null
    local.set 120
    struct.new $unit
    global.get 18
    ref.as_non_null
    local.tee 121
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 121
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 122
    struct.new $unit
    local.get 122
    struct.get $"('0 -> boolean)" 1
    local.get 122
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    global.get 15
    ref.as_non_null
    local.tee 123
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 123
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 124
    i32.const 1
    struct.new $boolean
    local.get 124
    struct.get $"('0 -> boolean)" 1
    local.get 124
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 120
    struct.get $"(boolean -> unit)" 1
    local.get 120
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 119
    local.get 119
    global.set 50
    global.get 26
    ref.as_non_null
    local.set 126
    struct.new $unit
    global.get 19
    ref.as_non_null
    local.tee 127
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 127
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 128
    struct.new $unit
    local.get 128
    struct.get $"('0 -> boolean)" 1
    local.get 128
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    global.get 15
    ref.as_non_null
    local.tee 129
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 129
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 130
    i32.const 0
    struct.new $boolean
    local.get 130
    struct.get $"('0 -> boolean)" 1
    local.get 130
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 126
    struct.get $"(boolean -> unit)" 1
    local.get 126
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 125
    local.get 125
    global.set 51
    global.get 26
    ref.as_non_null
    local.set 132
    struct.new $unit
    global.get 20
    ref.as_non_null
    local.tee 133
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 133
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 134
    struct.new $unit
    local.get 134
    struct.get $"('0 -> boolean)" 1
    local.get 134
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    global.get 15
    ref.as_non_null
    local.tee 135
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 135
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 136
    i32.const 0
    struct.new $boolean
    local.get 136
    struct.get $"('0 -> boolean)" 1
    local.get 136
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 132
    struct.get $"(boolean -> unit)" 1
    local.get 132
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 131
    local.get 131
    global.set 52
    global.get 26
    ref.as_non_null
    local.set 138
    i32.const 1
    struct.new $boolean
    global.get 15
    ref.as_non_null
    local.tee 139
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 139
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 140
    i32.const 1
    struct.new $boolean
    local.get 140
    struct.get $"('0 -> boolean)" 1
    local.get 140
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 138
    struct.get $"(boolean -> unit)" 1
    local.get 138
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 137
    local.get 137
    global.set 53
    global.get 26
    ref.as_non_null
    local.set 142
    i32.const 1
    struct.new $boolean
    global.get 16
    ref.as_non_null
    local.tee 143
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 143
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 144
    i32.const 0
    struct.new $boolean
    local.get 144
    struct.get $"('0 -> boolean)" 1
    local.get 144
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 142
    struct.get $"(boolean -> unit)" 1
    local.get 142
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 141
    local.get 141
    global.set 54
    global.get 26
    ref.as_non_null
    local.set 146
    i32.const 0
    struct.new $boolean
    global.get 17
    ref.as_non_null
    local.tee 147
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 147
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 148
    i32.const 1
    struct.new $boolean
    local.get 148
    struct.get $"('0 -> boolean)" 1
    local.get 148
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 146
    struct.get $"(boolean -> unit)" 1
    local.get 146
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 145
    local.get 145
    global.set 55
    global.get 26
    ref.as_non_null
    local.set 150
    i32.const 1
    struct.new $boolean
    global.get 18
    ref.as_non_null
    local.tee 151
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 151
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 152
    i32.const 0
    struct.new $boolean
    local.get 152
    struct.get $"('0 -> boolean)" 1
    local.get 152
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 150
    struct.get $"(boolean -> unit)" 1
    local.get 150
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 149
    local.get 149
    global.set 56
    global.get 26
    ref.as_non_null
    local.set 154
    i32.const 0
    struct.new $boolean
    global.get 19
    ref.as_non_null
    local.tee 155
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 155
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 156
    i32.const 1
    struct.new $boolean
    local.get 156
    struct.get $"('0 -> boolean)" 1
    local.get 156
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 154
    struct.get $"(boolean -> unit)" 1
    local.get 154
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 153
    local.get 153
    global.set 57
    global.get 26
    ref.as_non_null
    local.set 158
    i32.const 1
    struct.new $boolean
    global.get 20
    ref.as_non_null
    local.tee 159
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 159
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 160
    i32.const 0
    struct.new $boolean
    local.get 160
    struct.get $"('0 -> boolean)" 1
    local.get 160
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 158
    struct.get $"(boolean -> unit)" 1
    local.get 158
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 157
    local.get 157
    global.set 58
    global.get 26
    ref.as_non_null
    local.set 162
    i32.const 97
    struct.new $glyph
    global.get 15
    ref.as_non_null
    local.tee 163
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 163
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 164
    i32.const 97
    struct.new $glyph
    local.get 164
    struct.get $"('0 -> boolean)" 1
    local.get 164
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 162
    struct.get $"(boolean -> unit)" 1
    local.get 162
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 161
    local.get 161
    global.set 59
    global.get 26
    ref.as_non_null
    local.set 166
    i32.const 97
    struct.new $glyph
    global.get 16
    ref.as_non_null
    local.tee 167
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 167
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 168
    i32.const 98
    struct.new $glyph
    local.get 168
    struct.get $"('0 -> boolean)" 1
    local.get 168
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 166
    struct.get $"(boolean -> unit)" 1
    local.get 166
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 165
    local.get 165
    global.set 60
    global.get 26
    ref.as_non_null
    local.set 170
    i32.const 97
    struct.new $glyph
    global.get 17
    ref.as_non_null
    local.tee 171
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 171
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 172
    i32.const 98
    struct.new $glyph
    local.get 172
    struct.get $"('0 -> boolean)" 1
    local.get 172
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 170
    struct.get $"(boolean -> unit)" 1
    local.get 170
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 169
    local.get 169
    global.set 61
    global.get 26
    ref.as_non_null
    local.set 174
    i32.const 98
    struct.new $glyph
    global.get 18
    ref.as_non_null
    local.tee 175
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 175
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 176
    i32.const 97
    struct.new $glyph
    local.get 176
    struct.get $"('0 -> boolean)" 1
    local.get 176
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 174
    struct.get $"(boolean -> unit)" 1
    local.get 174
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 173
    local.get 173
    global.set 62
    global.get 26
    ref.as_non_null
    local.set 178
    i32.const 97
    struct.new $glyph
    global.get 19
    ref.as_non_null
    local.tee 179
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 179
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 180
    i32.const 98
    struct.new $glyph
    local.get 180
    struct.get $"('0 -> boolean)" 1
    local.get 180
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 178
    struct.get $"(boolean -> unit)" 1
    local.get 178
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 177
    local.get 177
    global.set 63
    global.get 26
    ref.as_non_null
    local.set 182
    i32.const 98
    struct.new $glyph
    global.get 20
    ref.as_non_null
    local.tee 183
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 183
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 184
    i32.const 97
    struct.new $glyph
    local.get 184
    struct.get $"('0 -> boolean)" 1
    local.get 184
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 182
    struct.get $"(boolean -> unit)" 1
    local.get 182
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 181
    local.get 181
    global.set 64
    global.get 26
    ref.as_non_null
    local.set 186
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    global.get 15
    ref.as_non_null
    local.tee 187
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 187
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 188
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    local.get 188
    struct.get $"('0 -> boolean)" 1
    local.get 188
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 186
    struct.get $"(boolean -> unit)" 1
    local.get 186
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 185
    local.get 185
    global.set 65
    global.get 26
    ref.as_non_null
    local.set 190
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    global.get 16
    ref.as_non_null
    local.tee 191
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 191
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 192
    i32.const 100
    i32.const 101
    i32.const 102
    array.new_fixed $string 3
    local.get 192
    struct.get $"('0 -> boolean)" 1
    local.get 192
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 190
    struct.get $"(boolean -> unit)" 1
    local.get 190
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 189
    local.get 189
    global.set 66
    global.get 26
    ref.as_non_null
    local.set 194
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    global.get 17
    ref.as_non_null
    local.tee 195
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 195
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 196
    i32.const 100
    i32.const 101
    i32.const 102
    array.new_fixed $string 3
    local.get 196
    struct.get $"('0 -> boolean)" 1
    local.get 196
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 194
    struct.get $"(boolean -> unit)" 1
    local.get 194
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 193
    local.get 193
    global.set 67
    global.get 26
    ref.as_non_null
    local.set 198
    i32.const 100
    i32.const 101
    i32.const 102
    array.new_fixed $string 3
    global.get 18
    ref.as_non_null
    local.tee 199
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 199
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 200
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    local.get 200
    struct.get $"('0 -> boolean)" 1
    local.get 200
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 198
    struct.get $"(boolean -> unit)" 1
    local.get 198
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 197
    local.get 197
    global.set 68
    global.get 26
    ref.as_non_null
    local.set 202
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    global.get 17
    ref.as_non_null
    local.tee 203
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 203
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 204
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    local.get 204
    struct.get $"('0 -> boolean)" 1
    local.get 204
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 202
    struct.get $"(boolean -> unit)" 1
    local.get 202
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 201
    local.get 201
    global.set 69
    global.get 26
    ref.as_non_null
    local.set 206
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    global.get 18
    ref.as_non_null
    local.tee 207
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 207
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 208
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    local.get 208
    struct.get $"('0 -> boolean)" 1
    local.get 208
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 206
    struct.get $"(boolean -> unit)" 1
    local.get 206
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 205
    local.get 205
    global.set 70
    global.get 26
    ref.as_non_null
    local.set 210
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    global.get 19
    ref.as_non_null
    local.tee 211
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 211
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 212
    i32.const 100
    i32.const 101
    i32.const 102
    array.new_fixed $string 3
    local.get 212
    struct.get $"('0 -> boolean)" 1
    local.get 212
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 210
    struct.get $"(boolean -> unit)" 1
    local.get 210
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 209
    local.get 209
    global.set 71
    global.get 26
    ref.as_non_null
    local.set 214
    i32.const 100
    i32.const 101
    i32.const 102
    array.new_fixed $string 3
    global.get 20
    ref.as_non_null
    local.tee 215
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 215
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 216
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    local.get 216
    struct.get $"('0 -> boolean)" 1
    local.get 216
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 214
    struct.get $"(boolean -> unit)" 1
    local.get 214
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 213
    local.get 213
    global.set 72
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
  (func (;43;) (type $"(raw) (string -> unit)") (param $0 (ref $string)) (param (ref $capture)) (result (ref $unit))
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
    i32.const 43
    call_indirect (type 30)
    struct.new $unit
  )
  (func (;44;) (type $"(raw) (string -> (string -> string))") (param $1 (ref $string)) (param (ref $capture)) (result (ref $"(string -> string)"))
    i32.const 45
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(string -> (string -> string))"
  )
  (func (;45;) (type $"(raw) (string -> string)") (param $unit (ref $string)) (param (ref $capture)) (result (ref $string))
    (local (ref $string) i32 i32 (ref $string))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $string)
    local.set 2
    local.get 2
    array.len
    local.tee 3
    local.get $unit
    array.len
    local.tee 4
    i32.add
    array.new_default $string
    local.tee 5
    i32.const 0
    local.get 2
    i32.const 0
    local.get 3
    array.copy $string $string
    local.get 5
    local.get 3
    local.get $unit
    i32.const 0
    local.get 4
    array.copy $string $string
    local.get 5
  )
  (func (;46;) (type $"(raw) (unit -> '1)") (param $condition#0 (ref $unit)) (param (ref $capture)) (result anyref)
    (local (ref $"(unit -> '2)"))
    global.get 21
    ref.as_non_null
    local.set 2
    struct.new $unit
    local.get 2
    struct.get $"(unit -> '2)" 1
    local.get 2
    struct.get $"(unit -> '2)" 0
    call_indirect (type $"(raw) (unit -> '2)")
    ref.cast (ref any)
  )
  (func (;47;) (type $"(raw) (boolean -> unit)") (param $s#1 (ref $boolean)) (param (ref $capture)) (result (ref $unit))
    (local (ref $"(unit -> '1)"))
    local.get $s#1
    ref.cast (ref $boolean)
    struct.get $boolean 0
    if (result (ref $unit)) ;; label = @1
      struct.new $unit
    else
      global.get 25
      ref.as_non_null
      ref.cast (ref $"(unit -> '1)")
      local.set 2
      struct.new $unit
      local.get 2
      struct.get $"(unit -> '1)" 1
      local.get 2
      struct.get $"(unit -> '1)" 0
      call_indirect (type $"(raw) (unit -> '1)")
      ref.cast (ref $unit)
    end
  )
  (func (;48;) (type $"(raw) (string -> integer)") (param $s#2 (ref $string)) (param (ref $capture)) (result (ref $integer))
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
  (func (;49;) (type $"(raw) (string -> unit)") (param $s1#3 (ref $string)) (param (ref $capture)) (result (ref $unit))
    (local (ref $"(string -> unit)"))
    global.get 23
    ref.as_non_null
    local.set 2
    local.get $s1#3
    ref.cast (ref $string)
    local.get 2
    struct.get $"(string -> unit)" 1
    local.get 2
    struct.get $"(string -> unit)" 0
    call_indirect (type $"(raw) (string -> unit)")
    ref.cast (ref $unit)
  )
  (func (;50;) (type $"(raw) (string -> (string -> string))") (param $s2#4 (ref $string)) (param (ref $capture)) (result (ref $"(string -> string)"))
    i32.const 51
    local.get $s2#4
    ref.cast (ref any)
    array.new_fixed $capture 1
    struct.new $"(string -> string)"
  )
  (func (;51;) (type $"(raw) (string -> string)") (param (ref $string) (ref $capture)) (result (ref $string))
    (local (ref $string) (ref $"(string -> string)") (ref $"(string -> (string -> string))"))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $string)
    local.set 2
    global.get 24
    ref.as_non_null
    local.set 4
    local.get 2
    ref.cast (ref $string)
    local.get 4
    struct.get $"(string -> (string -> string))" 1
    local.get 4
    struct.get $"(string -> (string -> string))" 0
    call_indirect (type $"(raw) (string -> (string -> string))")
    ref.cast (ref $"(string -> string)")
    local.set 3
    local.get 0
    ref.cast (ref $string)
    local.get 3
    struct.get $"(string -> string)" 1
    local.get 3
    struct.get $"(string -> string)" 0
    call_indirect (type $"(raw) (string -> string)")
    ref.cast (ref $string)
  )
)
