module ControlFlowTest
  import std
  let _ = std:assert ((if true then 1 else 2) == 1)
  let _ = std:assert (if 1 + 3 == 4 then true else false)
  let _ = std:assert ((if true then ()) == () and (if false then ()) == ())
  let _ = std:assert ((fn a => a) true)
end

