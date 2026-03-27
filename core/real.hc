module real =
  use bundle
  use bundle::ops

  (*>
  Absolute value for reals.

  - Arguments:
    - `value`: Real input.
  - Returns: Non-negative magnitude.
  *)
  let abs = fn value => if value < 0.0 then 0.0 - value else value

  (*>
  Numeric sign for reals.

  - Arguments:
    - `value`: Real input.
  - Returns: `-1.0`, `0.0`, or `1.0`.
  *)
  let signum = fn value => if value < 0.0 then -1.0 else if value > 0.0 then 1.0 else 0.0

  (*>
  Checks whether a real is strictly positive.

  - Arguments:
    - `value`: Real input.
  - Returns: `true` when `value > 0.0`.
  *)
  let is_positive = fn value => value > 0.0

  (*>
  Checks whether a real is strictly negative.

  - Arguments:
    - `value`: Real input.
  - Returns: `true` when `value < 0.0`.
  *)
  let is_negative = fn value => value < 0.0
  --> @HIDDEN
  let max_significant_digits = 17

  (*>
  Truncates a real toward zero.

  - Arguments:
    - `value`: Real input.
  - Returns: Integer part of `value`.
  *)
  let trunc_to_integer : Real -> Integer = fn value => (wasm : Integer) => (
    get value
    struct.get $real 0
    i64.trunc_f64_s
    struct.new $integer
  )

  (*>
  Converts an integer to a real.

  - Arguments:
    - `value`: Integer input.
  - Returns: Equivalent real value.
  *)
  let integer_to_real : Integer -> Real = fn value => (wasm : Real) => (
    get value
    struct.get $integer 0
    f64.convert_i64_s
    struct.new $real
  )

  --> @HIDDEN
  let power10 = fn exponent =>
    if exponent == 0 then 1 else 10 * (power10 (exponent - 1))

  --> @HIDDEN
  let power10_real = fn exponent =>
    if exponent == 0 then
      1.0
    else
      10.0 * (power10_real (exponent - 1))

  --> @HIDDEN
  let show_fixed_width = fn width value =>
    if width == 0 then
      ""
    else
      let divisor = power10 (width - 1) in
      let digit = value / divisor in
      let rest = value mod divisor in
      (bundle::integer::digit_to_string digit) + (show_fixed_width (width - 1) rest)

  --> @HIDDEN
  let repeat_zeros = fn count =>
    if count < 1 then
      ""
    else
      "0" + (repeat_zeros (count - 1))

  --> @HIDDEN
  let round_to_integer = fn value =>
    let truncated = trunc_to_integer value in
    let fractional = value - (integer_to_real truncated) in
    if (fractional > 0.5) or (fractional == 0.5) then
      truncated + 1
    else
      truncated

  --> @HIDDEN
  let decimal_exponent = fn value =>
    if (value > 10.0) or (value == 10.0) then
      1 + (decimal_exponent (value / 10.0))
    else if value < 1.0 then
      -1 + (decimal_exponent (value * 10.0))
    else
      0

  --> @HIDDEN
  let scale_by_power10 = fn value exponent =>
    if exponent == 0 then
      value
    else if exponent > 0 then
      scale_by_power10 (value * 10.0) (exponent - 1)
    else
      scale_by_power10 (value / 10.0) (exponent + 1)

  --> @HIDDEN
  let rounded_significand = fn normalized precision =>
    round_to_integer (normalized * (power10_real (precision - 1)))

  --> @HIDDEN
  let significand_for_precision = fn normalized precision =>
    let rounded = rounded_significand normalized precision in
    if rounded == (power10 precision) then
      rounded / 10
    else
      rounded

  --> @HIDDEN
  let exponent_carry_for_precision = fn normalized precision =>
    if rounded_significand normalized precision == (power10 precision) then
      1
    else
      0

  --> @HIDDEN
  let adjusted_exponent_for_precision = fn exponent normalized precision =>
    exponent + (exponent_carry_for_precision normalized precision)

  --> @HIDDEN
  let candidate_value = fn significand precision exponent =>
    let normalized = (integer_to_real significand) / (power10_real (precision - 1)) in
    scale_by_power10 normalized exponent

  --> @HIDDEN
  let candidate_matches_precision = fn magnitude normalized exponent precision =>
    let significand = significand_for_precision normalized precision in
    let adjusted_exponent = adjusted_exponent_for_precision exponent normalized precision in
    candidate_value significand precision adjusted_exponent == magnitude

  --> @HIDDEN
  let choose_precision = fn magnitude normalized exponent precision =>
    if precision > max_significant_digits then
      max_significant_digits
    else if candidate_matches_precision magnitude normalized exponent precision then
      precision
    else
      choose_precision magnitude normalized exponent (precision + 1)

  --> @HIDDEN
  let trailing_fraction_zero_count = fn width value =>
    if width == 0 then
      0
    else if value == 0 then
      width
    else if (value mod 10) == 0 then
      1 + (trailing_fraction_zero_count (width - 1) (value / 10))
    else
      0

  --> @HIDDEN
  let render_fixed = fn significand precision exponent =>
    if (exponent > (precision - 1)) or (exponent == (precision - 1)) then
      (bundle::show::show significand) + (repeat_zeros (exponent - (precision - 1)))
    else if (exponent > 0) or (exponent == 0) then
      let fractional_width = precision - (exponent + 1) in
      let divisor = power10 fractional_width in
      let whole = significand / divisor in
      let fractional = significand mod divisor in
      let trailing_zeros = trailing_fraction_zero_count fractional_width fractional in
      let trimmed_width = fractional_width - trailing_zeros in
      if trimmed_width == 0 then
        bundle::show::show whole
      else
        let trimmed_fractional = fractional / (power10 trailing_zeros) in
        (bundle::show::show whole) + "." + (show_fixed_width trimmed_width trimmed_fractional)
    else
      let leading_zeros = (0 - exponent) - 1 in
      let fractional_width = leading_zeros + precision in
      let trailing_zeros = trailing_fraction_zero_count fractional_width significand in
      let trimmed_width = fractional_width - trailing_zeros in
      if trimmed_width == 0 then
        "0"
      else
        let trimmed_fractional = significand / (power10 trailing_zeros) in
        "0." + (show_fixed_width trimmed_width trimmed_fractional)

  (*>
  Detects IEEE NaN.

  - Arguments:
    - `value`: Real input.
  - Returns: `true` when `value` is NaN.
  *)
  let is_nan = fn value => value != value

  (*>
  Detects positive or negative infinity.

  - Arguments:
    - `value`: Real input.
  - Returns: `true` when `value` is infinite.
  *)
  let is_infinite = fn value =>
    let delta = value - value in
    delta != delta

  --> @HIDDEN
  let is_negative_zero = fn value =>
    (value == 0.0) and ((1.0 / value) < 0.0)

  (*>
  Returns the smaller of two reals.

  - Arguments:
    - `left`: First real.
    - `right`: Second real.
  - Returns: Minimum value.
  *)
  let min = bundle::ops::min

  (*>
  Returns the larger of two reals.

  - Arguments:
    - `left`: First real.
    - `right`: Second real.
  - Returns: Maximum value.
  *)
  let max = bundle::ops::max

  (*>
  Restricts a real to an inclusive range.

  - Arguments:
    - `lower`: Lower bound.
    - `upper`: Upper bound.
    - `value`: Real to clamp.
  - Returns: Clamped real.

  ```hc
  let normalized = real::clamp 0.0 1.0 raw_value
  ```
  *)
  let clamp = bundle::ops::clamp

  --> @HIDDEN
  impl bundle::Default bundle::Real =
    let default = 0.0
  end

  --> @HIDDEN
  impl bundle::show::Show bundle::Real =
    let show = fn value =>
      if is_nan value then
        "nan"
      else
        let negative = (value < 0.0) or (is_negative_zero value) in
        let magnitude = abs value in
        if is_infinite magnitude then
          if negative then "-inf" else "inf"
        else if magnitude == 0.0 then
          if negative then "-0" else "0"
        else
          let exponent = decimal_exponent magnitude in
          let normalized = scale_by_power10 magnitude (0 - exponent) in
          let precision = choose_precision magnitude normalized exponent 1 in
          let significand = significand_for_precision normalized precision in
          let adjusted_exponent = adjusted_exponent_for_precision exponent normalized precision in
          let rendered = render_fixed significand precision adjusted_exponent in
          if negative then "-" + rendered else rendered
  end
end
