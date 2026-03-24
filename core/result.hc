module result =
  use bundle::ops
  use bundle::opt
  type Result: err ok = | Ok ok | Err err

  let map = fn f result => match result with
    | Ok value => Ok (f value)
    | Err error => Err error

  let map_err = fn f result => match result with
    | Ok value => Ok value
    | Err error => Err (f error)

  let and_then = fn f result => match result with
    | Ok value => f value
    | Err error => Err error

  let and_with = fn next result => match result with
    | Ok _ => next
    | Err error => Err error

  let or_else = fn f result => match result with
    | Ok value => Ok value
    | Err error => f error

  let or_with = fn fallback result => match result with
    | Ok value => Ok value
    | Err _ => fallback

  let is_ok = fn result => match result with
    | Ok _ => true
    | Err _ => false

  let is_err = fn result => match result with
    | Ok _ => false
    | Err _ => true

  let ok = fn result => match result with
    | Ok value => Some value
    | Err _ => None

  let err = fn result => match result with
    | Ok _ => None
    | Err error => Some error

  let unwrap_or = fn backup result => match result with
    | Ok value => value
    | Err _ => backup

  let unwrap_or_else = fn backup_fn result => match result with
    | Ok value => value
    | Err error => backup_fn error

  let contains = fn value result => match result with
    | Ok inner => inner == value
    | Err _ => false

  let contains_err = fn error result => match result with
    | Ok _ => false
    | Err inner => inner == error

  let map_or = fn backup f result => match result with
    | Ok value => f value
    | Err _ => backup

  let map_or_else = fn backup_fn f result => match result with
    | Ok value => f value
    | Err error => backup_fn error

  let to_option = fn result => match result with
    | Ok value => Some value
    | Err _ => None

  let error_option = fn result => match result with
    | Ok _ => None
    | Err error => Some error

  impl bundle::Default for err ok in Result err ok where bundle::Default ok =
    let default = Ok bundle::default
  end

  impl bundle::ops::Equal for err ok in Result err ok where bundle::ops::Equal err, bundle::ops::Equal ok =
    let [==] = fn left right => match left with
      | Ok left_value =>
        match right with
          | Ok right_value => bundle::ops::[==] left_value right_value
          | Err _ => false
      | Err left_error =>
        match right with
          | Ok _ => false
          | Err right_error => bundle::ops::[==] left_error right_error
  end

  impl bundle::show::Show for err ok in Result err ok where bundle::show::Show err, bundle::show::Show ok =
    let show = fn result =>
      match result with
        | Ok value => "Ok(" + (bundle::show::show value) + ")"
        | Err error => "Err(" + (bundle::show::show error) + ")"
  end

  impl bundle::hkt::Applicative for err in Result err =
    let apply = fn wrapped_fn wrapped_value =>
      and_then (fn f => map f wrapped_value) wrapped_fn
  end

  impl bundle::hkt::Traversable for err in Result err =
    let traverse = fn f result => match result with
      | Ok value => bundle::hkt::map Ok (f value)
      | Err error => bundle::hkt::new (Err error)
  end

  impl bundle::hkt::Foldable for err in Result err =
    let fold = fn step initial result => match result with
      | Ok value => step initial value
      | Err _ => initial
  end

  impl bundle::hkt::Alternative for err in Result err where bundle::Default err =
    let empty = Err bundle::default
    let or_else = fn left right => match left with
      | Ok value => Ok value
      | Err _ => right
  end

  impl bundle::hkt::Functor for err in Result err =
    let fmap = fn f result => map f result
  end

  impl bundle::hkt::Bifunctor Result =
    let bimap = fn left_fn right_fn result => match result with
      | Ok value => Ok (right_fn value)
      | Err error => Err (left_fn error)
  end

  impl bundle::hkt::Zip for err in Result err =
    let zip_with = fn f left right =>
      and_then (fn left_value => map (fn right_value => f left_value right_value) right) left
  end

  impl bundle::hkt::Filterable for err in Result err where bundle::Default err =
    let filter = fn predicate result => match result with
      | Ok value => if predicate value then Ok value else Err bundle::default
      | Err error => Err error
  end

  impl bundle::hkt::Monad for err in Result err =
    let new = fn value => Ok value
    let flat_map = fn f result => match result with
      | Ok value => f value
      | Err error => Err error
  end

end
