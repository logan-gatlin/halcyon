module string =
  use core
  use core::ops
  let empty = ""
  let concat = fn left right => left + right

  -- Parameter order: replace value needle replacement.
  -- Replaces all non-overlapping occurrences of `needle`; empty `needle` keeps `value` unchanged.
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

  -- Parameter order: split value delimiter.
  -- Splits `value` on each non-overlapping `delimiter`; empty `delimiter` returns [value].
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

  let is_empty = fn value => value == ""
  let non_empty = fn value => value != ""

  impl bundle::Default bundle::String =
    let default = ""
  end
end
