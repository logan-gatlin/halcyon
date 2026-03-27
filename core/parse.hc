module parse =
  use bundle
  use bundle::ops
  use bundle::opt

  (*>
  Parses a value from plain string text.

  Implement `Parse` for types that can be reconstructed from user input.
  Parsing failures return `None`.

  ```hc
  let parsed : Option Integer = parse "42"
  ```
  *)
  trait Parse: t =
    let parse : String -> bundle::opt::Option t
  end

  --> @HIDDEN
  let ascii_zero = 48
  --> @HIDDEN
  let ascii_nine = 57
  --> @HIDDEN
  let ascii_plus = 43
  --> @HIDDEN
  let ascii_minus = 45
  --> @HIDDEN
  let ascii_dot = 46

  --> @HIDDEN
  let string_length : String -> Integer = fn value => (wasm : Integer) => (
    get value
    array.len
    i64.extend_i32_u
    struct.new $integer
  )

  --> @HIDDEN
  let string_code_at : String -> Integer -> Integer = fn value index => (wasm : Integer) => (
    (local $length i32)
    (local $length_i64 i64)
    (local $index_i64 i64)
    (local $index_i32 i32)
    (local $result i64)

    i64.const -1
    set $result

    get value
    array.len
    set $length

    get $length
    i64.extend_i32_u
    set $length_i64

    get index
    struct.get $integer 0
    set $index_i64

    get $index_i64
    i64.const 0
    i64.lt
    if
    else
      get $index_i64
      get $length_i64
      i64.lt
      if
        get $index_i64
        i32.wrap_i64
        set $index_i32

        get value
        get $index_i32
        array.get i8
        i32.const 255
        i32.and
        i64.extend_i32_u
        set $result
      end
    end

    get $result
    struct.new $integer
  )

  --> @HIDDEN
  let natural_from_integer : Integer -> Natural = fn value => (wasm : Natural) => (
    get value
    struct.get $integer 0
    struct.new $natural
  )

  --> @HIDDEN
  let glyph_from_code : Integer -> Glyph = fn code => (wasm : Glyph) => (
    get code
    struct.get $integer 0
    i32.wrap_i64
    struct.new $word
  )

  --> @HIDDEN
  let is_ascii_digit = fn code => (code >= ascii_zero) and (code <= ascii_nine)

  --> @HIDDEN
  let digit_value = fn code => code - ascii_zero

  --> @HIDDEN
  let parse_unsigned_integer_with_length = fn text length index acc =>
    if index == length then
      Some acc
    else
      let code = string_code_at text index in
      if is_ascii_digit code then
        parse_unsigned_integer_with_length text length (index + 1) ((acc * 10) + (digit_value code))
      else
        None

  --> @HIDDEN
  let parse_real_integer_part = fn text length index acc consumed =>
    if index == length then
      if consumed then Some (index, acc) else None
    else
      let code = string_code_at text index in
      if is_ascii_digit code then
        parse_real_integer_part text length (index + 1) ((acc * 10) + (digit_value code)) true
      else if consumed then
        Some (index, acc)
      else
        None

  --> @HIDDEN
  let parse_real_fractional_part = fn text length index acc scale consumed =>
    if index == length then
      if consumed then Some acc else None
    else
      let code = string_code_at text index in
      if is_ascii_digit code then
        let digit_as_real = bundle::real::integer_to_real (digit_value code) in
        parse_real_fractional_part
          text
          length
          (index + 1)
          (acc + (digit_as_real / scale))
          (scale * 10.0)
          true
      else
        None

  --> @HIDDEN
  let parse_finite_real = fn text =>
    let length = string_length text in
    if length == 0 then
      None
    else
      let first = string_code_at text 0 in
      let sign_and_start =
        if first == ascii_minus then
          if length == 1 then None else Some (-1, 1)
        else if first == ascii_plus then
          if length == 1 then None else Some (1, 1)
        else
          Some (1, 0)
      in
      match sign_and_start with
        | None => None
        | Some (sign, start_index) =>
          match parse_real_integer_part text length start_index 0 false with
            | None => None
            | Some (next_index, integer_part) =>
              let integer_as_real = bundle::real::integer_to_real integer_part in
              let signed_integer = if sign < 0 then 0.0 - integer_as_real else integer_as_real in
              if next_index == length then
                Some signed_integer
              else if (string_code_at text next_index) == ascii_dot then
                match parse_real_fractional_part text length (next_index + 1) 0.0 10.0 false with
                  | None => None
                  | Some fraction =>
                    let signed_fraction = if sign < 0 then 0.0 - fraction else fraction in
                    Some (signed_integer + signed_fraction)
              else
                None

  --> @HIDDEN
  impl Parse () =
    let parse = fn value =>
      if value == "()" then Some () else None
  end

  --> @HIDDEN
  impl Parse Boolean =
    let parse = fn value =>
      if value == "true" then
        Some true
      else if value == "false" then
        Some false
      else
        None
  end

  --> @HIDDEN
  impl Parse Integer =
    let parse = fn value =>
      let length = string_length value in
      if length == 0 then
        None
      else
        let first = string_code_at value 0 in
        if first == ascii_minus then
          if length == 1 then
            None
          else
            match parse_unsigned_integer_with_length value length 1 0 with
              | Some magnitude => Some (0 - magnitude)
              | None => None
        else if first == ascii_plus then
          if length == 1 then
            None
          else
            parse_unsigned_integer_with_length value length 1 0
        else
          parse_unsigned_integer_with_length value length 0 0
  end

  --> @HIDDEN
  impl Parse Natural =
    let parse = fn value =>
      let length = string_length value in
      if length == 0 then
        None
      else
        match parse_unsigned_integer_with_length value length 0 0 with
          | Some parsed => Some (natural_from_integer parsed)
          | None => None
  end

  --> @HIDDEN
  impl Parse Real =
    let parse = fn value =>
      if value == "nan" then
        Some (0.0 / 0.0)
      else if value == "inf" then
        Some (1.0 / 0.0)
      else if value == "-inf" then
        Some ((0.0 - 1.0) / 0.0)
      else
        parse_finite_real value
  end

  --> @HIDDEN
  impl Parse String =
    let parse = fn value => Some value
  end

  --> @HIDDEN
  impl Parse Glyph =
    let parse = fn value =>
      if string_length value == 1 then
        Some (glyph_from_code (string_code_at value 0))
      else
        None
  end
end
