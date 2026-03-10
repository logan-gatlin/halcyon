module core =
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

  module ops =
    use core
    trait Equal : a =
      let [==] : a -> a -> Boolean
    end

    trait Compare : a =
      let [<] : a -> a -> Boolean
      let [>] : a -> a -> Boolean
    end

    trait Add : a =
      let [+] : a -> a -> a
    end

    trait Subtract : a =
      let [-] : a -> a -> a
      let [~] : a -> a
    end

    trait Multiply : a =
      let [*] : a -> a -> a
    end

    trait Divide : a =
      let [/] : a -> a -> a
    end

    trait Remainder : a =
      let [%] : a -> a -> a
    end

    trait Bitwise : a =
      let [and] : a -> a -> a
      let [or] : a -> a -> a
      let [xor] : a -> a -> a
      let [not] : a -> a
    end


    impl Equal () =
      let [==] = fn _ _ => (wasm : Boolean) => (
        i32.const 1
        struct.new $word
      )
    end

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

    impl Remainder Integer =
      let [%] = fn left right => (wasm : Integer) => (
        get left
        struct.get $integer 0
        get right
        struct.get $integer 0
        i64.rem
        struct.new $integer
      )
    end

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

    let [!=] = fn left right => not (left == right)

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

    let [>>] = fn first second value => second (first value)
    let [<<] = fn first second value => first (second value)
    let [|>] = fn value f => f value
    let [;] = fn _ kept => kept

    let [<=] = fn left right => if left < right then true else left == right
    let [>=] = fn left right => if left > right then true else left == right
  end

  let array_empty : for a in Array a = (wasm : for a in Array a) => (
    i32.const 0
    array.new_default any
  )

  let array_concat : for a in Array a -> Array a -> Array a =
    fn left right =>
      (wasm : for a in Array a) => (
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

  let array_push : for a in a -> Array a -> Array a =
    fn value arr =>
      (wasm : for a in Array a) => (
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

  let print_string = fn (value : String) => (wasm : ()) => (
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

  impl ops::Add for a in Array a =
    let [+] = fn left right => array_concat left right
  end

  trait Default : self =
    let default : self
  end

  impl Default () =
    let default = ()
  end

  impl Default Integer = 
    let default = 0
  end

  impl Default Real = 
    let default = 0.0
  end

  impl Default Boolean =
    let default = false
  end

  impl Default String =
    let default = ""
  end

  impl Default for a in Array a =
    let default = []
  end

  module opt =
    type Option: a = | Some a | None
    let map = fn f opt => match opt with
      | Some a => Some (f a)
      | None => None
    let is_some = fn opt => match opt with
      | Some a => true
      | None => false
    let unwrap_or = fn backup opt => match opt with
      | Some a => a
      | None => backup
    let unwrap_or_else = fn backup_fn opt => match opt with
      | Some a => a
      | None => backup_fn ()
  end

  (*
    Everything in the core::prelude module is implicitly in scope always. Do
    not write actual implementations in here, instead write aliases to things
    defined elsewhere.
  *)
  module prelude =
    type ~Integer = core::Integer
    type ~Real = core::Real
    type ~Glyph = core::Glyph
    type ~String = core::String
    trait ~Default = core::Default
    let [+] = core::ops::[+]
    let [-] = core::ops::[-]
    let [~] = core::ops::[~]
    let [*] = core::ops::[*]
    let [/] = core::ops::[/]
    let [%] = core::ops::[%]
    let [|>] = core::ops::[|>]
    let [>>] = core::ops::[>>]
    let [<<] = core::ops::[<<]
    let [and] = core::ops::[and]
    let [or] = core::ops::[or]
    let [xor] = core::ops::[xor]
    let [not] = core::ops::[not]
    let [==] = core::ops::[==]
    let [!=] = core::ops::[!=]
    let [<] = core::ops::[<]
    let [>] = core::ops::[>]
    let [<=] = core::ops::[<=]
    let [>=] = core::ops::[>=]
    let default = core::default
  end

end
