module integer =
  use core::ops

  let abs = fn value => if value < 0 then 0 - value else value
  let signum = fn value => if value < 0 then -1 else if value > 0 then 1 else 0
  let is_even = fn value => (value % 2) == 0
  let is_odd = fn value => not (is_even value)
  let min = core::ops::min
  let max = core::ops::max
  let clamp = core::ops::clamp

  impl bundle::Default bundle::Integer =
    let default = 0
  end
end
