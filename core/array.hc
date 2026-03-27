module array =
  use bundle
  use bundle::ops
  use bundle::opt
  use bundle::hkt

  (*>
  Empty array value.

  - Arguments: none.
  - Returns: `[]` with polymorphic element type.
  *)
  let empty : for a in Array a = (wasm : for a in Array a) => (
    i32.const 0
    array.new_default any
  )

  (*>
  Concatenates two arrays.

  - Arguments:
    - `left`: First array.
    - `right`: Second array.
  - Returns: New array with all `left` items followed by all `right` items.

  ```hc
  let merged = array::concat [1, 2] [3, 4]
  ```
  *)
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

  (*>
  Appends one value to the end of an array.

  - Arguments:
    - `value`: Item to append.
    - `arr`: Source array.
  - Returns: New array with `value` added at the end.

  ```hc
  let extended = array::push 3 [1, 2]
  ```
  *)
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

  (*>
  Builds a single-item array.

  - Arguments:
    - `value`: Item to store.
  - Returns: Array containing only `value`.
  *)
  let singleton = fn value => [value]

  (*>
  Checks whether an array has no elements.

  - Arguments:
    - `arr`: Array to inspect.
  - Returns: `true` when `arr` is empty.
  *)
  let is_empty = fn arr => match arr with
    | [] => true
    | [_, ..] => false

  (*>
  Checks whether an array has at least one element.

  - Arguments:
    - `arr`: Array to inspect.
  - Returns: `true` when `arr` is non-empty.
  *)
  let non_empty = fn arr => not (is_empty arr)

  (*>
  Returns the first element as an option.

  - Arguments:
    - `arr`: Array to inspect.
  - Returns: `Some first_value` or `None` for empty arrays.
  *)
  let head = fn arr => match arr with
    | [] => None
    | [value, ..] => Some value

  (*>
  Maps a function over every array element.

  - Arguments:
    - `f`: Mapping function.
    - `arr`: Source array.
  - Returns: New array with mapped values.

  ```hc
  let doubled = array::map (fn n => n * 2) [1, 2, 3]
  ```
  *)
  let map = fn f arr =>
    match arr with
      | [] => []
      | [head, ..tail] => [f head] + (array::map f tail)

  (*>
  Flattens one level of nested arrays.

  - Arguments:
    - `arr`: Array of arrays.
  - Returns: Concatenated flat array.

  ```hc
  let flat = array::flatten [[1, 2], [3], []]
  ```
  *)
  let flatten = fn arr =>
    match arr with
      | [] => []
      | [head, ..tail] => array::concat head (array::flatten tail)

  --> @HIDDEN
  impl bundle::hkt::Monad Array =
    let new = fn value => singleton value
    let flat_map = fn f arr => match arr with
      | [] => []
      | [value, ..tail] => array::concat (f value) (hkt::flat_map f tail)
  end

  (*>
  Traverses an array with an effectful function.

  - Arguments:
    - `f`: Effectful mapping function.
    - `arr`: Source array.
  - Returns: Effect wrapping the rebuilt array.

  ```hc
  let parsed = array::traverse parse_int ["1", "2", "3"]
  ```
  *)
  let traverse = fn f arr => match arr with
    | [] => bundle::hkt::new []
    | [value, ..tail] =>
      bundle::hkt::lift2
        (fn mapped_value mapped_tail => array::concat [mapped_value] mapped_tail)
        (f value)
        (array::traverse f tail)

  (*>
  Left-folds an array.

  - Arguments:
    - `step`: Accumulator step function.
    - `initial`: Initial accumulator value.
    - `arr`: Source array.
  - Returns: Final accumulated result.

  ```hc
  let total = array::fold (fn acc n => acc + n) 0 [1, 2, 3]
  ```
  *)
  let fold = fn step initial arr => match arr with
    | [] => initial
    | [value, ..tail] => array::fold step (step initial value) tail

  (*>
  Zips two arrays with a combining function.

  - Arguments:
    - `f`: Combining function.
    - `left`: First array.
    - `right`: Second array.
  - Returns: Array of combined values up to the shorter input length.

  ```hc
  let sums = array::zip_with (fn a b => a + b) [1, 2, 3] [10, 20]
  ```
  *)
  let zip_with = fn f left right => match left with
    | [] => []
    | [left_value, ..left_tail] =>
      match right with
        | [] => []
        | [right_value, ..right_tail] =>
          [f left_value right_value] + (array::zip_with f left_tail right_tail)

  (*>
  Filters array elements by a predicate.

  - Arguments:
    - `predicate`: Keep-condition for values.
    - `arr`: Source array.
  - Returns: Array containing only values that satisfy `predicate`.

  ```hc
  let large = array::filter (fn n => n > 2) [1, 2, 3, 4]
  ```
  *)
  let filter = fn predicate arr => match arr with
    | [] => []
    | [value, ..tail] =>
      if predicate value
        then [value] + (array::filter predicate tail)
        else array::filter predicate tail

  --> @HIDDEN
  let equal_with = fn compare left right =>
    match left with
      | [] => array::is_empty right
      | [left_value, ..left_tail] =>
        match right with
          | [] => false
          | [right_value, ..right_tail] =>
            if compare left_value right_value
              then array::equal_with compare left_tail right_tail
              else false

  --> @HIDDEN
  let show_items = fn arr =>
    match arr with
      | [] => ""
      | [value] => bundle::show::show value
      | [value, ..tail] => (bundle::show::show value) + ", " + (show_items tail)

  --> @HIDDEN
  impl Default for a in Array a =
    let default = []
  end

  --> @HIDDEN
  impl ops::Add for a in Array a =
    let [+] = fn left right => array::concat left right
  end

  --> @HIDDEN
  impl ops::Equal for a in Array a where ops::Equal a =
    let [==] = fn left right =>
      let compare = bundle::ops::[==] in
      array::equal_with compare left right
  end

  --> @HIDDEN
  impl bundle::show::Show for a in Array a where bundle::show::Show a =
    let show = fn arr => "[" + (show_items arr) + "]"
  end

  --> @HIDDEN
  impl bundle::hkt::Applicative Array =
    let apply = fn wrapped_fn wrapped_value =>
      flat_map
        (fn f => array::map f wrapped_value)
        wrapped_fn
  end

  --> @HIDDEN
  impl bundle::hkt::Traversable Array =
    let traverse = fn f arr => array::traverse f arr
  end

  --> @HIDDEN
  impl bundle::hkt::Foldable Array =
    let fold = fn step initial arr => array::fold step initial arr
  end

  --> @HIDDEN
  impl bundle::hkt::Alternative Array =
    let empty = []
    let or_else = fn left right => array::concat left right
  end

  --> @HIDDEN
  impl bundle::hkt::Functor Array =
    let fmap = fn f arr => array::map f arr
  end

  --> @HIDDEN
  impl bundle::hkt::Zip Array =
    let zip_with = fn f left right => array::zip_with f left right
  end

  --> @HIDDEN
  impl bundle::hkt::Filterable Array =
    let filter = fn predicate arr => array::filter predicate arr
  end


end
