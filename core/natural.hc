module natural =
  use bundle::ops

  (*>
  Checks whether a natural number is even.

  - Arguments:
    - `value`: Natural number.
  - Returns: `true` when divisible by `2n`.
  *)
  let is_even = fn value => (value mod 2n) == 0n

  (*>
  Checks whether a natural number is odd.

  - Arguments:
    - `value`: Natural number.
  - Returns: `true` when not even.
  *)
  let is_odd = fn value => not (is_even value)
  --> @HIDDEN
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

  --> @HIDDEN
  let show_magnitude = fn value =>
    let quotient = value / 10n in
    let remainder = value mod 10n in
    let prefix = if quotient == 0n then "" else show_magnitude quotient in
    prefix + (digit_to_string remainder)

  (*>
  Returns the smaller of two natural numbers.

  - Arguments:
    - `left`: First natural number.
    - `right`: Second natural number.
  - Returns: Minimum of `left` and `right`.
  *)
  let min = bundle::ops::min

  (*>
  Returns the larger of two natural numbers.

  - Arguments:
    - `left`: First natural number.
    - `right`: Second natural number.
  - Returns: Maximum of `left` and `right`.
  *)
  let max = bundle::ops::max

  (*>
  Restricts a natural number to an inclusive range.

  - Arguments:
    - `lower`: Lower bound.
    - `upper`: Upper bound.
    - `value`: Natural number to clamp.
  - Returns: Clamped natural number.
  *)
  let clamp = bundle::ops::clamp

  --> @HIDDEN
  impl bundle::Default bundle::Natural =
    let default = 0n
  end

  --> @HIDDEN
  impl bundle::show::Show bundle::Natural =
    let show = show_magnitude
  end
end
