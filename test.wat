(module
  (type $integer (;0;) (struct (field i64)))
  (type $capture (;1;) (array (mut anyref)))
  (type $"(raw) integer -> integer" (;2;) (func (param (ref $integer) (ref $capture)) (result (ref $integer))))
  (type $"integer -> integer" (;3;) (struct (field (ref $"(raw) integer -> integer")) (field (ref $capture))))
  (type $"(raw) integer -> integer -> integer" (;4;) (func (param (ref $integer) (ref $capture)) (result (ref $"integer -> integer"))))
  (type $real (;5;) (struct (field f64)))
  (type $"(raw) real -> real" (;6;) (func (param (ref $real) (ref $capture)) (result (ref $real))))
  (type $"real -> real" (;7;) (struct (field (ref $"(raw) real -> real")) (field (ref $capture))))
  (type $"(raw) real -> real -> real" (;8;) (func (param (ref $real) (ref $capture)) (result (ref $"real -> real"))))
  (type $boolean (;9;) (struct (field i32)))
  (type $"(raw) boolean -> boolean" (;10;) (func (param (ref $boolean) (ref $capture)) (result (ref $boolean))))
  (type $"boolean -> boolean" (;11;) (struct (field (ref $"(raw) boolean -> boolean")) (field (ref $capture))))
  (type $"(raw) boolean -> boolean -> boolean" (;12;) (func (param (ref $boolean) (ref $capture)) (result (ref $"boolean -> boolean"))))
  (type $glyph (;13;) (struct (field i32)))
  (type $"()" (;14;) (struct))
  (type $string (;15;) (array (mut i8)))
  (type $"(raw) '0 -> boolean" (;16;) (func (param anyref (ref $capture)) (result (ref $boolean))))
  (type $"'0 -> boolean" (;17;) (struct (field (ref $"(raw) '0 -> boolean")) (field (ref $capture))))
  (type $"(raw) '0 -> '0 -> boolean" (;18;) (func (param anyref (ref $capture)) (result (ref $"'0 -> boolean"))))
  (type $"(raw) boolean -> ()" (;19;) (func (param (ref $boolean) (ref $capture)) (result (ref $"()"))))
  (type (;20;) (func))
  (type $"boolean -> ()" (;21;) (struct (field (ref $"(raw) boolean -> ()")) (field (ref $capture))))
  (table (;0;) 41 41 funcref)
  (start 40)
  (elem (;0;) (i32.const 0) func 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40)
  (func (;0;) (type $"(raw) integer -> integer") (param $b (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local $a (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set $a
    local.get $a
    struct.get $integer 0
    local.get $b
    struct.get $integer 0
    i64.add
    struct.new $integer
  )
  (func (;1;) (type $"(raw) integer -> integer -> integer") (param $a (ref $integer)) (param (ref $capture)) (result (ref $"integer -> integer"))
    ref.func 0
    local.get $a
    array.new_fixed $capture 1
    struct.new $"integer -> integer"
  )
  (func (;2;) (type $"(raw) integer -> integer") (param $b (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local $a (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set $a
    local.get $a
    struct.get $integer 0
    local.get $b
    struct.get $integer 0
    i64.sub
    struct.new $integer
  )
  (func (;3;) (type $"(raw) integer -> integer -> integer") (param $a (ref $integer)) (param (ref $capture)) (result (ref $"integer -> integer"))
    ref.func 2
    local.get $a
    array.new_fixed $capture 1
    struct.new $"integer -> integer"
  )
  (func (;4;) (type $"(raw) integer -> integer") (param $b (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local $a (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set $a
    local.get $a
    struct.get $integer 0
    local.get $b
    struct.get $integer 0
    i64.mul
    struct.new $integer
  )
  (func (;5;) (type $"(raw) integer -> integer -> integer") (param $a (ref $integer)) (param (ref $capture)) (result (ref $"integer -> integer"))
    ref.func 4
    local.get $a
    array.new_fixed $capture 1
    struct.new $"integer -> integer"
  )
  (func (;6;) (type $"(raw) integer -> integer") (param $b (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local $a (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set $a
    local.get $a
    struct.get $integer 0
    local.get $b
    struct.get $integer 0
    i64.div_s
    struct.new $integer
  )
  (func (;7;) (type $"(raw) integer -> integer -> integer") (param $a (ref $integer)) (param (ref $capture)) (result (ref $"integer -> integer"))
    ref.func 6
    local.get $a
    array.new_fixed $capture 1
    struct.new $"integer -> integer"
  )
  (func (;8;) (type $"(raw) integer -> integer") (param $b (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local $a (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set $a
    local.get $a
    struct.get $integer 0
    local.get $b
    struct.get $integer 0
    i64.rem_s
    struct.new $integer
  )
  (func (;9;) (type $"(raw) integer -> integer -> integer") (param $a (ref $integer)) (param (ref $capture)) (result (ref $"integer -> integer"))
    ref.func 8
    local.get $a
    array.new_fixed $capture 1
    struct.new $"integer -> integer"
  )
  (func (;10;) (type $"(raw) real -> real") (param $b (ref $real)) (param (ref $capture)) (result (ref $real))
    (local $a (ref $real))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set $a
    local.get $a
    struct.get $real 0
    local.get $b
    struct.get $real 0
    f64.add
    struct.new $real
  )
  (func (;11;) (type $"(raw) real -> real -> real") (param $a (ref $real)) (param (ref $capture)) (result (ref $"real -> real"))
    ref.func 10
    local.get $a
    array.new_fixed $capture 1
    struct.new $"real -> real"
  )
  (func (;12;) (type $"(raw) real -> real") (param $b (ref $real)) (param (ref $capture)) (result (ref $real))
    (local $a (ref $real))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set $a
    local.get $a
    struct.get $real 0
    local.get $b
    struct.get $real 0
    f64.sub
    struct.new $real
  )
  (func (;13;) (type $"(raw) real -> real -> real") (param $a (ref $real)) (param (ref $capture)) (result (ref $"real -> real"))
    ref.func 12
    local.get $a
    array.new_fixed $capture 1
    struct.new $"real -> real"
  )
  (func (;14;) (type $"(raw) real -> real") (param $b (ref $real)) (param (ref $capture)) (result (ref $real))
    (local $a (ref $real))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set $a
    local.get $a
    struct.get $real 0
    local.get $b
    struct.get $real 0
    f64.mul
    struct.new $real
  )
  (func (;15;) (type $"(raw) real -> real -> real") (param $a (ref $real)) (param (ref $capture)) (result (ref $"real -> real"))
    ref.func 14
    local.get $a
    array.new_fixed $capture 1
    struct.new $"real -> real"
  )
  (func (;16;) (type $"(raw) real -> real") (param $b (ref $real)) (param (ref $capture)) (result (ref $real))
    (local $a (ref $real))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set $a
    local.get $a
    struct.get $real 0
    local.get $b
    struct.get $real 0
    f64.div
    struct.new $real
  )
  (func (;17;) (type $"(raw) real -> real -> real") (param $a (ref $real)) (param (ref $capture)) (result (ref $"real -> real"))
    ref.func 16
    local.get $a
    array.new_fixed $capture 1
    struct.new $"real -> real"
  )
  (func (;18;) (type $"(raw) boolean -> boolean") (param $b (ref $boolean)) (param (ref $capture)) (result (ref $boolean))
    (local $a (ref $boolean))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $boolean)
    local.set $a
    local.get $a
    struct.get $boolean 0
    local.get $b
    struct.get $boolean 0
    i32.and
    struct.new $boolean
  )
  (func (;19;) (type $"(raw) boolean -> boolean -> boolean") (param $a (ref $boolean)) (param (ref $capture)) (result (ref $"boolean -> boolean"))
    ref.func 18
    local.get $a
    array.new_fixed $capture 1
    struct.new $"boolean -> boolean"
  )
  (func (;20;) (type $"(raw) boolean -> boolean") (param $b (ref $boolean)) (param (ref $capture)) (result (ref $boolean))
    (local $a (ref $boolean))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $boolean)
    local.set $a
    local.get $a
    struct.get $boolean 0
    local.get $b
    struct.get $boolean 0
    i32.or
    struct.new $boolean
  )
  (func (;21;) (type $"(raw) boolean -> boolean -> boolean") (param $a (ref $boolean)) (param (ref $capture)) (result (ref $"boolean -> boolean"))
    ref.func 20
    local.get $a
    array.new_fixed $capture 1
    struct.new $"boolean -> boolean"
  )
  (func (;22;) (type $"(raw) boolean -> boolean") (param $b (ref $boolean)) (param (ref $capture)) (result (ref $boolean))
    (local $a (ref $boolean))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $boolean)
    local.set $a
    local.get $a
    struct.get $boolean 0
    local.get $b
    struct.get $boolean 0
    i32.xor
    struct.new $boolean
  )
  (func (;23;) (type $"(raw) boolean -> boolean -> boolean") (param $a (ref $boolean)) (param (ref $capture)) (result (ref $"boolean -> boolean"))
    ref.func 22
    local.get $a
    array.new_fixed $capture 1
    struct.new $"boolean -> boolean"
  )
  (func (;24;) (type $"(raw) '0 -> boolean") (param $b anyref) (param (ref $capture)) (result (ref $boolean))
    (local $a anyref) (local i32) (local $index i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set $a
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get $a
                br_on_cast 0 (;@6;) anyref (ref $integer)
                br_on_cast 1 (;@5;) anyref (ref $real)
                br_on_cast 2 (;@4;) anyref (ref $boolean)
                br_on_cast 3 (;@3;) anyref (ref $glyph)
                br_on_cast 4 (;@2;) anyref (ref $"()")
                br_on_cast 5 (;@1;) anyref (ref $string)
                unreachable
              end
              ref.cast (ref $integer)
              struct.get $integer 0
              local.get $b
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.eq
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $b
            ref.cast (ref $real)
            struct.get $real 0
            f64.eq
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $b
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.eq
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $b
        ref.cast (ref $glyph)
        struct.get $glyph 0
        i32.eq
        struct.new $boolean
        return
      end
      i32.const 1
      struct.new $boolean
      return
    end
    ref.cast (ref $string)
    array.len
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    local.get $a
    ref.cast (ref $string)
    array.len
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.lt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    i32.const 0
    local.set 3
    local.get $a
    ref.cast (ref $string)
    array.len
    local.set $index
    loop ;; label = @1
      local.get $a
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $b
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      i32.ne
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 3
      i32.const 1
      i32.add
      local.tee 3
      local.get $index
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;25;) (type $"(raw) '0 -> '0 -> boolean") (param $a anyref) (param (ref $capture)) (result (ref $"'0 -> boolean"))
    ref.func 24
    local.get $a
    array.new_fixed $capture 1
    struct.new $"'0 -> boolean"
  )
  (func (;26;) (type $"(raw) '0 -> boolean") (param $b anyref) (param (ref $capture)) (result (ref $boolean))
    (local $a anyref) (local i32) (local $index i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set $a
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get $a
                br_on_cast 0 (;@6;) anyref (ref $integer)
                br_on_cast 1 (;@5;) anyref (ref $real)
                br_on_cast 2 (;@4;) anyref (ref $boolean)
                br_on_cast 3 (;@3;) anyref (ref $glyph)
                br_on_cast 4 (;@2;) anyref (ref $"()")
                br_on_cast 5 (;@1;) anyref (ref $string)
                unreachable
              end
              ref.cast (ref $integer)
              struct.get $integer 0
              local.get $b
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.ne
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $b
            ref.cast (ref $real)
            struct.get $real 0
            f64.ne
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $b
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.ne
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $b
        ref.cast (ref $glyph)
        struct.get $glyph 0
        i32.ne
        struct.new $boolean
        return
      end
      i32.const 0
      struct.new $boolean
      return
    end
    ref.cast (ref $string)
    array.len
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    local.get $a
    ref.cast (ref $string)
    array.len
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.lt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    i32.const 0
    local.set 3
    local.get $a
    ref.cast (ref $string)
    array.len
    local.set $index
    loop ;; label = @1
      local.get $a
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $b
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      i32.eq
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 3
      i32.const 1
      i32.add
      local.tee 3
      local.get $index
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;27;) (type $"(raw) '0 -> '0 -> boolean") (param $a anyref) (param (ref $capture)) (result (ref $"'0 -> boolean"))
    ref.func 26
    local.get $a
    array.new_fixed $capture 1
    struct.new $"'0 -> boolean"
  )
  (func (;28;) (type $"(raw) '0 -> boolean") (param $b anyref) (param (ref $capture)) (result (ref $boolean))
    (local $a anyref) (local i32) (local $index i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set $a
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get $a
                br_on_cast 0 (;@6;) anyref (ref $integer)
                br_on_cast 1 (;@5;) anyref (ref $real)
                br_on_cast 2 (;@4;) anyref (ref $boolean)
                br_on_cast 3 (;@3;) anyref (ref $glyph)
                br_on_cast 4 (;@2;) anyref (ref $"()")
                br_on_cast 5 (;@1;) anyref (ref $string)
                unreachable
              end
              ref.cast (ref $integer)
              struct.get $integer 0
              local.get $b
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.le_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $b
            ref.cast (ref $real)
            struct.get $real 0
            f64.le
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $b
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.le_u
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $b
        ref.cast (ref $glyph)
        struct.get $glyph 0
        i32.le_u
        struct.new $boolean
        return
      end
      i32.const 1
      struct.new $boolean
      return
    end
    ref.cast (ref $string)
    array.len
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    local.get $a
    ref.cast (ref $string)
    array.len
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.lt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    i32.const 0
    local.set 3
    local.get $a
    ref.cast (ref $string)
    array.len
    local.set $index
    loop ;; label = @1
      local.get $a
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $b
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      i32.gt_u
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 3
      i32.const 1
      i32.add
      local.tee 3
      local.get $index
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;29;) (type $"(raw) '0 -> '0 -> boolean") (param $a anyref) (param (ref $capture)) (result (ref $"'0 -> boolean"))
    ref.func 28
    local.get $a
    array.new_fixed $capture 1
    struct.new $"'0 -> boolean"
  )
  (func (;30;) (type $"(raw) '0 -> boolean") (param $b anyref) (param (ref $capture)) (result (ref $boolean))
    (local $a anyref) (local i32) (local $index i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set $a
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get $a
                br_on_cast 0 (;@6;) anyref (ref $integer)
                br_on_cast 1 (;@5;) anyref (ref $real)
                br_on_cast 2 (;@4;) anyref (ref $boolean)
                br_on_cast 3 (;@3;) anyref (ref $glyph)
                br_on_cast 4 (;@2;) anyref (ref $"()")
                br_on_cast 5 (;@1;) anyref (ref $string)
                unreachable
              end
              ref.cast (ref $integer)
              struct.get $integer 0
              local.get $b
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.ge_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $b
            ref.cast (ref $real)
            struct.get $real 0
            f64.ge
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $b
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.ge_u
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $b
        ref.cast (ref $glyph)
        struct.get $glyph 0
        i32.ge_u
        struct.new $boolean
        return
      end
      i32.const 1
      struct.new $boolean
      return
    end
    ref.cast (ref $string)
    array.len
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    local.get $a
    ref.cast (ref $string)
    array.len
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.lt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    i32.const 0
    local.set 3
    local.get $a
    ref.cast (ref $string)
    array.len
    local.set $index
    loop ;; label = @1
      local.get $a
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $b
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      i32.lt_u
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 3
      i32.const 1
      i32.add
      local.tee 3
      local.get $index
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;31;) (type $"(raw) '0 -> '0 -> boolean") (param $a anyref) (param (ref $capture)) (result (ref $"'0 -> boolean"))
    ref.func 30
    local.get $a
    array.new_fixed $capture 1
    struct.new $"'0 -> boolean"
  )
  (func (;32;) (type $"(raw) '0 -> boolean") (param $b anyref) (param (ref $capture)) (result (ref $boolean))
    (local $a anyref) (local i32) (local $index i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set $a
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get $a
                br_on_cast 0 (;@6;) anyref (ref $integer)
                br_on_cast 1 (;@5;) anyref (ref $real)
                br_on_cast 2 (;@4;) anyref (ref $boolean)
                br_on_cast 3 (;@3;) anyref (ref $glyph)
                br_on_cast 4 (;@2;) anyref (ref $"()")
                br_on_cast 5 (;@1;) anyref (ref $string)
                unreachable
              end
              ref.cast (ref $integer)
              struct.get $integer 0
              local.get $b
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.lt_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $b
            ref.cast (ref $real)
            struct.get $real 0
            f64.lt
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $b
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.lt_u
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $b
        ref.cast (ref $glyph)
        struct.get $glyph 0
        i32.lt_u
        struct.new $boolean
        return
      end
      i32.const 0
      struct.new $boolean
      return
    end
    ref.cast (ref $string)
    array.len
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    local.get $a
    ref.cast (ref $string)
    array.len
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.lt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    i32.const 0
    local.set 3
    local.get $a
    ref.cast (ref $string)
    array.len
    local.set $index
    loop ;; label = @1
      local.get $a
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $b
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      i32.ge_u
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 3
      i32.const 1
      i32.add
      local.tee 3
      local.get $index
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;33;) (type $"(raw) '0 -> '0 -> boolean") (param $a anyref) (param (ref $capture)) (result (ref $"'0 -> boolean"))
    ref.func 32
    local.get $a
    array.new_fixed $capture 1
    struct.new $"'0 -> boolean"
  )
  (func (;34;) (type $"(raw) '0 -> boolean") (param $b anyref) (param (ref $capture)) (result (ref $boolean))
    (local $a anyref) (local i32) (local $index i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set $a
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get $a
                br_on_cast 0 (;@6;) anyref (ref $integer)
                br_on_cast 1 (;@5;) anyref (ref $real)
                br_on_cast 2 (;@4;) anyref (ref $boolean)
                br_on_cast 3 (;@3;) anyref (ref $glyph)
                br_on_cast 4 (;@2;) anyref (ref $"()")
                br_on_cast 5 (;@1;) anyref (ref $string)
                unreachable
              end
              ref.cast (ref $integer)
              struct.get $integer 0
              local.get $b
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.gt_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $b
            ref.cast (ref $real)
            struct.get $real 0
            f64.gt
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $b
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.gt_u
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $b
        ref.cast (ref $glyph)
        struct.get $glyph 0
        i32.gt_u
        struct.new $boolean
        return
      end
      i32.const 0
      struct.new $boolean
      return
    end
    ref.cast (ref $string)
    array.len
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    local.get $a
    ref.cast (ref $string)
    array.len
    local.get $b
    ref.cast (ref $string)
    array.len
    i32.lt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    i32.const 0
    local.set 3
    local.get $a
    ref.cast (ref $string)
    array.len
    local.set $index
    loop ;; label = @1
      local.get $a
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $b
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      i32.le_u
      if ;; label = @2
        i32.const 0
        struct.new $boolean
        return
      end
      local.get 3
      i32.const 1
      i32.add
      local.tee 3
      local.get $index
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;35;) (type $"(raw) '0 -> '0 -> boolean") (param $a anyref) (param (ref $capture)) (result (ref $"'0 -> boolean"))
    ref.func 34
    local.get $a
    array.new_fixed $capture 1
    struct.new $"'0 -> boolean"
  )
  (func (;36;) (type $"(raw) integer -> integer") (param $a (ref $integer)) (param (ref $capture)) (result (ref $integer))
    i64.const 0
    local.get $a
    struct.get $integer 0
    i64.sub
    struct.new $integer
  )
  (func (;37;) (type $"(raw) real -> real") (param $a (ref $real)) (param (ref $capture)) (result (ref $real))
    local.get $a
    struct.get $real 0
    f64.neg
    struct.new $real
  )
  (func (;38;) (type $"(raw) boolean -> boolean") (param $a (ref $boolean)) (param (ref $capture)) (result (ref $boolean))
    local.get $a
    struct.get $boolean 0
    i32.eqz
    struct.new $boolean
  )
  (func (;39;) (type $"(raw) boolean -> ()") (param $a (ref $boolean)) (param (ref $capture)) (result (ref $"()"))
    local.get $a
    struct.get $boolean 0
    i32.eqz
    if ;; label = @1
      unreachable
    end
    struct.new $"()"
  )
  (func (;40;) (type 20)
    (local (ref $"boolean -> ()") (ref $"boolean -> ()") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"integer -> integer") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"real -> real") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"real -> real") (ref $"boolean -> ()") (ref $"integer -> integer") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"integer -> integer") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"integer -> integer") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"integer -> integer") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"integer -> integer") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"real -> real") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"real -> real") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"real -> real") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"real -> real") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"boolean -> boolean") (ref $"boolean -> ()") (ref $"boolean -> boolean") (ref $"boolean -> ()") (ref $"boolean -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean") (ref $"boolean -> ()") (ref $"'0 -> boolean"))
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 0
    i32.const 0
    struct.new $boolean
    array.new_fixed $capture 0
    call 38
    local.get 0
    struct.get $"boolean -> ()" 1
    local.get 0
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 1
    i32.const 0
    struct.new $boolean
    array.new_fixed $capture 0
    call 38
    array.new_fixed $capture 0
    call 38
    array.new_fixed $capture 0
    call 38
    local.get 1
    struct.get $"boolean -> ()" 1
    local.get 1
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 2
    i64.const 1
    struct.new $integer
    array.new_fixed $capture 0
    call 36
    array.new_fixed $capture 0
    call 25
    local.set 3
    i64.const 0
    struct.new $integer
    array.new_fixed $capture 0
    call 3
    local.set 4
    i64.const 1
    struct.new $integer
    local.get 4
    struct.get $"integer -> integer" 1
    local.get 4
    struct.get $"integer -> integer" 0
    call_ref $"(raw) integer -> integer"
    local.get 3
    struct.get $"'0 -> boolean" 1
    local.get 3
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 2
    struct.get $"boolean -> ()" 1
    local.get 2
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 5
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    array.new_fixed $capture 0
    call 37
    array.new_fixed $capture 0
    call 25
    local.set 6
    f64.const 0x0p+0 (;=0;)
    struct.new $real
    array.new_fixed $capture 0
    call 13
    local.set 7
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    local.get 7
    struct.get $"real -> real" 1
    local.get 7
    struct.get $"real -> real" 0
    call_ref $"(raw) real -> real"
    local.get 6
    struct.get $"'0 -> boolean" 1
    local.get 6
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 5
    struct.get $"boolean -> ()" 1
    local.get 5
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 8
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    array.new_fixed $capture 0
    call 37
    array.new_fixed $capture 0
    call 25
    local.set 9
    f64.const 0x0p+0 (;=0;)
    struct.new $real
    array.new_fixed $capture 0
    call 13
    local.set 10
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    local.get 10
    struct.get $"real -> real" 1
    local.get 10
    struct.get $"real -> real" 0
    call_ref $"(raw) real -> real"
    local.get 9
    struct.get $"'0 -> boolean" 1
    local.get 9
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 8
    struct.get $"boolean -> ()" 1
    local.get 8
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 11
    i64.const 1
    struct.new $integer
    array.new_fixed $capture 0
    call 1
    local.set 12
    i64.const 2
    struct.new $integer
    local.get 12
    struct.get $"integer -> integer" 1
    local.get 12
    struct.get $"integer -> integer" 0
    call_ref $"(raw) integer -> integer"
    array.new_fixed $capture 0
    call 25
    local.set 13
    i64.const 3
    struct.new $integer
    local.get 13
    struct.get $"'0 -> boolean" 1
    local.get 13
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 11
    struct.get $"boolean -> ()" 1
    local.get 11
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 14
    i64.const 1
    struct.new $integer
    array.new_fixed $capture 0
    call 3
    local.set 15
    i64.const 2
    struct.new $integer
    local.get 15
    struct.get $"integer -> integer" 1
    local.get 15
    struct.get $"integer -> integer" 0
    call_ref $"(raw) integer -> integer"
    array.new_fixed $capture 0
    call 25
    local.set 16
    i64.const 1
    struct.new $integer
    array.new_fixed $capture 0
    call 36
    local.get 16
    struct.get $"'0 -> boolean" 1
    local.get 16
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 14
    struct.get $"boolean -> ()" 1
    local.get 14
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 17
    i64.const 1
    struct.new $integer
    array.new_fixed $capture 0
    call 5
    local.set 18
    i64.const 2
    struct.new $integer
    local.get 18
    struct.get $"integer -> integer" 1
    local.get 18
    struct.get $"integer -> integer" 0
    call_ref $"(raw) integer -> integer"
    array.new_fixed $capture 0
    call 25
    local.set 19
    i64.const 2
    struct.new $integer
    local.get 19
    struct.get $"'0 -> boolean" 1
    local.get 19
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 17
    struct.get $"boolean -> ()" 1
    local.get 17
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 20
    i64.const 1
    struct.new $integer
    array.new_fixed $capture 0
    call 7
    local.set 21
    i64.const 2
    struct.new $integer
    local.get 21
    struct.get $"integer -> integer" 1
    local.get 21
    struct.get $"integer -> integer" 0
    call_ref $"(raw) integer -> integer"
    array.new_fixed $capture 0
    call 25
    local.set 22
    i64.const 0
    struct.new $integer
    local.get 22
    struct.get $"'0 -> boolean" 1
    local.get 22
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 20
    struct.get $"boolean -> ()" 1
    local.get 20
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 23
    i64.const 1
    struct.new $integer
    array.new_fixed $capture 0
    call 9
    local.set 24
    i64.const 2
    struct.new $integer
    local.get 24
    struct.get $"integer -> integer" 1
    local.get 24
    struct.get $"integer -> integer" 0
    call_ref $"(raw) integer -> integer"
    array.new_fixed $capture 0
    call 25
    local.set 25
    i64.const 1
    struct.new $integer
    local.get 25
    struct.get $"'0 -> boolean" 1
    local.get 25
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 23
    struct.get $"boolean -> ()" 1
    local.get 23
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 26
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    array.new_fixed $capture 0
    call 11
    local.set 27
    f64.const 0x1p+1 (;=2;)
    struct.new $real
    local.get 27
    struct.get $"real -> real" 1
    local.get 27
    struct.get $"real -> real" 0
    call_ref $"(raw) real -> real"
    array.new_fixed $capture 0
    call 25
    local.set 28
    f64.const 0x1.8p+1 (;=3;)
    struct.new $real
    local.get 28
    struct.get $"'0 -> boolean" 1
    local.get 28
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 26
    struct.get $"boolean -> ()" 1
    local.get 26
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 29
    f64.const 0x1p+1 (;=2;)
    struct.new $real
    array.new_fixed $capture 0
    call 13
    local.set 30
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    local.get 30
    struct.get $"real -> real" 1
    local.get 30
    struct.get $"real -> real" 0
    call_ref $"(raw) real -> real"
    array.new_fixed $capture 0
    call 25
    local.set 31
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    local.get 31
    struct.get $"'0 -> boolean" 1
    local.get 31
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 29
    struct.get $"boolean -> ()" 1
    local.get 29
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 32
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    array.new_fixed $capture 0
    call 15
    local.set 33
    f64.const 0x1p+1 (;=2;)
    struct.new $real
    local.get 33
    struct.get $"real -> real" 1
    local.get 33
    struct.get $"real -> real" 0
    call_ref $"(raw) real -> real"
    array.new_fixed $capture 0
    call 25
    local.set 34
    f64.const 0x1p+1 (;=2;)
    struct.new $real
    local.get 34
    struct.get $"'0 -> boolean" 1
    local.get 34
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 32
    struct.get $"boolean -> ()" 1
    local.get 32
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 35
    f64.const 0x1p+0 (;=1;)
    struct.new $real
    array.new_fixed $capture 0
    call 17
    local.set 36
    f64.const 0x1p+1 (;=2;)
    struct.new $real
    local.get 36
    struct.get $"real -> real" 1
    local.get 36
    struct.get $"real -> real" 0
    call_ref $"(raw) real -> real"
    array.new_fixed $capture 0
    call 25
    local.set 37
    f64.const 0x1p-1 (;=0.5;)
    struct.new $real
    local.get 37
    struct.get $"'0 -> boolean" 1
    local.get 37
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 35
    struct.get $"boolean -> ()" 1
    local.get 35
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 38
    i32.const 1
    struct.new $boolean
    array.new_fixed $capture 0
    call 19
    local.set 39
    i32.const 1
    struct.new $boolean
    local.get 39
    struct.get $"boolean -> boolean" 1
    local.get 39
    struct.get $"boolean -> boolean" 0
    call_ref $"(raw) boolean -> boolean"
    local.get 38
    struct.get $"boolean -> ()" 1
    local.get 38
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 40
    i32.const 1
    struct.new $boolean
    array.new_fixed $capture 0
    call 21
    local.set 41
    i32.const 0
    struct.new $boolean
    local.get 41
    struct.get $"boolean -> boolean" 1
    local.get 41
    struct.get $"boolean -> boolean" 0
    call_ref $"(raw) boolean -> boolean"
    local.get 40
    struct.get $"boolean -> ()" 1
    local.get 40
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 42
    i32.const 1
    struct.new $boolean
    array.new_fixed $capture 0
    call 23
    local.set 43
    i32.const 0
    struct.new $boolean
    local.get 43
    struct.get $"boolean -> boolean" 1
    local.get 43
    struct.get $"boolean -> boolean" 0
    call_ref $"(raw) boolean -> boolean"
    local.get 42
    struct.get $"boolean -> ()" 1
    local.get 42
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 44
    struct.new $"()"
    array.new_fixed $capture 0
    call 25
    local.set 45
    struct.new $"()"
    local.get 45
    struct.get $"'0 -> boolean" 1
    local.get 45
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    array.new_fixed $capture 0
    call 25
    local.set 46
    i32.const 1
    struct.new $boolean
    local.get 46
    struct.get $"'0 -> boolean" 1
    local.get 46
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 44
    struct.get $"boolean -> ()" 1
    local.get 44
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 47
    struct.new $"()"
    array.new_fixed $capture 0
    call 27
    local.set 48
    struct.new $"()"
    local.get 48
    struct.get $"'0 -> boolean" 1
    local.get 48
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    array.new_fixed $capture 0
    call 25
    local.set 49
    i32.const 0
    struct.new $boolean
    local.get 49
    struct.get $"'0 -> boolean" 1
    local.get 49
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 47
    struct.get $"boolean -> ()" 1
    local.get 47
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 50
    struct.new $"()"
    array.new_fixed $capture 0
    call 29
    local.set 51
    struct.new $"()"
    local.get 51
    struct.get $"'0 -> boolean" 1
    local.get 51
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    array.new_fixed $capture 0
    call 25
    local.set 52
    i32.const 1
    struct.new $boolean
    local.get 52
    struct.get $"'0 -> boolean" 1
    local.get 52
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 50
    struct.get $"boolean -> ()" 1
    local.get 50
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 53
    struct.new $"()"
    array.new_fixed $capture 0
    call 31
    local.set 54
    struct.new $"()"
    local.get 54
    struct.get $"'0 -> boolean" 1
    local.get 54
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    array.new_fixed $capture 0
    call 25
    local.set 55
    i32.const 1
    struct.new $boolean
    local.get 55
    struct.get $"'0 -> boolean" 1
    local.get 55
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 53
    struct.get $"boolean -> ()" 1
    local.get 53
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 56
    struct.new $"()"
    array.new_fixed $capture 0
    call 33
    local.set 57
    struct.new $"()"
    local.get 57
    struct.get $"'0 -> boolean" 1
    local.get 57
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    array.new_fixed $capture 0
    call 25
    local.set 58
    i32.const 0
    struct.new $boolean
    local.get 58
    struct.get $"'0 -> boolean" 1
    local.get 58
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 56
    struct.get $"boolean -> ()" 1
    local.get 56
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 59
    struct.new $"()"
    array.new_fixed $capture 0
    call 35
    local.set 60
    struct.new $"()"
    local.get 60
    struct.get $"'0 -> boolean" 1
    local.get 60
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    array.new_fixed $capture 0
    call 25
    local.set 61
    i32.const 0
    struct.new $boolean
    local.get 61
    struct.get $"'0 -> boolean" 1
    local.get 61
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 59
    struct.get $"boolean -> ()" 1
    local.get 59
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 62
    i32.const 1
    struct.new $boolean
    array.new_fixed $capture 0
    call 25
    local.set 63
    i32.const 1
    struct.new $boolean
    local.get 63
    struct.get $"'0 -> boolean" 1
    local.get 63
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 62
    struct.get $"boolean -> ()" 1
    local.get 62
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 64
    i32.const 1
    struct.new $boolean
    array.new_fixed $capture 0
    call 27
    local.set 65
    i32.const 0
    struct.new $boolean
    local.get 65
    struct.get $"'0 -> boolean" 1
    local.get 65
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 64
    struct.get $"boolean -> ()" 1
    local.get 64
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 66
    i32.const 0
    struct.new $boolean
    array.new_fixed $capture 0
    call 29
    local.set 67
    i32.const 1
    struct.new $boolean
    local.get 67
    struct.get $"'0 -> boolean" 1
    local.get 67
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 66
    struct.get $"boolean -> ()" 1
    local.get 66
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 68
    i32.const 1
    struct.new $boolean
    array.new_fixed $capture 0
    call 31
    local.set 69
    i32.const 0
    struct.new $boolean
    local.get 69
    struct.get $"'0 -> boolean" 1
    local.get 69
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 68
    struct.get $"boolean -> ()" 1
    local.get 68
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 70
    i32.const 0
    struct.new $boolean
    array.new_fixed $capture 0
    call 33
    local.set 71
    i32.const 1
    struct.new $boolean
    local.get 71
    struct.get $"'0 -> boolean" 1
    local.get 71
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 70
    struct.get $"boolean -> ()" 1
    local.get 70
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 72
    i32.const 1
    struct.new $boolean
    array.new_fixed $capture 0
    call 35
    local.set 73
    i32.const 0
    struct.new $boolean
    local.get 73
    struct.get $"'0 -> boolean" 1
    local.get 73
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 72
    struct.get $"boolean -> ()" 1
    local.get 72
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 74
    i32.const 97
    struct.new $glyph
    array.new_fixed $capture 0
    call 25
    local.set 75
    i32.const 97
    struct.new $glyph
    local.get 75
    struct.get $"'0 -> boolean" 1
    local.get 75
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 74
    struct.get $"boolean -> ()" 1
    local.get 74
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 76
    i32.const 97
    struct.new $glyph
    array.new_fixed $capture 0
    call 27
    local.set 77
    i32.const 98
    struct.new $glyph
    local.get 77
    struct.get $"'0 -> boolean" 1
    local.get 77
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 76
    struct.get $"boolean -> ()" 1
    local.get 76
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 78
    i32.const 97
    struct.new $glyph
    array.new_fixed $capture 0
    call 29
    local.set 79
    i32.const 98
    struct.new $glyph
    local.get 79
    struct.get $"'0 -> boolean" 1
    local.get 79
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 78
    struct.get $"boolean -> ()" 1
    local.get 78
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 80
    i32.const 98
    struct.new $glyph
    array.new_fixed $capture 0
    call 31
    local.set 81
    i32.const 97
    struct.new $glyph
    local.get 81
    struct.get $"'0 -> boolean" 1
    local.get 81
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 80
    struct.get $"boolean -> ()" 1
    local.get 80
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 82
    i32.const 97
    struct.new $glyph
    array.new_fixed $capture 0
    call 33
    local.set 83
    i32.const 98
    struct.new $glyph
    local.get 83
    struct.get $"'0 -> boolean" 1
    local.get 83
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 82
    struct.get $"boolean -> ()" 1
    local.get 82
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 84
    i32.const 98
    struct.new $glyph
    array.new_fixed $capture 0
    call 35
    local.set 85
    i32.const 97
    struct.new $glyph
    local.get 85
    struct.get $"'0 -> boolean" 1
    local.get 85
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 84
    struct.get $"boolean -> ()" 1
    local.get 84
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 86
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    array.new_fixed $capture 0
    call 25
    local.set 87
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    local.get 87
    struct.get $"'0 -> boolean" 1
    local.get 87
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 86
    struct.get $"boolean -> ()" 1
    local.get 86
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 88
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    array.new_fixed $capture 0
    call 27
    local.set 89
    i32.const 100
    i32.const 101
    i32.const 102
    array.new_fixed $string 3
    local.get 89
    struct.get $"'0 -> boolean" 1
    local.get 89
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 88
    struct.get $"boolean -> ()" 1
    local.get 88
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 90
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    array.new_fixed $capture 0
    call 29
    local.set 91
    i32.const 100
    i32.const 101
    i32.const 102
    array.new_fixed $string 3
    local.get 91
    struct.get $"'0 -> boolean" 1
    local.get 91
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 90
    struct.get $"boolean -> ()" 1
    local.get 90
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 92
    i32.const 100
    i32.const 101
    i32.const 102
    array.new_fixed $string 3
    array.new_fixed $capture 0
    call 31
    local.set 93
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    local.get 93
    struct.get $"'0 -> boolean" 1
    local.get 93
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 92
    struct.get $"boolean -> ()" 1
    local.get 92
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 94
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    array.new_fixed $capture 0
    call 29
    local.set 95
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    local.get 95
    struct.get $"'0 -> boolean" 1
    local.get 95
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 94
    struct.get $"boolean -> ()" 1
    local.get 94
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 96
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    array.new_fixed $capture 0
    call 31
    local.set 97
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    local.get 97
    struct.get $"'0 -> boolean" 1
    local.get 97
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 96
    struct.get $"boolean -> ()" 1
    local.get 96
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 98
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    array.new_fixed $capture 0
    call 33
    local.set 99
    i32.const 100
    i32.const 101
    i32.const 102
    array.new_fixed $string 3
    local.get 99
    struct.get $"'0 -> boolean" 1
    local.get 99
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 98
    struct.get $"boolean -> ()" 1
    local.get 98
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
    ref.func 39
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 100
    i32.const 100
    i32.const 101
    i32.const 102
    array.new_fixed $string 3
    array.new_fixed $capture 0
    call 35
    local.set 101
    i32.const 97
    i32.const 98
    i32.const 99
    array.new_fixed $string 3
    local.get 101
    struct.get $"'0 -> boolean" 1
    local.get 101
    struct.get $"'0 -> boolean" 0
    call_ref $"(raw) '0 -> boolean"
    local.get 100
    struct.get $"boolean -> ()" 1
    local.get 100
    struct.get $"boolean -> ()" 0
    call_ref $"(raw) boolean -> ()"
    drop
  )
)
