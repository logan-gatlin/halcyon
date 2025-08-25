module Demo =
  import builtin
  type list = fn a => Pair of a * (list a) | Nil of ()
  type intlist = list builtin:integer
end
