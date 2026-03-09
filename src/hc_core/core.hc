module core =

  trait equal : a =
    let [==] : a -> a -> boolean
  end

  trait compare : a =
    let [<] : a -> a -> boolean
    let [>] : a -> a -> boolean
  end

  trait add : a =
    let [+] : a -> a -> a
  end

  trait subtract : a =
    let [-] : a -> a -> a
    let [~] : a -> a
  end

  trait multiply : a =
    let [*] : a -> a -> a
  end

  trait divide : a =
    let [/] : a -> a -> a
  end

  trait remainder : a =
    let [%] : a -> a -> a
  end

  trait bitwise : a =
    let [and] : a -> a -> a
    let [or] : a -> a -> a
    let [xor] : a -> a -> a
    let [not] : a -> a
  end

  wasm => (
    (type $integer (struct i64))
    (type $real (struct f64))
    (type $word (struct i32))
    (type $string (array i8))
    (type $unit (struct))
    (import "wasi_snapshot_preview1" "fd_write"
      (func $fd_write (param i32 i32 i32 i32) (result i32)))
    (memory $mem 1)
  )

  impl equal : core::unit =
    let [==] = fn _ _ => (wasm : core::boolean) => (
      i32.const 1
      struct.new $word
    )
  end

  impl compare : core::unit =
    let [<] = fn _ _ => (wasm : core::boolean) => (
      i32.const 0
      struct.new $word
    )

    let [>] = fn _ _ => (wasm : core::boolean) => (
      i32.const 0
      struct.new $word
    )
  end

  impl equal : core::integer =
    let [==] = fn left right => (wasm : core::boolean) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.eq
      struct.new $word
    )
  end

  impl compare : core::integer =
    let [<] = fn left right => (wasm : core::boolean) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.lt
      struct.new $word
    )

    let [>] = fn left right => (wasm : core::boolean) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.gt
      struct.new $word
    )
  end

  impl add : core::integer =
    let [+] = fn left right => (wasm : core::integer) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.add
      struct.new $integer
    )
  end

  impl subtract : core::integer =
    let [-] = fn left right => (wasm : core::integer) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.sub
      struct.new $integer
    )

    let [~] = fn value => (wasm : core::integer) => (
      i64.const 0
      get value
      struct.get $integer 0
      i64.sub
      struct.new $integer
    )
  end

  impl multiply : core::integer =
    let [*] = fn left right => (wasm : core::integer) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.mul
      struct.new $integer
    )
  end

  impl divide : core::integer =
    let [/] = fn left right => (wasm : core::integer) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.div
      struct.new $integer
    )
  end

  impl remainder : core::integer =
    let [%] = fn left right => (wasm : core::integer) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.rem
      struct.new $integer
    )
  end

  impl bitwise : core::integer =
    let [and] = fn left right => (wasm : core::integer) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.and
      struct.new $integer
    )

    let [or] = fn left right => (wasm : core::integer) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.or
      struct.new $integer
    )

    let [xor] = fn left right => (wasm : core::integer) => (
      get left
      struct.get $integer 0
      get right
      struct.get $integer 0
      i64.xor
      struct.new $integer
    )

    let [not] = fn value => (wasm : core::integer) => (
      get value
      struct.get $integer 0
      i64.const 0
      i64.const 1
      i64.sub
      i64.xor
      struct.new $integer
    )
  end

  impl equal : core::real =
    let [==] = fn left right => (wasm : core::boolean) => (
      get left
      struct.get $real 0
      get right
      struct.get $real 0
      f64.eq
      struct.new $word
    )
  end

  impl compare : core::real =
    let [<] = fn left right => (wasm : core::boolean) => (
      get left
      struct.get $real 0
      get right
      struct.get $real 0
      f64.lt
      struct.new $word
    )

    let [>] = fn left right => (wasm : core::boolean) => (
      get left
      struct.get $real 0
      get right
      struct.get $real 0
      f64.gt
      struct.new $word
    )
  end

  impl add : core::real =
    let [+] = fn left right => (wasm : core::real) => (
      get left
      struct.get $real 0
      get right
      struct.get $real 0
      f64.add
      struct.new $real
    )
  end

  impl subtract : core::real =
    let [-] = fn left right => (wasm : core::real) => (
      get left
      struct.get $real 0
      get right
      struct.get $real 0
      f64.sub
      struct.new $real
    )

    let [~] = fn value => (wasm : core::real) => (
      f64.const 0
      get value
      struct.get $real 0
      f64.sub
      struct.new $real
    )
  end

  impl multiply : core::real =
    let [*] = fn left right => (wasm : core::real) => (
      get left
      struct.get $real 0
      get right
      struct.get $real 0
      f64.mul
      struct.new $real
    )
  end

  impl divide : core::real =
    let [/] = fn left right => (wasm : core::real) => (
      get left
      struct.get $real 0
      get right
      struct.get $real 0
      f64.div
      struct.new $real
    )
  end

  impl equal : core::boolean =
    let [==] = fn left right => (wasm : core::boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.eq
      struct.new $word
    )
  end

  impl compare : core::boolean =
    let [<] = fn left right => (wasm : core::boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.lt
      struct.new $word
    )

    let [>] = fn left right => (wasm : core::boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.gt
      struct.new $word
    )
  end

  impl bitwise : core::boolean =
    let [and] = fn left right => (wasm : core::boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.and
      struct.new $word
    )

    let [or] = fn left right => (wasm : core::boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.or
      struct.new $word
    )

    let [xor] = fn left right => (wasm : core::boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.xor
      struct.new $word
    )

    let [not] = fn value => (wasm : core::boolean) => (
      get value
      struct.get $word 0
      i32.const 1
      i32.xor
      struct.new $word
    )
  end

  let [!=] = fn left right => not (left == right)

  impl equal : core::glyph =
    let [==] = fn left right => (wasm : core::boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.eq
      struct.new $word
    )
  end

  impl compare : core::glyph =
    let [<] = fn left right => (wasm : core::boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.lt
      struct.new $word
    )

    let [>] = fn left right => (wasm : core::boolean) => (
      get left
      struct.get $word 0
      get right
      struct.get $word 0
      i32.gt
      struct.new $word
    )
  end

  impl add : core::string =
    let [+] = fn left right => (wasm : core::string) => (
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

  impl equal : core::string =
    let [==] = fn left right => (wasm : core::boolean) => (
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

  impl compare : core::string =
    let [<] = fn left right => (wasm : core::boolean) => (
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

    let [>] = fn left right => (wasm : core::boolean) => (
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

  let [>>] = fn first second value => second (first value)
  let [<<] = fn first second value => first (second value)
  let [|>] = fn value f => f value
  let [;] = fn _ kept => kept

  let [<=] = fn left right => if left < right then true else left == right
  let [>=] = fn left right => if left > right then true else left == right

  let array_empty : for a in array a = (wasm : for a in array a) => (
    i32.const 0
    array.new_default any
  )

  let array_concat : for a in array a -> array a -> array a =
    fn left right =>
      (wasm : for a in array a) => (
        (local $left (array any))
        (local $right (array any))
        (local $left_len i32)
        (local $right_len i32)
        (local $result (array any))

        get left
        ref.cast_array any
        set $left

        get right
        ref.cast_array any
        set $right

        get $left
        array.len
        set $left_len

        get $right
        array.len
        set $right_len

        get $left_len
        get $right_len
        i32.add
        array.new_default any
        set $result

        get $result
        i32.const 0
        get $left
        i32.const 0
        get $left_len
        array.copy any any

        get $result
        get $left_len
        get $right
        i32.const 0
        get $right_len
        array.copy any any

        get $result
      )

  let array_push : for a in a -> array a -> array a =
    fn value arr =>
      (wasm : for a in array a) => (
        (local $arr (array any))
        (local $len i32)
        (local $result (array any))

        get arr
        ref.cast_array any
        set $arr

        get $arr
        array.len
        set $len

        get $len
        i32.const 1
        i32.add
        array.new_default any
        set $result

        get $result
        i32.const 0
        get $arr
        i32.const 0
        get $len
        array.copy any any

        get $result
        get $len
        get value
        array.new_fixed any 1
        i32.const 0
        i32.const 1
        array.copy any any

        get $result
      )

  let print_string = fn (value : core::string) => (wasm : core::unit) => (
    (local $str $string)
    (local $len i32)
    (local $index i32)

    get value
    ref.cast_array i8
    set $str

    get $str
    array.len
    set $len

    i32.const 0
    set $index

    block
    loop
      get $index
      get $len
      i32.eq
      break.if 1

      i32.const 12
      get $index
      i32.add
      get $str
      get $index
      array.get i8
      i32.store8

      get $index
      i32.const 1
      i32.add
      set $index
      break 0
    end
    end

    i32.const 0
    i32.const 12
    i32.store

    i32.const 4
    get $len
    i32.store

    i32.const 1
    i32.const 0
    i32.const 1
    i32.const 8
    call $fd_write
    drop

    struct.new $unit
  )

  impl add : for a in array a =
    let [+] = fn left right => array_concat left right
  end

  trait default : self =
    let default : self
  end

  impl default : () =
    let default = ()
  end

  impl default : core::integer = 
    let default = 0
  end

  impl default : core::real = 
    let default = 0.0
  end

  impl default : core::boolean =
    let default = false
  end

  impl default : core::string =
    let default = ""
  end

  impl default : for a in array a =
    let default = []
  end

end
