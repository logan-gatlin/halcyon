module integer =
  use bundle::ops

  (*>
  Absolute value.

  - Arguments:
    - `value`: Integer input.
  - Returns: Non-negative magnitude of `value`.
  *)
  let abs = fn value => if value < 0 then 0 - value else value

  (*>
  Numeric sign.

  - Arguments:
    - `value`: Integer input.
  - Returns: `-1`, `0`, or `1`.
  *)
  let signum = fn value => if value < 0 then -1 else if value > 0 then 1 else 0

  (*>
  Checks whether an integer is even.

  - Arguments:
    - `value`: Integer input.
  - Returns: `true` when divisible by `2`.
  *)
  let is_even = fn value => (value mod 2) == 0

  (*>
  Checks whether an integer is odd.

  - Arguments:
    - `value`: Integer input.
  - Returns: `true` when not even.
  *)
  let is_odd = fn value => not (is_even value)
  --> @HIDDEN
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

  --> @HIDDEN
  let show_magnitude = fn value =>
    let quotient = value / 10 in
    let remainder = value mod 10 in
    let digit = if remainder < 0 then 0 - remainder else remainder in
    let prefix = if quotient == 0 then "" else show_magnitude quotient in
    prefix + (digit_to_string digit)

  (*>
  Returns the smaller of two integers.

  - Arguments:
    - `left`: First integer.
    - `right`: Second integer.
  - Returns: Minimum of `left` and `right`.
  *)
  let min = bundle::ops::min

  (*>
  Returns the larger of two integers.

  - Arguments:
    - `left`: First integer.
    - `right`: Second integer.
  - Returns: Maximum of `left` and `right`.
  *)
  let max = bundle::ops::max

  (*>
  Restricts an integer to an inclusive range.

  - Arguments:
    - `lower`: Lower bound.
    - `upper`: Upper bound.
    - `value`: Integer to clamp.
  - Returns: Clamped integer.
  *)
  let clamp = bundle::ops::clamp

  --> @HIDDEN
  impl bundle::Default bundle::Integer =
    let default = 0
  end

  --> @HIDDEN
  impl bundle::show::Show bundle::Integer =
    let show = fn value =>
      if value < 0 then
        "-" + (show_magnitude value)
      else
        show_magnitude value
  end
end
