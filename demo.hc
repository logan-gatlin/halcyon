module A = 
  import std
  type v = A of std:integer | B of std:real
  let a = A 1
  let b = B 1.0
  let c = match a with
    | A a => a == 1
    | B b => b == 1.0
end
