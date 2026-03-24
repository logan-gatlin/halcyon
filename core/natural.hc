module natural =
  use bundle::ops

  let is_even = fn value => (value mod 2n) == 0n
  let is_odd = fn value => not (is_even value)
  let digit_to_string = fn digit =>
    match digit with
      | 0n => "0"
      | 1n => "1"
      | 2n => "2"
      | 3n => "3"
      | 4n => "4"
      | 5n => "5"
      | 6n => "6"
      | 7n => "7"
      | 8n => "8"
      | 9n => "9"
      | _ => "?"

  let show_magnitude = fn value =>
    let quotient = value / 10n in
    let remainder = value mod 10n in
    let prefix = if quotient == 0n then "" else show_magnitude quotient in
    prefix + (digit_to_string remainder)

  let min = bundle::ops::min
  let max = bundle::ops::max
  let clamp = bundle::ops::clamp

  impl bundle::Default bundle::Natural =
    let default = 0n
  end

  impl bundle::show::Show bundle::Natural =
    let show = show_magnitude
  end
end
