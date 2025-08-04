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
  (type $"(unit -> '4)" (;35;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> '4)" (;36;) (func (param (ref $unit) (ref $capture)) (result anyref)))
  (type $"(unit -> '2)" (;37;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> '2)" (;38;) (func (param (ref $unit) (ref $capture)) (result anyref)))
  (type $"(boolean -> unit)" (;39;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (boolean -> unit)" (;40;) (func (param (ref $boolean) (ref $capture)) (result (ref $unit))))
  (type $"(unit -> unit)" (;41;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> unit)" (;42;) (func (param (ref $unit) (ref $capture)) (result (ref $unit))))
  (type $"(integer -> (integer -> (integer -> integer)))" (;43;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (integer -> (integer -> (integer -> integer)))" (;44;) (func (param (ref $integer) (ref $capture)) (result (ref $"(integer -> (integer -> integer))"))))
  (type $"('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))))))" (;45;) (struct (field i32) (field (ref $capture))))
  (type $"('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))))))" (;46;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))))))" (;47;) (func (param anyref (ref $capture)) (result (ref $"('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))))))"))))
  (type $"('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))))" (;48;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))))))" (;49;) (func (param anyref (ref $capture)) (result (ref $"('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))))"))))
  (type $"('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))))" (;50;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))))" (;51;) (func (param anyref (ref $capture)) (result (ref $"('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))))"))))
  (type $"('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))" (;52;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))))" (;53;) (func (param anyref (ref $capture)) (result (ref $"('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))"))))
  (type $"('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))" (;54;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))" (;55;) (func (param anyref (ref $capture)) (result (ref $"('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))"))))
  (type $"('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))" (;56;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))" (;57;) (func (param anyref (ref $capture)) (result (ref $"('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))"))))
  (type $"('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))" (;58;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))" (;59;) (func (param anyref (ref $capture)) (result (ref $"('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))"))))
  (type $"('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))" (;60;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))" (;61;) (func (param anyref (ref $capture)) (result (ref $"('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))"))))
  (type $"('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))" (;62;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))" (;63;) (func (param anyref (ref $capture)) (result (ref $"('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))"))))
  (type $"('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))" (;64;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))" (;65;) (func (param anyref (ref $capture)) (result (ref $"('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))"))))
  (type $"('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))" (;66;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))" (;67;) (func (param anyref (ref $capture)) (result (ref $"('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))"))))
  (type $"('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))" (;68;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))" (;69;) (func (param anyref (ref $capture)) (result (ref $"('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))"))))
  (type $"('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))" (;70;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))" (;71;) (func (param anyref (ref $capture)) (result (ref $"('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))"))))
  (type $"('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))" (;72;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))" (;73;) (func (param anyref (ref $capture)) (result (ref $"('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))"))))
  (type $"('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))" (;74;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))" (;75;) (func (param anyref (ref $capture)) (result (ref $"('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))"))))
  (type $"('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))" (;76;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))" (;77;) (func (param anyref (ref $capture)) (result (ref $"('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))"))))
  (type $"('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))" (;78;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))" (;79;) (func (param anyref (ref $capture)) (result (ref $"('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))"))))
  (type $"('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))" (;80;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))" (;81;) (func (param anyref (ref $capture)) (result (ref $"('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))"))))
  (type $"('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))" (;82;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))" (;83;) (func (param anyref (ref $capture)) (result (ref $"('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))"))))
  (type $"('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))" (;84;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))" (;85;) (func (param anyref (ref $capture)) (result (ref $"('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))"))))
  (type $"('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))" (;86;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))" (;87;) (func (param anyref (ref $capture)) (result (ref $"('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))"))))
  (type $"('23 -> ('24 -> ('25 -> ('26 -> unit))))" (;88;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))" (;89;) (func (param anyref (ref $capture)) (result (ref $"('23 -> ('24 -> ('25 -> ('26 -> unit))))"))))
  (type $"('24 -> ('25 -> ('26 -> unit)))" (;90;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('23 -> ('24 -> ('25 -> ('26 -> unit))))" (;91;) (func (param anyref (ref $capture)) (result (ref $"('24 -> ('25 -> ('26 -> unit)))"))))
  (type $"('25 -> ('26 -> unit))" (;92;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('24 -> ('25 -> ('26 -> unit)))" (;93;) (func (param anyref (ref $capture)) (result (ref $"('25 -> ('26 -> unit))"))))
  (type $"('26 -> unit)" (;94;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('25 -> ('26 -> unit))" (;95;) (func (param anyref (ref $capture)) (result (ref $"('26 -> unit)"))))
  (type $"(raw) ('26 -> unit)" (;96;) (func (param anyref (ref $capture)) (result (ref $unit))))
  (import "sys" "print_string" (func (;0;) (type 30)))
  (import "sys" "memory" (memory (;0;) 1))
  (table (;0;) 86 86 funcref)
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
  (global (;25;) (mut (ref null $"(unit -> '4)")) ref.null $"(unit -> '4)")
  (global (;26;) (mut (ref null $"(boolean -> unit)")) ref.null $"(boolean -> unit)")
  (global (;27;) (mut (ref null $"(string -> integer)")) ref.null $"(string -> integer)")
  (global (;28;) (mut (ref null $"(string -> unit)")) ref.null $"(string -> unit)")
  (global (;29;) (mut (ref null $"(string -> (string -> string))")) ref.null $"(string -> (string -> string))")
  (global (;30;) (mut (ref null $"('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))))))")) ref.null $"('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))))))")
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
  (export "FunctionTest:_#8" (global 30))
  (start 1)
  (elem (;0;) (i32.const 0) func 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 0 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63 64 65 66 67 68 69 70 71 72 73 74 75 76 77 78 79 80 81 82 83 84 85)
  (func (;1;) (type 0)
    (local $0 (ref $"(unit -> '4)")) (local (ref $"(boolean -> unit)") (ref $"(string -> integer)") (ref $"(string -> unit)") (ref $"(string -> (string -> string))") (ref $"(boolean -> unit)") (ref $"(unit -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"(boolean -> unit)") (ref $"(integer -> integer)") (ref $"(integer -> (integer -> integer))") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"(boolean -> unit)") (ref $"(integer -> integer)") (ref $"(integer -> (integer -> integer))") (ref $"(integer -> (integer -> (integer -> integer)))") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"(boolean -> unit)") (ref $"(integer -> integer)") (ref $integer) (ref $integer) (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))))))") (ref $"(boolean -> unit)") (ref $"(unit -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $unit) (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)"))
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
    struct.new $"(unit -> '4)"
    local.set $0
    local.get $0
    global.set 25
    i32.const 1
    drop
    i32.const 47
    array.new_fixed $capture 0
    struct.new $"(boolean -> unit)"
    local.set 1
    local.get 1
    global.set 26
    i32.const 1
    drop
    i32.const 48
    array.new_fixed $capture 0
    struct.new $"(string -> integer)"
    local.set 2
    local.get 2
    global.set 27
    i32.const 1
    drop
    i32.const 49
    array.new_fixed $capture 0
    struct.new $"(string -> unit)"
    local.set 3
    local.get 3
    global.set 28
    i32.const 1
    drop
    i32.const 50
    array.new_fixed $capture 0
    struct.new $"(string -> (string -> string))"
    local.set 4
    local.get 4
    global.set 29
    i32.const 1
    drop
    global.get 26
    ref.as_non_null
    local.set 5
    i32.const 52
    array.new_fixed $capture 0
    struct.new $"(unit -> unit)"
    local.set 6
    struct.new $unit
    local.get 6
    struct.get $"(unit -> unit)" 1
    local.get 6
    struct.get $"(unit -> unit)" 0
    call_indirect (type $"(raw) (unit -> unit)")
    ref.cast (ref $unit)
    global.get 15
    ref.as_non_null
    local.tee 7
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 7
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 8
    struct.new $unit
    local.get 8
    struct.get $"('0 -> boolean)" 1
    local.get 8
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 5
    struct.get $"(boolean -> unit)" 1
    local.get 5
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 9
    local.get 9
    global.get 15
    ref.as_non_null
    local.tee 10
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 10
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 11
    struct.new $unit
    local.get 11
    struct.get $"('0 -> boolean)" 1
    local.get 11
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    struct.get $boolean 0
    drop
    global.get 26
    ref.as_non_null
    local.set 12
    i32.const 53
    array.new_fixed $capture 0
    struct.new $"(integer -> (integer -> integer))"
    local.set 14
    i64.const 1
    struct.new $integer
    local.get 14
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 14
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    ref.cast (ref $"(integer -> integer)")
    local.set 13
    i64.const 2
    struct.new $integer
    local.get 13
    struct.get $"(integer -> integer)" 1
    local.get 13
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    ref.cast (ref $integer)
    global.get 15
    ref.as_non_null
    local.tee 15
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 15
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 16
    i64.const 3
    struct.new $integer
    local.get 16
    struct.get $"('0 -> boolean)" 1
    local.get 16
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 12
    struct.get $"(boolean -> unit)" 1
    local.get 12
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 17
    local.get 17
    global.get 15
    ref.as_non_null
    local.tee 18
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 18
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 19
    struct.new $unit
    local.get 19
    struct.get $"('0 -> boolean)" 1
    local.get 19
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    struct.get $boolean 0
    drop
    global.get 26
    ref.as_non_null
    local.set 20
    i32.const 55
    array.new_fixed $capture 0
    struct.new $"(integer -> (integer -> (integer -> integer)))"
    local.set 23
    i64.const 1
    struct.new $integer
    local.get 23
    struct.get $"(integer -> (integer -> (integer -> integer)))" 1
    local.get 23
    struct.get $"(integer -> (integer -> (integer -> integer)))" 0
    call_indirect (type $"(raw) (integer -> (integer -> (integer -> integer)))")
    ref.cast (ref $"(integer -> (integer -> integer))")
    local.set 22
    i64.const 2
    struct.new $integer
    local.get 22
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 22
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    ref.cast (ref $"(integer -> integer)")
    local.set 21
    i64.const 3
    struct.new $integer
    local.get 21
    struct.get $"(integer -> integer)" 1
    local.get 21
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    ref.cast (ref $integer)
    global.get 15
    ref.as_non_null
    local.tee 24
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 24
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 25
    i64.const 6
    struct.new $integer
    local.get 25
    struct.get $"('0 -> boolean)" 1
    local.get 25
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 20
    struct.get $"(boolean -> unit)" 1
    local.get 20
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 26
    local.get 26
    global.get 15
    ref.as_non_null
    local.tee 27
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 27
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 28
    struct.new $unit
    local.get 28
    struct.get $"('0 -> boolean)" 1
    local.get 28
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    struct.get $boolean 0
    drop
    global.get 26
    ref.as_non_null
    local.set 29
    i64.const 1
    struct.new $integer
    local.set 31
    local.get 31
    local.set 32
    i32.const 1
    drop
    i32.const 58
    local.get 32
    ref.cast (ref any)
    array.new_fixed $capture 1
    struct.new $"(integer -> integer)"
    local.set 30
    i64.const 2
    struct.new $integer
    local.get 30
    struct.get $"(integer -> integer)" 1
    local.get 30
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    ref.cast (ref $integer)
    global.get 15
    ref.as_non_null
    local.tee 33
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 33
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 34
    i64.const 3
    struct.new $integer
    local.get 34
    struct.get $"('0 -> boolean)" 1
    local.get 34
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 29
    struct.get $"(boolean -> unit)" 1
    local.get 29
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 35
    local.get 35
    global.get 15
    ref.as_non_null
    local.tee 36
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 36
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 37
    struct.new $unit
    local.get 37
    struct.get $"('0 -> boolean)" 1
    local.get 37
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    struct.get $boolean 0
    drop
    i32.const 59
    array.new_fixed $capture 0
    struct.new $"('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))))))"
    local.set 38
    local.get 38
    global.set 30
    i32.const 1
    drop
    global.get 26
    ref.as_non_null
    local.set 39
    i32.const 85
    array.new_fixed $capture 0
    struct.new $"(unit -> unit)"
    local.set 40
    struct.new $unit
    local.get 40
    struct.get $"(unit -> unit)" 1
    local.get 40
    struct.get $"(unit -> unit)" 0
    call_indirect (type $"(raw) (unit -> unit)")
    ref.cast (ref $unit)
    global.get 15
    ref.as_non_null
    local.tee 41
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 41
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 42
    struct.new $unit
    local.get 42
    struct.get $"('0 -> boolean)" 1
    local.get 42
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 39
    struct.get $"(boolean -> unit)" 1
    local.get 39
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    local.set 43
    local.get 43
    global.get 15
    ref.as_non_null
    local.tee 44
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 44
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 45
    struct.new $unit
    local.get 45
    struct.get $"('0 -> boolean)" 1
    local.get 45
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    struct.get $boolean 0
    drop
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
  (func (;46;) (type $"(raw) (unit -> '4)") (param $condition#0 (ref $unit)) (param (ref $capture)) (result anyref)
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
    (local (ref $"(unit -> '4)"))
    local.get $s#1
    ref.cast (ref $boolean)
    struct.get $boolean 0
    if (result (ref $unit)) ;; label = @1
      struct.new $unit
    else
      global.get 25
      ref.as_non_null
      ref.cast (ref $"(unit -> '4)")
      local.set 2
      struct.new $unit
      local.get 2
      struct.get $"(unit -> '4)" 1
      local.get 2
      struct.get $"(unit -> '4)" 0
      call_indirect (type $"(raw) (unit -> '4)")
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
  (func (;51;) (type $"(raw) (string -> string)") (param $a#0 (ref $string)) (param (ref $capture)) (result (ref $string))
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
    local.get $a#0
    ref.cast (ref $string)
    local.get 3
    struct.get $"(string -> string)" 1
    local.get 3
    struct.get $"(string -> string)" 0
    call_indirect (type $"(raw) (string -> string)")
    ref.cast (ref $string)
  )
  (func (;52;) (type $"(raw) (unit -> unit)") (param $a#1 (ref $unit)) (param (ref $capture)) (result (ref $unit))
    local.get $a#1
    ref.cast (ref $unit)
  )
  (func (;53;) (type $"(raw) (integer -> (integer -> integer))") (param $b#2 (ref $integer)) (param (ref $capture)) (result (ref $"(integer -> integer)"))
    i32.const 54
    local.get $b#2
    ref.cast (ref any)
    array.new_fixed $capture 1
    struct.new $"(integer -> integer)"
  )
  (func (;54;) (type $"(raw) (integer -> integer)") (param $a#3 (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local (ref $integer) (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)"))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 2
    local.get 2
    ref.cast (ref $integer)
    global.get 3
    ref.as_non_null
    local.tee 3
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 3
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    local.set 4
    local.get $a#3
    ref.cast (ref $integer)
    local.get 4
    struct.get $"(integer -> integer)" 1
    local.get 4
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
  )
  (func (;55;) (type $"(raw) (integer -> (integer -> (integer -> integer)))") (param $b#4 (ref $integer)) (param (ref $capture)) (result (ref $"(integer -> (integer -> integer))"))
    i32.const 56
    local.get $b#4
    ref.cast (ref any)
    array.new_fixed $capture 1
    struct.new $"(integer -> (integer -> integer))"
  )
  (func (;56;) (type $"(raw) (integer -> (integer -> integer))") (param $c#5 (ref $integer)) (param (ref $capture)) (result (ref $"(integer -> integer)"))
    (local $a#3 (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set $a#3
    i32.const 57
    local.get $a#3
    ref.cast (ref any)
    local.get $c#5
    ref.cast (ref any)
    array.new_fixed $capture 2
    struct.new $"(integer -> integer)"
  )
  (func (;57;) (type $"(raw) (integer -> integer)") (param $b#7 (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local $a#6 (ref $integer)) (local (ref $integer) (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)") (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)"))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set $a#6
    local.get 1
    i32.const 1
    array.get $capture
    ref.cast (ref $integer)
    local.set 3
    local.get $a#6
    ref.cast (ref $integer)
    global.get 3
    ref.as_non_null
    local.tee 4
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 4
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    local.set 5
    local.get 3
    ref.cast (ref $integer)
    local.get 5
    struct.get $"(integer -> integer)" 1
    local.get 5
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    global.get 3
    ref.as_non_null
    local.tee 6
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 6
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    local.set 7
    local.get $b#7
    ref.cast (ref $integer)
    local.get 7
    struct.get $"(integer -> integer)" 1
    local.get 7
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
  )
  (func (;58;) (type $"(raw) (integer -> integer)") (param $a#9 (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local (ref $integer) (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)"))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 2
    local.get 2
    ref.cast (ref $integer)
    global.get 3
    ref.as_non_null
    local.tee 3
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 3
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    local.set 4
    local.get $a#9
    ref.cast (ref $integer)
    local.get 4
    struct.get $"(integer -> integer)" 1
    local.get 4
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
  )
  (func (;59;) (type $"(raw) ('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))))))") (param $b#10 anyref) (param (ref $capture)) (result (ref $"('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))))))"))
    i32.const 60
    array.new_fixed $capture 0
    struct.new $"('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))))))"
  )
  (func (;60;) (type $"(raw) ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))))))") (param $c#11 anyref) (param (ref $capture)) (result (ref $"('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))))"))
    i32.const 61
    array.new_fixed $capture 0
    struct.new $"('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))))"
  )
  (func (;61;) (type $"(raw) ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))))") (param $d#12 anyref) (param (ref $capture)) (result (ref $"('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))))"))
    i32.const 62
    array.new_fixed $capture 0
    struct.new $"('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))))"
  )
  (func (;62;) (type $"(raw) ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))))") (param $e#13 anyref) (param (ref $capture)) (result (ref $"('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))"))
    i32.const 63
    array.new_fixed $capture 0
    struct.new $"('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))"
  )
  (func (;63;) (type $"(raw) ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))))") (param $f#14 anyref) (param (ref $capture)) (result (ref $"('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))"))
    i32.const 64
    array.new_fixed $capture 0
    struct.new $"('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))"
  )
  (func (;64;) (type $"(raw) ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))))") (param $g#15 anyref) (param (ref $capture)) (result (ref $"('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))"))
    i32.const 65
    array.new_fixed $capture 0
    struct.new $"('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))"
  )
  (func (;65;) (type $"(raw) ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))))") (param $h#16 anyref) (param (ref $capture)) (result (ref $"('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))"))
    i32.const 66
    array.new_fixed $capture 0
    struct.new $"('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))"
  )
  (func (;66;) (type $"(raw) ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))))") (param $i#17 anyref) (param (ref $capture)) (result (ref $"('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))"))
    i32.const 67
    array.new_fixed $capture 0
    struct.new $"('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))"
  )
  (func (;67;) (type $"(raw) ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))))") (param $j#18 anyref) (param (ref $capture)) (result (ref $"('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))"))
    i32.const 68
    array.new_fixed $capture 0
    struct.new $"('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))"
  )
  (func (;68;) (type $"(raw) ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))))") (param $k#19 anyref) (param (ref $capture)) (result (ref $"('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))"))
    i32.const 69
    array.new_fixed $capture 0
    struct.new $"('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))"
  )
  (func (;69;) (type $"(raw) ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))))") (param $l#20 anyref) (param (ref $capture)) (result (ref $"('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))"))
    i32.const 70
    array.new_fixed $capture 0
    struct.new $"('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))"
  )
  (func (;70;) (type $"(raw) ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))))") (param $m#21 anyref) (param (ref $capture)) (result (ref $"('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))"))
    i32.const 71
    array.new_fixed $capture 0
    struct.new $"('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))"
  )
  (func (;71;) (type $"(raw) ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))))") (param $n#22 anyref) (param (ref $capture)) (result (ref $"('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))"))
    i32.const 72
    array.new_fixed $capture 0
    struct.new $"('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))"
  )
  (func (;72;) (type $"(raw) ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))))") (param $o#23 anyref) (param (ref $capture)) (result (ref $"('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))"))
    i32.const 73
    array.new_fixed $capture 0
    struct.new $"('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))"
  )
  (func (;73;) (type $"(raw) ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))))") (param $p#24 anyref) (param (ref $capture)) (result (ref $"('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))"))
    i32.const 74
    array.new_fixed $capture 0
    struct.new $"('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))"
  )
  (func (;74;) (type $"(raw) ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))))") (param $q#25 anyref) (param (ref $capture)) (result (ref $"('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))"))
    i32.const 75
    array.new_fixed $capture 0
    struct.new $"('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))"
  )
  (func (;75;) (type $"(raw) ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))))") (param $r#26 anyref) (param (ref $capture)) (result (ref $"('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))"))
    i32.const 76
    array.new_fixed $capture 0
    struct.new $"('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))"
  )
  (func (;76;) (type $"(raw) ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))))") (param $s#27 anyref) (param (ref $capture)) (result (ref $"('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))"))
    i32.const 77
    array.new_fixed $capture 0
    struct.new $"('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))"
  )
  (func (;77;) (type $"(raw) ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))))") (param $t#28 anyref) (param (ref $capture)) (result (ref $"('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))"))
    i32.const 78
    array.new_fixed $capture 0
    struct.new $"('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))"
  )
  (func (;78;) (type $"(raw) ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))))") (param $u#29 anyref) (param (ref $capture)) (result (ref $"('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))"))
    i32.const 79
    array.new_fixed $capture 0
    struct.new $"('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))"
  )
  (func (;79;) (type $"(raw) ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit))))))") (param $v#30 anyref) (param (ref $capture)) (result (ref $"('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))"))
    i32.const 80
    array.new_fixed $capture 0
    struct.new $"('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))"
  )
  (func (;80;) (type $"(raw) ('22 -> ('23 -> ('24 -> ('25 -> ('26 -> unit)))))") (param $w#31 anyref) (param (ref $capture)) (result (ref $"('23 -> ('24 -> ('25 -> ('26 -> unit))))"))
    i32.const 81
    array.new_fixed $capture 0
    struct.new $"('23 -> ('24 -> ('25 -> ('26 -> unit))))"
  )
  (func (;81;) (type $"(raw) ('23 -> ('24 -> ('25 -> ('26 -> unit))))") (param $x#32 anyref) (param (ref $capture)) (result (ref $"('24 -> ('25 -> ('26 -> unit)))"))
    i32.const 82
    array.new_fixed $capture 0
    struct.new $"('24 -> ('25 -> ('26 -> unit)))"
  )
  (func (;82;) (type $"(raw) ('24 -> ('25 -> ('26 -> unit)))") (param $y#33 anyref) (param (ref $capture)) (result (ref $"('25 -> ('26 -> unit))"))
    i32.const 83
    array.new_fixed $capture 0
    struct.new $"('25 -> ('26 -> unit))"
  )
  (func (;83;) (type $"(raw) ('25 -> ('26 -> unit))") (param $z#34 anyref) (param (ref $capture)) (result (ref $"('26 -> unit)"))
    i32.const 84
    array.new_fixed $capture 0
    struct.new $"('26 -> unit)"
  )
  (func (;84;) (type $"(raw) ('26 -> unit)") (param $unit anyref) (param (ref $capture)) (result (ref $unit))
    struct.new $unit
  )
  (func (;85;) (type $"(raw) (unit -> unit)") (param (ref $unit) (ref $capture)) (result (ref $unit))
    struct.new $unit
  )
)
