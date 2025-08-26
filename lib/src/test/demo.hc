module Demo =
  import builtin

  type list = fn t => Pair of t * (list t) | Nil of ()

  let push = fn item list => match list with
    | Pair of (head, tail) => Pair (head, push item tail)
    | Nil of () => Pair (item, Nil ())

  let nil = Nil ()
  
  let list = nil |> push 1 |> push 2 |> push 3
end

