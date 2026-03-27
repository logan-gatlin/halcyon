module big-num =
  use bundle
  use bundle::ops

  (*>
  Arbitrary-precision natural number.

  `BigNat` stores non-negative integers without fixed-width overflow.
  *)
  type BigNat = | Nat (Array Natural)

  (*>
  Arbitrary-precision signed integer.

  `BigInt` stores signed integers without fixed-width overflow.
  *)
  type BigInt =
    | BigZero
    | BigPositive BigNat
    | BigNegative BigNat

  --> @HIDDEN
  type DigitsAndRemainder = | DigitsAndRemainder (Array Natural, Natural)
  --> @HIDDEN
  type NaturalAndRemainder = | NaturalAndRemainder (BigNat, Natural)
  --> @HIDDEN
  type DigitsAndNatural = | DigitsAndNatural (Array Natural, BigNat)
  --> @HIDDEN
  type NaturalDivision = | NaturalDivision (BigNat, BigNat)
  --> @HIDDEN
  type IntegerDivision = | IntegerDivision (BigInt, BigInt)

  --> @HIDDEN
  let integer_to_natural : Integer -> Natural = fn value =>
    if value < 0 then
      0n
    else
      (wasm : Natural) => (
        get value
        struct.get $integer 0
        struct.new $natural
      )

  --> @HIDDEN
  let reverse_into = fn remaining acc =>
    match remaining with
      | [] => acc
      | [head, ..tail] => reverse_into tail [head, ..acc]

  --> @HIDDEN
  let reverse = fn values => reverse_into values []

  --> @HIDDEN
  let length = fn values =>
    match values with
      | [] => 0n
      | [_, ..tail] => 1n + (length tail)

  --> @HIDDEN
  let trim_msf_zeros = fn values =>
    match values with
      | [] => []
      | [head, ..tail] =>
        if head == 0n then
          trim_msf_zeros tail
        else
          [head, ..tail]

  --> @HIDDEN
  let normalize_digit_array = fn digits =>
    reverse (trim_msf_zeros (reverse digits))

  --> @HIDDEN
  let natural_digits = fn value =>
    match value with
      | Nat digits => digits

  --> @HIDDEN
  let make_natural = fn digits => Nat (normalize_digit_array digits)

  --> @HIDDEN
  let natural_zero = make_natural []
  --> @HIDDEN
  let natural_one = make_natural [1n]

  --> @HIDDEN
  let magnitude_digits_from_natural = fn value =>
    let quotient = value / 10n in
    let remainder = value mod 10n in
    if quotient == 0n then
      if remainder == 0n then [] else [remainder]
    else
      [remainder, ..(magnitude_digits_from_natural quotient)]

  (*>
  Converts a primitive natural into `BigNat`.

  - Arguments:
    - `value`: Primitive `Natural` value.
  - Returns: Equivalent arbitrary-precision natural.
  *)
  let natural_from_natural = fn value =>
    make_natural (magnitude_digits_from_natural value)

  (*>
  Converts a primitive integer into `BigNat`.

  - Arguments:
    - `value`: Primitive `Integer` value.
  - Returns: Equivalent `BigNat`; negative inputs saturate to zero.
  *)
  let natural_from_integer = fn value =>
    if value < 0 then
      natural_zero
    else
      natural_from_natural (integer_to_natural value)

  --> @HIDDEN
  let natural_is_zero = fn value =>
    match natural_digits value with
      | [] => true
      | _ => false

  --> @HIDDEN
  let compare_digits_msf = fn left right =>
    match left with
      | [] => 0
      | [left_head, ..left_tail] =>
        match right with
          | [] => 0
          | [right_head, ..right_tail] =>
            if left_head < right_head then
              -1
            else if left_head > right_head then
              1
            else
              compare_digits_msf left_tail right_tail

  --> @HIDDEN
  let natural_compare = fn left right =>
    let left_digits = natural_digits left in
    let right_digits = natural_digits right in
    let left_length = length left_digits in
    let right_length = length right_digits in
    if left_length < right_length then
      -1
    else if left_length > right_length then
      1
    else
      compare_digits_msf (reverse left_digits) (reverse right_digits)

  --> @HIDDEN
  let comparison_is_negative = fn value => value < 0
  --> @HIDDEN
  let comparison_is_zero = fn value => value == 0
  --> @HIDDEN
  let comparison_is_positive = fn value => value > 0

  --> @HIDDEN
  let add_digit_arrays = fn carry left right =>
    match left with
      | [] =>
        match right with
          | [] =>
            if carry == 0n then
              []
            else
              [carry]
          | [right_head, ..right_tail] =>
            let total = right_head + carry in
            [(total mod 10n), ..(add_digit_arrays (total / 10n) [] right_tail)]
      | [left_head, ..left_tail] =>
        match right with
          | [] =>
            let total = left_head + carry in
            [(total mod 10n), ..(add_digit_arrays (total / 10n) left_tail [])]
          | [right_head, ..right_tail] =>
            let total = left_head + right_head + carry in
            [(total mod 10n), ..(add_digit_arrays (total / 10n) left_tail right_tail)]

  --> @HIDDEN
  let natural_add = fn left right =>
    make_natural (add_digit_arrays 0n (natural_digits left) (natural_digits right))

  --> @HIDDEN
  let subtract_digit_arrays_exact = fn borrow left right =>
    match left with
      | [] => []
      | [left_head, ..left_tail] =>
        match right with
          | [] =>
            if left_head < borrow then
              [((left_head + 10n) - borrow), ..(subtract_digit_arrays_exact 1n left_tail [])]
            else
              [(left_head - borrow), ..(subtract_digit_arrays_exact 0n left_tail [])]
          | [right_head, ..right_tail] =>
            let subtrahend = right_head + borrow in
            if left_head < subtrahend then
              [((left_head + 10n) - subtrahend), ..(subtract_digit_arrays_exact 1n left_tail right_tail)]
            else
              [(left_head - subtrahend), ..(subtract_digit_arrays_exact 0n left_tail right_tail)]

  --> @HIDDEN
  let natural_sub_exact = fn left right =>
    make_natural (subtract_digit_arrays_exact 0n (natural_digits left) (natural_digits right))

  --> @HIDDEN
  let natural_sub = fn left right =>
    if comparison_is_negative (natural_compare left right) then
      natural_zero
    else
      natural_sub_exact left right

  --> @HIDDEN
  let multiply_digit_array_small = fn carry factor digits =>
    match digits with
      | [] =>
        if carry == 0n then
          []
        else
          [carry]
      | [head, ..tail] =>
        let product = (head * factor) + carry in
        [(product mod 10n), ..(multiply_digit_array_small (product / 10n) factor tail)]

  --> @HIDDEN
  let natural_mul_small = fn value factor =>
    if (factor < 1n) or (natural_is_zero value) then
      natural_zero
    else
      make_natural (multiply_digit_array_small 0n factor (natural_digits value))

  --> @HIDDEN
  let natural_add_small = fn value addend =>
    if addend == 0n then
      value
    else
      natural_add value (natural_from_natural addend)

  --> @HIDDEN
  let prepend_zeros = fn count digits =>
    if count < 1n then
      digits
    else
      prepend_zeros (count - 1n) [0n, ..digits]

  --> @HIDDEN
  let multiply_with_digits = fn left_digits right_digits shift acc =>
    match right_digits with
      | [] => acc
      | [right_head, ..right_tail] =>
        let partial = prepend_zeros shift (multiply_digit_array_small 0n right_head left_digits) in
        let next = natural_add acc (make_natural partial) in
        multiply_with_digits left_digits right_tail (shift + 1n) next

  --> @HIDDEN
  let natural_mul = fn left right =>
    if (natural_is_zero left) or (natural_is_zero right) then
      natural_zero
    else
      multiply_with_digits (natural_digits left) (natural_digits right) 0n natural_zero

  --> @HIDDEN
  let divide_msf_by_small : Array Natural -> Natural -> Natural -> Array Natural -> DigitsAndRemainder =
    fn msf_digits divisor remainder quotient_reversed =>
    match msf_digits with
      | [] => DigitsAndRemainder (quotient_reversed, remainder)
      | [digit, ..tail] =>
        let current = (remainder * 10n) + digit in
        let quotient_digit = current / divisor in
        let next_remainder = current mod divisor in
        divide_msf_by_small tail divisor next_remainder [quotient_digit, ..quotient_reversed]

  --> @HIDDEN
  let natural_div_mod_small : BigNat -> Natural -> NaturalAndRemainder = fn value divisor =>
    if divisor == 0n then
      NaturalAndRemainder (natural_zero, 0n)
    else
      let msf_digits = reverse (natural_digits value) in
      let result = divide_msf_by_small msf_digits divisor 0n [] in
      match result with
        | DigitsAndRemainder (quotient_digits, remainder) =>
          NaturalAndRemainder (make_natural quotient_digits, remainder)

  --> @HIDDEN
  let choose_quotient_digit = fn remainder divisor candidate =>
    if candidate == 0n then
      0n
    else
      let product = natural_mul_small divisor candidate in
      if (natural_compare product remainder) < 1 then
        candidate
      else
        choose_quotient_digit remainder divisor (candidate - 1n)

  --> @HIDDEN
  let divide_msf : Array Natural -> BigNat -> BigNat -> Array Natural -> DigitsAndNatural =
    fn msf_digits divisor remainder quotient_reversed =>
    match msf_digits with
      | [] => DigitsAndNatural (quotient_reversed, remainder)
      | [digit, ..tail] =>
        let shifted = natural_add_small (natural_mul_small remainder 10n) digit in
        let quotient_digit = choose_quotient_digit shifted divisor 9n in
        let product = natural_mul_small divisor quotient_digit in
        let next_remainder = natural_sub_exact shifted product in
        divide_msf tail divisor next_remainder [quotient_digit, ..quotient_reversed]

  --> @HIDDEN
  let natural_div_mod : BigNat -> BigNat -> NaturalDivision = fn left right =>
    if natural_is_zero right then
      NaturalDivision (natural_zero, natural_zero)
    else if comparison_is_negative (natural_compare left right) then
      NaturalDivision (natural_zero, left)
    else
      let msf_digits = reverse (natural_digits left) in
      let result = divide_msf msf_digits right natural_zero [] in
      match result with
        | DigitsAndNatural (quotient_digits, remainder) =>
          NaturalDivision (make_natural quotient_digits, remainder)

  --> @HIDDEN
  let natural_div = fn left right =>
    match natural_div_mod left right with
      | NaturalDivision (quotient, _) => quotient

  --> @HIDDEN
  let natural_rem = fn left right =>
    match natural_div_mod left right with
      | NaturalDivision (_, remainder) => remainder

  --> @HIDDEN
  let natural_to_bits = fn value =>
    if natural_is_zero value then
      []
    else
      let division = natural_div_mod_small value 2n in
      match division with
        | NaturalAndRemainder (quotient, remainder) =>
          [remainder, ..(natural_to_bits quotient)]

  --> @HIDDEN
  let natural_from_bits = fn bits =>
    let from_msf = fn msf_bits acc =>
      match msf_bits with
        | [] => acc
        | [bit, ..tail] =>
          from_msf tail (natural_add_small (natural_mul_small acc 2n) bit)
    in
      from_msf (reverse bits) natural_zero

  --> @HIDDEN
  let natural_bit_length = fn value =>
    length (natural_to_bits value)

  --> @HIDDEN
  let flip_bit = fn bit => if bit == 0n then 1n else 0n

  --> @HIDDEN
  let bits_not = fn bits =>
    match bits with
      | [] => []
      | [bit, ..tail] => [flip_bit bit, ..(bits_not tail)]

  --> @HIDDEN
  let bits_add_one = fn bits =>
    let loop = fn carry remaining =>
      match remaining with
        | [] =>
          if carry == 0n then [] else [1n]
        | [bit, ..tail] =>
          if carry == 0n then
            [bit, ..tail]
          else
            let total = bit + 1n in
            [(total mod 2n), ..(loop (total / 2n) tail)]
    in
      loop 1n bits

  --> @HIDDEN
  let truncate_bits = fn width bits =>
    if width < 1n then
      []
    else
      match bits with
        | [] => []
        | [bit, ..tail] => [bit, ..(truncate_bits (width - 1n) tail)]

  --> @HIDDEN
  let pad_msf_zeros = fn count msf_bits =>
    if count < 1n then
      msf_bits
    else
      pad_msf_zeros (count - 1n) [0n, ..msf_bits]

  --> @HIDDEN
  let pad_bits_to = fn width bits =>
    let current = length bits in
    if current > width then
      truncate_bits width bits
    else if current == width then
      bits
    else
      reverse (pad_msf_zeros (width - current) (reverse bits))

  --> @HIDDEN
  let most_significant_bit = fn bits =>
    match bits with
      | [] => 0n
      | [bit] => bit
      | [_, ..tail] => most_significant_bit tail

  --> @HIDDEN
  let combine_bits_with = fn op left right =>
    match left with
      | [] =>
        match right with
          | [] => []
          | [right_bit, ..right_tail] =>
            [op 0n right_bit, ..(combine_bits_with op [] right_tail)]
      | [left_bit, ..left_tail] =>
        match right with
          | [] =>
            [op left_bit 0n, ..(combine_bits_with op left_tail [])]
          | [right_bit, ..right_tail] =>
            [op left_bit right_bit, ..(combine_bits_with op left_tail right_tail)]

  --> @HIDDEN
  let bit_and = fn left right => if (left == 1n) and (right == 1n) then 1n else 0n
  --> @HIDDEN
  let bit_or = fn left right => if (left == 1n) or (right == 1n) then 1n else 0n
  --> @HIDDEN
  let bit_xor = fn left right => if left == right then 0n else 1n

  --> @HIDDEN
  let natural_bitwise_with = fn op left right =>
    natural_from_bits (combine_bits_with op (natural_to_bits left) (natural_to_bits right))

  --> @HIDDEN
  let natural_bit_not = fn value =>
    natural_from_bits (bits_not (natural_to_bits value))

  --> @HIDDEN
  let integer_is_zero = fn value =>
    match value with
      | BigZero => true
      | _ => false

  --> @HIDDEN
  let integer_is_negative = fn value =>
    match value with
      | BigNegative _ => true
      | _ => false

  --> @HIDDEN
  let integer_abs = fn value =>
    match value with
      | BigZero => natural_zero
      | BigPositive magnitude => magnitude
      | BigNegative magnitude => magnitude

  --> @HIDDEN
  let integer_from_parts = fn negative magnitude =>
    if natural_is_zero magnitude then
      BigZero
    else if negative then
      BigNegative magnitude
    else
      BigPositive magnitude

  (*>
  Converts a `BigNat` into a non-negative `BigInt`.

  - Arguments:
    - `value`: Natural magnitude.
  - Returns: Non-negative `BigInt` with the same magnitude.
  *)
  let integer_from_natural = fn value =>
    integer_from_parts false value

  (*>
  Converts a primitive integer into `BigInt`.

  - Arguments:
    - `value`: Primitive `Integer` value.
  - Returns: Equivalent arbitrary-precision integer.

  ```hc
  let big = big-num::integer_from_integer 9223372036854775807
  ```
  *)
  let integer_from_integer = fn value =>
    if value == 0 then
      BigZero
    else if value < 0 then
      BigNegative (natural_from_natural (integer_to_natural (0 - value)))
    else
      BigPositive (natural_from_natural (integer_to_natural value))

  --> @HIDDEN
  let integer_negate = fn value =>
    match value with
      | BigZero => BigZero
      | BigPositive magnitude => BigNegative magnitude
      | BigNegative magnitude => BigPositive magnitude

  --> @HIDDEN
  let integer_compare = fn left right =>
    match left with
      | BigNegative left_magnitude =>
        match right with
          | BigNegative right_magnitude =>
            0 - (natural_compare left_magnitude right_magnitude)
          | _ => -1
      | BigZero =>
        match right with
          | BigNegative _ => 1
          | BigZero => 0
          | BigPositive _ => -1
      | BigPositive left_magnitude =>
        match right with
          | BigNegative _ => 1
          | BigZero => 1
          | BigPositive right_magnitude =>
            natural_compare left_magnitude right_magnitude

  --> @HIDDEN
  let integer_add = fn left right =>
    match left with
      | BigZero => right
      | BigPositive left_magnitude =>
        match right with
          | BigZero => left
          | BigPositive right_magnitude =>
            integer_from_parts false (natural_add left_magnitude right_magnitude)
          | BigNegative right_magnitude =>
            let ordering = natural_compare left_magnitude right_magnitude in
            if comparison_is_zero ordering then
              BigZero
            else if comparison_is_positive ordering then
              integer_from_parts false (natural_sub_exact left_magnitude right_magnitude)
            else
              integer_from_parts true (natural_sub_exact right_magnitude left_magnitude)
      | BigNegative left_magnitude =>
        match right with
          | BigZero => left
          | BigNegative right_magnitude =>
            integer_from_parts true (natural_add left_magnitude right_magnitude)
          | BigPositive right_magnitude =>
            let ordering = natural_compare left_magnitude right_magnitude in
            if comparison_is_zero ordering then
              BigZero
            else if comparison_is_positive ordering then
              integer_from_parts true (natural_sub_exact left_magnitude right_magnitude)
            else
              integer_from_parts false (natural_sub_exact right_magnitude left_magnitude)

  --> @HIDDEN
  let integer_sub = fn left right =>
    integer_add left (integer_negate right)

  --> @HIDDEN
  let integer_mul = fn left right =>
    if (integer_is_zero left) or (integer_is_zero right) then
      BigZero
    else
      let negative = (integer_is_negative left) xor (integer_is_negative right) in
      integer_from_parts negative (natural_mul (integer_abs left) (integer_abs right))

  --> @HIDDEN
  let integer_div_rem : BigInt -> BigInt -> IntegerDivision = fn left right =>
    if integer_is_zero right then
      IntegerDivision (BigZero, BigZero)
    else if integer_is_zero left then
      IntegerDivision (BigZero, BigZero)
    else
      let division = natural_div_mod (integer_abs left) (integer_abs right) in
      match division with
        | NaturalDivision (quotient_magnitude, remainder_magnitude) =>
          let quotient_negative = (integer_is_negative left) xor (integer_is_negative right) in
          let remainder_negative = integer_is_negative left in
          IntegerDivision (
            integer_from_parts quotient_negative quotient_magnitude,
            integer_from_parts remainder_negative remainder_magnitude
          )

  --> @HIDDEN
  let integer_div = fn left right =>
    match integer_div_rem left right with
      | IntegerDivision (quotient, _) => quotient

  --> @HIDDEN
  let integer_rem = fn left right =>
    match integer_div_rem left right with
      | IntegerDivision (_, remainder) => remainder

  --> @HIDDEN
  let integer_bit_width = fn value =>
    let magnitude_width = natural_bit_length (integer_abs value) in
    if magnitude_width == 0n then
      1n
    else
      magnitude_width + 1n

  --> @HIDDEN
  let integer_to_twos_bits = fn value width =>
    match value with
      | BigZero => pad_bits_to width []
      | BigPositive magnitude => pad_bits_to width (natural_to_bits magnitude)
      | BigNegative magnitude =>
        let padded = pad_bits_to width (natural_to_bits magnitude) in
        truncate_bits width (bits_add_one (bits_not padded))

  --> @HIDDEN
  let integer_from_twos_bits = fn bits =>
    let width = length bits in
    if (most_significant_bit bits) == 0n then
      integer_from_parts false (natural_from_bits bits)
    else
      let magnitude_bits = truncate_bits width (bits_add_one (bits_not bits)) in
      integer_from_parts true (natural_from_bits magnitude_bits)

  --> @HIDDEN
  let integer_bitwise_binary = fn op left right =>
    let width = max (integer_bit_width left) (integer_bit_width right) in
    let left_bits = integer_to_twos_bits left width in
    let right_bits = integer_to_twos_bits right width in
    integer_from_twos_bits (truncate_bits width (combine_bits_with op left_bits right_bits))

  --> @HIDDEN
  let integer_bit_not = fn value =>
    let width = integer_bit_width value in
    let bits = integer_to_twos_bits value width in
    integer_from_twos_bits (truncate_bits width (bits_not bits))

  --> @HIDDEN
  let digit_to_string = bundle::natural::digit_to_string

  --> @HIDDEN
  let show_digits_msf = fn digits =>
    match digits with
      | [] => ""
      | [digit, ..tail] =>
        (digit_to_string digit) + (show_digits_msf tail)

  --> @HIDDEN
  let show_natural = fn value =>
    let msf_digits = reverse (natural_digits value) in
    match msf_digits with
      | [] => "0"
      | _ => show_digits_msf msf_digits

  --> @HIDDEN
  let show_integer = fn value =>
    match value with
      | BigZero => "0"
      | BigPositive magnitude => show_natural magnitude
      | BigNegative magnitude => "-" + (show_natural magnitude)

  (*>
  Returns `10^exponent` as a `BigNat`.

  - Arguments:
    - `exponent`: Decimal exponent.
  - Returns: `1` for exponent `< 1`, otherwise `10^exponent`.

  ```hc
  let trillion = big-num::natural_pow10 12
  ```
  *)
  let natural_pow10 = fn exponent =>
    if exponent < 1 then
      natural_one
    else
      natural_mul_small (natural_pow10 (exponent - 1)) 10n

  (*>
  Short alias for `natural_from_integer`.

  - Arguments:
    - `value`: Primitive integer.
  - Returns: Converted `BigNat`.
  *)
  let natural = natural_from_integer

  (*>
  Short alias for `integer_from_integer`.

  - Arguments:
    - `value`: Primitive integer.
  - Returns: Converted `BigInt`.
  *)
  let integer = integer_from_integer

  --> @HIDDEN
  impl bundle::Default BigNat =
    let default = natural_zero
  end

  --> @HIDDEN
  impl bundle::show::Show BigNat =
    let show = fn value => show_natural value
  end

  --> @HIDDEN
  impl ops::Equal BigNat =
    let [==] = fn left right => comparison_is_zero (natural_compare left right)
  end

  --> @HIDDEN
  impl ops::Compare BigNat =
    let [<] = fn left right => comparison_is_negative (natural_compare left right)
    let [>] = fn left right => comparison_is_positive (natural_compare left right)
  end

  --> @HIDDEN
  impl ops::Add BigNat =
    let [+] = fn left right => natural_add left right
  end

  --> @HIDDEN
  impl ops::Subtract BigNat =
    let [-] = fn left right => natural_sub left right
    let [~] = fn _ => natural_zero
  end

  --> @HIDDEN
  impl ops::Multiply BigNat =
    let [*] = fn left right => natural_mul left right
  end

  --> @HIDDEN
  impl ops::Divide BigNat =
    let [/] = fn left right => natural_div left right
  end

  --> @HIDDEN
  impl ops::Remainder BigNat =
    let [mod] = fn left right => natural_rem left right
  end

  --> @HIDDEN
  impl ops::Bitwise BigNat =
    let [and] = fn left right => natural_bitwise_with bit_and left right
    let [or] = fn left right => natural_bitwise_with bit_or left right
    let [xor] = fn left right => natural_bitwise_with bit_xor left right
    let [not] = fn value => natural_bit_not value
  end

  --> @HIDDEN
  impl bundle::Default BigInt =
    let default = BigZero
  end

  --> @HIDDEN
  impl bundle::show::Show BigInt =
    let show = fn value => show_integer value
  end

  --> @HIDDEN
  impl ops::Equal BigInt =
    let [==] = fn left right => comparison_is_zero (integer_compare left right)
  end

  --> @HIDDEN
  impl ops::Compare BigInt =
    let [<] = fn left right => comparison_is_negative (integer_compare left right)
    let [>] = fn left right => comparison_is_positive (integer_compare left right)
  end

  --> @HIDDEN
  impl ops::Add BigInt =
    let [+] = fn left right => integer_add left right
  end

  --> @HIDDEN
  impl ops::Subtract BigInt =
    let [-] = fn left right => integer_sub left right
    let [~] = fn value => integer_negate value
  end

  --> @HIDDEN
  impl ops::Multiply BigInt =
    let [*] = fn left right => integer_mul left right
  end

  --> @HIDDEN
  impl ops::Divide BigInt =
    let [/] = fn left right => integer_div left right
  end

  --> @HIDDEN
  impl ops::Remainder BigInt =
    let [mod] = fn left right => integer_rem left right
  end

  --> @HIDDEN
  impl ops::Bitwise BigInt =
    let [and] = fn left right => integer_bitwise_binary bit_and left right
    let [or] = fn left right => integer_bitwise_binary bit_or left right
    let [xor] = fn left right => integer_bitwise_binary bit_xor left right
    let [not] = fn value => integer_bit_not value
  end
end
