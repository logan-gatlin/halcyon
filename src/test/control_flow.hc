module ControlFlowTest
  let _ = assert ((if true then 1 else 2) == 1)
  let _ = assert (if 1 + 3 == 4 then true else false)
  let _ = assert ((if true then ()) == () and (if false then ()) == ())
  let _ = assert ((fn a => a) true)
end

