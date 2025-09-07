module test = 
  let pow = fn base power =>
    if power == 1 then base
    else let b = pow base (power / 2) in
    b + b + b

  let i = pow 1 2
end
