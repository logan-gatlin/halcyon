module string =
  use bundle
  use bundle::ops

  (*>
  Empty string value.

  - Arguments: none.
  - Returns: `""`.
  *)
  let empty = ""

  (*>
  Concatenates two strings.

  - Arguments:
    - `left`: Prefix string.
    - `right`: Suffix string.
  - Returns: `left + right`.
  *)
  let concat = fn left right => left + right

  (*>
  Replaces all non-overlapping `needle` occurrences in `value`.

  - Arguments:
    - `value`: Source text.
    - `needle`: Substring to search for.
    - `replacement`: Substring to insert for each match.
  - Returns: Updated string. If `needle` is empty, returns `value` unchanged.

  ```hc
  let out = string::replace "banana" "na" "xo"
  ```
  *)
  let replace : String -> String -> String -> String =
    fn value needle replacement => (wasm : String) => (
      (local $result $string)
      (local $next $string)
      (local $value_len i32)
      (local $needle_len i32)
      (local $replacement_len i32)
      (local $index i32)
      (local $needle_index i32)
      (local $match i32)
      (local $result_len i32)
      (local $value_byte i32)

      get value
      array.len
      set $value_len

      get needle
      array.len
      set $needle_len

      get replacement
      array.len
      set $replacement_len

      get value
      set $result

      get $needle_len
      i32.const 0
      i32.ne
      if
        i32.const 0
        array.new_default i8
        set $result

        i32.const 0
        set $index

        block
        loop
          get $index
          get $value_len
          i32.eq
          break.if 1

          i32.const 0
          set $match

          get $index
          get $needle_len
          i32.add
          get $value_len
          i32.le
          if
            i32.const 1
            set $match

            i32.const 0
            set $needle_index

            block
            loop
              get $needle_index
              get $needle_len
              i32.eq
              break.if 1

              get value
              get $index
              get $needle_index
              i32.add
              array.get i8

              get needle
              get $needle_index
              array.get i8
              i32.ne
              if
                i32.const 0
                set $match
                break 2
              end

              get $needle_index
              i32.const 1
              i32.add
              set $needle_index
              break 0
            end
            end
          end

          get $match
          i32.const 1
          i32.eq
          if
            get $result
            array.len
            set $result_len

            get $result_len
            get $replacement_len
            i32.add
            array.new_default i8
            set $next

            get $next
            i32.const 0
            get $result
            i32.const 0
            get $result_len
            array.copy i8 i8

            get $next
            get $result_len
            get replacement
            i32.const 0
            get $replacement_len
            array.copy i8 i8

            get $next
            set $result

            get $index
            get $needle_len
            i32.add
            set $index
          else
            get $result
            array.len
            set $result_len

            get $result_len
            i32.const 1
            i32.add
            array.new_default i8
            set $next

            get $next
            i32.const 0
            get $result
            i32.const 0
            get $result_len
            array.copy i8 i8

            get value
            get $index
            array.get i8
            set $value_byte

            get $next
            get $result_len
            get $value_byte
            array.new_fixed i8 1
            i32.const 0
            i32.const 1
            array.copy i8 i8

            get $next
            set $result

            get $index
            i32.const 1
            i32.add
            set $index
          end

          break 0
        end
        end
      end

      get $result
    )

  (*>
  Splits `value` on each non-overlapping `delimiter`.

  - Arguments:
    - `value`: Source text.
    - `delimiter`: Separator string.
  - Returns: Array of segments. If `delimiter` is empty, returns `[value]`.

  ```hc
  let parts = string::split "a--b--c" "--"
  ```
  *)
  let split : String -> String -> Array String =
    fn value delimiter => (wasm : Array String) => (
      (local $result (array any))
      (local $next_result (array any))
      (local $value_len i32)
      (local $delimiter_len i32)
      (local $index i32)
      (local $delimiter_index i32)
      (local $match i32)
      (local $segment_start i32)
      (local $segment_len i32)
      (local $result_len i32)
      (local $segment $string)

      get value
      array.len
      set $value_len

      get delimiter
      array.len
      set $delimiter_len

      i32.const 0
      array.new_default any
      set $result

      get $delimiter_len
      i32.const 0
      i32.eq
      if
        i32.const 1
        array.new_default any
        set $result

        get $result
        i32.const 0
        get value
        array.new_fixed any 1
        i32.const 0
        i32.const 1
        array.copy any any
      else
        i32.const 0
        set $segment_start

        i32.const 0
        set $index

        block
        loop
          get $index
          get $delimiter_len
          i32.add
          get $value_len
          i32.gt
          break.if 1

          i32.const 1
          set $match

          i32.const 0
          set $delimiter_index

          block
          loop
            get $delimiter_index
            get $delimiter_len
            i32.eq
            break.if 1

            get value
            get $index
            get $delimiter_index
            i32.add
            array.get i8

            get delimiter
            get $delimiter_index
            array.get i8
            i32.ne
            if
              i32.const 0
              set $match
              break 2
            end

            get $delimiter_index
            i32.const 1
            i32.add
            set $delimiter_index
            break 0
          end
          end

          get $match
          i32.const 1
          i32.eq
          if
            get $index
            get $segment_start
            i32.sub
            set $segment_len

            get $segment_len
            array.new_default i8
            set $segment

            get $segment
            i32.const 0
            get value
            get $segment_start
            get $segment_len
            array.copy i8 i8

            get $result
            array.len
            set $result_len

            get $result_len
            i32.const 1
            i32.add
            array.new_default any
            set $next_result

            get $next_result
            i32.const 0
            get $result
            i32.const 0
            get $result_len
            array.copy any any

            get $next_result
            get $result_len
            get $segment
            array.new_fixed any 1
            i32.const 0
            i32.const 1
            array.copy any any

            get $next_result
            set $result

            get $index
            get $delimiter_len
            i32.add
            set $segment_start

            get $segment_start
            set $index
          else
            get $index
            i32.const 1
            i32.add
            set $index
          end

          break 0
        end
        end

        get $value_len
        get $segment_start
        i32.sub
        set $segment_len

        get $segment_len
        array.new_default i8
        set $segment

        get $segment
        i32.const 0
        get value
        get $segment_start
        get $segment_len
        array.copy i8 i8

        get $result
        array.len
        set $result_len

        get $result_len
        i32.const 1
        i32.add
        array.new_default any
        set $next_result

        get $next_result
        i32.const 0
        get $result
        i32.const 0
        get $result_len
        array.copy any any

        get $next_result
        get $result_len
        get $segment
        array.new_fixed any 1
        i32.const 0
        i32.const 1
        array.copy any any

        get $next_result
        set $result
      end

      get $result
    )

  (*>
  Checks whether a string is empty.

  - Arguments:
    - `value`: String to inspect.
  - Returns: `true` when `value == ""`.
  *)
  let is_empty = fn value => value == ""

  (*>
  Checks whether a string is non-empty.

  - Arguments:
    - `value`: String to inspect.
  - Returns: `true` when `value != ""`.
  *)
  let non_empty = fn value => value != ""

  (*>
  Escapes control characters and quotes for display.

  - Arguments:
    - `value`: Raw string.
  - Returns: Escaped string content without surrounding quotes.
  *)
  let escape = fn value =>
    let escaped_backslash = replace value "\\" "\\\\" in
    let escaped_quote = replace escaped_backslash "\"" "\\\"" in
    let escaped_newline = replace escaped_quote "\n" "\\n" in
    let escaped_carriage_return = replace escaped_newline "\r" "\\r" in
    let escaped_tab = replace escaped_carriage_return "\t" "\\t" in
    let escaped_backspace = replace escaped_tab "\b" "\\b" in
    replace escaped_backspace "\0" "\\0"

  (*>
  Produces a quoted, escaped string literal representation.

  - Arguments:
    - `value`: Raw string.
  - Returns: String wrapped in double quotes with escape sequences.

  ```hc
  let rendered = string::quote "a\nb"
  ```
  *)
  let quote = fn value => "\"" + (escape value) + "\""

  --> @HIDDEN
  impl bundle::Default bundle::String =
    let default = ""
  end

  --> @HIDDEN
  impl bundle::show::Show bundle::String =
    let show = fn value => quote value
  end
end
