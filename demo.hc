module List = 
  import std

  type List = Cons of std:string * List | Nil of std:unit

  let push = fn item list => match list with
    | Cons of (head, tail) => Cons (head, push item tail)
    | Nil of () => Cons (item, Nil ())

  let print = fn list => match list with
    | Cons of (s, l) =>  std:print_string s; print l
    | Nil => ()

  let () = print (Nil () |> push "one" |> push "two" |> push "three")
end
