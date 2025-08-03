module A
  import std
  let _ = std:string_print
    (std:string_concatenate "Hello " "World!!")
  type Option = Some of integer | None of unit
  let o1 = Some 1
  let o2 = None ()
  let m = (match o1 with Some a -> a | None -> 0)
end
