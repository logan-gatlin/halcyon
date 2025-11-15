module ControlFlowTest =
  let () = std::assert ((if true then 1 else 2) == 1)
  let () = std::assert (if 1 + 3 == 4 then true else false)
  let () = std::assert ((fn a => a) true)
  let () = std::assert ((match (1, 2) with
    | (_, 1) => 1
    | (2, _) => 2
    | (_, _) => 3
  ) == 3)
  let () = std::assert (match ((1, 2), (3, 4)) with
    | ((1, 2), (4, 3)) => false
    | ((a, b), (c, d)) => (a + b + c + d) == 10
    | _ => false)
end

