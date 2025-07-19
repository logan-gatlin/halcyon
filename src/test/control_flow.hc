assert ((if true then 1 else 2) == 1);
assert (if 1 + 3 == 4 then true else false);
assert ((if true then ()) == () and (if false then ()) == ());
assert ((fn a => a) true)

