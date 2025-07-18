// Unary ops
assert not false;
assert not not not false;
assert (-1 == 0 - 1);
assert (-.1.0 == 0.0 -. 1.0);
assert (-1.0 == 0.0 -. 1.0); // Special case

// Integer arithmetic
assert (1 + 2 == 3);
assert (1 - 2 == -1);
assert (1 * 2 == 2);
assert (1 / 2 == 0);
assert (1 % 2 == 1);

// Real arithmetic
assert (1.0 +. 2.0 == 3.0);
assert (2.0 -. 1.0 == 1.0);
assert (1.0 *. 2.0 == 2.0);
assert (1.0 /. 2.0 == 0.5);

// Boolean logical
assert (true and true);
assert (true or false);
assert (true xor false);

// Unit comparison
assert (() == () == true);
assert (() != () == false);
assert (() <= () == true);
assert (() >= () == true);
assert (() < () == false);
assert (() > () == false);

// Boolean comparison
assert (true == true);
assert (true != false);
assert (false <= true);
assert (true >= false);
assert (false < true);
assert (true > false);

// Glyph comparison
assert ('a' == 'a');
assert ('a' != 'b');
assert ('a' <= 'b');
assert ('b' >= 'a');
assert ('a' < 'b');
assert ('b' > 'a');

// String comparison
assert ("abc" == "abc");
assert ("abc" != "def");
assert ("abc" <= "def");
assert ("def" >= "abc");
assert ("abc" <= "abc");
assert ("abc" >= "abc");
assert ("abc" < "def");
assert ("def" > "abc")
