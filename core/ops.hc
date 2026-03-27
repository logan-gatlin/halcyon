module ops =
  use bundle

  (*>
  Implements the `[==]` operator for a type.
  The `[!=]` operator is implemented for all types that implement `Equal`.
  The `[<=]` and `[>=]` operators are implemented for all types which implement both `Equal` and `Compare`
  *)
  trait Equal : a =
    let [==] : a -> a -> Boolean
  end

  (*>
  Implements the `[<]` and `[>]` operators for a type.
  The `[<=]` and `[>=]` operators are implemented for all types which implement both `Equal` and `Compare`
  *)
  trait Compare : a =
    let [<] : a -> a -> Boolean
    let [>] : a -> a -> Boolean
  end

  (*>
  Implements the `[+]` operator for a type.
  *)
  trait Add : a =
    let [+] : a -> a -> a
  end

  (*>
  Implements the `[-]` (`a - b`) and `[~]` (`-a`) operators for a type.
  These are typically defined as subtraction and negation respectively.
  *)
  trait Subtract : a =
    let [-] : a -> a -> a
    let [~] : a -> a
  end

  (*>
  Implements the `[*]` operator for a type.
  This is typically defined as multiplication.
  *)
  trait Multiply : a =
    let [*] : a -> a -> a
  end

  (*>
  Implements the `[/]` operator for a type.
  This is typically defined as division.
  *)
  trait Divide : a =
    let [/] : a -> a -> a
  end

  (*>
  Implements the `[mod]` operator for a type.
  This is typically defined as the modulus operation.
  *)
  trait Remainder : a =
    let [mod] : a -> a -> a
  end

  (*>
  Defines the `[and]`, `[or]`, `[xor]`, and `[not]` operators for a type.
  These are typically defined as bitwise operations.
  *)
  trait Bitwise : a =
    let [and] : a -> a -> a
    let [or] : a -> a -> a
    let [xor] : a -> a -> a
    let [not] : a -> a
  end


  --> @HIDDEN
  impl Equal () =
    let [==] = fn _ _ => (wasm : Boolean) => (
      i32.const 1
      struct.new $word
    )
  end

  --> @HIDDEN
  impl Compare () =
    let [<] = fn _ _ => (wasm : Boolean) => (
      i32.const 0
      struct.new $word
    )

    let [>] = fn _ _ => (wasm : Boolean) => (
      i32.const 0
      struct.new $word
    )
  end

  --> @HIDDEN
  impl Equal Integer =
    let [==] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.eq
      struct.new $word
    )
  end

  --> @HIDDEN
  impl Compare Integer =
    let [<] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.lt
      struct.new $word
    )

    let [>] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.gt
      struct.new $word
    )
  end

  --> @HIDDEN
  impl Add Integer =
    let [+] = fn left right => (wasm : Integer) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.add
      struct.new $integer
    )
  end

  --> @HIDDEN
  impl Subtract Integer =
    let [-] = fn left right => (wasm : Integer) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.sub
      struct.new $integer
    )

    let [~] = fn value => (wasm : Integer) => (
      i64.const 0
      get value
      struct.get $integer 0
      i64.sub
      struct.new $integer
    )
  end

  --> @HIDDEN
  impl Multiply Integer =
    let [*] = fn left right => (wasm : Integer) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.mul
      struct.new $integer
    )
  end

  --> @HIDDEN
  impl Divide Integer =
    let [/] = fn left right => (wasm : Integer) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.div
      struct.new $integer
    )
  end

  --> @HIDDEN
  impl Remainder Integer =
    let [mod] = fn left right => (wasm : Integer) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.rem
      struct.new $integer
    )
  end

  --> @HIDDEN
  impl Bitwise Integer =
    let [and] = fn left right => (wasm : Integer) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.and
      struct.new $integer
    )

    let [or] = fn left right => (wasm : Integer) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.or
      struct.new $integer
    )

    let [xor] = fn left right => (wasm : Integer) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.xor
      struct.new $integer
    )

    let [not] = fn value => (wasm : Integer) => (
      get value
      struct.get $integer 0
      i64.const 0
      i64.const 1
      i64.sub
      i64.xor
      struct.new $integer
    )
  end

  --> @HIDDEN
  impl Equal Natural =
    let [==] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $natural 0
      get right
      struct.get $natural 0
      i64.eq
      struct.new $word
    )
  end

  --> @HIDDEN
  impl Compare Natural =
    let [<] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $natural 0
      get right
      struct.get $natural 0
      i64.lt
      struct.new $word
    )

    let [>] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $natural 0
      get right
      struct.get $natural 0
      i64.gt
      struct.new $word
    )
  end

  --> @HIDDEN
  impl Add Natural =
    let [+] = fn left right => (wasm : Natural) => (
      get left
      struct.get $natural 0
      get right
      struct.get $natural 0
      i64.add
      struct.new $natural
    )
  end

  --> @HIDDEN
  impl Subtract Natural =
    let [-] = fn left right => (wasm : Natural) => (
      (local $result i64)
      get left
      struct.get $natural 0
      get right
      struct.get $natural 0
      i64.lt
      if
        i64.const 0
        set $result
      else
        get left
        struct.get $natural 0
        get right
        struct.get $natural 0
        i64.sub
        set $result
      end
      get $result
      struct.new $natural
    )

    let [~] = fn _ => (wasm : Natural) => (
      i64.const 0
      struct.new $natural
    )
  end

  --> @HIDDEN
  impl Multiply Natural =
    let [*] = fn left right => (wasm : Natural) => (
      get left
      struct.get $natural 0
      get right
      struct.get $natural 0
      i64.mul
      struct.new $natural
    )
  end

  --> @HIDDEN
  impl Divide Natural =
    let [/] = fn left right => (wasm : Natural) => (
      get left
      struct.get $natural 0
      get right
      struct.get $natural 0
      i64.div
      struct.new $natural
    )
  end

  --> @HIDDEN
  impl Remainder Natural =
    let [mod] = fn left right => (wasm : Natural) => (
      get left
      struct.get $natural 0
      get right
      struct.get $natural 0
      i64.rem
      struct.new $natural
    )
  end

  --> @HIDDEN
  impl Equal Real =
    let [==] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $real 0
      get right
      struct.get $real 0
      f64.eq
      struct.new $word
    )
  end

  --> @HIDDEN
  impl Compare Real =
    let [<] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $real 0
      get right
      struct.get $real 0
      f64.lt
      struct.new $word
    )

    let [>] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $real 0
      get right
      struct.get $real 0
      f64.gt
      struct.new $word
    )
  end

  --> @HIDDEN
  impl Add Real =
    let [+] = fn left right => (wasm : Real) => (
      get left
      struct.get $real 0
      get right
      struct.get $real 0
      f64.add
      struct.new $real
    )
  end

  --> @HIDDEN
  impl Subtract Real =
    let [-] = fn left right => (wasm : Real) => (
      get left
      struct.get $real 0
      get right
      struct.get $real 0
      f64.sub
      struct.new $real
    )

    let [~] = fn value => (wasm : Real) => (
      f64.const 0
      get value
      struct.get $real 0
      f64.sub
      struct.new $real
    )
  end

  --> @HIDDEN
  impl Multiply Real =
    let [*] = fn left right => (wasm : Real) => (
      get left
      struct.get $real 0
      get right
      struct.get $real 0
      f64.mul
      struct.new $real
    )
  end

  --> @HIDDEN
  impl Divide Real =
    let [/] = fn left right => (wasm : Real) => (
      get left
      struct.get $real 0
      get right
      struct.get $real 0
      f64.div
      struct.new $real
    )
  end

  --> @HIDDEN
  impl Equal Boolean =
    let [==] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.eq
      struct.new $word
    )
  end

  --> @HIDDEN
  impl Compare Boolean =
    let [<] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.lt
      struct.new $word
    )

    let [>] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.gt
      struct.new $word
    )
  end

  --> @HIDDEN
  impl Bitwise Boolean =
    let [and] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.and
      struct.new $word
    )

    let [or] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.or
      struct.new $word
    )

    let [xor] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.xor
      struct.new $word
    )

    let [not] = fn value => (wasm : Boolean) => (
      get value
      struct.get $word 0
      i32.const 1
      i32.xor
      struct.new $word
    )
  end

  (*>
  Equivalent to `not (a == b)`
  *)
  let [!=] = fn left right => not (left == right)

  --> @HIDDEN
  impl Equal Glyph =
    let [==] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.eq
      struct.new $word
    )
  end

  --> @HIDDEN
  impl Compare Glyph =
    let [<] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.lt
      struct.new $word
    )

    let [>] = fn left right => (wasm : Boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.gt
      struct.new $word
    )
  end

  --> @HIDDEN
  impl Add String =
    let [+] = fn left right => (wasm : String) => (
      (local $left_len i32)
      (local $right_len i32)
      (local $result $string)

      get left
      array.len
      set $left_len

      get right
      array.len
      set $right_len

      get $left_len
      get $right_len
      i32.add
      array.new_default i8
      set $result

      get $result
      i32.const 0
      get left
      i32.const 0
      get $left_len
      array.copy i8 i8

      get $result
      get $left_len
      get right
      i32.const 0
      get $right_len
      array.copy i8 i8

      get $result
    )
  end

  --> @HIDDEN
  impl Equal String =
    let [==] = fn left right => (wasm : Boolean) => (
      (local $left_len i32)
      (local $right_len i32)
      (local $min_len i32)
      (local $index i32)
      (local $cmp i32)
      (local $left_byte i32)
      (local $right_byte i32)

      get left
      array.len
      set $left_len
      get right
      array.len
      set $right_len

      get $left_len
      set $min_len
      get $right_len
      get $left_len
      i32.lt
      if
        get $right_len
        set $min_len
      end

      i32.const 0
      set $index
      i32.const 0
      set $cmp

      block
      loop
        get $index
        get $min_len
        i32.eq
        break.if 1

        get left
        get $index
        array.get i8
        set $left_byte

        get right
        get $index
        array.get i8
        set $right_byte

        get $left_byte
        get $right_byte
        i32.lt
        if
          i32.const 0
          i32.const 1
          i32.sub
          set $cmp
          break 2
        end

        get $left_byte
        get $right_byte
        i32.gt
        if
          i32.const 1
          set $cmp
          break 2
        end

        get $index
        i32.const 1
        i32.add
        set $index
        break 0
      end
      end

      get $cmp
      i32.const 0
      i32.eq
      if
        get $left_len
        get $right_len
        i32.lt
        if
          i32.const 0
          i32.const 1
          i32.sub
          set $cmp
        else
          get $left_len
          get $right_len
          i32.gt
          if
            i32.const 1
            set $cmp
          end
        end
      end

      get $cmp
      i32.const 0
      i32.eq
      struct.new $word
    )
  end

  --> @HIDDEN
  impl Compare String =
    let [<] = fn left right => (wasm : Boolean) => (
      (local $left_len i32)
      (local $right_len i32)
      (local $min_len i32)
      (local $index i32)
      (local $cmp i32)
      (local $left_byte i32)
      (local $right_byte i32)

      get left
      array.len
      set $left_len
      get right
      array.len
      set $right_len

      get $left_len
      set $min_len
      get $right_len
      get $left_len
      i32.lt
      if
        get $right_len
        set $min_len
      end

      i32.const 0
      set $index
      i32.const 0
      set $cmp

      block
      loop
        get $index
        get $min_len
        i32.eq
        break.if 1

        get left
        get $index
        array.get i8
        set $left_byte

        get right
        get $index
        array.get i8
        set $right_byte

        get $left_byte
        get $right_byte
        i32.lt
        if
          i32.const 0
          i32.const 1
          i32.sub
          set $cmp
          break 2
        end

        get $left_byte
        get $right_byte
        i32.gt
        if
          i32.const 1
          set $cmp
          break 2
        end

        get $index
        i32.const 1
        i32.add
        set $index
        break 0
      end
      end

      get $cmp
      i32.const 0
      i32.eq
      if
        get $left_len
        get $right_len
        i32.lt
        if
          i32.const 0
          i32.const 1
          i32.sub
          set $cmp
        else
          get $left_len
          get $right_len
          i32.gt
          if
            i32.const 1
            set $cmp
          end
        end
      end

      get $cmp
      i32.const 0
      i32.lt
      struct.new $word
    )

    let [>] = fn left right => (wasm : Boolean) => (
      (local $left_len i32)
      (local $right_len i32)
      (local $min_len i32)
      (local $index i32)
      (local $cmp i32)
      (local $left_byte i32)
      (local $right_byte i32)

      get left
      array.len
      set $left_len
      get right
      array.len
      set $right_len

      get $left_len
      set $min_len
      get $right_len
      get $left_len
      i32.lt
      if
        get $right_len
        set $min_len
      end

      i32.const 0
      set $index
      i32.const 0
      set $cmp

      block
      loop
        get $index
        get $min_len
        i32.eq
        break.if 1

        get left
        get $index
        array.get i8
        set $left_byte

        get right
        get $index
        array.get i8
        set $right_byte

        get $left_byte
        get $right_byte
        i32.lt
        if
          i32.const 0
          i32.const 1
          i32.sub
          set $cmp
          break 2
        end

        get $left_byte
        get $right_byte
        i32.gt
        if
          i32.const 1
          set $cmp
          break 2
        end

        get $index
        i32.const 1
        i32.add
        set $index
        break 0
      end
      end

      get $cmp
      i32.const 0
      i32.eq
      if
        get $left_len
        get $right_len
        i32.lt
        if
          i32.const 0
          i32.const 1
          i32.sub
          set $cmp
        else
          get $left_len
          get $right_len
          i32.gt
          if
            i32.const 1
            set $cmp
          end
        end
      end

      get $cmp
      i32.const 0
      i32.gt
      struct.new $word
    )
  end

  (*>
  Function composition operator (`g . f`).
  Defined as `fn a b c => b (a c)`
  *)
  let [>>] = fn a b c => b (a c)

  (*>
  Reverse function composition operator (`f . g`).
  Defined as `fn a b c => a (b c)`
  *)
  let [<<] = fn a b c => a (b c)

  (*>
  Function pipe operator.
  Calls the function to the right with the argument on the left.

  ```hc
  do show 42 |> println
  ```
  *)
  let [|>] = fn value f => f value

  (*>
  Sequence operator that keeps the right value, igores the left value.
  *)
  let [;] = fn _ kept => kept

  (*>
  Less-than-or-equal-to operator.
  Defined as `(a < b) or (a == b)`
  *)
  let [<=] = fn left right => (left < right) or (left == right)

  (*>
  Greater-than-or-equal-to operator.
  Defined as `(a > b) or (a == b)`
  *)
  let [>=] = fn left right => (left > right) or (left == right)

  (*>
  Returns the smaller of two values.
  If neither value is smaller than the other, return the second value.
  *)
  let min = fn left right => if left < right then left else right

  (*>
  Returns the larger of two comparable values.
  If neither value is larger than the other, return the second value.
  *)
  let max = fn left right => if left > right then left else right

  (*>
  Restricts `value` to the inclusive range `[lower, upper]`.

  - Arguments:
    - `low`: Minimum allowed value.
    - `high`: Maximum allowed value.
    - `value`: Value to clamp.
  - Returns: `value` clamped into range.

  ```hc
  let percent = ops::clamp 0 100 raw_percent
  ```
  *)
  let clamp = fn low high value => min high (max low value)

  (*>
  Checks whether a value is within an inclusive range.

  - Arguments:
    - `low`: Lower bound.
    - `high`: Upper bound.
    - `value`: Value to test.
  - Returns: `true` when `lower <= value <= upper`.
  *)
  let between = fn low high value => (value >= low) and (value <= high)
end
