module function =
  use core::ops

  let identity = fn value => value
  let constant = fn value => fn _ => value
  let flip = fn f => fn left right => f right left
  let compose = fn first second value => second (first value)
  let pipe = fn value f => f value
end
