module opt =
  use bundle
  use bundle::ops

  (*>
  Optional value container.

  `Some a` represents presence; `None` represents absence.
  *)
  type Option: a = | Some a | None

  (*>
  Maps a function over an option.

  - Arguments:
    - `f`: Mapping function.
    - `opt`: Source option.
  - Returns: `Some (f value)` when present, otherwise `None`.
  *)
  let map = fn f opt => match opt with
    | Some a => Some (f a)
    | None => None

  --> @HIDDEN
  let map_or = fn backup f opt => match opt with
    | Some a => f a
    | None => backup

  --> @HIDDEN
  let map_or_else = fn backup_fn f opt => match opt with
    | Some a => f a
    | None => backup_fn ()

  (*>
  Chains an option-producing function.

  - Arguments:
    - `f`: Function from `a` to `Option b`.
    - `opt`: Source option.
  - Returns: Result of `f` for `Some`, otherwise `None`.

  ```hc
  let parsed = opt::and_then parse_port maybe_port_text
  ```
  *)
  let and_then = fn f opt => match opt with
    | Some a => f a
    | None => None

  --> @HIDDEN
  let and_with = fn next opt => match opt with
    | Some _ => next
    | None => None

  --> @HIDDEN
  let or_with = fn fallback opt => match opt with
    | Some a => Some a
    | None => fallback

  (*>
  Provides a fallback option lazily.

  - Arguments:
    - `fallback_fn`: Function called when `opt` is `None`.
    - `opt`: Source option.
  - Returns: Original `Some` value, or fallback option.
  *)
  let or_else = fn fallback_fn opt => match opt with
    | Some a => Some a
    | None => fallback_fn ()

  --> @HIDDEN
  let xor_with = fn left right => match left with
    | Some left_value =>
      match right with
        | Some _ => None
        | None => Some left_value
    | None => right

  (*>
  Zips two options into one pair.

  - Arguments:
    - `left`: First option.
    - `right`: Second option.
  - Returns: `Some (left_value, right_value)` when both are `Some`.
  *)
  let zip = fn left right => match left with
    | Some left_value =>
      match right with
        | Some right_value => Some (left_value, right_value)
        | None => None
    | None => None

  (*>
  Keeps an option only when its value satisfies a predicate.

  - Arguments:
    - `predicate`: Keep-condition.
    - `opt`: Source option.
  - Returns: Original `Some` when predicate passes, else `None`.
  *)
  let filter = fn predicate opt => match opt with
    | Some a => if predicate a then Some a else None
    | None => None

  (*>
  Returns `true` when an option contains a value.

  - Arguments:
    - `opt`: Option to inspect.
  - Returns: Presence flag.
  *)
  let is_some = fn opt => match opt with
    | Some a => true
    | None => false

  (*>
  Returns `true` when an option is empty.

  - Arguments:
    - `opt`: Option to inspect.
  - Returns: Absence flag.
  *)
  let is_none = [not] >> is_some

  (*>
  Checks whether an option equals a target value.

  - Arguments:
    - `value`: Value to compare against.
    - `opt`: Option to inspect.
  - Returns: `true` for `Some inner` where `inner == value`.
  *)
  let contains = fn value opt => match opt with
    | Some inner => inner == value
    | None => false

  (*>
  Converts an option into an array of zero or one element.

  - Arguments:
    - `opt`: Option to convert.
  - Returns: `[]` for `None`, `[value]` for `Some value`.
  *)
  let to_array = fn opt => match opt with
    | Some value => (wasm : for a in Array a) => (
      get value
      array.new_fixed any 1
    )
    | None => (wasm : for a in Array a) => (
      i32.const 0
      array.new_default any
    )

  (*>
  Returns the first array element as an option.

  - Arguments:
    - `arr`: Source array.
  - Returns: `Some first_value` or `None` when empty.
  *)
  let from_array_head = fn arr => match arr with
    | [value, ..] => Some value
    | [] => None

  --> @HIDDEN
  let zip_with = fn f left right =>
    and_then (fn left_value => map (fn right_value => f left_value right_value) right) left

  --> @HIDDEN
  let match_or = fn none_value some_fn opt => match opt with
    | Some value => some_fn value
    | None => none_value

  (*>
  Extracts the value from an option, or uses a fallback.

  - Arguments:
    - `backup`: Fallback value.
    - `opt`: Source option.
  - Returns: Inner value for `Some`, otherwise `backup`.
  *)
  let unwrap_or = fn backup opt => match opt with
    | Some a => a
    | None => backup

  (*>
  Extracts the value from an option, or computes a fallback lazily.

  - Arguments:
    - `backup_fn`: Fallback thunk.
    - `opt`: Source option.
  - Returns: Inner value for `Some`, otherwise `backup_fn ()`.
  *)
  let unwrap_or_else = fn backup_fn opt => match opt with
    | Some a => a
    | None => backup_fn ()

  --> @HIDDEN
  impl bundle::Default for a in Option a =
    let default = None
  end

  --> @HIDDEN
  impl bundle::ops::Equal for a in Option a where bundle::ops::Equal a =
    let [==] = fn left right => match left with
      | Some left_value =>
        match right with
          | Some right_value => bundle::ops::[==] left_value right_value
          | None => false
      | None =>
        match right with
          | Some _ => false
          | None => true
  end

  --> @HIDDEN
  impl bundle::show::Show for a in Option a where bundle::show::Show a =
    let show = fn value =>
      match value with
        | Some inner => "Some(" + (bundle::show::show inner) + ")"
        | None => "None"
  end

  --> @HIDDEN
  impl bundle::hkt::Applicative Option =
    let apply = fn wrapped_fn wrapped_value =>
      opt::and_then (fn f => opt::map f wrapped_value) wrapped_fn
  end

  --> @HIDDEN
  impl bundle::hkt::Traversable Option =
    let traverse = fn f opt => match opt with
      | Some value => bundle::hkt::map (f value) Some
      | None => bundle::hkt::new None
  end

  --> @HIDDEN
  impl bundle::hkt::Foldable Option =
    let fold = fn step initial opt => match opt with
      | Some value => step initial value
      | None => initial
  end

  --> @HIDDEN
  impl bundle::hkt::Alternative Option =
    let empty = None
    let or_else = fn left right => opt::or_with right left
  end

  --> @HIDDEN
  impl bundle::hkt::Functor Option =
    let fmap = fn f value => opt::map f value
  end

  --> @HIDDEN
  impl bundle::hkt::Zip Option =
    let zip_with = fn f left right => opt::zip_with f left right
  end

  --> @HIDDEN
  impl bundle::hkt::Filterable Option =
    let filter = fn predicate value => opt::filter predicate value
  end

  --> @HIDDEN
  impl bundle::hkt::Monad Option =
    let new = fn value => Some value
    let flat_map = fn f value => opt::and_then f value
  end
end
