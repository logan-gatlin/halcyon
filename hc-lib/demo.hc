module List = 
  import std
  import opt
  import list
  import string

  let o = opt:None ()
  let () = o
    |> opt:iterate std:print_string

  let l1 = list:Nil ()
    |> list:push "Hello"
    |> list:push "World"

  let len = std:assert (list:length l1 == 2)

  let l2 = list:Nil ()
    |> list:push "Hello"
    |> list:push "Sailor"

  let l3 = list:concatenate l1 l2
  let () = list:fold string:concatenate "" l3 |> std:print_string
end
