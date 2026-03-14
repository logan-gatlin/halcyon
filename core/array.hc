module array =
  use core
  use core::ops
  use core::opt
  let empty : for a in Array a = (wasm : for a in Array a) => (
    i32.const 0
    array.new_default any
  )

  let concat : for a in Array a -> Array a -> Array a =
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

  let append_many = concat

  let push : for a in a -> Array a -> Array a =
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

  let append = fn arr value => push value arr
  let singleton = fn value => [value]

  let is_empty = fn arr => match arr with
    | [] => true
    | [_, ..] => false

  let non_empty = fn arr => not (is_empty arr)

  let head = fn arr => match arr with
    | [] => None
    | [value, ..] => Some value

  let head_or = fn backup arr => match arr with
    | [] => backup
    | [value, ..] => value

  let head_or_else = fn backup_fn arr => match arr with
    | [] => backup_fn ()
    | [value, ..] => value

  let flat_map = fn f arr =>
    match arr with
      | [] => []
      | [value, ..] => f value

  impl Default for a in Array a =
    let default = []
  end

  impl ops::Add for a in Array a =
    let [+] = fn left right => array::concat left right
  end

  impl bundle::hkt::Applicative Array =
    let apply = fn wrapped_fn wrapped_value =>
      array::flat_map
        (fn f => array::flat_map (fn value => [f value]) wrapped_value)
        wrapped_fn
  end

  impl bundle::hkt::Traversable Array =
    let traverse = fn f arr => match arr with
      | [] => bundle::hkt::new []
      | [value, ..] => bundle::hkt::map (fn mapped => [mapped]) (f value)
  end

  impl bundle::hkt::Foldable Array =
    let fold = fn step initial arr => match arr with
      | [] => initial
      | [value, ..] => step initial value
  end

  impl bundle::hkt::Alternative Array =
    let empty = []
    let or_else = array::concat
  end

  impl bundle::hkt::Functor Array =
    let fmap = fn f arr => array::flat_map (fn value => [f value]) arr
  end

  impl bundle::hkt::Zip Array =
    let zip_with = fn f left right => match left with
      | [] => []
      | [left_value, ..] =>
        match right with
          | [] => []
          | [right_value, ..] => [f left_value right_value]
  end

  impl bundle::hkt::Filterable Array =
    let filter = fn predicate arr => match arr with
      | [] => []
      | [value, ..] => if predicate value then [value] else []
  end

  impl bundle::hkt::Monad Array =
    let new = array::singleton
    let flatmap = array::flat_map
  end

end
