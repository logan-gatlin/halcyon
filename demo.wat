(module
  (type (;0;) (func))
  (type $integer (;1;) (struct (field i64)))
  (type $capture (;2;) (array (mut anyref)))
  (type $"(raw) (integer -> integer)" (;3;) (func (param (ref $integer) (ref $capture)) (result (ref $integer))))
  (type $"(integer -> integer)" (;4;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (integer -> (integer -> integer))" (;5;) (func (param (ref $integer) (ref $capture)) (result (ref $"(integer -> integer)"))))
  (type $"(integer -> (integer -> integer))" (;6;) (struct (field i32) (field (ref $capture))))
  (type $real (;7;) (struct (field f64)))
  (type $"(raw) (real -> real)" (;8;) (func (param (ref $real) (ref $capture)) (result (ref $real))))
  (type $"(real -> real)" (;9;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (real -> (real -> real))" (;10;) (func (param (ref $real) (ref $capture)) (result (ref $"(real -> real)"))))
  (type $"(real -> (real -> real))" (;11;) (struct (field i32) (field (ref $capture))))
  (type $boolean (;12;) (struct (field i32)))
  (type $"(raw) (boolean -> boolean)" (;13;) (func (param (ref $boolean) (ref $capture)) (result (ref $boolean))))
  (type $"(boolean -> boolean)" (;14;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (boolean -> (boolean -> boolean))" (;15;) (func (param (ref $boolean) (ref $capture)) (result (ref $"(boolean -> boolean)"))))
  (type $"(boolean -> (boolean -> boolean))" (;16;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('0 -> boolean)" (;17;) (func (param anyref (ref $capture)) (result (ref $boolean))))
  (type $"('0 -> boolean)" (;18;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('0 -> ('0 -> boolean))" (;19;) (func (param anyref (ref $capture)) (result (ref $"('0 -> boolean)"))))
  (type $"('0 -> ('0 -> boolean))" (;20;) (struct (field i32) (field (ref $capture))))
  (type $glyph (;21;) (struct (field i32)))
  (type $unit (;22;) (struct))
  (type $string (;23;) (array (mut i8)))
  (type $"(raw) (unit -> '0)" (;24;) (func (param (ref $unit) (ref $capture)) (result anyref)))
  (type $"(unit -> '0)" (;25;) (struct (field i32) (field (ref $capture))))
  (type $"(boolean -> unit)" (;26;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (boolean -> unit)" (;27;) (func (param (ref $boolean) (ref $capture)) (result (ref $unit))))
  (type $"(unit -> '3)" (;28;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> '3)" (;29;) (func (param (ref $unit) (ref $capture)) (result anyref)))
  (type $"(unit -> unit)" (;30;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (unit -> unit)" (;31;) (func (param (ref $unit) (ref $capture)) (result (ref $unit))))
  (type $"(integer -> (integer -> (integer -> integer)))" (;32;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) (integer -> (integer -> (integer -> integer)))" (;33;) (func (param (ref $integer) (ref $capture)) (result (ref $"(integer -> (integer -> integer))"))))
  (type $"('0 -> ('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))))))" (;34;) (struct (field i32) (field (ref $capture))))
  (type $"('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))))))" (;35;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('0 -> ('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))))))" (;36;) (func (param anyref (ref $capture)) (result (ref $"('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))))))"))))
  (type $"('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))))" (;37;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))))))" (;38;) (func (param anyref (ref $capture)) (result (ref $"('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))))"))))
  (type $"('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))))" (;39;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))))" (;40;) (func (param anyref (ref $capture)) (result (ref $"('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))))"))))
  (type $"('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))" (;41;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))))" (;42;) (func (param anyref (ref $capture)) (result (ref $"('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))"))))
  (type $"('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))" (;43;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))" (;44;) (func (param anyref (ref $capture)) (result (ref $"('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))"))))
  (type $"('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))" (;45;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))" (;46;) (func (param anyref (ref $capture)) (result (ref $"('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))"))))
  (type $"('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))" (;47;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))" (;48;) (func (param anyref (ref $capture)) (result (ref $"('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))"))))
  (type $"('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))" (;49;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))" (;50;) (func (param anyref (ref $capture)) (result (ref $"('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))"))))
  (type $"('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))" (;51;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))" (;52;) (func (param anyref (ref $capture)) (result (ref $"('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))"))))
  (type $"('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))" (;53;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))" (;54;) (func (param anyref (ref $capture)) (result (ref $"('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))"))))
  (type $"('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))" (;55;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))" (;56;) (func (param anyref (ref $capture)) (result (ref $"('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))"))))
  (type $"('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))" (;57;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))" (;58;) (func (param anyref (ref $capture)) (result (ref $"('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))"))))
  (type $"('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))" (;59;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))" (;60;) (func (param anyref (ref $capture)) (result (ref $"('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))"))))
  (type $"('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))" (;61;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))" (;62;) (func (param anyref (ref $capture)) (result (ref $"('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))"))))
  (type $"('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))" (;63;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))" (;64;) (func (param anyref (ref $capture)) (result (ref $"('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))"))))
  (type $"('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))" (;65;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))" (;66;) (func (param anyref (ref $capture)) (result (ref $"('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))"))))
  (type $"('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))" (;67;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))" (;68;) (func (param anyref (ref $capture)) (result (ref $"('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))"))))
  (type $"('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))" (;69;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))" (;70;) (func (param anyref (ref $capture)) (result (ref $"('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))"))))
  (type $"('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))" (;71;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))" (;72;) (func (param anyref (ref $capture)) (result (ref $"('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))"))))
  (type $"('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))" (;73;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))" (;74;) (func (param anyref (ref $capture)) (result (ref $"('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))"))))
  (type $"('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))" (;75;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))" (;76;) (func (param anyref (ref $capture)) (result (ref $"('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))"))))
  (type $"('22 -> ('23 -> ('24 -> ('25 -> unit))))" (;77;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))" (;78;) (func (param anyref (ref $capture)) (result (ref $"('22 -> ('23 -> ('24 -> ('25 -> unit))))"))))
  (type $"('23 -> ('24 -> ('25 -> unit)))" (;79;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('22 -> ('23 -> ('24 -> ('25 -> unit))))" (;80;) (func (param anyref (ref $capture)) (result (ref $"('23 -> ('24 -> ('25 -> unit)))"))))
  (type $"('24 -> ('25 -> unit))" (;81;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('23 -> ('24 -> ('25 -> unit)))" (;82;) (func (param anyref (ref $capture)) (result (ref $"('24 -> ('25 -> unit))"))))
  (type $"('25 -> unit)" (;83;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) ('24 -> ('25 -> unit))" (;84;) (func (param anyref (ref $capture)) (result (ref $"('25 -> unit)"))))
  (type $"(raw) ('25 -> unit)" (;85;) (func (param anyref (ref $capture)) (result (ref $unit))))
  (table (;0;) 73 73 funcref)
  (global (;0;) (mut (ref null $"(integer -> (integer -> integer))")) ref.null $"(integer -> (integer -> integer))")
  (global (;1;) (mut (ref null $"(integer -> (integer -> integer))")) ref.null $"(integer -> (integer -> integer))")
  (global (;2;) (mut (ref null $"(integer -> (integer -> integer))")) ref.null $"(integer -> (integer -> integer))")
  (global (;3;) (mut (ref null $"(integer -> (integer -> integer))")) ref.null $"(integer -> (integer -> integer))")
  (global (;4;) (mut (ref null $"(integer -> (integer -> integer))")) ref.null $"(integer -> (integer -> integer))")
  (global (;5;) (mut (ref null $"(real -> (real -> real))")) ref.null $"(real -> (real -> real))")
  (global (;6;) (mut (ref null $"(real -> (real -> real))")) ref.null $"(real -> (real -> real))")
  (global (;7;) (mut (ref null $"(real -> (real -> real))")) ref.null $"(real -> (real -> real))")
  (global (;8;) (mut (ref null $"(real -> (real -> real))")) ref.null $"(real -> (real -> real))")
  (global (;9;) (mut (ref null $"(boolean -> (boolean -> boolean))")) ref.null $"(boolean -> (boolean -> boolean))")
  (global (;10;) (mut (ref null $"(boolean -> (boolean -> boolean))")) ref.null $"(boolean -> (boolean -> boolean))")
  (global (;11;) (mut (ref null $"(boolean -> (boolean -> boolean))")) ref.null $"(boolean -> (boolean -> boolean))")
  (global (;12;) (mut (ref null $"('0 -> ('0 -> boolean))")) ref.null $"('0 -> ('0 -> boolean))")
  (global (;13;) (mut (ref null $"('0 -> ('0 -> boolean))")) ref.null $"('0 -> ('0 -> boolean))")
  (global (;14;) (mut (ref null $"('0 -> ('0 -> boolean))")) ref.null $"('0 -> ('0 -> boolean))")
  (global (;15;) (mut (ref null $"('0 -> ('0 -> boolean))")) ref.null $"('0 -> ('0 -> boolean))")
  (global (;16;) (mut (ref null $"('0 -> ('0 -> boolean))")) ref.null $"('0 -> ('0 -> boolean))")
  (global (;17;) (mut (ref null $"('0 -> ('0 -> boolean))")) ref.null $"('0 -> ('0 -> boolean))")
  (global (;18;) (mut (ref null $"(unit -> '0)")) ref.null $"(unit -> '0)")
  (global (;19;) (mut (ref null $"(boolean -> unit)")) ref.null $"(boolean -> unit)")
  (global (;20;) (mut (ref null $unit)) ref.null $unit)
  (global (;21;) (mut (ref null $unit)) ref.null $unit)
  (global (;22;) (mut (ref null $unit)) ref.null $unit)
  (global (;23;) (mut (ref null $unit)) ref.null $unit)
  (global (;24;) (mut (ref null $"('0 -> ('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))))))")) ref.null $"('0 -> ('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))))))")
  (global (;25;) (mut (ref null $unit)) ref.null $unit)
  (export "builtin:BinaryOp+" (global 0))
  (export "builtin:BinaryOp-" (global 1))
  (export "builtin:BinaryOp*" (global 2))
  (export "builtin:BinaryOp/" (global 3))
  (export "builtin:BinaryOp%" (global 4))
  (export "builtin:BinaryOp+." (global 5))
  (export "builtin:BinaryOp-." (global 6))
  (export "builtin:BinaryOp*." (global 7))
  (export "builtin:BinaryOp/." (global 8))
  (export "builtin:BinaryOpand" (global 9))
  (export "builtin:BinaryOpor" (global 10))
  (export "builtin:BinaryOpxor" (global 11))
  (export "builtin:BinaryOp==" (global 12))
  (export "builtin:BinaryOp!=" (global 13))
  (export "builtin:BinaryOp<=" (global 14))
  (export "builtin:BinaryOp>=" (global 15))
  (export "builtin:BinaryOp<" (global 16))
  (export "builtin:BinaryOp>" (global 17))
  (export "builtin:panic" (global 18))
  (export "std:assert" (global 19))
  (export "2_-1" (global 20))
  (export "2_-4" (global 21))
  (export "2_-8" (global 22))
  (export "2_-11" (global 23))
  (export "2_-38" (global 24))
  (export "2_-39" (global 25))
  (start 0)
  (elem (;0;) (i32.const 0) func 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63 64 65 66 67 68 69 70 71 72)
  (func (;0;) (type 0)
    (local (ref $"(boolean -> unit)") (ref $"(unit -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"(boolean -> unit)") (ref $"(integer -> integer)") (ref $"(integer -> (integer -> integer))") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"(boolean -> unit)") (ref $"(integer -> integer)") (ref $"(integer -> (integer -> integer))") (ref $"(integer -> (integer -> (integer -> integer)))") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"(boolean -> unit)") (ref $"(integer -> integer)")) (local $13FunctionTest-2a-9 anyref) (local (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)") (ref $"(boolean -> unit)") (ref $"(unit -> unit)") (ref $"('0 -> ('0 -> boolean))") (ref $"('0 -> boolean)"))
    i32.const 2
    array.new_fixed $capture 0
    struct.new $"(integer -> (integer -> integer))"
    global.set 0
    i32.const 4
    array.new_fixed $capture 0
    struct.new $"(integer -> (integer -> integer))"
    global.set 1
    i32.const 6
    array.new_fixed $capture 0
    struct.new $"(integer -> (integer -> integer))"
    global.set 2
    i32.const 8
    array.new_fixed $capture 0
    struct.new $"(integer -> (integer -> integer))"
    global.set 3
    i32.const 10
    array.new_fixed $capture 0
    struct.new $"(integer -> (integer -> integer))"
    global.set 4
    i32.const 12
    array.new_fixed $capture 0
    struct.new $"(real -> (real -> real))"
    global.set 5
    i32.const 14
    array.new_fixed $capture 0
    struct.new $"(real -> (real -> real))"
    global.set 6
    i32.const 16
    array.new_fixed $capture 0
    struct.new $"(real -> (real -> real))"
    global.set 7
    i32.const 18
    array.new_fixed $capture 0
    struct.new $"(real -> (real -> real))"
    global.set 8
    i32.const 20
    array.new_fixed $capture 0
    struct.new $"(boolean -> (boolean -> boolean))"
    global.set 9
    i32.const 22
    array.new_fixed $capture 0
    struct.new $"(boolean -> (boolean -> boolean))"
    global.set 10
    i32.const 24
    array.new_fixed $capture 0
    struct.new $"(boolean -> (boolean -> boolean))"
    global.set 11
    i32.const 26
    array.new_fixed $capture 0
    struct.new $"('0 -> ('0 -> boolean))"
    global.set 12
    i32.const 28
    array.new_fixed $capture 0
    struct.new $"('0 -> ('0 -> boolean))"
    global.set 13
    i32.const 30
    array.new_fixed $capture 0
    struct.new $"('0 -> ('0 -> boolean))"
    global.set 14
    i32.const 32
    array.new_fixed $capture 0
    struct.new $"('0 -> ('0 -> boolean))"
    global.set 15
    i32.const 34
    array.new_fixed $capture 0
    struct.new $"('0 -> ('0 -> boolean))"
    global.set 16
    i32.const 36
    array.new_fixed $capture 0
    struct.new $"('0 -> ('0 -> boolean))"
    global.set 17
    i32.const 37
    array.new_fixed $capture 0
    struct.new $"(unit -> '0)"
    global.set 18
    i32.const 38
    array.new_fixed $capture 0
    struct.new $"(boolean -> unit)"
    global.set 19
    global.get 19
    ref.as_non_null
    local.set 0
    i32.const 39
    array.new_fixed $capture 0
    struct.new $"(unit -> unit)"
    local.set 1
    struct.new $unit
    local.get 1
    struct.get $"(unit -> unit)" 1
    local.get 1
    struct.get $"(unit -> unit)" 0
    call_indirect (type $"(raw) (unit -> unit)")
    ref.cast (ref $unit)
    global.get 12
    ref.as_non_null
    local.tee 2
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 2
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 3
    struct.new $unit
    local.get 3
    struct.get $"('0 -> boolean)" 1
    local.get 3
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 0
    struct.get $"(boolean -> unit)" 1
    local.get 0
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    global.set 20
    global.get 19
    ref.as_non_null
    local.set 4
    i32.const 40
    array.new_fixed $capture 0
    struct.new $"(integer -> (integer -> integer))"
    local.set 6
    i64.const 1
    struct.new $integer
    local.get 6
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 6
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    ref.cast (ref $"(integer -> integer)")
    local.set 5
    i64.const 2
    struct.new $integer
    local.get 5
    struct.get $"(integer -> integer)" 1
    local.get 5
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    ref.cast (ref $integer)
    global.get 12
    ref.as_non_null
    local.tee 7
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 7
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 8
    i64.const 3
    struct.new $integer
    local.get 8
    struct.get $"('0 -> boolean)" 1
    local.get 8
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 4
    struct.get $"(boolean -> unit)" 1
    local.get 4
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    global.set 21
    global.get 19
    ref.as_non_null
    local.set 9
    i32.const 42
    array.new_fixed $capture 0
    struct.new $"(integer -> (integer -> (integer -> integer)))"
    local.set 12
    i64.const 1
    struct.new $integer
    local.get 12
    struct.get $"(integer -> (integer -> (integer -> integer)))" 1
    local.get 12
    struct.get $"(integer -> (integer -> (integer -> integer)))" 0
    call_indirect (type $"(raw) (integer -> (integer -> (integer -> integer)))")
    ref.cast (ref $"(integer -> (integer -> integer))")
    local.set 11
    i64.const 2
    struct.new $integer
    local.get 11
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 11
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    ref.cast (ref $"(integer -> integer)")
    local.set 10
    i64.const 3
    struct.new $integer
    local.get 10
    struct.get $"(integer -> integer)" 1
    local.get 10
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    ref.cast (ref $integer)
    global.get 12
    ref.as_non_null
    local.tee 13
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 13
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 14
    i64.const 6
    struct.new $integer
    local.get 14
    struct.get $"('0 -> boolean)" 1
    local.get 14
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 9
    struct.get $"(boolean -> unit)" 1
    local.get 9
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    global.set 22
    global.get 19
    ref.as_non_null
    local.set 15
    i64.const 1
    struct.new $integer
    local.set $13FunctionTest-2a-9
    i32.const 45
    local.get $13FunctionTest-2a-9
    ref.cast (ref any)
    array.new_fixed $capture 1
    struct.new $"(integer -> integer)"
    local.set 16
    i64.const 2
    struct.new $integer
    local.get 16
    struct.get $"(integer -> integer)" 1
    local.get 16
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    ref.cast (ref $integer)
    global.get 12
    ref.as_non_null
    local.tee 18
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 18
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 19
    i64.const 3
    struct.new $integer
    local.get 19
    struct.get $"('0 -> boolean)" 1
    local.get 19
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 15
    struct.get $"(boolean -> unit)" 1
    local.get 15
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    global.set 23
    i32.const 46
    array.new_fixed $capture 0
    struct.new $"('0 -> ('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))))))"
    global.set 24
    global.get 19
    ref.as_non_null
    local.set 20
    i32.const 72
    array.new_fixed $capture 0
    struct.new $"(unit -> unit)"
    local.set 21
    struct.new $unit
    local.get 21
    struct.get $"(unit -> unit)" 1
    local.get 21
    struct.get $"(unit -> unit)" 0
    call_indirect (type $"(raw) (unit -> unit)")
    ref.cast (ref $unit)
    global.get 12
    ref.as_non_null
    local.tee 22
    struct.get $"('0 -> ('0 -> boolean))" 1
    local.get 22
    struct.get $"('0 -> ('0 -> boolean))" 0
    call_indirect (type $"(raw) ('0 -> ('0 -> boolean))")
    local.set 23
    struct.new $unit
    local.get 23
    struct.get $"('0 -> boolean)" 1
    local.get 23
    struct.get $"('0 -> boolean)" 0
    call_indirect (type $"(raw) ('0 -> boolean)")
    local.get 20
    struct.get $"(boolean -> unit)" 1
    local.get 20
    struct.get $"(boolean -> unit)" 0
    call_indirect (type $"(raw) (boolean -> unit)")
    ref.cast (ref $unit)
    global.set 25
  )
  (func (;1;) (type $"(raw) (integer -> integer)") (param $b (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local $a (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set $a
    local.get $a
    struct.get $integer 0
    local.get $b
    struct.get $integer 0
    i64.add
    struct.new $integer
  )
  (func (;2;) (type $"(raw) (integer -> (integer -> integer))") (param $a (ref $integer)) (param (ref $capture)) (result (ref $"(integer -> integer)"))
    i32.const 1
    local.get $a
    array.new_fixed $capture 1
    struct.new $"(integer -> integer)"
  )
  (func (;3;) (type $"(raw) (integer -> integer)") (param $b (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local $a (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set $a
    local.get $a
    struct.get $integer 0
    local.get $b
    struct.get $integer 0
    i64.sub
    struct.new $integer
  )
  (func (;4;) (type $"(raw) (integer -> (integer -> integer))") (param $a (ref $integer)) (param (ref $capture)) (result (ref $"(integer -> integer)"))
    i32.const 3
    local.get $a
    array.new_fixed $capture 1
    struct.new $"(integer -> integer)"
  )
  (func (;5;) (type $"(raw) (integer -> integer)") (param $b (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local $a (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set $a
    local.get $a
    struct.get $integer 0
    local.get $b
    struct.get $integer 0
    i64.mul
    struct.new $integer
  )
  (func (;6;) (type $"(raw) (integer -> (integer -> integer))") (param $a (ref $integer)) (param (ref $capture)) (result (ref $"(integer -> integer)"))
    i32.const 5
    local.get $a
    array.new_fixed $capture 1
    struct.new $"(integer -> integer)"
  )
  (func (;7;) (type $"(raw) (integer -> integer)") (param $b (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local $a (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set $a
    local.get $a
    struct.get $integer 0
    local.get $b
    struct.get $integer 0
    i64.div_s
    struct.new $integer
  )
  (func (;8;) (type $"(raw) (integer -> (integer -> integer))") (param $a (ref $integer)) (param (ref $capture)) (result (ref $"(integer -> integer)"))
    i32.const 7
    local.get $a
    array.new_fixed $capture 1
    struct.new $"(integer -> integer)"
  )
  (func (;9;) (type $"(raw) (integer -> integer)") (param $b (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local $a (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set $a
    local.get $a
    struct.get $integer 0
    local.get $b
    struct.get $integer 0
    i64.rem_s
    struct.new $integer
  )
  (func (;10;) (type $"(raw) (integer -> (integer -> integer))") (param $a (ref $integer)) (param (ref $capture)) (result (ref $"(integer -> integer)"))
    i32.const 9
    local.get $a
    array.new_fixed $capture 1
    struct.new $"(integer -> integer)"
  )
  (func (;11;) (type $"(raw) (real -> real)") (param $b (ref $real)) (param (ref $capture)) (result (ref $real))
    (local $a (ref $real))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set $a
    local.get $a
    struct.get $real 0
    local.get $b
    struct.get $real 0
    f64.add
    struct.new $real
  )
  (func (;12;) (type $"(raw) (real -> (real -> real))") (param $a (ref $real)) (param (ref $capture)) (result (ref $"(real -> real)"))
    i32.const 11
    local.get $a
    array.new_fixed $capture 1
    struct.new $"(real -> real)"
  )
  (func (;13;) (type $"(raw) (real -> real)") (param $b (ref $real)) (param (ref $capture)) (result (ref $real))
    (local $a (ref $real))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set $a
    local.get $a
    struct.get $real 0
    local.get $b
    struct.get $real 0
    f64.sub
    struct.new $real
  )
  (func (;14;) (type $"(raw) (real -> (real -> real))") (param $a (ref $real)) (param (ref $capture)) (result (ref $"(real -> real)"))
    i32.const 13
    local.get $a
    array.new_fixed $capture 1
    struct.new $"(real -> real)"
  )
  (func (;15;) (type $"(raw) (real -> real)") (param $b (ref $real)) (param (ref $capture)) (result (ref $real))
    (local $a (ref $real))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set $a
    local.get $a
    struct.get $real 0
    local.get $b
    struct.get $real 0
    f64.mul
    struct.new $real
  )
  (func (;16;) (type $"(raw) (real -> (real -> real))") (param $a (ref $real)) (param (ref $capture)) (result (ref $"(real -> real)"))
    i32.const 15
    local.get $a
    array.new_fixed $capture 1
    struct.new $"(real -> real)"
  )
  (func (;17;) (type $"(raw) (real -> real)") (param $b (ref $real)) (param (ref $capture)) (result (ref $real))
    (local $a (ref $real))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set $a
    local.get $a
    struct.get $real 0
    local.get $b
    struct.get $real 0
    f64.div
    struct.new $real
  )
  (func (;18;) (type $"(raw) (real -> (real -> real))") (param $a (ref $real)) (param (ref $capture)) (result (ref $"(real -> real)"))
    i32.const 17
    local.get $a
    array.new_fixed $capture 1
    struct.new $"(real -> real)"
  )
  (func (;19;) (type $"(raw) (boolean -> boolean)") (param $b (ref $boolean)) (param (ref $capture)) (result (ref $boolean))
    (local $a (ref $boolean))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $boolean)
    local.set $a
    local.get $a
    struct.get $boolean 0
    local.get $b
    struct.get $boolean 0
    i32.and
    struct.new $boolean
  )
  (func (;20;) (type $"(raw) (boolean -> (boolean -> boolean))") (param $a (ref $boolean)) (param (ref $capture)) (result (ref $"(boolean -> boolean)"))
    i32.const 19
    local.get $a
    array.new_fixed $capture 1
    struct.new $"(boolean -> boolean)"
  )
  (func (;21;) (type $"(raw) (boolean -> boolean)") (param $b (ref $boolean)) (param (ref $capture)) (result (ref $boolean))
    (local $a (ref $boolean))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $boolean)
    local.set $a
    local.get $a
    struct.get $boolean 0
    local.get $b
    struct.get $boolean 0
    i32.or
    struct.new $boolean
  )
  (func (;22;) (type $"(raw) (boolean -> (boolean -> boolean))") (param $a (ref $boolean)) (param (ref $capture)) (result (ref $"(boolean -> boolean)"))
    i32.const 21
    local.get $a
    array.new_fixed $capture 1
    struct.new $"(boolean -> boolean)"
  )
  (func (;23;) (type $"(raw) (boolean -> boolean)") (param $b (ref $boolean)) (param (ref $capture)) (result (ref $boolean))
    (local $a (ref $boolean))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $boolean)
    local.set $a
    local.get $a
    struct.get $boolean 0
    local.get $b
    struct.get $boolean 0
    i32.xor
    struct.new $boolean
  )
  (func (;24;) (type $"(raw) (boolean -> (boolean -> boolean))") (param $a (ref $boolean)) (param (ref $capture)) (result (ref $"(boolean -> boolean)"))
    i32.const 23
    local.get $a
    array.new_fixed $capture 1
    struct.new $"(boolean -> boolean)"
  )
  (func (;25;) (type $"(raw) ('0 -> boolean)") (param $b anyref) (param (ref $capture)) (result (ref $boolean))
    (local $a anyref) (local i32) (local $index i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set $a
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get $a
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
              local.get $b
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.eq
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $b
            ref.cast (ref $real)
            struct.get $real 0
            f64.eq
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $b
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.eq
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $b
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
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    local.get $a
    ref.cast (ref $string)
    array.len
    local.get $b
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
    local.get $a
    ref.cast (ref $string)
    array.len
    local.set $index
    loop ;; label = @1
      local.get $a
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $b
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
      local.get $index
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;26;) (type $"(raw) ('0 -> ('0 -> boolean))") (param $a anyref) (param (ref $capture)) (result (ref $"('0 -> boolean)"))
    i32.const 25
    local.get $a
    array.new_fixed $capture 1
    struct.new $"('0 -> boolean)"
  )
  (func (;27;) (type $"(raw) ('0 -> boolean)") (param $b anyref) (param (ref $capture)) (result (ref $boolean))
    (local $a anyref) (local i32) (local $index i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set $a
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get $a
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
              local.get $b
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.ne
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $b
            ref.cast (ref $real)
            struct.get $real 0
            f64.ne
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $b
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.ne
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $b
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
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    local.get $a
    ref.cast (ref $string)
    array.len
    local.get $b
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
    local.get $a
    ref.cast (ref $string)
    array.len
    local.set $index
    loop ;; label = @1
      local.get $a
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $b
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
      local.get $index
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;28;) (type $"(raw) ('0 -> ('0 -> boolean))") (param $a anyref) (param (ref $capture)) (result (ref $"('0 -> boolean)"))
    i32.const 27
    local.get $a
    array.new_fixed $capture 1
    struct.new $"('0 -> boolean)"
  )
  (func (;29;) (type $"(raw) ('0 -> boolean)") (param $b anyref) (param (ref $capture)) (result (ref $boolean))
    (local $a anyref) (local i32) (local $index i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set $a
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get $a
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
              local.get $b
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.le_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $b
            ref.cast (ref $real)
            struct.get $real 0
            f64.le
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $b
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.le_s
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $b
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
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    local.get $a
    ref.cast (ref $string)
    array.len
    local.get $b
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
    local.get $a
    ref.cast (ref $string)
    array.len
    local.set $index
    loop ;; label = @1
      local.get $a
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $b
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
      local.get $index
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;30;) (type $"(raw) ('0 -> ('0 -> boolean))") (param $a anyref) (param (ref $capture)) (result (ref $"('0 -> boolean)"))
    i32.const 29
    local.get $a
    array.new_fixed $capture 1
    struct.new $"('0 -> boolean)"
  )
  (func (;31;) (type $"(raw) ('0 -> boolean)") (param $b anyref) (param (ref $capture)) (result (ref $boolean))
    (local $a anyref) (local i32) (local $index i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set $a
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get $a
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
              local.get $b
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.ge_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $b
            ref.cast (ref $real)
            struct.get $real 0
            f64.ge
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $b
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.ge_s
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $b
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
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    local.get $a
    ref.cast (ref $string)
    array.len
    local.get $b
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
    local.get $a
    ref.cast (ref $string)
    array.len
    local.set $index
    loop ;; label = @1
      local.get $a
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $b
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
      local.get $index
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;32;) (type $"(raw) ('0 -> ('0 -> boolean))") (param $a anyref) (param (ref $capture)) (result (ref $"('0 -> boolean)"))
    i32.const 31
    local.get $a
    array.new_fixed $capture 1
    struct.new $"('0 -> boolean)"
  )
  (func (;33;) (type $"(raw) ('0 -> boolean)") (param $b anyref) (param (ref $capture)) (result (ref $boolean))
    (local $a anyref) (local i32) (local $index i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set $a
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get $a
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
              local.get $b
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.lt_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $b
            ref.cast (ref $real)
            struct.get $real 0
            f64.lt
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $b
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.lt_s
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $b
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
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    local.get $a
    ref.cast (ref $string)
    array.len
    local.get $b
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
    local.get $a
    ref.cast (ref $string)
    array.len
    local.set $index
    loop ;; label = @1
      local.get $a
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $b
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
      local.get $index
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;34;) (type $"(raw) ('0 -> ('0 -> boolean))") (param $a anyref) (param (ref $capture)) (result (ref $"('0 -> boolean)"))
    i32.const 33
    local.get $a
    array.new_fixed $capture 1
    struct.new $"('0 -> boolean)"
  )
  (func (;35;) (type $"(raw) ('0 -> boolean)") (param $b anyref) (param (ref $capture)) (result (ref $boolean))
    (local $a anyref) (local i32) (local $index i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set $a
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get $a
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
              local.get $b
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.gt_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $b
            ref.cast (ref $real)
            struct.get $real 0
            f64.gt
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $b
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.gt_s
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $b
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
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    local.get $a
    ref.cast (ref $string)
    array.len
    local.get $b
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
    local.get $a
    ref.cast (ref $string)
    array.len
    local.set $index
    loop ;; label = @1
      local.get $a
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $b
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
      local.get $index
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;36;) (type $"(raw) ('0 -> ('0 -> boolean))") (param $a anyref) (param (ref $capture)) (result (ref $"('0 -> boolean)"))
    i32.const 35
    local.get $a
    array.new_fixed $capture 1
    struct.new $"('0 -> boolean)"
  )
  (func (;37;) (type $"(raw) (unit -> '0)") (param $nothing (ref $unit)) (param (ref $capture)) (result anyref)
    (local $a (ref $integer))
    unreachable
  )
  (func (;38;) (type $"(raw) (boolean -> unit)") (param $4std-8to_test-0 (ref $boolean)) (param (ref $capture)) (result (ref $unit))
    (local (ref $"(unit -> '3)"))
    local.get $4std-8to_test-0
    ref.cast (ref $boolean)
    struct.get $boolean 0
    if (result (ref $unit)) ;; label = @1
      struct.new $unit
    else
      global.get 18
      ref.as_non_null
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
  (func (;39;) (type $"(raw) (unit -> unit)") (param $13FunctionTest-2a-0 (ref $unit)) (param (ref $capture)) (result (ref $unit))
    local.get $13FunctionTest-2a-0
    ref.cast (ref $unit)
  )
  (func (;40;) (type $"(raw) (integer -> (integer -> integer))") (param $13FunctionTest-2a-2 (ref $integer)) (param (ref $capture)) (result (ref $"(integer -> integer)"))
    i32.const 41
    local.get $13FunctionTest-2a-2
    ref.cast (ref any)
    array.new_fixed $capture 1
    struct.new $"(integer -> integer)"
  )
  (func (;41;) (type $"(raw) (integer -> integer)") (param $13FunctionTest-2b-3 (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local $13FunctionTest-2a-2 (ref $integer)) (local (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)"))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set $13FunctionTest-2a-2
    local.get $13FunctionTest-2a-2
    ref.cast (ref $integer)
    global.get 0
    ref.as_non_null
    local.tee 3
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 3
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    local.set 4
    local.get $13FunctionTest-2b-3
    ref.cast (ref $integer)
    local.get 4
    struct.get $"(integer -> integer)" 1
    local.get 4
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
  )
  (func (;42;) (type $"(raw) (integer -> (integer -> (integer -> integer)))") (param $13FunctionTest-2a-5 (ref $integer)) (param (ref $capture)) (result (ref $"(integer -> (integer -> integer))"))
    i32.const 43
    local.get $13FunctionTest-2a-5
    ref.cast (ref any)
    array.new_fixed $capture 1
    struct.new $"(integer -> (integer -> integer))"
  )
  (func (;43;) (type $"(raw) (integer -> (integer -> integer))") (param $13FunctionTest-2b-6 (ref $integer)) (param (ref $capture)) (result (ref $"(integer -> integer)"))
    (local $13FunctionTest-2a-5 (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set $13FunctionTest-2a-5
    i32.const 44
    local.get $13FunctionTest-2a-5
    ref.cast (ref any)
    local.get $13FunctionTest-2b-6
    ref.cast (ref any)
    array.new_fixed $capture 2
    struct.new $"(integer -> integer)"
  )
  (func (;44;) (type $"(raw) (integer -> integer)") (param $13FunctionTest-2c-7 (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local $13FunctionTest-2a-5 (ref $integer)) (local $13FunctionTest-2b-6 (ref $integer)) (local (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)") (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)"))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set $13FunctionTest-2a-5
    local.get 1
    i32.const 1
    array.get $capture
    ref.cast (ref $integer)
    local.set $13FunctionTest-2b-6
    local.get $13FunctionTest-2a-5
    ref.cast (ref $integer)
    global.get 0
    ref.as_non_null
    local.tee 4
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 4
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    local.set 5
    local.get $13FunctionTest-2b-6
    ref.cast (ref $integer)
    local.get 5
    struct.get $"(integer -> integer)" 1
    local.get 5
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
    global.get 0
    ref.as_non_null
    local.tee 6
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 6
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    local.set 7
    local.get $13FunctionTest-2c-7
    ref.cast (ref $integer)
    local.get 7
    struct.get $"(integer -> integer)" 1
    local.get 7
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
  )
  (func (;45;) (type $"(raw) (integer -> integer)") (param $13FunctionTest-2b-10 (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local $13FunctionTest-2a-9 (ref $integer)) (local (ref $"(integer -> (integer -> integer))") (ref $"(integer -> integer)"))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set $13FunctionTest-2a-9
    local.get $13FunctionTest-2a-9
    ref.cast (ref $integer)
    global.get 0
    ref.as_non_null
    local.tee 3
    struct.get $"(integer -> (integer -> integer))" 1
    local.get 3
    struct.get $"(integer -> (integer -> integer))" 0
    call_indirect (type $"(raw) (integer -> (integer -> integer))")
    local.set 4
    local.get $13FunctionTest-2b-10
    ref.cast (ref $integer)
    local.get 4
    struct.get $"(integer -> integer)" 1
    local.get 4
    struct.get $"(integer -> integer)" 0
    call_indirect (type $"(raw) (integer -> integer)")
  )
  (func (;46;) (type $"(raw) ('0 -> ('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))))))") (param $13FunctionTest-2a-12 anyref) (param (ref $capture)) (result (ref $"('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))))))"))
    i32.const 47
    array.new_fixed $capture 0
    struct.new $"('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))))))"
  )
  (func (;47;) (type $"(raw) ('1 -> ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))))))") (param $13FunctionTest-2b-13 anyref) (param (ref $capture)) (result (ref $"('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))))"))
    i32.const 48
    array.new_fixed $capture 0
    struct.new $"('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))))"
  )
  (func (;48;) (type $"(raw) ('2 -> ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))))") (param $13FunctionTest-2c-14 anyref) (param (ref $capture)) (result (ref $"('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))))"))
    i32.const 49
    array.new_fixed $capture 0
    struct.new $"('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))))"
  )
  (func (;49;) (type $"(raw) ('3 -> ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))))") (param $13FunctionTest-2d-15 anyref) (param (ref $capture)) (result (ref $"('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))"))
    i32.const 50
    array.new_fixed $capture 0
    struct.new $"('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))"
  )
  (func (;50;) (type $"(raw) ('4 -> ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))))") (param $13FunctionTest-2e-16 anyref) (param (ref $capture)) (result (ref $"('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))"))
    i32.const 51
    array.new_fixed $capture 0
    struct.new $"('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))"
  )
  (func (;51;) (type $"(raw) ('5 -> ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))))") (param $13FunctionTest-2f-17 anyref) (param (ref $capture)) (result (ref $"('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))"))
    i32.const 52
    array.new_fixed $capture 0
    struct.new $"('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))"
  )
  (func (;52;) (type $"(raw) ('6 -> ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))))") (param $13FunctionTest-2g-18 anyref) (param (ref $capture)) (result (ref $"('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))"))
    i32.const 53
    array.new_fixed $capture 0
    struct.new $"('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))"
  )
  (func (;53;) (type $"(raw) ('7 -> ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))))") (param $13FunctionTest-2h-19 anyref) (param (ref $capture)) (result (ref $"('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))"))
    i32.const 54
    array.new_fixed $capture 0
    struct.new $"('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))"
  )
  (func (;54;) (type $"(raw) ('8 -> ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))))") (param $13FunctionTest-2i-20 anyref) (param (ref $capture)) (result (ref $"('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))"))
    i32.const 55
    array.new_fixed $capture 0
    struct.new $"('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))"
  )
  (func (;55;) (type $"(raw) ('9 -> ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))))") (param $13FunctionTest-2j-21 anyref) (param (ref $capture)) (result (ref $"('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))"))
    i32.const 56
    array.new_fixed $capture 0
    struct.new $"('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))"
  )
  (func (;56;) (type $"(raw) ('10 -> ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))))") (param $13FunctionTest-2k-22 anyref) (param (ref $capture)) (result (ref $"('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))"))
    i32.const 57
    array.new_fixed $capture 0
    struct.new $"('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))"
  )
  (func (;57;) (type $"(raw) ('11 -> ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))))") (param $13FunctionTest-2l-23 anyref) (param (ref $capture)) (result (ref $"('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))"))
    i32.const 58
    array.new_fixed $capture 0
    struct.new $"('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))"
  )
  (func (;58;) (type $"(raw) ('12 -> ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))))") (param $13FunctionTest-2m-24 anyref) (param (ref $capture)) (result (ref $"('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))"))
    i32.const 59
    array.new_fixed $capture 0
    struct.new $"('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))"
  )
  (func (;59;) (type $"(raw) ('13 -> ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))))") (param $13FunctionTest-2n-25 anyref) (param (ref $capture)) (result (ref $"('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))"))
    i32.const 60
    array.new_fixed $capture 0
    struct.new $"('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))"
  )
  (func (;60;) (type $"(raw) ('14 -> ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))))") (param $13FunctionTest-2o-26 anyref) (param (ref $capture)) (result (ref $"('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))"))
    i32.const 61
    array.new_fixed $capture 0
    struct.new $"('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))"
  )
  (func (;61;) (type $"(raw) ('15 -> ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))))") (param $13FunctionTest-2p-27 anyref) (param (ref $capture)) (result (ref $"('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))"))
    i32.const 62
    array.new_fixed $capture 0
    struct.new $"('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))"
  )
  (func (;62;) (type $"(raw) ('16 -> ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))))") (param $13FunctionTest-2q-28 anyref) (param (ref $capture)) (result (ref $"('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))"))
    i32.const 63
    array.new_fixed $capture 0
    struct.new $"('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))"
  )
  (func (;63;) (type $"(raw) ('17 -> ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))))") (param $13FunctionTest-2r-29 anyref) (param (ref $capture)) (result (ref $"('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))"))
    i32.const 64
    array.new_fixed $capture 0
    struct.new $"('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))"
  )
  (func (;64;) (type $"(raw) ('18 -> ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))))") (param $13FunctionTest-2s-30 anyref) (param (ref $capture)) (result (ref $"('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))"))
    i32.const 65
    array.new_fixed $capture 0
    struct.new $"('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))"
  )
  (func (;65;) (type $"(raw) ('19 -> ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))))") (param $13FunctionTest-2t-31 anyref) (param (ref $capture)) (result (ref $"('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))"))
    i32.const 66
    array.new_fixed $capture 0
    struct.new $"('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))"
  )
  (func (;66;) (type $"(raw) ('20 -> ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit))))))") (param $13FunctionTest-2u-32 anyref) (param (ref $capture)) (result (ref $"('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))"))
    i32.const 67
    array.new_fixed $capture 0
    struct.new $"('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))"
  )
  (func (;67;) (type $"(raw) ('21 -> ('22 -> ('23 -> ('24 -> ('25 -> unit)))))") (param $13FunctionTest-2v-33 anyref) (param (ref $capture)) (result (ref $"('22 -> ('23 -> ('24 -> ('25 -> unit))))"))
    i32.const 68
    array.new_fixed $capture 0
    struct.new $"('22 -> ('23 -> ('24 -> ('25 -> unit))))"
  )
  (func (;68;) (type $"(raw) ('22 -> ('23 -> ('24 -> ('25 -> unit))))") (param $13FunctionTest-2w-34 anyref) (param (ref $capture)) (result (ref $"('23 -> ('24 -> ('25 -> unit)))"))
    i32.const 69
    array.new_fixed $capture 0
    struct.new $"('23 -> ('24 -> ('25 -> unit)))"
  )
  (func (;69;) (type $"(raw) ('23 -> ('24 -> ('25 -> unit)))") (param $13FunctionTest-2x-35 anyref) (param (ref $capture)) (result (ref $"('24 -> ('25 -> unit))"))
    i32.const 70
    array.new_fixed $capture 0
    struct.new $"('24 -> ('25 -> unit))"
  )
  (func (;70;) (type $"(raw) ('24 -> ('25 -> unit))") (param $13FunctionTest-2y-36 anyref) (param (ref $capture)) (result (ref $"('25 -> unit)"))
    i32.const 71
    array.new_fixed $capture 0
    struct.new $"('25 -> unit)"
  )
  (func (;71;) (type $"(raw) ('25 -> unit)") (param $13FunctionTest-2z-37 anyref) (param (ref $capture)) (result (ref $unit))
    struct.new $unit
  )
  (func (;72;) (type $"(raw) (unit -> unit)") (param $unit (ref $unit)) (param (ref $capture)) (result (ref $unit))
    struct.new $unit
  )
)
.get 3
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
      local.get $index
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;32;) (type $"(raw) ('0 -> ('0 -> boolean))") (param $a anyref) (param (ref $capture)) (result (ref $"('0 -> boolean)"))
    i32.const 31
    local.get $a
    array.new_fixed $capture 1
    struct.new $"('0 -> boolean)"
  )
  (func (;33;) (type $"(raw) ('0 -> boolean)") (param $b anyref) (param (ref $capture)) (result (ref $boolean))
    (local $a anyref) (local i32) (local $index i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set $a
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get $a
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
              local.get $b
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.lt_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $b
            ref.cast (ref $real)
            struct.get $real 0
            f64.lt
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $b
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.lt_s
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $b
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
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    local.get $a
    ref.cast (ref $string)
    array.len
    local.get $b
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
    local.get $a
    ref.cast (ref $string)
    array.len
    local.set $index
    loop ;; label = @1
      local.get $a
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $b
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
      local.get $index
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;34;) (type $"(raw) ('0 -> ('0 -> boolean))") (param $a anyref) (param (ref $capture)) (result (ref $"('0 -> boolean)"))
    i32.const 33
    local.get $a
    array.new_fixed $capture 1
    struct.new $"('0 -> boolean)"
  )
  (func (;35;) (type $"(raw) ('0 -> boolean)") (param $b anyref) (param (ref $capture)) (result (ref $boolean))
    (local $a anyref) (local i32) (local $index i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set $a
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get $a
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
              local.get $b
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.gt_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $b
            ref.cast (ref $real)
            struct.get $real 0
            f64.gt
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $b
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.gt_s
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $b
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
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    local.get $a
    ref.cast (ref $string)
    array.len
    local.get $b
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
    local.get $a
    ref.cast (ref $string)
    array.len
    local.set $index
    loop ;; label = @1
      local.get $a
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $b
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
      local.get $index
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;36;) (type $"(raw) ('0 -> ('0 -> boolean))") (param $a anyref) (param (ref $capture)) (result (ref $"('0 -> boolean)"))
    i32.const 35
    local.get $a
    array.new_fixed $capture 1
    struct.new $"('0 -> boolean)"
  )
  (func (;37;) (type $"(raw) (unit -> '0)") (param $nothing (ref $unit)) (param (ref $capture)) (result anyref)
    (local $a (ref $integer))
    unreachable
  )
  (func (;38;) (type $"(raw) (boolean -> unit)") (param $4std-8to_test-0 (ref $boolean)) (param (ref $capture)) (result (ref $unit))
    (local (ref $"(unit -> '3)"))
    local.get $4std-8to_test-0
    ref.cast (ref $boolean)
    struct.get $boolean 0
    if (result (ref $unit)) ;; label = @1
      struct.new $unit
    else
      global.get 18
      ref.as_non_null
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
  (func (;39;) (type $"(raw) (boolean -> boolean)") (param $a (ref $boolean)) (param (ref $capture)) (result (ref $boolean))
    local.get $a
    struct.get $boolean 0
    i32.eqz
    struct.new $boolean
  )
  (func (;40;) (type $"(raw) (integer -> integer)") (param $a (ref $integer)) (param (ref $capture)) (result (ref $integer))
    i64.const 0
    local.get $a
    struct.get $integer 0
    i64.sub
    struct.new $integer
  )
  (func (;41;) (type $"(raw) (real -> real)") (param $a (ref $real)) (param (ref $capture)) (result (ref $real))
    local.get $a
    struct.get $real 0
    f64.neg
    struct.new $real
  )
)
