module real =
  use core::ops

  let abs = fn value => if value < 0.0 then 0.0 - value else value
  let signum = fn value => if value < 0.0 then -1.0 else if value > 0.0 then 1.0 else 0.0
  let is_positive = fn value => value > 0.0
  let is_negative = fn value => value < 0.0
  let min = core::ops::min
  let max = core::ops::max
  let clamp = core::ops::clamp

  impl bundle::Default bundle::Real =
    let default = 0.0
  end
end
