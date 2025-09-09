module test = 
  let add1 = fn x => x + 1
  let mul2 = fn x => x * 2

  do (add1 >> mul2) 1
    |> format::integer
    |> std::println
end
