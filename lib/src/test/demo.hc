module test =
  let pow = fn base num => match num with
    | 0 => 1
    | 1 => base
    | n =>
      let b = pow base (n / 2) in
      b * b * (if n % 2 == 0 then 1 else base)
end
