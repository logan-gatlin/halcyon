module Test =
  let foo = 1
  type i = builtin::integer
end

module Demo =
  -- import println : integer -> () = sys::println
  let a = fn a => Test::foo

  let _ = if true then 1 else builtin::panic ()
  type list = fn t => Pair of t * (list t) | Nil of ()

  let nil = Nil ()
  let push = fn item list => match list with
    | Pair of (head, tail) => Pair (head, push item tail)
    | Nil of () => Pair (item, Nil ())
  
  let list = nil
    |> push 1
    |> push 2
    |> push 3
end

