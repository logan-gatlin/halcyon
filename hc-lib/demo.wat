(module
  (type (;0;) (func))
  (type $integer (;1;) (struct (field i64)))
  (type $capture (;2;) (array (mut anyref)))
  (type $"(integer -> integer)" (;3;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (integer -> integer)" (;4;) (func (param anyref (ref $capture)) (result anyref)))
  (type $real (;5;) (struct (field f64)))
  (type $"(real -> real)" (;6;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (real -> real)" (;7;) (func (param anyref (ref $capture)) (result anyref)))
  (type $boolean (;8;) (struct (field i32)))
  (type $"(boolean -> boolean)" (;9;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (boolean -> boolean)" (;10;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(integer -> (integer -> integer))" (;11;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (integer -> (integer -> integer))" (;12;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(real -> (real -> real))" (;13;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (real -> (real -> real))" (;14;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(boolean -> (boolean -> boolean))" (;15;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (boolean -> (boolean -> boolean))" (;16;) (func (param anyref (ref $capture)) (result anyref)))
  (type $glyph (;17;) (struct (field i32)))
  (type $unit (;18;) (struct))
  (type $string (;19;) (array (mut i8)))
  (type $"('0 -> ('0 -> boolean))" (;20;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('0 -> ('0 -> boolean))" (;21;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"('0 -> boolean)" (;22;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('0 -> boolean)" (;23;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(unit -> '0)" (;24;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> '0)" (;25;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(string -> integer)" (;26;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (string -> integer)" (;27;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(string -> unit)" (;28;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (string -> unit)" (;29;) (func (param anyref (ref $capture)) (result anyref)))
  (type (;30;) (func (param i32 i32)))
  (type $"(string -> (string -> string))" (;31;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (string -> (string -> string))" (;32;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(string -> string)" (;33;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (string -> string)" (;34;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(unit -> '1)" (;35;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> '1)" (;36;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(unit -> '2)" (;37;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> '2)" (;38;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(boolean -> unit)" (;39;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (boolean -> unit)" (;40;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(unit -> '4)" (;41;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> '4)" (;42;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"('0 -> Some of '0 | None of unit)" (;43;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('0 -> Some of '0 | None of unit)" (;44;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Some of '0 | None of unit" (;45;) (struct (field i32) (field anyref)))
  (type $"(unit -> Some of '0 | None of unit)" (;46;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> Some of '0 | None of unit)" (;47;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(('3 -> '6) -> (Some of '3 | None of unit -> Some of '6 | None of unit))" (;48;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (('3 -> '6) -> (Some of '3 | None of unit -> Some of '6 | None of unit))" (;49;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"('3 -> '6)" (;50;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('3 -> '6)" (;51;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Some of '3 | None of unit" (;52;) (struct (field i32) (field anyref)))
  (type $"(Some of '3 | None of unit -> Some of '6 | None of unit)" (;53;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (Some of '3 | None of unit -> Some of '6 | None of unit)" (;54;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Some of '6 | None of unit" (;55;) (struct (field i32) (field anyref)))
  (type $"('6 -> Some of '6 | None of unit)" (;56;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('6 -> Some of '6 | None of unit)" (;57;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(unit -> Some of '8 | None of unit)" (;58;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> Some of '8 | None of unit)" (;59;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Some of '8 | None of unit" (;60;) (struct (field i32) (field anyref)))
  (type $"(('6 -> '7) -> (Some of '9 | None of unit -> unit))" (;61;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (('6 -> '7) -> (Some of '9 | None of unit -> unit))" (;62;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"('6 -> '7)" (;63;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('6 -> '7)" (;64;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Some of '9 | None of unit" (;65;) (struct (field i32) (field anyref)))
  (type $"(Some of '9 | None of unit -> unit)" (;66;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (Some of '9 | None of unit -> unit)" (;67;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(Some of '9 | None of unit -> Some of '10 | None of unit)" (;68;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (Some of '9 | None of unit -> Some of '10 | None of unit)" (;69;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(('9 -> '10) -> (Some of '9 | None of unit -> Some of '10 | None of unit))" (;70;) (struct (field i32) (field (ref $capture))))
  (type $"('6 -> '6)" (;71;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('6 -> '6)" (;72;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Some of '10 | None of unit" (;73;) (struct (field i32) (field anyref)))
  (type $"(Some of '5 | None of unit -> '5)" (;74;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (Some of '5 | None of unit -> '5)" (;75;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Some of '5 | None of unit" (;76;) (struct (field i32) (field anyref)))
  (type $"(unit -> '7)" (;77;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> '7)" (;78;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(Some of '3 | None of unit -> boolean)" (;79;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (Some of '3 | None of unit -> boolean)" (;80;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(Some of '4 | None of unit -> boolean)" (;81;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (Some of '4 | None of unit -> boolean)" (;82;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Some of '4 | None of unit" (;83;) (struct (field i32) (field anyref)))
  (type $"(('0 * list:t) -> Pair of ('0 * list:t) | Nil of unit)" (;84;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (('0 * list:t) -> Pair of ('0 * list:t) | Nil of unit)" (;85;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Pair of ('0 * list:t) | Nil of unit" (;86;) (struct (field i32) (field anyref)))
  (type $"('0 * list:t)" (;87;) (struct (field anyref) (field (ref $"Pair of ('0 * list:t) | Nil of unit"))))
  (type $"(unit -> Pair of ('0 * list:t) | Nil of unit)" (;88;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> Pair of ('0 * list:t) | Nil of unit)" (;89;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(('3 -> '9) -> (list:t -> list:t))" (;90;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (('3 -> '9) -> (list:t -> list:t))" (;91;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"('3 -> '9)" (;92;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('3 -> '9)" (;93;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(('3 -> '9) -> (Pair of ('3 * list:t) | Nil of unit -> Pair of ('9 * list:t) | Nil of unit))" (;94;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (('3 -> '9) -> (Pair of ('3 * list:t) | Nil of unit -> Pair of ('9 * list:t) | Nil of unit))" (;95;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Pair of ('3 * list:t) | Nil of unit" (;96;) (struct (field i32) (field anyref)))
  (type $"(Pair of ('3 * list:t) | Nil of unit -> Pair of ('9 * list:t) | Nil of unit)" (;97;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (Pair of ('3 * list:t) | Nil of unit -> Pair of ('9 * list:t) | Nil of unit)" (;98;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Pair of ('9 * list:t) | Nil of unit" (;99;) (struct (field i32) (field anyref)))
  (type $"('3 * list:t)" (;100;) (struct (field anyref) (field (ref $"Pair of ('0 * list:t) | Nil of unit"))))
  (type $"(('9 * list:t) -> Pair of ('9 * list:t) | Nil of unit)" (;101;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (('9 * list:t) -> Pair of ('9 * list:t) | Nil of unit)" (;102;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(list:t -> list:t)" (;103;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (list:t -> list:t)" (;104;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"('9 * list:t)" (;105;) (struct (field anyref) (field (ref $"Pair of ('0 * list:t) | Nil of unit"))))
  (type $"(unit -> Pair of ('11 * list:t) | Nil of unit)" (;106;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> Pair of ('11 * list:t) | Nil of unit)" (;107;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Pair of ('11 * list:t) | Nil of unit" (;108;) (struct (field i32) (field anyref)))
  (type $"(('6 -> '7) -> (list:t -> unit))" (;109;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (('6 -> '7) -> (list:t -> unit))" (;110;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(list:t -> unit)" (;111;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (list:t -> unit)" (;112;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(('9 -> '10) -> (list:t -> list:t))" (;113;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (('9 -> '10) -> (list:t -> list:t))" (;114;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"('3 -> (list:t -> list:t))" (;115;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('3 -> (list:t -> list:t))" (;116;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"('3 -> (Pair of ('10 * list:t) | Nil of unit -> Pair of ('10 * list:t) | Nil of unit))" (;117;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('3 -> (Pair of ('10 * list:t) | Nil of unit -> Pair of ('10 * list:t) | Nil of unit))" (;118;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Pair of ('10 * list:t) | Nil of unit" (;119;) (struct (field i32) (field anyref)))
  (type $"(Pair of ('10 * list:t) | Nil of unit -> Pair of ('10 * list:t) | Nil of unit)" (;120;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (Pair of ('10 * list:t) | Nil of unit -> Pair of ('10 * list:t) | Nil of unit)" (;121;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"('10 * list:t)" (;122;) (struct (field anyref) (field (ref $"Pair of ('0 * list:t) | Nil of unit"))))
  (type $"(('10 * list:t) -> Pair of ('10 * list:t) | Nil of unit)" (;123;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (('10 * list:t) -> Pair of ('10 * list:t) | Nil of unit)" (;124;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(('14 * list:t) -> Pair of ('14 * list:t) | Nil of unit)" (;125;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (('14 * list:t) -> Pair of ('14 * list:t) | Nil of unit)" (;126;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(unit -> Pair of ('12 * list:t) | Nil of unit)" (;127;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> Pair of ('12 * list:t) | Nil of unit)" (;128;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Pair of ('12 * list:t) | Nil of unit" (;129;) (struct (field i32) (field anyref)))
  (type $"('3 * Pair of ('12 * list:t) | Nil of unit)" (;130;) (struct (field anyref) (field (ref $"Pair of ('12 * list:t) | Nil of unit"))))
  (type $"Pair of ('14 * list:t) | Nil of unit" (;131;) (struct (field i32) (field anyref)))
  (type $"(list:t -> integer)" (;132;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (list:t -> integer)" (;133;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(Pair of ('3 * list:t) | Nil of unit -> integer)" (;134;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (Pair of ('3 * list:t) | Nil of unit -> integer)" (;135;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(('9 -> ('6 -> '9)) -> ('4 -> (list:t -> '6)))" (;136;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (('9 -> ('6 -> '9)) -> ('4 -> (list:t -> '6)))" (;137;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"('9 -> ('6 -> '9))" (;138;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('9 -> ('6 -> '9))" (;139;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(('9 -> ('6 -> '9)) -> ('9 -> (Pair of ('4 * list:t) | Nil of unit -> '9)))" (;140;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (('9 -> ('6 -> '9)) -> ('9 -> (Pair of ('4 * list:t) | Nil of unit -> '9)))" (;141;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"('9 -> (Pair of ('4 * list:t) | Nil of unit -> '9))" (;142;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('9 -> (Pair of ('4 * list:t) | Nil of unit -> '9))" (;143;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Pair of ('4 * list:t) | Nil of unit" (;144;) (struct (field i32) (field anyref)))
  (type $"(Pair of ('4 * list:t) | Nil of unit -> '9)" (;145;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (Pair of ('4 * list:t) | Nil of unit -> '9)" (;146;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"('4 * list:t)" (;147;) (struct (field anyref) (field (ref $"Pair of ('0 * list:t) | Nil of unit"))))
  (type $"('6 -> '9)" (;148;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('6 -> '9)" (;149;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(list:t -> '6)" (;150;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (list:t -> '6)" (;151;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"('4 -> (list:t -> '6))" (;152;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('4 -> (list:t -> '6))" (;153;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(list:t -> (Pair of ('11 * list:t) | Nil of unit -> list:t))" (;154;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (list:t -> (Pair of ('11 * list:t) | Nil of unit -> list:t))" (;155;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(Pair of ('11 * list:t) | Nil of unit -> (Pair of ('11 * list:t) | Nil of unit -> Pair of ('11 * list:t) | Nil of unit))" (;156;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (Pair of ('11 * list:t) | Nil of unit -> (Pair of ('11 * list:t) | Nil of unit -> Pair of ('11 * list:t) | Nil of unit))" (;157;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(Pair of ('11 * list:t) | Nil of unit -> Pair of ('11 * list:t) | Nil of unit)" (;158;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (Pair of ('11 * list:t) | Nil of unit -> Pair of ('11 * list:t) | Nil of unit)" (;159;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"('11 * list:t)" (;160;) (struct (field anyref) (field (ref $"Pair of ('0 * list:t) | Nil of unit"))))
  (type $"(('11 * list:t) -> Pair of ('11 * list:t) | Nil of unit)" (;161;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (('11 * list:t) -> Pair of ('11 * list:t) | Nil of unit)" (;162;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(Pair of ('11 * list:t) | Nil of unit -> list:t)" (;163;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (Pair of ('11 * list:t) | Nil of unit -> list:t)" (;164;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(integer -> (list:t -> Some of '8 | None of unit))" (;165;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (integer -> (list:t -> Some of '8 | None of unit))" (;166;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(integer -> (Pair of ('8 * list:t) | Nil of unit -> Some of '8 | None of unit))" (;167;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (integer -> (Pair of ('8 * list:t) | Nil of unit -> Some of '8 | None of unit))" (;168;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Pair of ('8 * list:t) | Nil of unit" (;169;) (struct (field i32) (field anyref)))
  (type $"(Pair of ('8 * list:t) | Nil of unit -> Some of '8 | None of unit)" (;170;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (Pair of ('8 * list:t) | Nil of unit -> Some of '8 | None of unit)" (;171;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(integer * Pair of ('8 * list:t) | Nil of unit)" (;172;) (struct (field (ref $integer)) (field (ref $"Pair of ('8 * list:t) | Nil of unit"))))
  (type $"('8 * list:t)" (;173;) (struct (field anyref) (field (ref $"Pair of ('0 * list:t) | Nil of unit"))))
  (type $"('8 -> Some of '8 | None of unit)" (;174;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('8 -> Some of '8 | None of unit)" (;175;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Pair of ('5 * list:t) | Nil of unit" (;176;) (struct (field i32) (field anyref)))
  (type $"('5 * list:t)" (;177;) (struct (field anyref) (field (ref $"Pair of ('0 * list:t) | Nil of unit"))))
  (type $"(list:t -> Some of '8 | None of unit)" (;178;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (list:t -> Some of '8 | None of unit)" (;179;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Pair of ('6 * list:t) | Nil of unit" (;180;) (struct (field i32) (field anyref)))
  (type $"(unit -> Some of '12 | None of unit)" (;181;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> Some of '12 | None of unit)" (;182;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"Some of '12 | None of unit" (;183;) (struct (field i32) (field anyref)))
  (type $"(integer -> (integer -> unit))" (;184;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (integer -> (integer -> unit))" (;185;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(integer -> unit)" (;186;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (integer -> unit)" (;187;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(integer -> string)" (;188;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (integer -> string)" (;189;) (func (param anyref (ref $capture)) (result anyref)))
  (type $"(integer * integer)" (;190;) (struct (field (ref $integer)) (field (ref $integer))))
  (import "sys" "print_string" (func (;0;) (type 30)))
  (import "sys" "memory" (memory (;0;) 1))
  (table (;0;) 83 83 funcref)
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
  (global (;27;) (mut (ref null $"(string -> unit)")) ref.null $"(string -> unit)")
  (global (;28;) (mut (ref null $"(string -> integer)")) ref.null $"(string -> integer)")
  (global (;29;) (mut (ref null $"(string -> (string -> string))")) ref.null $"(string -> (string -> string))")
  (global (;30;) (mut (ref null $"(string -> unit)")) ref.null $"(string -> unit)")
  (global (;31;) (mut (ref null $"('0 -> Some of '0 | None of unit)")) ref.null $"('0 -> Some of '0 | None of unit)")
  (global (;32;) (mut (ref null $"(unit -> Some of '0 | None of unit)")) ref.null $"(unit -> Some of '0 | None of unit)")
  (global (;33;) (mut (ref null $"(('3 -> '6) -> (Some of '3 | None of unit -> Some of '6 | None of unit))")) ref.null $"(('3 -> '6) -> (Some of '3 | None of unit -> Some of '6 | None of unit))")
  (global (;34;) (mut (ref null $"(('6 -> '7) -> (Some of '9 | None of unit -> unit))")) ref.null $"(('6 -> '7) -> (Some of '9 | None of unit -> unit))")
  (global (;35;) (mut (ref null $"(Some of '5 | None of unit -> '5)")) ref.null $"(Some of '5 | None of unit -> '5)")
  (global (;36;) (mut (ref null $"(Some of '3 | None of unit -> boolean)")) ref.null $"(Some of '3 | None of unit -> boolean)")
  (global (;37;) (mut (ref null $"(Some of '4 | None of unit -> boolean)")) ref.null $"(Some of '4 | None of unit -> boolean)")
  (global (;38;) (mut (ref null $"(('0 * list:t) -> Pair of ('0 * list:t) | Nil of unit)")) ref.null $"(('0 * list:t) -> Pair of ('0 * list:t) | Nil of unit)")
  (global (;39;) (mut (ref null $"(unit -> Pair of ('0 * list:t) | Nil of unit)")) ref.null $"(unit -> Pair of ('0 * list:t) | Nil of unit)")
  (global (;40;) (mut (ref null $"(('3 -> '9) -> (list:t -> list:t))")) ref.null $"(('3 -> '9) -> (list:t -> list:t))")
  (global (;41;) (mut (ref null $"(('6 -> '7) -> (list:t -> unit))")) ref.null $"(('6 -> '7) -> (list:t -> unit))")
  (global (;42;) (mut (ref null $"('3 -> (list:t -> list:t))")) ref.null $"('3 -> (list:t -> list:t))")
  (global (;43;) (mut (ref null $"(list:t -> integer)")) ref.null $"(list:t -> integer)")
  (global (;44;) (mut (ref null $"(('9 -> ('6 -> '9)) -> ('4 -> (list:t -> '6)))")) ref.null $"(('9 -> ('6 -> '9)) -> ('4 -> (list:t -> '6)))")
  (global (;45;) (mut (ref null $"(list:t -> (Pair of ('11 * list:t) | Nil of unit -> list:t))")) ref.null $"(list:t -> (Pair of ('11 * list:t) | Nil of unit -> list:t))")
  (global (;46;) (mut (ref null $"(integer -> (list:t -> Some of '8 | None of unit))")) ref.null $"(integer -> (list:t -> Some of '8 | None of unit))")
  (global (;47;) (mut (ref null $"(integer -> (integer -> unit))")) ref.null $"(integer -> (integer -> unit))")
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
  (export "std:print_string" (global 27))
  (export "string:length" (global 28))
  (export "string:concatenate" (global 29))
  (export "string:print" (global 30))
  (export "opt:Some" (global 31))
  (export "opt:None" (global 32))
  (export "opt:map" (global 33))
  (export "opt:iterate" (global 34))
  (export "opt:unwrap" (global 35))
  (export "opt:is_some" (global 36))
  (export "opt:is_none" (global 37))
  (export "list:Pair" (global 38))
  (export "list:Nil" (global 39))
  (export "list:map" (global 40))
  (export "list:iterate" (global 41))
  (export "list:push" (global 42))
  (export "list:length" (global 43))
  (export "list:fold" (global 44))
  (export "list:concatenate" (global 45))
  (export "list:nth" (global 46))
  (export "FizzBuzz:fizzbuzz" (global 47))
  (start 1)
  (elem (;0;) (i32.const 0) func 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 0 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63 64 65 66 67 68 69 70 71 72 73 74 75 76 77 78 79 80 81 82)
  (func (;1;) (type 0)
    (local (ref $"(unit -> '1)") (ref $"(boolean -> unit)")) (local $0 (ref $"(string -> unit)")) (local (ref $"(string -> integer)") (ref $"(string -> (string -> string))") (ref $"(string -> unit)") (ref $"(('3 -> '6) -> (Some of '3 | None of unit -> Some of '6 | None of unit))") (ref $"(('6 -> '7) -> (Some of '9 | None of unit -> unit))") (ref $"(Some of '5 | None of unit -> '5)") (ref $"(Some of '3 | None of unit -> boolean)") (ref $"(Some of '4 | None of unit -> boolean)") (ref $"(('3 -> '9) -> (list:t -> list:t))") (ref $"(('6 -> '7) -> (list:t -> unit))") (ref $"('3 -> (list:t -> list:t))") (ref $"(list:t -> integer)") (ref $"(('9 -> ('6 -> '9)) -> ('4 -> (list:t -> '6)))") (ref $"(list:t -> (Pair of ('11 * list:t) | Nil of unit -> list:t))") (ref $"(integer -> (list:t -> Some of '8 | None of unit))") (ref $"(integer -> (integer -> unit))") (ref $unit) (ref $"(integer -> unit)") (ref $"(integer -> (integer -> unit))") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)"))
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
    local.set 0
    local.get 0
    global.set 25
    i32.const 47
    array.new_fixed $capture 0
    struct.new $"(boolean -> unit)"
    local.set 1
    local.get 1
    global.set 26
    i32.const 48
    array.new_fixed $capture 0
    struct.new $"(string -> unit)"
    local.set $0
    local.get $0
    global.set 27
    i32.const 49
    array.new_fixed $capture 0
    struct.new $"(string -> integer)"
    local.set 3
    local.get 3
    global.set 28
    i32.const 50
    array.new_fixed $capture 0
    struct.new $"(string -> (string -> string))"
    local.set 4
    local.get 4
    global.set 29
    i32.const 52
    array.new_fixed $capture 0
    struct.new $"(string -> unit)"
    local.set 5
    local.get 5
    global.set 30
    i32.const 53
    array.new_fixed $capture 0
    struct.new $"('0 -> Some of '0 | None of unit)"
    global.set 31
    i32.const 54
    array.new_fixed $capture 0
    struct.new $"(unit -> Some of '0 | None of unit)"
    global.set 32
    i32.const 55
    array.new_fixed $capture 0
    struct.new $"(('3 -> '6) -> (Some of '3 | None of unit -> Some of '6 | None of unit))"
    local.set 6
    local.get 6
    global.set 33
    i32.const 57
    array.new_fixed $capture 0
    struct.new $"(('6 -> '7) -> (Some of '9 | None of unit -> unit))"
    local.set 7
    local.get 7
    global.set 34
    i32.const 60
    array.new_fixed $capture 0
    struct.new $"(Some of '5 | None of unit -> '5)"
    local.set 8
    local.get 8
    global.set 35
    i32.const 61
    array.new_fixed $capture 0
    struct.new $"(Some of '3 | None of unit -> boolean)"
    local.set 9
    local.get 9
    global.set 36
    i32.const 62
    array.new_fixed $capture 0
    struct.new $"(Some of '4 | None of unit -> boolean)"
    local.set 10
    local.get 10
    global.set 37
    i32.const 63
    array.new_fixed $capture 0
    struct.new $"(('0 * list:t) -> Pair of ('0 * list:t) | Nil of unit)"
    global.set 38
    i32.const 64
    array.new_fixed $capture 0
    struct.new $"(unit -> Pair of ('0 * list:t) | Nil of unit)"
    global.set 39
    i32.const 65
    array.new_fixed $capture 0
    struct.new $"(('3 -> '9) -> (Pair of ('3 * list:t) | Nil of unit -> Pair of ('9 * list:t) | Nil of unit))"
    local.set 11
    local.get 11
    global.set 40
    i32.const 67
    array.new_fixed $capture 0
    struct.new $"(('6 -> '7) -> (list:t -> unit))"
    local.set 12
    local.get 12
    global.set 41
    i32.const 70
    array.new_fixed $capture 0
    struct.new $"('3 -> (Pair of ('10 * list:t) | Nil of unit -> Pair of ('10 * list:t) | Nil of unit))"
    local.set 13
    local.get 13
    global.set 42
    i32.const 72
    array.new_fixed $capture 0
    struct.new $"(Pair of ('3 * list:t) | Nil of unit -> integer)"
    local.set 14
    local.get 14
    global.set 43
    i32.const 73
    array.new_fixed $capture 0
    struct.new $"(('9 -> ('6 -> '9)) -> ('9 -> (Pair of ('4 * list:t) | Nil of unit -> '9)))"
    local.set 15
    local.get 15
    global.set 44
    i32.const 76
    array.new_fixed $capture 0
    struct.new $"(Pair of ('11 * list:t) | Nil of unit -> (Pair of ('11 * list:t) | Nil of unit -> Pair of ('11 * list:t) | Nil of unit))"
    local.set 16
    local.get 16
    global.set 45
    i32.const 78
    array.new_fixed $capture 0
    struct.new $"(integer -> (Pair of ('8 * list:t) | Nil of unit -> Some of '8 | None of unit))"
    local.set 17
    local.get 17
    global.set 46
    i32.const 80
    array.new_fixed $capture 0
    struct.new $"(integer -> (integer -> unit))"
    local.set 18
    local.get 18
    global.set 47
    global.get 47
    ref.as_non_null
    local.set 21
    i64.const 1
    struct.new $integer
    local.get 21
    struct.get $"(integer -> (integer -> unit))" 1
    local.get 21
    struct.get $"(integer -> (integer -> unit))" 0
    call_indirect (type $"(raw) (integer -> (integer -> unit))")
    ref.cast (ref $"(integer -> unit)")
    local.set 20
    i64.const 30
    struct.new $integer
    local.get 20
    struct.get $"(integer -> unit)" 1
    local.get 20
    struct.get $"(integer -> unit)" 0
    call_indirect (type $"(raw) (integer -> unit)")
    ref.cast (ref $unit)
    local.set 19
    local.get 19
    global.get 15
    ref.as_non_null
    local.tee 22
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 22
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    ref.cast (ref $"('0 -> boolean)")
    local.set 23
    struct.new $unit
    local.get 23
    struct.get $"('0 -> boolean)" 1
    local.get 23
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    ref.cast (ref $boolean)
    struct.get $boolean 0
    i32.const 1
    i32.xor
    br_if 0
  )
  (func (;2;) (type $"(raw) (integer -> integer)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $integer))
    local.get 0
    ref.cast (ref $integer)
    local.set $0
    i64.const 0
    local.get $0
    struct.get $integer 0
    i64.sub
    struct.new $integer
  )
  (func (;3;) (type $"(raw) (real -> real)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $real))
    local.get 0
    ref.cast (ref $real)
    local.set $0
    f64.const 0x0p+0 (;=0;)
    local.get $0
    struct.get $real 0
    f64.sub
    struct.new $real
  )
  (func (;4;) (type $"(raw) (boolean -> boolean)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $boolean))
    local.get 0
    ref.cast (ref $boolean)
    local.set $0
    local.get $0
    struct.get $boolean 0
    i32.eqz
    struct.new $boolean
  )
  (func (;5;) (type $"(raw) (integer -> (integer -> integer))") (param anyref (ref $capture)) (result anyref)
    (local $1 (ref $integer))
    local.get 0
    ref.cast (ref $integer)
    local.set $1
    i32.const 5
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(integer -> (integer -> integer))"
  )
  (func (;6;) (type $"(raw) (integer -> integer)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $integer)) (local (ref $integer))
    local.get 0
    ref.cast (ref $integer)
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 3
    local.get 3
    struct.get $integer 0
    local.get $0
    struct.get $integer 0
    i64.add
    struct.new $integer
  )
  (func (;7;) (type $"(raw) (integer -> (integer -> integer))") (param anyref (ref $capture)) (result anyref)
    (local $1 (ref $integer))
    local.get 0
    ref.cast (ref $integer)
    local.set $1
    i32.const 7
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(integer -> (integer -> integer))"
  )
  (func (;8;) (type $"(raw) (integer -> integer)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $integer)) (local (ref $integer))
    local.get 0
    ref.cast (ref $integer)
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 3
    local.get 3
    struct.get $integer 0
    local.get $0
    struct.get $integer 0
    i64.sub
    struct.new $integer
  )
  (func (;9;) (type $"(raw) (integer -> (integer -> integer))") (param anyref (ref $capture)) (result anyref)
    (local $1 (ref $integer))
    local.get 0
    ref.cast (ref $integer)
    local.set $1
    i32.const 9
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(integer -> (integer -> integer))"
  )
  (func (;10;) (type $"(raw) (integer -> integer)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $integer)) (local (ref $integer))
    local.get 0
    ref.cast (ref $integer)
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 3
    local.get 3
    struct.get $integer 0
    local.get $0
    struct.get $integer 0
    i64.mul
    struct.new $integer
  )
  (func (;11;) (type $"(raw) (integer -> (integer -> integer))") (param anyref (ref $capture)) (result anyref)
    (local $1 (ref $integer))
    local.get 0
    ref.cast (ref $integer)
    local.set $1
    i32.const 11
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(integer -> (integer -> integer))"
  )
  (func (;12;) (type $"(raw) (integer -> integer)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $integer)) (local (ref $integer))
    local.get 0
    ref.cast (ref $integer)
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 3
    local.get 3
    struct.get $integer 0
    local.get $0
    struct.get $integer 0
    i64.div_s
    struct.new $integer
  )
  (func (;13;) (type $"(raw) (integer -> (integer -> integer))") (param anyref (ref $capture)) (result anyref)
    (local $1 (ref $integer))
    local.get 0
    ref.cast (ref $integer)
    local.set $1
    i32.const 13
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(integer -> (integer -> integer))"
  )
  (func (;14;) (type $"(raw) (integer -> integer)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $integer)) (local (ref $integer))
    local.get 0
    ref.cast (ref $integer)
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 3
    local.get 3
    struct.get $integer 0
    local.get $0
    struct.get $integer 0
    i64.rem_s
    struct.new $integer
  )
  (func (;15;) (type $"(raw) (real -> (real -> real))") (param anyref (ref $capture)) (result anyref)
    (local $1 (ref $real))
    local.get 0
    ref.cast (ref $real)
    local.set $1
    i32.const 15
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(real -> (real -> real))"
  )
  (func (;16;) (type $"(raw) (real -> real)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $real)) (local (ref $real))
    local.get 0
    ref.cast (ref $real)
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set 3
    local.get 3
    struct.get $real 0
    local.get $0
    struct.get $real 0
    f64.add
    struct.new $real
  )
  (func (;17;) (type $"(raw) (real -> (real -> real))") (param anyref (ref $capture)) (result anyref)
    (local $1 (ref $real))
    local.get 0
    ref.cast (ref $real)
    local.set $1
    i32.const 17
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(real -> (real -> real))"
  )
  (func (;18;) (type $"(raw) (real -> real)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $real)) (local (ref $real))
    local.get 0
    ref.cast (ref $real)
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set 3
    local.get 3
    struct.get $real 0
    local.get $0
    struct.get $real 0
    f64.sub
    struct.new $real
  )
  (func (;19;) (type $"(raw) (real -> (real -> real))") (param anyref (ref $capture)) (result anyref)
    (local $1 (ref $real))
    local.get 0
    ref.cast (ref $real)
    local.set $1
    i32.const 19
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(real -> (real -> real))"
  )
  (func (;20;) (type $"(raw) (real -> real)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $real)) (local (ref $real))
    local.get 0
    ref.cast (ref $real)
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set 3
    local.get 3
    struct.get $real 0
    local.get $0
    struct.get $real 0
    f64.mul
    struct.new $real
  )
  (func (;21;) (type $"(raw) (real -> (real -> real))") (param anyref (ref $capture)) (result anyref)
    (local $1 (ref $real))
    local.get 0
    ref.cast (ref $real)
    local.set $1
    i32.const 21
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(real -> (real -> real))"
  )
  (func (;22;) (type $"(raw) (real -> real)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $real)) (local (ref $real))
    local.get 0
    ref.cast (ref $real)
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set 3
    local.get 3
    struct.get $real 0
    local.get $0
    struct.get $real 0
    f64.div
    struct.new $real
  )
  (func (;23;) (type $"(raw) (boolean -> (boolean -> boolean))") (param anyref (ref $capture)) (result anyref)
    (local $1 (ref $boolean))
    local.get 0
    ref.cast (ref $boolean)
    local.set $1
    i32.const 23
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(boolean -> (boolean -> boolean))"
  )
  (func (;24;) (type $"(raw) (boolean -> boolean)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $boolean)) (local (ref $boolean))
    local.get 0
    ref.cast (ref $boolean)
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $boolean)
    local.set 3
    local.get 3
    struct.get $boolean 0
    local.get $0
    struct.get $boolean 0
    i32.and
    struct.new $boolean
  )
  (func (;25;) (type $"(raw) (boolean -> (boolean -> boolean))") (param anyref (ref $capture)) (result anyref)
    (local $1 (ref $boolean))
    local.get 0
    ref.cast (ref $boolean)
    local.set $1
    i32.const 25
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(boolean -> (boolean -> boolean))"
  )
  (func (;26;) (type $"(raw) (boolean -> boolean)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $boolean)) (local (ref $boolean))
    local.get 0
    ref.cast (ref $boolean)
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $boolean)
    local.set 3
    local.get 3
    struct.get $boolean 0
    local.get $0
    struct.get $boolean 0
    i32.or
    struct.new $boolean
  )
  (func (;27;) (type $"(raw) (boolean -> (boolean -> boolean))") (param anyref (ref $capture)) (result anyref)
    (local $1 (ref $boolean))
    local.get 0
    ref.cast (ref $boolean)
    local.set $1
    i32.const 27
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(boolean -> (boolean -> boolean))"
  )
  (func (;28;) (type $"(raw) (boolean -> boolean)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $boolean)) (local (ref $boolean))
    local.get 0
    ref.cast (ref $boolean)
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $boolean)
    local.set 3
    local.get 3
    struct.get $boolean 0
    local.get $0
    struct.get $boolean 0
    i32.xor
    struct.new $boolean
  )
  (func (;29;) (type $"(raw) ('0 -> ('0 -> boolean))") (param anyref (ref $capture)) (result anyref)
    (local $1 anyref)
    local.get 0
    local.set $1
    i32.const 29
    local.get $1
    array.new_fixed $capture 1
    struct.new $"('0 -> ('0 -> boolean))"
  )
  (func (;30;) (type $"(raw) ('0 -> boolean)") (param anyref (ref $capture)) (result anyref)
    (local $0 anyref) (local anyref i32 i32)
    local.get 0
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 3
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 3
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
    local.get 3
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
    local.set 4
    local.get 3
    ref.cast (ref $string)
    array.len
    local.set 5
    loop ;; label = @1
      local.get 3
      ref.cast (ref $string)
      local.get 4
      array.get_u $string
      local.get $0
      ref.cast (ref $string)
      local.get 4
      array.get_u $string
      i32.ne
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 4
      i32.const 1
      i32.add
      local.tee 4
      local.get 5
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;31;) (type $"(raw) ('0 -> ('0 -> boolean))") (param anyref (ref $capture)) (result anyref)
    (local $1 anyref)
    local.get 0
    local.set $1
    i32.const 31
    local.get $1
    array.new_fixed $capture 1
    struct.new $"('0 -> ('0 -> boolean))"
  )
  (func (;32;) (type $"(raw) ('0 -> boolean)") (param anyref (ref $capture)) (result anyref)
    (local $0 anyref) (local anyref i32 i32)
    local.get 0
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 3
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 3
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
    local.get 3
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
    local.set 4
    local.get 3
    ref.cast (ref $string)
    array.len
    local.set 5
    loop ;; label = @1
      local.get 3
      ref.cast (ref $string)
      local.get 4
      array.get_u $string
      local.get $0
      ref.cast (ref $string)
      local.get 4
      array.get_u $string
      i32.eq
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 4
      i32.const 1
      i32.add
      local.tee 4
      local.get 5
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;33;) (type $"(raw) ('0 -> ('0 -> boolean))") (param anyref (ref $capture)) (result anyref)
    (local $1 anyref)
    local.get 0
    local.set $1
    i32.const 33
    local.get $1
    array.new_fixed $capture 1
    struct.new $"('0 -> ('0 -> boolean))"
  )
  (func (;34;) (type $"(raw) ('0 -> boolean)") (param anyref (ref $capture)) (result anyref)
    (local $0 anyref) (local anyref i32 i32)
    local.get 0
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 3
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 3
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
    local.get 3
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
    local.set 4
    local.get 3
    ref.cast (ref $string)
    array.len
    local.set 5
    loop ;; label = @1
      local.get 3
      ref.cast (ref $string)
      local.get 4
      array.get_u $string
      local.get $0
      ref.cast (ref $string)
      local.get 4
      array.get_u $string
      i32.gt_u
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 4
      i32.const 1
      i32.add
      local.tee 4
      local.get 5
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;35;) (type $"(raw) ('0 -> ('0 -> boolean))") (param anyref (ref $capture)) (result anyref)
    (local $1 anyref)
    local.get 0
    local.set $1
    i32.const 35
    local.get $1
    array.new_fixed $capture 1
    struct.new $"('0 -> ('0 -> boolean))"
  )
  (func (;36;) (type $"(raw) ('0 -> boolean)") (param anyref (ref $capture)) (result anyref)
    (local $0 anyref) (local anyref i32 i32)
    local.get 0
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 3
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 3
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
    local.get 3
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
    local.set 4
    local.get 3
    ref.cast (ref $string)
    array.len
    local.set 5
    loop ;; label = @1
      local.get 3
      ref.cast (ref $string)
      local.get 4
      array.get_u $string
      local.get $0
      ref.cast (ref $string)
      local.get 4
      array.get_u $string
      i32.lt_u
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 4
      i32.const 1
      i32.add
      local.tee 4
      local.get 5
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;37;) (type $"(raw) ('0 -> ('0 -> boolean))") (param anyref (ref $capture)) (result anyref)
    (local $1 anyref)
    local.get 0
    local.set $1
    i32.const 37
    local.get $1
    array.new_fixed $capture 1
    struct.new $"('0 -> ('0 -> boolean))"
  )
  (func (;38;) (type $"(raw) ('0 -> boolean)") (param anyref (ref $capture)) (result anyref)
    (local $0 anyref) (local anyref i32 i32)
    local.get 0
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 3
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 3
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
    local.get 3
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
    local.set 4
    local.get 3
    ref.cast (ref $string)
    array.len
    local.set 5
    loop ;; label = @1
      local.get 3
      ref.cast (ref $string)
      local.get 4
      array.get_u $string
      local.get $0
      ref.cast (ref $string)
      local.get 4
      array.get_u $string
      i32.ge_u
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 4
      i32.const 1
      i32.add
      local.tee 4
      local.get 5
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;39;) (type $"(raw) ('0 -> ('0 -> boolean))") (param anyref (ref $capture)) (result anyref)
    (local $1 anyref)
    local.get 0
    local.set $1
    i32.const 39
    local.get $1
    array.new_fixed $capture 1
    struct.new $"('0 -> ('0 -> boolean))"
  )
  (func (;40;) (type $"(raw) ('0 -> boolean)") (param anyref (ref $capture)) (result anyref)
    (local $0 anyref) (local anyref i32 i32)
    local.get 0
    local.set $0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 3
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 3
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
    local.get 3
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
    local.set 4
    local.get 3
    ref.cast (ref $string)
    array.len
    local.set 5
    loop ;; label = @1
      local.get 3
      ref.cast (ref $string)
      local.get 4
      array.get_u $string
      local.get $0
      ref.cast (ref $string)
      local.get 4
      array.get_u $string
      i32.le_u
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 4
      i32.const 1
      i32.add
      local.tee 4
      local.get 5
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;41;) (type $"(raw) (unit -> '0)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $unit))
    local.get 0
    ref.cast (ref $unit)
    local.set $0
    unreachable
  )
  (func (;42;) (type $"(raw) (string -> integer)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $string))
    local.get 0
    ref.cast (ref $string)
    local.set $0
    local.get $0
    array.len
    i64.extend_i32_u
    struct.new $integer
  )
  (func (;43;) (type $"(raw) (string -> unit)") (param anyref (ref $capture)) (result anyref)
    (local $0 (ref $string)) (local i32 i32)
    local.get 0
    ref.cast (ref $string)
    local.set $0
    i32.const 0
    local.set 3
    local.get $0
    array.len
    local.set 4
    loop ;; label = @1
      local.get 3
      local.get 4
      i32.lt_u
      if ;; label = @2
        local.get 3
        local.get $0
        local.get 3
        array.get_u $string
        i32.store8
        local.get 3
        i32.const 1
        i32.add
        local.set 3
        br 1 (;@1;)
      end
    end
    i32.const 0
    local.get 4
    i32.const 43
    call_indirect (type 30)
    struct.new $unit
  )
  (func (;44;) (type $"(raw) (string -> (string -> string))") (param anyref (ref $capture)) (result anyref)
    (local $1 (ref $string))
    local.get 0
    ref.cast (ref $string)
    local.set $1
    i32.const 45
    local.get $1
    array.new_fixed $capture 1
    struct.new $"(string -> (string -> string))"
  )
  (func (;45;) (type $"(raw) (string -> string)") (param anyref (ref $capture)) (result anyref)
    (local $unit (ref $string)) (local (ref $string) i32 i32 (ref $string))
    local.get 0
    ref.cast (ref $string)
    local.set $unit
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $string)
    local.set 3
    local.get 3
    array.len
    local.tee 4
    local.get $unit
    array.len
    local.tee 5
    i32.add
    array.new_default $string
    local.tee 6
    i32.const 0
    local.get 3
    i32.const 0
    local.get 4
    array.copy $string $string
    local.get 6
    local.get 4
    local.get $unit
    i32.const 0
    local.get 5
    array.copy $string $string
    local.get 6
  )
  (func (;46;) (type $"(raw) (unit -> '1)") (param anyref (ref $capture)) (result anyref)
    (local $condition#0 (ref $unit)) (local (ref $"(unit -> '2)"))
    local.get 0
    ref.cast (ref $unit)
    local.set $condition#0
    global.get 21
    ref.as_non_null
    local.set 3
    struct.new $unit
    local.get 3
    struct.get $"(unit -> '2)" 1
    local.get 3
    struct.get $"(unit -> '2)" 0
    call_indirect (type $"(raw) (unit -> '2)")
    ref.cast (ref any)
  )
  (func (;47;) (type $"(raw) (boolean -> unit)") (param anyref (ref $capture)) (result anyref)
    (local $s#1 (ref $boolean)) (local (ref $"(unit -> '4)"))
    local.get 0
    ref.cast (ref $boolean)
    local.set $s#1
    local.get $s#1
    struct.get $boolean 0
    if (result (ref $unit)) ;; label = @1
      struct.new $unit
    else
      global.get 25
      ref.as_non_null
      local.set 3
      struct.new $unit
      local.get 3
      struct.get $"(unit -> '4)" 1
      local.get 3
      struct.get $"(unit -> '4)" 0
      call_indirect (type $"(raw) (unit -> '4)")
      ref.cast (ref $unit)
    end
  )
  (func (;48;) (type $"(raw) (string -> unit)") (param anyref (ref $capture)) (result anyref)
    (local $s#0 (ref $string)) (local (ref $"(string -> unit)"))
    local.get 0
    ref.cast (ref $string)
    local.set $s#0
    global.get 23
    ref.as_non_null
    local.set 3
    local.get $s#0
    local.get 3
    struct.get $"(string -> unit)" 1
    local.get 3
    struct.get $"(string -> unit)" 0
    call_indirect (type $"(raw) (string -> unit)")
    ref.cast (ref $unit)
  )
  (func (;49;) (type $"(raw) (string -> integer)") (param anyref (ref $capture)) (result anyref)
    (local $s1#1 (ref $string)) (local (ref $"(string -> integer)"))
    local.get 0
    ref.cast (ref $string)
    local.set $s1#1
    global.get 22
    ref.as_non_null
    local.set 3
    local.get $s1#1
    local.get 3
    struct.get $"(string -> integer)" 1
    local.get 3
    struct.get $"(string -> integer)" 0
    call_indirect (type $"(raw) (string -> integer)")
    ref.cast (ref $integer)
  )
  (func (;50;) (type $"(raw) (string -> (string -> string))") (param anyref (ref $capture)) (result anyref)
    (local $s2#2 (ref $string))
    local.get 0
    ref.cast (ref $string)
    local.set $s2#2
    i32.const 51
    local.get $s2#2
    ref.cast (ref any)
    array.new_fixed $capture 1
    struct.new $"(string -> string)"
  )
  (func (;51;) (type $"(raw) (string -> string)") (param anyref (ref $capture)) (result anyref)
    (local $s#3 (ref $string)) (local (ref $string) (ref $"(string -> string)") (ref $"(string -> (string -> string))"))
    local.get 0
    ref.cast (ref $string)
    local.set $s#3
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $string)
    local.set 3
    global.get 24
    ref.as_non_null
    local.set 5
    local.get 3
    local.get 5
    struct.get $"(string -> (string -> string))" 1
    local.get 5
    struct.get $"(string -> (string -> string))" 0
    call_indirect (type $"(raw) (string -> (string -> string))")
    ref.cast (ref $"(string -> string)")
    local.set 4
    local.get $s#3
    local.get 4
    struct.get $"(string -> string)" 1
    local.get 4
    struct.get $"(string -> string)" 0
    call_indirect (type $"(raw) (string -> string)")
    ref.cast (ref $string)
  )
  (func (;52;) (type $"(raw) (string -> unit)") (param anyref (ref $capture)) (result anyref)
    (local $a (ref $string)) (local (ref $"(string -> unit)"))
    local.get 0
    ref.cast (ref $string)
    local.set $a
    global.get 23
    ref.as_non_null
    local.set 3
    local.get $a
    local.get 3
    struct.get $"(string -> unit)" 1
    local.get 3
    struct.get $"(string -> unit)" 0
    call_indirect (type $"(raw) (string -> unit)")
    ref.cast (ref $unit)
  )
  (func (;53;) (type $"(raw) ('0 -> Some of '0 | None of unit)") (param anyref (ref $capture)) (result anyref)
    (local $a anyref)
    local.get 0
    local.set $a
    i32.const 0
    local.get $a
    struct.new $"Some of '0 | None of unit"
  )
  (func (;54;) (type $"(raw) (unit -> Some of '0 | None of unit)") (param anyref (ref $capture)) (result anyref)
    (local $operation#0 (ref $unit))
    local.get 0
    ref.cast (ref $unit)
    local.set $operation#0
    i32.const 1
    local.get $operation#0
    struct.new $"Some of '0 | None of unit"
  )
  (func (;55;) (type $"(raw) (('3 -> '6) -> (Some of '3 | None of unit -> Some of '6 | None of unit))") (param anyref (ref $capture)) (result anyref)
    (local $maybe#1 (ref $"('3 -> '6)"))
    local.get 0
    ref.cast (ref $"('3 -> '6)")
    local.set $maybe#1
    i32.const 56
    local.get $maybe#1
    ref.cast (ref any)
    array.new_fixed $capture 1
    struct.new $"(Some of '3 | None of unit -> Some of '6 | None of unit)"
  )
  (func (;56;) (type $"(raw) (Some of '3 | None of unit -> Some of '6 | None of unit)") (param anyref (ref $capture)) (result anyref)
    (local $operation#4 (ref $"Some of '3 | None of unit")) (local (ref $"('3 -> '6)") (ref $"Some of '3 | None of unit") anyref anyref (ref $"('6 -> Some of '6 | None of unit)") (ref $"('3 -> '6)") (ref $"Some of '3 | None of unit") (ref $"(unit -> Some of '8 | None of unit)"))
    local.get 0
    ref.cast (ref $"Some of '3 | None of unit")
    local.set $operation#4
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $"('3 -> '6)")
    local.set 3
    local.get $operation#4
    local.set 4
    block (result (ref $"Some of '6 | None of unit")) ;; label = @1
      block ;; label = @2
        i32.const 0
        local.get 4
        struct.get $"Some of '0 | None of unit" 0
        i32.eq
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 4
        struct.get $"Some of '0 | None of unit" 1
        ref.cast (ref any)
        local.set 5
        local.get 5
        local.set 6
        global.get 31
        ref.as_non_null
        local.set 7
        local.get 3
        local.set 8
        local.get 6
        local.get 8
        struct.get $"('3 -> '6)" 1
        local.get 8
        struct.get $"('3 -> '6)" 0
        call_indirect (type $"(raw) ('3 -> '6)")
        ref.cast (ref any)
        local.get 7
        struct.get $"('6 -> Some of '6 | None of unit)" 1
        local.get 7
        struct.get $"('6 -> Some of '6 | None of unit)" 0
        call_indirect (type $"(raw) ('6 -> Some of '6 | None of unit)")
        ref.cast (ref $"Some of '6 | None of unit")
        br 1 (;@1;)
      end
      block ;; label = @2
        local.get 4
        local.set 9
        global.get 32
        ref.as_non_null
        local.set 10
        struct.new $unit
        local.get 10
        struct.get $"(unit -> Some of '8 | None of unit)" 1
        local.get 10
        struct.get $"(unit -> Some of '8 | None of unit)" 0
        call_indirect (type $"(raw) (unit -> Some of '8 | None of unit)")
        ref.cast (ref $"Some of '8 | None of unit")
        br 1 (;@1;)
      end
      unreachable
    end
  )
  (func (;57;) (type $"(raw) (('6 -> '7) -> (Some of '9 | None of unit -> unit))") (param anyref (ref $capture)) (result anyref)
    (local $maybe#5 (ref $"('6 -> '7)"))
    local.get 0
    ref.cast (ref $"('6 -> '7)")
    local.set $maybe#5
    i32.const 58
    local.get $maybe#5
    ref.cast (ref any)
    array.new_fixed $capture 1
    struct.new $"(Some of '9 | None of unit -> unit)"
  )
  (func (;58;) (type $"(raw) (Some of '9 | None of unit -> unit)") (param anyref (ref $capture)) (result anyref)
    (local $a#6 (ref $"Some of '9 | None of unit")) (local $operation#4 (ref $"('6 -> '7)")) (local (ref $"(Some of '9 | None of unit -> Some of '10 | None of unit)") (ref $"(('9 -> '10) -> (Some of '9 | None of unit -> Some of '10 | None of unit))"))
    local.get 0
    ref.cast (ref $"Some of '9 | None of unit")
    local.set $a#6
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $"('6 -> '7)")
    local.set $operation#4
    global.get 33
    ref.as_non_null
    local.set 5
    i32.const 59
    local.get $operation#4
    ref.cast (ref any)
    array.new_fixed $capture 1
    struct.new $"('6 -> '6)"
    local.get 5
    struct.get $"(('9 -> '10) -> (Some of '9 | None of unit -> Some of '10 | None of unit))" 1
    local.get 5
    struct.get $"(('9 -> '10) -> (Some of '9 | None of unit -> Some of '10 | None of unit))" 0
    call_indirect (type $"(raw) (('3 -> '6) -> (Some of '3 | None of unit -> Some of '6 | None of unit))")
    ref.cast (ref $"(Some of '9 | None of unit -> Some of '10 | None of unit)")
    local.set 4
    local.get $a#6
    local.get 4
    struct.get $"(Some of '9 | None of unit -> Some of '10 | None of unit)" 1
    local.get 4
    struct.get $"(Some of '9 | None of unit -> Some of '10 | None of unit)" 0
    call_indirect (type $"(raw) (Some of '9 | None of unit -> Some of '10 | None of unit)")
    ref.cast (ref $"Some of '10 | None of unit")
    drop
    struct.new $unit
  )
  (func (;59;) (type $"(raw) ('6 -> '6)") (param anyref (ref $capture)) (result anyref)
    (local $maybe#7 anyref) (local (ref $"('6 -> '7)") (ref $"('6 -> '7)"))
    local.get 0
    local.set $maybe#7
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $"('6 -> '7)")
    local.set 3
    local.get 3
    local.set 4
    local.get $maybe#7
    local.get 4
    struct.get $"('6 -> '7)" 1
    local.get 4
    struct.get $"('6 -> '7)" 0
    call_indirect (type $"(raw) ('6 -> '7)")
    ref.cast (ref any)
    drop
    local.get $maybe#7
  )
  (func (;60;) (type $"(raw) (Some of '5 | None of unit -> '5)") (param anyref (ref $capture)) (result anyref)
    (local $maybe#10 (ref $"Some of '5 | None of unit")) (local (ref $"Some of '5 | None of unit") anyref) (local $_#11 anyref) (local $_#12 (ref $"Some of '5 | None of unit")) (local (ref $"(unit -> '7)"))
    local.get 0
    ref.cast (ref $"Some of '5 | None of unit")
    local.set $maybe#10
    local.get $maybe#10
    local.set 3
    block (result anyref) ;; label = @1
      block ;; label = @2
        i32.const 0
        local.get 3
        struct.get $"Some of '0 | None of unit" 0
        i32.eq
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 3
        struct.get $"Some of '0 | None of unit" 1
        ref.cast (ref any)
        local.set 4
        local.get 4
        local.set $_#11
        local.get $_#11
        br 1 (;@1;)
      end
      block ;; label = @2
        local.get 3
        local.set $_#12
        global.get 25
        ref.as_non_null
        local.set 7
        struct.new $unit
        local.get 7
        struct.get $"(unit -> '7)" 1
        local.get 7
        struct.get $"(unit -> '7)" 0
        call_indirect (type $"(raw) (unit -> '7)")
        ref.cast (ref any)
        br 1 (;@1;)
      end
      unreachable
    end
  )
  (func (;61;) (type $"(raw) (Some of '3 | None of unit -> boolean)") (param anyref (ref $capture)) (result anyref)
    (local $maybe#13 (ref $"Some of '3 | None of unit")) (local (ref $"Some of '3 | None of unit") anyref anyref (ref $"Some of '3 | None of unit"))
    local.get 0
    ref.cast (ref $"Some of '3 | None of unit")
    local.set $maybe#13
    local.get $maybe#13
    local.set 3
    block (result (ref $boolean)) ;; label = @1
      block ;; label = @2
        i32.const 0
        local.get 3
        struct.get $"Some of '0 | None of unit" 0
        i32.eq
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 3
        struct.get $"Some of '0 | None of unit" 1
        ref.cast (ref any)
        local.set 4
        local.get 4
        local.set 5
        i32.const 1
        struct.new $boolean
        br 1 (;@1;)
      end
      block ;; label = @2
        local.get 3
        local.set 6
        i32.const 0
        struct.new $boolean
        br 1 (;@1;)
      end
      unreachable
    end
  )
  (func (;62;) (type $"(raw) (Some of '4 | None of unit -> boolean)") (param anyref (ref $capture)) (result anyref)
    (local $a (ref $"Some of '4 | None of unit")) (local (ref $"(Some of '4 | None of unit -> boolean)") (ref $"(boolean -> boolean)"))
    local.get 0
    ref.cast (ref $"Some of '4 | None of unit")
    local.set $a
    global.get 36
    ref.as_non_null
    local.set 3
    local.get $a
    local.get 3
    struct.get $"(Some of '4 | None of unit -> boolean)" 1
    local.get 3
    struct.get $"(Some of '4 | None of unit -> boolean)" 0
    call_indirect (type $"(raw) (Some of '4 | None of unit -> boolean)")
    ref.cast (ref $boolean)
    global.get 2
    ref.as_non_null
    local.tee 4
    struct.get $"(boolean -> boolean)" 1
    local.get 4
    struct.get $"(boolean -> boolean)" 0
    call_indirect (type $"(raw) (boolean -> boolean)")
  )
  (func (;63;) (type $"(raw) (('0 * list:t) -> Pair of ('0 * list:t) | Nil of unit)") (param anyref (ref $capture)) (result anyref)
    (local $a (ref $"('0 * list:t)"))
    local.get 0
    ref.cast (ref $"('0 * list:t)")
    local.set $a
    i32.const 0
    local.get $a
    struct.new $"Pair of ('0 * list:t) | Nil of unit"
  )
  (func (;64;) (type $"(raw) (unit -> Pair of ('0 * list:t) | Nil of unit)") (param anyref (ref $capture)) (result anyref)
    (local $operation#0 (ref $unit))
    local.get 0
    ref.cast (ref $unit)
    local.set $operation#0
    i32.const 1
    local.get $operation#0
    struct.new $"Pair of ('0 * list:t) | Nil of unit"
  )
  (func (;65;) (type $"(raw) (('3 -> '9) -> (Pair of ('3 * list:t) | Nil of unit -> Pair of ('9 * list:t) | Nil of unit))") (param anyref (ref $capture)) (result anyref)
    (local $list#1 (ref $"('3 -> '9)"))
    local.get 0
    ref.cast (ref $"('3 -> '9)")
    local.set $list#1
    i32.const 66
    local.get $list#1
    ref.cast (ref any)
    local.get $list#1
    ref.cast (ref any)
    array.new_fixed $capture 2
    struct.new $"(Pair of ('3 * list:t) | Nil of unit -> Pair of ('9 * list:t) | Nil of unit)"
  )
  (func (;66;) (type $"(raw) (Pair of ('3 * list:t) | Nil of unit -> Pair of ('9 * list:t) | Nil of unit)") (param anyref (ref $capture)) (result anyref)
    (local $operation#4 (ref $"Pair of ('3 * list:t) | Nil of unit")) (local (ref $"('3 -> '9)") (ref $"('3 -> '9)") (ref $"Pair of ('3 * list:t) | Nil of unit") (ref $"('3 * list:t)") anyref anyref (ref $"Pair of ('0 * list:t) | Nil of unit") (ref $"Pair of ('0 * list:t) | Nil of unit") (ref $"(('9 * list:t) -> Pair of ('9 * list:t) | Nil of unit)") (ref $"('3 -> '9)") (ref $"(list:t -> list:t)") (ref $"(('3 -> '9) -> (list:t -> list:t))") (ref $unit) (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"(unit -> Pair of ('11 * list:t) | Nil of unit)"))
    local.get 0
    ref.cast (ref $"Pair of ('3 * list:t) | Nil of unit")
    local.set $operation#4
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $"('3 -> '9)")
    local.set 3
    local.get 1
    i32.const 1
    array.get $capture
    ref.cast (ref $"('3 -> '9)")
    local.set 4
    local.get $operation#4
    local.set 5
    block (result (ref $"Pair of ('9 * list:t) | Nil of unit")) ;; label = @1
      block ;; label = @2
        i32.const 0
        local.get 5
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 0
        i32.eq
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 5
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 1
        ref.cast (ref $"('3 * list:t)")
        local.set 6
        local.get 6
        struct.get $"('3 * list:t)" 0
        local.set 7
        local.get 7
        local.set 8
        local.get 6
        struct.get $"('3 * list:t)" 1
        local.set 9
        local.get 9
        local.set 10
        global.get 38
        ref.as_non_null
        local.set 11
        local.get 4
        local.set 12
        local.get 8
        local.get 12
        struct.get $"('3 -> '9)" 1
        local.get 12
        struct.get $"('3 -> '9)" 0
        call_indirect (type $"(raw) ('3 -> '9)")
        ref.cast (ref any)
        local.get 10
        global.get 40
        ref.as_non_null
        local.set 14
        local.get 4
        local.get 14
        struct.get $"(('3 -> '9) -> (list:t -> list:t))" 1
        local.get 14
        struct.get $"(('3 -> '9) -> (list:t -> list:t))" 0
        call_indirect (type $"(raw) (('3 -> '9) -> (list:t -> list:t))")
        ref.cast (ref $"(list:t -> list:t)")
        local.tee 13
        struct.get $"(list:t -> list:t)" 1
        local.get 13
        struct.get $"(list:t -> list:t)" 0
        call_indirect (type $"(raw) (list:t -> list:t)")
        ref.cast (ref $"Pair of ('0 * list:t) | Nil of unit")
        struct.new $"('9 * list:t)"
        local.get 11
        struct.get $"(('9 * list:t) -> Pair of ('9 * list:t) | Nil of unit)" 1
        local.get 11
        struct.get $"(('9 * list:t) -> Pair of ('9 * list:t) | Nil of unit)" 0
        call_indirect (type $"(raw) (('9 * list:t) -> Pair of ('9 * list:t) | Nil of unit)")
        ref.cast (ref $"Pair of ('9 * list:t) | Nil of unit")
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const 1
        local.get 5
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 0
        i32.eq
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 5
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 1
        ref.cast (ref $unit)
        local.set 15
        local.get 15
        global.get 15
        ref.as_non_null
        local.tee 16
        struct.get $"('0 -> ('0 -> boolean))" 1
        local.get 16
        struct.get $"('0 -> ('0 -> boolean))" 0
        call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
        ref.cast (ref $"('0 -> boolean)")
        local.set 17
        struct.new $unit
        local.get 17
        struct.get $"('0 -> boolean)" 1
        local.get 17
        struct.get $"('0 -> boolean)" 0
        call_indirect (type $"(raw) ('0 -> boolean)")
        ref.cast (ref $boolean)
        struct.get $boolean 0
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        global.get 39
        ref.as_non_null
        local.set 18
        struct.new $unit
        local.get 18
        struct.get $"(unit -> Pair of ('11 * list:t) | Nil of unit)" 1
        local.get 18
        struct.get $"(unit -> Pair of ('11 * list:t) | Nil of unit)" 0
        call_indirect (type $"(raw) (unit -> Pair of ('11 * list:t) | Nil of unit)")
        ref.cast (ref $"Pair of ('11 * list:t) | Nil of unit")
        br 1 (;@1;)
      end
      unreachable
    end
  )
  (func (;67;) (type $"(raw) (('6 -> '7) -> (list:t -> unit))") (param anyref (ref $capture)) (result anyref)
    (local $list#5 (ref $"('6 -> '7)"))
    local.get 0
    ref.cast (ref $"('6 -> '7)")
    local.set $list#5
    i32.const 68
    local.get $list#5
    ref.cast (ref any)
    array.new_fixed $capture 1
    struct.new $"(list:t -> unit)"
  )
  (func (;68;) (type $"(raw) (list:t -> unit)") (param anyref (ref $capture)) (result anyref)
    (local $a#6 (ref $"Pair of ('0 * list:t) | Nil of unit")) (local $operation#4 (ref $"('6 -> '7)")) (local (ref $"(list:t -> list:t)") (ref $"(('9 -> '10) -> (list:t -> list:t))"))
    local.get 0
    ref.cast (ref $"Pair of ('0 * list:t) | Nil of unit")
    local.set $a#6
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $"('6 -> '7)")
    local.set $operation#4
    global.get 40
    ref.as_non_null
    local.set 5
    i32.const 69
    local.get $operation#4
    ref.cast (ref any)
    array.new_fixed $capture 1
    struct.new $"('6 -> '6)"
    local.get 5
    struct.get $"(('9 -> '10) -> (list:t -> list:t))" 1
    local.get 5
    struct.get $"(('9 -> '10) -> (list:t -> list:t))" 0
    call_indirect (type $"(raw) (('9 -> '10) -> (list:t -> list:t))")
    ref.cast (ref $"(list:t -> list:t)")
    local.set 4
    local.get $a#6
    local.get 4
    struct.get $"(list:t -> list:t)" 1
    local.get 4
    struct.get $"(list:t -> list:t)" 0
    call_indirect (type $"(raw) (list:t -> list:t)")
    ref.cast (ref $"Pair of ('0 * list:t) | Nil of unit")
    drop
    struct.new $unit
  )
  (func (;69;) (type $"(raw) ('6 -> '6)") (param anyref (ref $capture)) (result anyref)
    (local $item#7 anyref) (local (ref $"('6 -> '7)") (ref $"('6 -> '7)"))
    local.get 0
    local.set $item#7
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $"('6 -> '7)")
    local.set 3
    local.get 3
    local.set 4
    local.get $item#7
    local.get 4
    struct.get $"('6 -> '7)" 1
    local.get 4
    struct.get $"('6 -> '7)" 0
    call_indirect (type $"(raw) ('6 -> '7)")
    ref.cast (ref any)
    drop
    local.get $item#7
  )
  (func (;70;) (type $"(raw) ('3 -> (Pair of ('10 * list:t) | Nil of unit -> Pair of ('10 * list:t) | Nil of unit))") (param anyref (ref $capture)) (result anyref)
    (local $list#8 anyref)
    local.get 0
    local.set $list#8
    i32.const 71
    local.get $list#8
    ref.cast (ref any)
    local.get $list#8
    ref.cast (ref any)
    array.new_fixed $capture 2
    struct.new $"(Pair of ('10 * list:t) | Nil of unit -> Pair of ('10 * list:t) | Nil of unit)"
  )
  (func (;71;) (type $"(raw) (Pair of ('10 * list:t) | Nil of unit -> Pair of ('10 * list:t) | Nil of unit)") (param anyref (ref $capture)) (result anyref)
    (local $list#11 (ref $"Pair of ('10 * list:t) | Nil of unit")) (local anyref anyref (ref $"Pair of ('10 * list:t) | Nil of unit")) (local $head#12 (ref $"('10 * list:t)")) (local anyref) (local $tail#13 anyref) (local (ref $"Pair of ('0 * list:t) | Nil of unit") (ref $"Pair of ('0 * list:t) | Nil of unit") (ref $"(('10 * list:t) -> Pair of ('10 * list:t) | Nil of unit)") (ref $"(list:t -> list:t)") (ref $"('3 -> (list:t -> list:t))") (ref $unit) (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"(('14 * list:t) -> Pair of ('14 * list:t) | Nil of unit)") (ref $"(unit -> Pair of ('12 * list:t) | Nil of unit)"))
    local.get 0
    ref.cast (ref $"Pair of ('10 * list:t) | Nil of unit")
    local.set $list#11
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 3
    local.get 1
    i32.const 1
    array.get $capture
    ref.cast (ref any)
    local.set 4
    local.get $list#11
    local.set 5
    block (result (ref $"Pair of ('10 * list:t) | Nil of unit")) ;; label = @1
      block ;; label = @2
        i32.const 0
        local.get 5
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 0
        i32.eq
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 5
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 1
        ref.cast (ref $"('10 * list:t)")
        local.set $head#12
        local.get $head#12
        struct.get $"('10 * list:t)" 0
        local.set 7
        local.get 7
        local.set $tail#13
        local.get $head#12
        struct.get $"('10 * list:t)" 1
        local.set 9
        local.get 9
        local.set 10
        global.get 38
        ref.as_non_null
        local.set 11
        local.get $tail#13
        global.get 42
        ref.as_non_null
        local.set 13
        local.get 4
        local.get 13
        struct.get $"('3 -> (list:t -> list:t))" 1
        local.get 13
        struct.get $"('3 -> (list:t -> list:t))" 0
        call_indirect (type $"(raw) ('3 -> (list:t -> list:t))")
        ref.cast (ref $"(list:t -> list:t)")
        local.set 12
        local.get 10
        local.get 12
        struct.get $"(list:t -> list:t)" 1
        local.get 12
        struct.get $"(list:t -> list:t)" 0
        call_indirect (type $"(raw) (list:t -> list:t)")
        ref.cast (ref $"Pair of ('0 * list:t) | Nil of unit")
        struct.new $"('10 * list:t)"
        local.get 11
        struct.get $"(('10 * list:t) -> Pair of ('10 * list:t) | Nil of unit)" 1
        local.get 11
        struct.get $"(('10 * list:t) -> Pair of ('10 * list:t) | Nil of unit)" 0
        call_indirect (type $"(raw) (('10 * list:t) -> Pair of ('10 * list:t) | Nil of unit)")
        ref.cast (ref $"Pair of ('10 * list:t) | Nil of unit")
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const 1
        local.get 5
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 0
        i32.eq
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 5
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 1
        ref.cast (ref $unit)
        local.set 14
        local.get 14
        global.get 15
        ref.as_non_null
        local.tee 15
        struct.get $"('0 -> ('0 -> boolean))" 1
        local.get 15
        struct.get $"('0 -> ('0 -> boolean))" 0
        call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
        ref.cast (ref $"('0 -> boolean)")
        local.set 16
        struct.new $unit
        local.get 16
        struct.get $"('0 -> boolean)" 1
        local.get 16
        struct.get $"('0 -> boolean)" 0
        call_indirect (type $"(raw) ('0 -> boolean)")
        ref.cast (ref $boolean)
        struct.get $boolean 0
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        global.get 38
        ref.as_non_null
        local.set 17
        local.get 4
        global.get 39
        ref.as_non_null
        local.set 18
        struct.new $unit
        local.get 18
        struct.get $"(unit -> Pair of ('12 * list:t) | Nil of unit)" 1
        local.get 18
        struct.get $"(unit -> Pair of ('12 * list:t) | Nil of unit)" 0
        call_indirect (type $"(raw) (unit -> Pair of ('12 * list:t) | Nil of unit)")
        ref.cast (ref $"Pair of ('12 * list:t) | Nil of unit")
        struct.new $"('3 * Pair of ('12 * list:t) | Nil of unit)"
        local.get 17
        struct.get $"(('14 * list:t) -> Pair of ('14 * list:t) | Nil of unit)" 1
        local.get 17
        struct.get $"(('14 * list:t) -> Pair of ('14 * list:t) | Nil of unit)" 0
        call_indirect (type $"(raw) (('14 * list:t) -> Pair of ('14 * list:t) | Nil of unit)")
        ref.cast (ref $"Pair of ('14 * list:t) | Nil of unit")
        br 1 (;@1;)
      end
      unreachable
    end
  )
  (func (;72;) (type $"(raw) (Pair of ('3 * list:t) | Nil of unit -> integer)") (param anyref (ref $capture)) (result anyref)
    (local $op#14 (ref $"Pair of ('3 * list:t) | Nil of unit")) (local (ref $"Pair of ('3 * list:t) | Nil of unit") (ref $"('3 * list:t)") anyref anyref (ref $"Pair of ('0 * list:t) | Nil of unit") (ref $"Pair of ('0 * list:t) | Nil of unit") (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)") (ref $"(list:t -> integer)") (ref $unit) (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)"))
    local.get 0
    ref.cast (ref $"Pair of ('3 * list:t) | Nil of unit")
    local.set $op#14
    local.get $op#14
    local.set 3
    block (result (ref $integer)) ;; label = @1
      block ;; label = @2
        i32.const 0
        local.get 3
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 0
        i32.eq
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 3
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 1
        ref.cast (ref $"('3 * list:t)")
        local.set 4
        local.get 4
        struct.get $"('3 * list:t)" 0
        local.set 5
        local.get 5
        local.set 6
        local.get 4
        struct.get $"('3 * list:t)" 1
        local.set 7
        local.get 7
        local.set 8
        i64.const 1
        struct.new $integer
        global.get 3
        ref.as_non_null
        local.tee 9
        struct.get $"(integer -> (integer -> integer))" 1
        local.get 9
        struct.get $"(integer -> (integer -> integer))" 0
        call_indirect (type $"(raw) (integer -> (integer -> integer))")
        ref.cast (ref $"(integer -> integer)")
        local.set 10
        global.get 43
        ref.as_non_null
        local.set 11
        local.get 8
        local.get 11
        struct.get $"(list:t -> integer)" 1
        local.get 11
        struct.get $"(list:t -> integer)" 0
        call_indirect (type $"(raw) (list:t -> integer)")
        ref.cast (ref $integer)
        local.get 10
        struct.get $"(integer -> integer)" 1
        local.get 10
        struct.get $"(integer -> integer)" 0
        call_indirect (type $"(raw) (integer -> integer)")
        ref.cast (ref $integer)
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const 1
        local.get 3
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 0
        i32.eq
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 3
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 1
        ref.cast (ref $unit)
        local.set 12
        local.get 12
        global.get 15
        ref.as_non_null
        local.tee 13
        struct.get $"('0 -> ('0 -> boolean))" 1
        local.get 13
        struct.get $"('0 -> ('0 -> boolean))" 0
        call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
        ref.cast (ref $"('0 -> boolean)")
        local.set 14
        struct.new $unit
        local.get 14
        struct.get $"('0 -> boolean)" 1
        local.get 14
        struct.get $"('0 -> boolean)" 0
        call_indirect (type $"(raw) ('0 -> boolean)")
        ref.cast (ref $boolean)
        struct.get $boolean 0
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        i64.const 0
        struct.new $integer
        br 1 (;@1;)
      end
      unreachable
    end
  )
  (func (;73;) (type $"(raw) (('9 -> ('6 -> '9)) -> ('9 -> (Pair of ('4 * list:t) | Nil of unit -> '9)))") (param anyref (ref $capture)) (result anyref)
    (local $acc#15 (ref $"('9 -> ('6 -> '9))"))
    local.get 0
    ref.cast (ref $"('9 -> ('6 -> '9))")
    local.set $acc#15
    i32.const 74
    local.get $acc#15
    ref.cast (ref any)
    local.get $acc#15
    ref.cast (ref any)
    array.new_fixed $capture 2
    struct.new $"('9 -> (Pair of ('4 * list:t) | Nil of unit -> '9))"
  )
  (func (;74;) (type $"(raw) ('9 -> (Pair of ('4 * list:t) | Nil of unit -> '9))") (param anyref (ref $capture)) (result anyref)
    (local $list#16 anyref) (local (ref $"('9 -> ('6 -> '9))") (ref $"('9 -> ('6 -> '9))"))
    local.get 0
    local.set $list#16
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $"('9 -> ('6 -> '9))")
    local.set 3
    local.get 1
    i32.const 1
    array.get $capture
    ref.cast (ref $"('9 -> ('6 -> '9))")
    local.set 4
    i32.const 75
    local.get 4
    ref.cast (ref any)
    local.get $list#16
    ref.cast (ref any)
    local.get 4
    ref.cast (ref any)
    local.get $list#16
    ref.cast (ref any)
    array.new_fixed $capture 4
    struct.new $"(Pair of ('4 * list:t) | Nil of unit -> '9)"
  )
  (func (;75;) (type $"(raw) (Pair of ('4 * list:t) | Nil of unit -> '9)") (param anyref (ref $capture)) (result anyref)
    (local $list1#19 (ref $"Pair of ('4 * list:t) | Nil of unit")) (local (ref $"('9 -> ('6 -> '9))") anyref (ref $"('9 -> ('6 -> '9))") anyref (ref $"Pair of ('4 * list:t) | Nil of unit") (ref $"('4 * list:t)") anyref anyref (ref $"Pair of ('0 * list:t) | Nil of unit") (ref $"Pair of ('0 * list:t) | Nil of unit") (ref $"('6 -> '9)") (ref $"('9 -> ('6 -> '9))") (ref $"(list:t -> '6)") (ref $"('4 -> (list:t -> '6))") (ref $"(('9 -> ('6 -> '9)) -> ('4 -> (list:t -> '6)))") (ref $unit) (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)"))
    local.get 0
    ref.cast (ref $"Pair of ('4 * list:t) | Nil of unit")
    local.set $list1#19
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $"('9 -> ('6 -> '9))")
    local.set 3
    local.get 1
    i32.const 1
    array.get $capture
    ref.cast (ref any)
    local.set 4
    local.get 1
    i32.const 2
    array.get $capture
    ref.cast (ref $"('9 -> ('6 -> '9))")
    local.set 5
    local.get 1
    i32.const 3
    array.get $capture
    ref.cast (ref any)
    local.set 6
    local.get $list1#19
    local.set 7
    block (result anyref) ;; label = @1
      block ;; label = @2
        i32.const 0
        local.get 7
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 0
        i32.eq
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 7
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 1
        ref.cast (ref $"('4 * list:t)")
        local.set 8
        local.get 8
        struct.get $"('4 * list:t)" 0
        local.set 9
        local.get 9
        local.set 10
        local.get 8
        struct.get $"('4 * list:t)" 1
        local.set 11
        local.get 11
        local.set 12
        local.get 5
        local.set 14
        local.get 6
        local.get 14
        struct.get $"('9 -> ('6 -> '9))" 1
        local.get 14
        struct.get $"('9 -> ('6 -> '9))" 0
        call_indirect (type $"(raw) ('9 -> ('6 -> '9))")
        ref.cast (ref $"('6 -> '9)")
        local.set 13
        global.get 44
        ref.as_non_null
        local.set 17
        local.get 5
        local.get 17
        struct.get $"(('9 -> ('6 -> '9)) -> ('4 -> (list:t -> '6)))" 1
        local.get 17
        struct.get $"(('9 -> ('6 -> '9)) -> ('4 -> (list:t -> '6)))" 0
        call_indirect (type $"(raw) (('9 -> ('6 -> '9)) -> ('4 -> (list:t -> '6)))")
        ref.cast (ref $"('4 -> (list:t -> '6))")
        local.set 16
        local.get 10
        local.get 16
        struct.get $"('4 -> (list:t -> '6))" 1
        local.get 16
        struct.get $"('4 -> (list:t -> '6))" 0
        call_indirect (type $"(raw) ('4 -> (list:t -> '6))")
        ref.cast (ref $"(list:t -> '6)")
        local.set 15
        local.get 12
        local.get 15
        struct.get $"(list:t -> '6)" 1
        local.get 15
        struct.get $"(list:t -> '6)" 0
        call_indirect (type $"(raw) (list:t -> '6)")
        ref.cast (ref any)
        local.get 13
        struct.get $"('6 -> '9)" 1
        local.get 13
        struct.get $"('6 -> '9)" 0
        call_indirect (type $"(raw) ('6 -> '9)")
        ref.cast (ref any)
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const 1
        local.get 7
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 0
        i32.eq
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 7
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 1
        ref.cast (ref $unit)
        local.set 18
        local.get 18
        global.get 15
        ref.as_non_null
        local.tee 19
        struct.get $"('0 -> ('0 -> boolean))" 1
        local.get 19
        struct.get $"('0 -> ('0 -> boolean))" 0
        call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
        ref.cast (ref $"('0 -> boolean)")
        local.set 20
        struct.new $unit
        local.get 20
        struct.get $"('0 -> boolean)" 1
        local.get 20
        struct.get $"('0 -> boolean)" 0
        call_indirect (type $"(raw) ('0 -> boolean)")
        ref.cast (ref $boolean)
        struct.get $boolean 0
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 6
        br 1 (;@1;)
      end
      unreachable
    end
  )
  (func (;76;) (type $"(raw) (Pair of ('11 * list:t) | Nil of unit -> (Pair of ('11 * list:t) | Nil of unit -> Pair of ('11 * list:t) | Nil of unit))") (param anyref (ref $capture)) (result anyref)
    (local $list2#20 (ref $"Pair of ('11 * list:t) | Nil of unit"))
    local.get 0
    ref.cast (ref $"Pair of ('11 * list:t) | Nil of unit")
    local.set $list2#20
    i32.const 77
    local.get $list2#20
    ref.cast (ref any)
    array.new_fixed $capture 1
    struct.new $"(Pair of ('11 * list:t) | Nil of unit -> Pair of ('11 * list:t) | Nil of unit)"
  )
  (func (;77;) (type $"(raw) (Pair of ('11 * list:t) | Nil of unit -> Pair of ('11 * list:t) | Nil of unit)") (param anyref (ref $capture)) (result anyref)
    (local $n#23 (ref $"Pair of ('11 * list:t) | Nil of unit")) (local (ref $"Pair of ('11 * list:t) | Nil of unit") (ref $"Pair of ('11 * list:t) | Nil of unit") (ref $"('11 * list:t)") anyref anyref (ref $"Pair of ('0 * list:t) | Nil of unit") (ref $"Pair of ('0 * list:t) | Nil of unit") (ref $"(('11 * list:t) -> Pair of ('11 * list:t) | Nil of unit)") (ref $"(Pair of ('11 * list:t) | Nil of unit -> list:t)") (ref $"(list:t -> (Pair of ('11 * list:t) | Nil of unit -> list:t))") (ref $unit) (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)"))
    local.get 0
    ref.cast (ref $"Pair of ('11 * list:t) | Nil of unit")
    local.set $n#23
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $"Pair of ('11 * list:t) | Nil of unit")
    local.set 3
    local.get 3
    local.set 4
    block (result (ref $"Pair of ('11 * list:t) | Nil of unit")) ;; label = @1
      block ;; label = @2
        i32.const 0
        local.get 4
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 0
        i32.eq
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 4
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 1
        ref.cast (ref $"('11 * list:t)")
        local.set 5
        local.get 5
        struct.get $"('11 * list:t)" 0
        local.set 6
        local.get 6
        local.set 7
        local.get 5
        struct.get $"('11 * list:t)" 1
        local.set 8
        local.get 8
        local.set 9
        global.get 38
        ref.as_non_null
        local.set 10
        local.get 7
        global.get 45
        ref.as_non_null
        local.set 12
        local.get 9
        local.get 12
        struct.get $"(list:t -> (Pair of ('11 * list:t) | Nil of unit -> list:t))" 1
        local.get 12
        struct.get $"(list:t -> (Pair of ('11 * list:t) | Nil of unit -> list:t))" 0
        call_indirect (type $"(raw) (list:t -> (Pair of ('11 * list:t) | Nil of unit -> list:t))")
        ref.cast (ref $"(Pair of ('11 * list:t) | Nil of unit -> list:t)")
        local.set 11
        local.get $n#23
        local.get 11
        struct.get $"(Pair of ('11 * list:t) | Nil of unit -> list:t)" 1
        local.get 11
        struct.get $"(Pair of ('11 * list:t) | Nil of unit -> list:t)" 0
        call_indirect (type $"(raw) (Pair of ('11 * list:t) | Nil of unit -> list:t)")
        ref.cast (ref $"Pair of ('0 * list:t) | Nil of unit")
        struct.new $"('11 * list:t)"
        local.get 10
        struct.get $"(('11 * list:t) -> Pair of ('11 * list:t) | Nil of unit)" 1
        local.get 10
        struct.get $"(('11 * list:t) -> Pair of ('11 * list:t) | Nil of unit)" 0
        call_indirect (type $"(raw) (('11 * list:t) -> Pair of ('11 * list:t) | Nil of unit)")
        ref.cast (ref $"Pair of ('11 * list:t) | Nil of unit")
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const 1
        local.get 4
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 0
        i32.eq
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 4
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 1
        ref.cast (ref $unit)
        local.set 13
        local.get 13
        global.get 15
        ref.as_non_null
        local.tee 14
        struct.get $"('0 -> ('0 -> boolean))" 1
        local.get 14
        struct.get $"('0 -> ('0 -> boolean))" 0
        call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
        ref.cast (ref $"('0 -> boolean)")
        local.set 15
        struct.new $unit
        local.get 15
        struct.get $"('0 -> boolean)" 1
        local.get 15
        struct.get $"('0 -> boolean)" 0
        call_indirect (type $"(raw) ('0 -> boolean)")
        ref.cast (ref $boolean)
        struct.get $boolean 0
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get $n#23
        br 1 (;@1;)
      end
      unreachable
    end
  )
  (func (;78;) (type $"(raw) (integer -> (Pair of ('8 * list:t) | Nil of unit -> Some of '8 | None of unit))") (param anyref (ref $capture)) (result anyref)
    (local $list#24 (ref $integer))
    local.get 0
    ref.cast (ref $integer)
    local.set $list#24
    i32.const 79
    local.get $list#24
    ref.cast (ref any)
    array.new_fixed $capture 1
    struct.new $"(Pair of ('8 * list:t) | Nil of unit -> Some of '8 | None of unit)"
  )
  (func (;79;) (type $"(raw) (Pair of ('8 * list:t) | Nil of unit -> Some of '8 | None of unit)") (param anyref (ref $capture)) (result anyref)
    (local $from#0 (ref $"Pair of ('8 * list:t) | Nil of unit")) (local (ref $integer) (ref $"(integer * Pair of ('8 * list:t) | Nil of unit)") (ref $integer) (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"Pair of ('8 * list:t) | Nil of unit") (ref $"('8 * list:t)") anyref anyref (ref $"Pair of ('0 * list:t) | Nil of unit") (ref $"Pair of ('0 * list:t) | Nil of unit") (ref $"('8 -> Some of '8 | None of unit)") (ref $integer) (ref $integer) (ref $"Pair of ('5 * list:t) | Nil of unit") (ref $"('5 * list:t)") anyref anyref (ref $"Pair of ('0 * list:t) | Nil of unit") (ref $"Pair of ('0 * list:t) | Nil of unit") (ref $"(list:t -> Some of '8 | None of unit)") (ref $"(integer -> (list:t -> Some of '8 | None of unit))") (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)") (ref $integer) (ref $integer) (ref $"Pair of ('6 * list:t) | Nil of unit") (ref $unit) (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"(unit -> Some of '12 | None of unit)"))
    local.get 0
    ref.cast (ref $"Pair of ('8 * list:t) | Nil of unit")
    local.set $from#0
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 3
    local.get 3
    local.get $from#0
    struct.new $"(integer * Pair of ('8 * list:t) | Nil of unit)"
    local.set 4
    block (result (ref $"Some of '8 | None of unit")) ;; label = @1
      block ;; label = @2
        local.get 4
        struct.get $"(integer * Pair of ('8 * list:t) | Nil of unit)" 0
        local.set 5
        local.get 5
        global.get 15
        ref.as_non_null
        local.tee 6
        struct.get $"('0 -> ('0 -> boolean))" 1
        local.get 6
        struct.get $"('0 -> ('0 -> boolean))" 0
        call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
        ref.cast (ref $"('0 -> boolean)")
        local.set 7
        i64.const 0
        struct.new $integer
        local.get 7
        struct.get $"('0 -> boolean)" 1
        local.get 7
        struct.get $"('0 -> boolean)" 0
        call_indirect (type $"(raw) ('0 -> boolean)")
        ref.cast (ref $boolean)
        struct.get $boolean 0
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 4
        struct.get $"(integer * Pair of ('8 * list:t) | Nil of unit)" 1
        local.set 8
        i32.const 0
        local.get 8
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 0
        i32.eq
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 8
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 1
        ref.cast (ref $"('8 * list:t)")
        local.set 9
        local.get 9
        struct.get $"('8 * list:t)" 0
        local.set 10
        local.get 10
        local.set 11
        local.get 9
        struct.get $"('8 * list:t)" 1
        local.set 12
        local.get 12
        local.set 13
        global.get 31
        ref.as_non_null
        local.set 14
        local.get 11
        local.get 14
        struct.get $"('8 -> Some of '8 | None of unit)" 1
        local.get 14
        struct.get $"('8 -> Some of '8 | None of unit)" 0
        call_indirect (type $"(raw) ('8 -> Some of '8 | None of unit)")
        ref.cast (ref $"Some of '8 | None of unit")
        br 1 (;@1;)
      end
      block ;; label = @2
        local.get 4
        struct.get $"(integer * Pair of ('8 * list:t) | Nil of unit)" 0
        local.set 15
        local.get 15
        local.set 16
        local.get 4
        struct.get $"(integer * Pair of ('8 * list:t) | Nil of unit)" 1
        local.set 17
        i32.const 0
        local.get 17
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 0
        i32.eq
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 17
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 1
        ref.cast (ref $"('5 * list:t)")
        local.set 18
        local.get 18
        struct.get $"('5 * list:t)" 0
        local.set 19
        local.get 19
        local.set 20
        local.get 18
        struct.get $"('5 * list:t)" 1
        local.set 21
        local.get 21
        local.set 22
        global.get 46
        ref.as_non_null
        local.set 24
        local.get 16
        global.get 4
        ref.as_non_null
        local.tee 25
        struct.get $"(integer -> (integer -> integer))" 1
        local.get 25
        struct.get $"(integer -> (integer -> integer))" 0
        call_indirect (type $"(raw) (integer -> (integer -> integer))")
        ref.cast (ref $"(integer -> integer)")
        local.set 26
        i64.const 1
        struct.new $integer
        local.get 26
        struct.get $"(integer -> integer)" 1
        local.get 26
        struct.get $"(integer -> integer)" 0
        call_indirect (type $"(raw) (integer -> integer)")
        ref.cast (ref $integer)
        local.get 24
        struct.get $"(integer -> (list:t -> Some of '8 | None of unit))" 1
        local.get 24
        struct.get $"(integer -> (list:t -> Some of '8 | None of unit))" 0
        call_indirect (type $"(raw) (integer -> (list:t -> Some of '8 | None of unit))")
        ref.cast (ref $"(list:t -> Some of '8 | None of unit)")
        local.set 23
        local.get 22
        local.get 23
        struct.get $"(list:t -> Some of '8 | None of unit)" 1
        local.get 23
        struct.get $"(list:t -> Some of '8 | None of unit)" 0
        call_indirect (type $"(raw) (list:t -> Some of '8 | None of unit)")
        ref.cast (ref $"Some of '8 | None of unit")
        br 1 (;@1;)
      end
      block ;; label = @2
        local.get 4
        struct.get $"(integer * Pair of ('8 * list:t) | Nil of unit)" 0
        local.set 27
        local.get 27
        local.set 28
        local.get 4
        struct.get $"(integer * Pair of ('8 * list:t) | Nil of unit)" 1
        local.set 29
        i32.const 1
        local.get 29
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 0
        i32.eq
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 29
        struct.get $"Pair of ('0 * list:t) | Nil of unit" 1
        ref.cast (ref $unit)
        local.set 30
        local.get 30
        global.get 15
        ref.as_non_null
        local.tee 31
        struct.get $"('0 -> ('0 -> boolean))" 1
        local.get 31
        struct.get $"('0 -> ('0 -> boolean))" 0
        call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
        ref.cast (ref $"('0 -> boolean)")
        local.set 32
        struct.new $unit
        local.get 32
        struct.get $"('0 -> boolean)" 1
        local.get 32
        struct.get $"('0 -> boolean)" 0
        call_indirect (type $"(raw) ('0 -> boolean)")
        ref.cast (ref $boolean)
        struct.get $boolean 0
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        global.get 32
        ref.as_non_null
        local.set 33
        struct.new $unit
        local.get 33
        struct.get $"(unit -> Some of '12 | None of unit)" 1
        local.get 33
        struct.get $"(unit -> Some of '12 | None of unit)" 0
        call_indirect (type $"(raw) (unit -> Some of '12 | None of unit)")
        ref.cast (ref $"Some of '12 | None of unit")
        br 1 (;@1;)
      end
      unreachable
    end
  )
  (func (;80;) (type $"(raw) (integer -> (integer -> unit))") (param anyref (ref $capture)) (result anyref)
    (local $to#1 (ref $integer))
    local.get 0
    ref.cast (ref $integer)
    local.set $to#1
    i32.const 81
    local.get $to#1
    ref.cast (ref any)
    local.get $to#1
    ref.cast (ref any)
    local.get $to#1
    ref.cast (ref any)
    array.new_fixed $capture 3
    struct.new $"(integer -> unit)"
  )
  (func (;81;) (type $"(raw) (integer -> unit)") (param anyref (ref $capture)) (result anyref)
    (local $num#3 (ref $integer)) (local (ref $integer) (ref $integer) (ref $integer) (ref $"(integer -> string)") (ref $"(integer -> string)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"(string -> unit)") (ref $"(integer -> string)") (ref $"(integer -> unit)") (ref $"(integer -> (integer -> unit))") (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)"))
    local.get 0
    ref.cast (ref $integer)
    local.set $num#3
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 3
    local.get 1
    i32.const 1
    array.get $capture
    ref.cast (ref $integer)
    local.set 4
    local.get 1
    i32.const 2
    array.get $capture
    ref.cast (ref $integer)
    local.set 5
    i32.const 82
    array.new_fixed $capture 0
    struct.new $"(integer -> string)"
    local.set 6
    local.get 6
    local.set 7
    local.get 5
    global.get 17
    ref.as_non_null
    local.tee 8
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 8
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    ref.cast (ref $"('0 -> boolean)")
    local.set 9
    local.get $num#3
    local.get 9
    struct.get $"('0 -> boolean)" 1
    local.get 9
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    ref.cast (ref $boolean)
    struct.get $boolean 0
    if (result (ref $unit)) ;; label = @1
      global.get 27
      ref.as_non_null
      local.set 10
      local.get 7
      local.set 11
      local.get 5
      local.get 11
      struct.get $"(integer -> string)" 1
      local.get 11
      struct.get $"(integer -> string)" 0
      call_indirect (type $"(raw) (integer -> string)")
      ref.cast (ref $string)
      local.get 10
      struct.get $"(string -> unit)" 1
      local.get 10
      struct.get $"(string -> unit)" 0
      call_indirect (type $"(raw) (string -> unit)")
      ref.cast (ref $unit)
      drop
      global.get 47
      ref.as_non_null
      local.set 13
      local.get 5
      global.get 3
      ref.as_non_null
      local.tee 14
      struct.get $"(integer -> (integer -> integer))" 1
      local.get 14
      struct.get $"(integer -> (integer -> integer))" 0
      call_indirect (type $"(raw) (integer -> (integer -> integer))")
      ref.cast (ref $"(integer -> integer)")
      local.set 15
      i64.const 1
      struct.new $integer
      local.get 15
      struct.get $"(integer -> integer)" 1
      local.get 15
      struct.get $"(integer -> integer)" 0
      call_indirect (type $"(raw) (integer -> integer)")
      ref.cast (ref $integer)
      local.get 13
      struct.get $"(integer -> (integer -> unit))" 1
      local.get 13
      struct.get $"(integer -> (integer -> unit))" 0
      call_indirect (type $"(raw) (integer -> (integer -> unit))")
      ref.cast (ref $"(integer -> unit)")
      local.set 12
      local.get $num#3
      local.get 12
      struct.get $"(integer -> unit)" 1
      local.get 12
      struct.get $"(integer -> unit)" 0
      call_indirect (type $"(raw) (integer -> unit)")
      ref.cast (ref $unit)
    else
      struct.new $unit
    end
  )
  (func (;82;) (type $"(raw) (integer -> string)") (param anyref (ref $capture)) (result anyref)
    (local (ref $integer) (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)") (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)") (ref $"(integer * integer)") (ref $integer) (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $integer) (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $integer) (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $integer) (ref $integer) (ref $integer) (ref $integer) (ref $integer) (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $integer) (ref $integer) (ref $integer) (ref $integer))
    local.get 0
    ref.cast (ref $integer)
    local.set 2
    local.get 2
    global.get 7
    ref.as_non_null
    local.tee 3
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 3
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    ref.cast (ref $"(integer -> integer)")
    local.set 4
    i64.const 3
    struct.new $integer
    local.get 4
    struct.get $"(integer -> integer)" 1
    local.get 4
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    ref.cast (ref $integer)
    local.get 2
    global.get 7
    ref.as_non_null
    local.tee 5
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 5
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    ref.cast (ref $"(integer -> integer)")
    local.set 6
    i64.const 5
    struct.new $integer
    local.get 6
    struct.get $"(integer -> integer)" 1
    local.get 6
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    ref.cast (ref $integer)
    struct.new $"(integer * integer)"
    local.set 7
    block (result (ref $string)) ;; label = @1
      block ;; label = @2
        local.get 7
        struct.get $"(integer * integer)" 0
        local.set 8
        local.get 8
        global.get 15
        ref.as_non_null
        local.tee 9
        struct.get $"('0 -> ('0 -> boolean))" 1
        local.get 9
        struct.get $"('0 -> ('0 -> boolean))" 0
        call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
        ref.cast (ref $"('0 -> boolean)")
        local.set 10
        i64.const 0
        struct.new $integer
        local.get 10
        struct.get $"('0 -> boolean)" 1
        local.get 10
        struct.get $"('0 -> boolean)" 0
        call_indirect (type $"(raw) ('0 -> boolean)")
        ref.cast (ref $boolean)
        struct.get $boolean 0
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 7
        struct.get $"(integer * integer)" 1
        local.set 11
        local.get 11
        global.get 15
        ref.as_non_null
        local.tee 12
        struct.get $"('0 -> ('0 -> boolean))" 1
        local.get 12
        struct.get $"('0 -> ('0 -> boolean))" 0
        call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
        ref.cast (ref $"('0 -> boolean)")
        local.set 13
        i64.const 0
        struct.new $integer
        local.get 13
        struct.get $"('0 -> boolean)" 1
        local.get 13
        struct.get $"('0 -> boolean)" 0
        call_indirect (type $"(raw) ('0 -> boolean)")
        ref.cast (ref $boolean)
        struct.get $boolean 0
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        i32.const 70
        i32.const 105
        i32.const 122
        i32.const 122
        i32.const 66
        i32.const 117
        i32.const 122
        i32.const 122
        array.new_fixed $string 8
        br 1 (;@1;)
      end
      block ;; label = @2
        local.get 7
        struct.get $"(integer * integer)" 0
        local.set 14
        local.get 14
        global.get 15
        ref.as_non_null
        local.tee 15
        struct.get $"('0 -> ('0 -> boolean))" 1
        local.get 15
        struct.get $"('0 -> ('0 -> boolean))" 0
        call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
        ref.cast (ref $"('0 -> boolean)")
        local.set 16
        i64.const 0
        struct.new $integer
        local.get 16
        struct.get $"('0 -> boolean)" 1
        local.get 16
        struct.get $"('0 -> boolean)" 0
        call_indirect (type $"(raw) ('0 -> boolean)")
        ref.cast (ref $boolean)
        struct.get $boolean 0
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        local.get 7
        struct.get $"(integer * integer)" 1
        local.set 17
        local.get 17
        local.set 18
        i32.const 70
        i32.const 105
        i32.const 122
        i32.const 122
        array.new_fixed $string 4
        br 1 (;@1;)
      end
      block ;; label = @2
        local.get 7
        struct.get $"(integer * integer)" 0
        local.set 19
        local.get 19
        local.set 20
        local.get 7
        struct.get $"(integer * integer)" 1
        local.set 21
        local.get 21
        global.get 15
        ref.as_non_null
        local.tee 22
        struct.get $"('0 -> ('0 -> boolean))" 1
        local.get 22
        struct.get $"('0 -> ('0 -> boolean))" 0
        call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
        ref.cast (ref $"('0 -> boolean)")
        local.set 23
        i64.const 0
        struct.new $integer
        local.get 23
        struct.get $"('0 -> boolean)" 1
        local.get 23
        struct.get $"('0 -> boolean)" 0
        call_indirect (type $"(raw) ('0 -> boolean)")
        ref.cast (ref $boolean)
        struct.get $boolean 0
        i32.const 1
        i32.xor
        br_if 0 (;@2;)
        i32.const 66
        i32.const 117
        i32.const 122
        i32.const 122
        array.new_fixed $string 4
        br 1 (;@1;)
      end
      block ;; label = @2
        local.get 7
        struct.get $"(integer * integer)" 0
        local.set 24
        local.get 24
        local.set 25
        local.get 7
        struct.get $"(integer * integer)" 1
        local.set 26
        local.get 26
        local.set 27
        i32.const 45
        i32.const 45
        i32.const 45
        array.new_fixed $string 3
        br 1 (;@1;)
      end
      unreachable
    end
  )
)
