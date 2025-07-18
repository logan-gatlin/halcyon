// Identity
assert ((fn a => a) () == ());

// Type hints
assert ((fn (a: integer) b => a + b) 1 2 == 3);

// Currying
assert ((fn a b c => a + b + c) 1 2 3 == 6);

// Closure captures
assert ((let a = 1 in
  fn b => a + b
) 2 == 3);

// Lots of arguments
(fn a b c d e f g h i j k l m n o p q r s t u v w x y z => ());

// No arguments
assert ((fn => ())() == ())
