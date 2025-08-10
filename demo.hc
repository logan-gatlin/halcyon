module List = 
  import std
  type Option = fn t =>
    Some of t | None of std:unit

  let f = fn a => a
  let _ = f 1
  let _ = f 2.0

end
