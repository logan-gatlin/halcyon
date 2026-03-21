module opt =
  use core
  use bundle::ops
  type Option: a = | Some a | None

  let map = fn f opt => match opt with
    | Some a => Some (f a)
    | None => None

  let map_or = fn backup f opt => match opt with
    | Some a => f a
    | None => backup

  let map_or_else = fn backup_fn f opt => match opt with
    | Some a => f a
    | None => backup_fn ()

  let and_then = fn f opt => match opt with
    | Some a => f a
    | None => None

  let and_with = fn next opt => match opt with
    | Some _ => next
    | None => None

  let or_with = fn fallback opt => match opt with
    | Some a => Some a
    | None => fallback

  let or_else = fn fallback_fn opt => match opt with
    | Some a => Some a
    | None => fallback_fn ()

  let xor_with = fn left right => match left with
    | Some left_value =>
      match right with
        | Some _ => None
        | None => Some left_value
    | None => right

  let zip = fn left right => match left with
    | Some left_value =>
      match right with
        | Some right_value => Some (left_value, right_value)
        | None => None
    | None => None

  let filter = fn predicate opt => match opt with
    | Some a => if predicate a then Some a else None
    | None => None

  let is_some = fn opt => match opt with
    | Some a => true
    | None => false

  let is_none = [not] >> is_some

  let contains = fn value opt => match opt with
    | Some inner => inner == value
    | None => false

  let to_array = fn opt => match opt with
    | Some value => (wasm : for a in Array a) => (
      get value
      array.new_fixed any 1
    )
    | None => (wasm : for a in Array a) => (
      i32.const 0
      array.new_default any
    )

  let from_array_head = fn arr => match arr with
    | [value, ..] => Some value
    | [] => None

  let zip_with = fn f left right =>
    and_then (fn left_value => map (fn right_value => f left_value right_value) right) left

  let match_or = fn none_value some_fn opt => match opt with
    | Some value => some_fn value
    | None => none_value

  let expect_or = fn message backup opt =>
    match opt with
      | Some value => value
      | None => backup

  let unwrap_or = fn backup opt => match opt with
    | Some a => a
    | None => backup

  let unwrap_or_else = fn backup_fn opt => match opt with
    | Some a => a
    | None => backup_fn ()

  impl bundle::Default for a in Option a =
    let default = None
  end

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

  impl bundle::show::Show for a in Option a where bundle::show::Show a =
    let show = fn value =>
      match value with
        | Some inner => "Some(" + (bundle::show::show inner) + ")"
        | None => "None"
  end

  impl bundle::hkt::Applicative Option =
    let apply = fn wrapped_fn wrapped_value =>
      opt::and_then (fn f => opt::map f wrapped_value) wrapped_fn
  end

  impl bundle::hkt::Traversable Option =
    let traverse = fn f opt => match opt with
      | Some value => bundle::hkt::map Some (f value)
      | None => bundle::hkt::new None
  end

  impl bundle::hkt::Foldable Option =
    let fold = fn step initial opt => match opt with
      | Some value => step initial value
      | None => initial
  end

  impl bundle::hkt::Alternative Option =
    let empty = None
    let or_else = fn left right => opt::or_with right left
  end

  impl bundle::hkt::Functor Option =
    let fmap = fn f value => opt::map f value
  end

  impl bundle::hkt::Zip Option =
    let zip_with = fn f left right => opt::zip_with f left right
  end

  impl bundle::hkt::Filterable Option =
    let filter = fn predicate value => opt::filter predicate value
  end

  impl bundle::hkt::Monad Option =
    let new = fn value => Some value
    let flatmap = fn f value => opt::and_then f value
  end
end
