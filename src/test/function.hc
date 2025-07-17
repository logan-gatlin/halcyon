// Identity
(fn a => a) ();

// Type hints
(fn (a: integer) b => a + b);

// Currying
(fn a b c => a + b + c) 1 2 3;

// Closure captures
(let a = 1 in
  fn b => a + b
);

// Lots of arguments
(fn a b c d e f g h i j k l m n o p q r s t u v w x y z => ())

