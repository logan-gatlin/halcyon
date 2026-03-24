module integer =
  use bundle::ops

  let abs = fn value => if value < 0 then 0 - value else value
  let signum = fn value => if value < 0 then -1 else if value > 0 then 1 else 0
  let is_even = fn value => (value mod 2) == 0
  let is_odd = fn value => not (is_even value)
  let digit_to_string = fn digit =>
    match digit with
      | 0 => "0"
      | 1 => "1"
      | 2 => "2"
      | 3 => "3"
      | 4 => "4"
      | 5 => "5"
      | 6 => "6"
      | 7 => "7"
      | 8 => "8"
      | 9 => "9"
      | _ => "?"

  let show_magnitude = fn value =>
    let quotient = value / 10 in
    let remainder = value mod 10 in
    let digit = if remainder < 0 then 0 - remainder else remainder in
    let prefix = if quotient == 0 then "" else show_magnitude quotient in
    prefix + (digit_to_string digit)

  let min = bundle::ops::min
  let max = bundle::ops::max
  let clamp = bundle::ops::clamp

  impl bundle::Default bundle::Integer =
    let default = 0
  end

  impl bundle::show::Show bundle::Integer =
    let show = fn value =>
      if value < 0 then
        "-" + (show_magnitude value)
      else
        show_magnitude value
  end
end
