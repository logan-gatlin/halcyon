module FunctionTest = 
  import std
  -- Identity
  let () = std::assert ((fn a => a) () == ())

  -- Type hints
  let () = std::assert ((fn (a: std::integer) b => a + b) 1 2 == 3)

  -- Currying
  let () = std::assert ((fn a b c => a + b + c) 1 2 3 == 6)

  -- Closure captures
  let () = std::assert ((let a = 1 in
    fn b => a + b
  ) 2 == 3)

  -- Lots of arguments
  let _ = (fn a b c d e f g h i j k l m n o p q r s t u v w x y z => ())

  -- No arguments
  let () = std::assert ((fn => ())() == ())
end
