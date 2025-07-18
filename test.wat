(module
  (type (;0;) (func (param i32)))
  (type $integer (;1;) (struct (field i64)))
  (type $capture (;2;) (array (mut anyref)))
  (type $"(raw) integer -> integer" (;3;) (func (param (ref $integer) (ref $capture)) (result (ref $integer))))
  (type $"integer -> integer" (;4;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) integer -> integer -> integer" (;5;) (func (param (ref $integer) (ref $capture)) (result (ref $"integer -> integer"))))
  (type $real (;6;) (struct (field f64)))
  (type $"(raw) real -> real" (;7;) (func (param (ref $real) (ref $capture)) (result (ref $real))))
  (type $"real -> real" (;8;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) real -> real -> real" (;9;) (func (param (ref $real) (ref $capture)) (result (ref $"real -> real"))))
  (type $boolean (;10;) (struct (field i32)))
  (type $"(raw) boolean -> boolean" (;11;) (func (param (ref $boolean) (ref $capture)) (result (ref $boolean))))
  (type $"boolean -> boolean" (;12;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) boolean -> boolean -> boolean" (;13;) (func (param (ref $boolean) (ref $capture)) (result (ref $"boolean -> boolean"))))
  (type $glyph (;14;) (struct (field i32)))
  (type $"()" (;15;) (struct))
  (type $string (;16;) (array (mut i8)))
  (type $"(raw) '0 -> boolean" (;17;) (func (param anyref (ref $capture)) (result (ref $boolean))))
  (type $"'0 -> boolean" (;18;) (struct (field i32) (field (ref $capture))))
  (type $"(raw) '0 -> '0 -> boolean" (;19;) (func (param anyref (ref $capture)) (result (ref $"'0 -> boolean"))))
  (type $"(raw) boolean -> ()" (;20;) (func (param (ref $boolean) (ref $capture)) (result (ref $"()"))))
  (type (;21;) (func))
  (type $"boolean -> ()" (;22;) (struct (field i32) (field (ref $capture))))
  (import "sys" "print_integer" (func (;0;) (type 0)))
  (table (;0;) 42 42 funcref)
  (start 41)
  (elem (;0;) (i32.const 0) func 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41)
  (func (;1;) (type $"(raw) integer -> integer") (param $a (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 2
    local.get 2
    struct.get $integer 0
    local.get $a
    struct.get $integer 0
    i64.add
    struct.new $integer
  )
  (func (;2;) (type $"(raw) integer -> integer -> integer") (param $b (ref $integer)) (param (ref $capture)) (result (ref $"integer -> integer"))
    i32.const 1
    local.get $b
    array.new_fixed $capture 1
    struct.new $"integer -> integer"
  )
  (func (;3;) (type $"(raw) integer -> integer") (param $a (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 2
    local.get 2
    struct.get $integer 0
    local.get $a
    struct.get $integer 0
    i64.sub
    struct.new $integer
  )
  (func (;4;) (type $"(raw) integer -> integer -> integer") (param $b (ref $integer)) (param (ref $capture)) (result (ref $"integer -> integer"))
    i32.const 3
    local.get $b
    array.new_fixed $capture 1
    struct.new $"integer -> integer"
  )
  (func (;5;) (type $"(raw) integer -> integer") (param $a (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 2
    local.get 2
    struct.get $integer 0
    local.get $a
    struct.get $integer 0
    i64.mul
    struct.new $integer
  )
  (func (;6;) (type $"(raw) integer -> integer -> integer") (param $b (ref $integer)) (param (ref $capture)) (result (ref $"integer -> integer"))
    i32.const 5
    local.get $b
    array.new_fixed $capture 1
    struct.new $"integer -> integer"
  )
  (func (;7;) (type $"(raw) integer -> integer") (param $a (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 2
    local.get 2
    struct.get $integer 0
    local.get $a
    struct.get $integer 0
    i64.div_s
    struct.new $integer
  )
  (func (;8;) (type $"(raw) integer -> integer -> integer") (param $b (ref $integer)) (param (ref $capture)) (result (ref $"integer -> integer"))
    i32.const 7
    local.get $b
    array.new_fixed $capture 1
    struct.new $"integer -> integer"
  )
  (func (;9;) (type $"(raw) integer -> integer") (param $a (ref $integer)) (param (ref $capture)) (result (ref $integer))
    (local (ref $integer))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $integer)
    local.set 2
    local.get 2
    struct.get $integer 0
    local.get $a
    struct.get $integer 0
    i64.rem_s
    struct.new $integer
  )
  (func (;10;) (type $"(raw) integer -> integer -> integer") (param $b (ref $integer)) (param (ref $capture)) (result (ref $"integer -> integer"))
    i32.const 9
    local.get $b
    array.new_fixed $capture 1
    struct.new $"integer -> integer"
  )
  (func (;11;) (type $"(raw) real -> real") (param $a (ref $real)) (param (ref $capture)) (result (ref $real))
    (local (ref $real))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set 2
    local.get 2
    struct.get $real 0
    local.get $a
    struct.get $real 0
    f64.add
    struct.new $real
  )
  (func (;12;) (type $"(raw) real -> real -> real") (param $b (ref $real)) (param (ref $capture)) (result (ref $"real -> real"))
    i32.const 11
    local.get $b
    array.new_fixed $capture 1
    struct.new $"real -> real"
  )
  (func (;13;) (type $"(raw) real -> real") (param $a (ref $real)) (param (ref $capture)) (result (ref $real))
    (local (ref $real))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set 2
    local.get 2
    struct.get $real 0
    local.get $a
    struct.get $real 0
    f64.sub
    struct.new $real
  )
  (func (;14;) (type $"(raw) real -> real -> real") (param $b (ref $real)) (param (ref $capture)) (result (ref $"real -> real"))
    i32.const 13
    local.get $b
    array.new_fixed $capture 1
    struct.new $"real -> real"
  )
  (func (;15;) (type $"(raw) real -> real") (param $a (ref $real)) (param (ref $capture)) (result (ref $real))
    (local (ref $real))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set 2
    local.get 2
    struct.get $real 0
    local.get $a
    struct.get $real 0
    f64.mul
    struct.new $real
  )
  (func (;16;) (type $"(raw) real -> real -> real") (param $b (ref $real)) (param (ref $capture)) (result (ref $"real -> real"))
    i32.const 15
    local.get $b
    array.new_fixed $capture 1
    struct.new $"real -> real"
  )
  (func (;17;) (type $"(raw) real -> real") (param $a (ref $real)) (param (ref $capture)) (result (ref $real))
    (local (ref $real))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $real)
    local.set 2
    local.get 2
    struct.get $real 0
    local.get $a
    struct.get $real 0
    f64.div
    struct.new $real
  )
  (func (;18;) (type $"(raw) real -> real -> real") (param $b (ref $real)) (param (ref $capture)) (result (ref $"real -> real"))
    i32.const 17
    local.get $b
    array.new_fixed $capture 1
    struct.new $"real -> real"
  )
  (func (;19;) (type $"(raw) boolean -> boolean") (param $a (ref $boolean)) (param (ref $capture)) (result (ref $boolean))
    (local (ref $boolean))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $boolean)
    local.set 2
    local.get 2
    struct.get $boolean 0
    local.get $a
    struct.get $boolean 0
    i32.and
    struct.new $boolean
  )
  (func (;20;) (type $"(raw) boolean -> boolean -> boolean") (param $b (ref $boolean)) (param (ref $capture)) (result (ref $"boolean -> boolean"))
    i32.const 19
    local.get $b
    array.new_fixed $capture 1
    struct.new $"boolean -> boolean"
  )
  (func (;21;) (type $"(raw) boolean -> boolean") (param $a (ref $boolean)) (param (ref $capture)) (result (ref $boolean))
    (local (ref $boolean))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $boolean)
    local.set 2
    local.get 2
    struct.get $boolean 0
    local.get $a
    struct.get $boolean 0
    i32.or
    struct.new $boolean
  )
  (func (;22;) (type $"(raw) boolean -> boolean -> boolean") (param $b (ref $boolean)) (param (ref $capture)) (result (ref $"boolean -> boolean"))
    i32.const 21
    local.get $b
    array.new_fixed $capture 1
    struct.new $"boolean -> boolean"
  )
  (func (;23;) (type $"(raw) boolean -> boolean") (param $a (ref $boolean)) (param (ref $capture)) (result (ref $boolean))
    (local (ref $boolean))
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref $boolean)
    local.set 2
    local.get 2
    struct.get $boolean 0
    local.get $a
    struct.get $boolean 0
    i32.xor
    struct.new $boolean
  )
  (func (;24;) (type $"(raw) boolean -> boolean -> boolean") (param $b (ref $boolean)) (param (ref $capture)) (result (ref $"boolean -> boolean"))
    i32.const 23
    local.get $b
    array.new_fixed $capture 1
    struct.new $"boolean -> boolean"
  )
  (func (;25;) (type $"(raw) '0 -> boolean") (param $a anyref) (param (ref $capture)) (result (ref $boolean))
    (local anyref i32 i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 2
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 2
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
              local.get $a
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.eq
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $a
            ref.cast (ref $real)
            struct.get $real 0
            f64.eq
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $a
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.eq
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $a
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
    local.get $a
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    local.get 2
    ref.cast (ref $string)
    array.len
    local.get $a
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
    local.get 2
    ref.cast (ref $string)
    array.len
    local.set 4
    loop ;; label = @1
      local.get 2
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $a
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
      local.get 4
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;26;) (type $"(raw) '0 -> '0 -> boolean") (param $b anyref) (param (ref $capture)) (result (ref $"'0 -> boolean"))
    i32.const 25
    local.get $b
    array.new_fixed $capture 1
    struct.new $"'0 -> boolean"
  )
  (func (;27;) (type $"(raw) '0 -> boolean") (param $a anyref) (param (ref $capture)) (result (ref $boolean))
    (local anyref i32 i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 2
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 2
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
              local.get $a
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.ne
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $a
            ref.cast (ref $real)
            struct.get $real 0
            f64.ne
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $a
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.ne
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $a
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
    local.get $a
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    local.get 2
    ref.cast (ref $string)
    array.len
    local.get $a
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
    local.get 2
    ref.cast (ref $string)
    array.len
    local.set 4
    loop ;; label = @1
      local.get 2
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $a
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
      local.get 4
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;28;) (type $"(raw) '0 -> '0 -> boolean") (param $b anyref) (param (ref $capture)) (result (ref $"'0 -> boolean"))
    i32.const 27
    local.get $b
    array.new_fixed $capture 1
    struct.new $"'0 -> boolean"
  )
  (func (;29;) (type $"(raw) '0 -> boolean") (param $a anyref) (param (ref $capture)) (result (ref $boolean))
    (local anyref i32 i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 2
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 2
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
              local.get $a
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.le_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $a
            ref.cast (ref $real)
            struct.get $real 0
            f64.le
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $a
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.le_u
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $a
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
    local.get $a
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    local.get 2
    ref.cast (ref $string)
    array.len
    local.get $a
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
    local.get 2
    ref.cast (ref $string)
    array.len
    local.set 4
    loop ;; label = @1
      local.get 2
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $a
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
      local.get 4
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;30;) (type $"(raw) '0 -> '0 -> boolean") (param $b anyref) (param (ref $capture)) (result (ref $"'0 -> boolean"))
    i32.const 29
    local.get $b
    array.new_fixed $capture 1
    struct.new $"'0 -> boolean"
  )
  (func (;31;) (type $"(raw) '0 -> boolean") (param $a anyref) (param (ref $capture)) (result (ref $boolean))
    (local anyref i32 i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 2
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 2
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
              local.get $a
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.ge_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $a
            ref.cast (ref $real)
            struct.get $real 0
            f64.ge
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $a
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.ge_u
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $a
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
    local.get $a
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    local.get 2
    ref.cast (ref $string)
    array.len
    local.get $a
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
    local.get 2
    ref.cast (ref $string)
    array.len
    local.set 4
    loop ;; label = @1
      local.get 2
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $a
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
      local.get 4
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;32;) (type $"(raw) '0 -> '0 -> boolean") (param $b anyref) (param (ref $capture)) (result (ref $"'0 -> boolean"))
    i32.const 31
    local.get $b
    array.new_fixed $capture 1
    struct.new $"'0 -> boolean"
  )
  (func (;33;) (type $"(raw) '0 -> boolean") (param $a anyref) (param (ref $capture)) (result (ref $boolean))
    (local anyref i32 i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 2
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 2
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
              local.get $a
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.lt_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $a
            ref.cast (ref $real)
            struct.get $real 0
            f64.lt
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $a
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.lt_u
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $a
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
    local.get $a
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 0
      struct.new $boolean
      return
    end
    local.get 2
    ref.cast (ref $string)
    array.len
    local.get $a
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
    local.get 2
    ref.cast (ref $string)
    array.len
    local.set 4
    loop ;; label = @1
      local.get 2
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $a
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
      local.get 4
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;34;) (type $"(raw) '0 -> '0 -> boolean") (param $b anyref) (param (ref $capture)) (result (ref $"'0 -> boolean"))
    i32.const 33
    local.get $b
    array.new_fixed $capture 1
    struct.new $"'0 -> boolean"
  )
  (func (;35;) (type $"(raw) '0 -> boolean") (param $a anyref) (param (ref $capture)) (result (ref $boolean))
    (local anyref i32 i32)
    local.get 1
    i32.const 0
    array.get $capture
    ref.cast (ref any)
    local.set 2
    block (result anyref) ;; label = @1
      block (result anyref) ;; label = @2
        block (result anyref) ;; label = @3
          block (result anyref) ;; label = @4
            block (result anyref) ;; label = @5
              block (result anyref) ;; label = @6
                local.get 2
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
              local.get $a
              ref.cast (ref $integer)
              struct.get $integer 0
              i64.gt_s
              struct.new $boolean
              return
            end
            ref.cast (ref $real)
            struct.get $real 0
            local.get $a
            ref.cast (ref $real)
            struct.get $real 0
            f64.gt
            struct.new $boolean
            return
          end
          ref.cast (ref $boolean)
          struct.get $boolean 0
          local.get $a
          ref.cast (ref $boolean)
          struct.get $boolean 0
          i32.gt_u
          struct.new $boolean
          return
        end
        ref.cast (ref $glyph)
        struct.get $glyph 0
        local.get $a
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
    local.get $a
    ref.cast (ref $string)
    array.len
    i32.gt_u
    if ;; label = @1
      i32.const 1
      struct.new $boolean
      return
    end
    local.get 2
    ref.cast (ref $string)
    array.len
    local.get $a
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
    local.get 2
    ref.cast (ref $string)
    array.len
    local.set 4
    loop ;; label = @1
      local.get 2
      ref.cast (ref $string)
      local.get 3
      array.get_u $string
      local.get $a
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
      local.get 4
      i32.lt_u
      br_if 0 (;@1;)
    end
    i32.const 1
    struct.new $boolean
    return
  )
  (func (;36;) (type $"(raw) '0 -> '0 -> boolean") (param $a anyref) (param (ref $capture)) (result (ref $"'0 -> boolean"))
    i32.const 35
    local.get $a
    array.new_fixed $capture 1
    struct.new $"'0 -> boolean"
  )
  (func (;37;) (type $"(raw) integer -> integer") (param $a (ref $integer)) (param (ref $capture)) (result (ref $integer))
    i64.const 0
    local.get $a
    struct.get $integer 0
    i64.sub
    struct.new $integer
  )
  (func (;38;) (type $"(raw) real -> real") (param $a (ref $real)) (param (ref $capture)) (result (ref $real))
    local.get $a
    struct.get $real 0
    f64.neg
    struct.new $real
  )
  (func (;39;) (type $"(raw) boolean -> boolean") (param $a (ref $boolean)) (param (ref $capture)) (result (ref $boolean))
    local.get $a
    struct.get $boolean 0
    i32.eqz
    struct.new $boolean
  )
  (func (;40;) (type $"(raw) boolean -> ()") (param (ref $boolean) (ref $capture)) (result (ref $"()"))
    local.get 0
    struct.get $boolean 0
    i32.eqz
    if ;; label = @1
      unreachable
    end
    struct.new $"()"
  )
  (func (;41;) (type 21)
    (local (ref $"boolean -> ()"))
    i32.const 40
    array.new_fixed $capture 0
    struct.new $"boolean -> ()"
    ref.cast (ref $"boolean -> ()")
    local.set 0
    i32.const 1
    struct.new $boolean
    local.get 0
    struct.get $"boolean -> ()" 1
    local.get 0
    struct.get $"boolean -> ()" 0
    call_indirect (type $"(raw) boolean -> ()")
    drop
  )
)
