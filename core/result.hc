module result =
  use bundle::ops
  use bundle::opt

  (*>
  Success-or-error container.

  `Ok ok` carries success values; `Err err` carries failure values.
  *)
  type Result: err ok = | Ok ok | Err err

  (*>
  Maps the success value.

  - Arguments:
    - `f`: Mapping function for `Ok` values.
    - `result`: Source result.
  - Returns: Mapped `Ok` or unchanged `Err`.

  ```hc
  let bumped = result::map (fn n => n + 1) outcome
  ```
  *)
  let map = fn f result => match result with
    | Ok value => Ok (f value)
    | Err error => Err error

  (*>
  Maps the error value.

  - Arguments:
    - `f`: Mapping function for `Err` values.
    - `result`: Source result.
  - Returns: Mapped `Err` or unchanged `Ok`.

  ```hc
  let message_result = result::map_err show status_result
  ```
  *)
  let map_err = fn f result => match result with
    | Ok value => Ok value
    | Err error => Err (f error)

  (*>
  Chains a result-producing function.

  - Arguments:
    - `f`: Function from `ok` to `Result err next`.
    - `result`: Source result.
  - Returns: Chained result; propagates existing errors.

  ```hc
  let loaded = result::and_then parse_config raw_config
  ```
  *)
  let and_then = fn f result => match result with
    | Ok value => f value
    | Err error => Err error

  --> @HIDDEN
  let and_with = fn next result => match result with
    | Ok _ => next
    | Err error => Err error

  (*>
  Recovers from an error with a fallback function.

  - Arguments:
    - `f`: Recovery function from `err` to `Result err ok`.
    - `result`: Source result.
  - Returns: Original `Ok` or recovered result.

  ```hc
  let configured = result::or_else (fn _ => Ok defaults) loaded
  ```
  *)
  let or_else = fn f result => match result with
    | Ok value => Ok value
    | Err error => f error

  --> @HIDDEN
  let or_with = fn fallback result => match result with
    | Ok value => Ok value
    | Err _ => fallback

  (*>
  Returns `true` for successful results.

  - Arguments:
    - `result`: Result to inspect.
  - Returns: Success flag.
  *)
  let is_ok = fn result => match result with
    | Ok _ => true
    | Err _ => false

  (*>
  Returns `true` for failed results.

  - Arguments:
    - `result`: Result to inspect.
  - Returns: Error flag.
  *)
  let is_err = fn result => match result with
    | Ok _ => false
    | Err _ => true

  (*>
  Extracts `Ok` as an option.

  - Arguments:
    - `result`: Result to inspect.
  - Returns: `Some ok` for success, otherwise `None`.
  *)
  let ok = fn result => match result with
    | Ok value => Some value
    | Err _ => None

  (*>
  Extracts `Err` as an option.

  - Arguments:
    - `result`: Result to inspect.
  - Returns: `Some err` for failure, otherwise `None`.
  *)
  let err = fn result => match result with
    | Ok _ => None
    | Err error => Some error

  (*>
  Extracts the success value, or returns a backup.

  - Arguments:
    - `backup`: Fallback value.
    - `result`: Source result.
  - Returns: `Ok` value or `backup`.
  *)
  let unwrap_or = fn backup result => match result with
    | Ok value => value
    | Err _ => backup

  (*>
  Extracts the success value, or computes a fallback from the error.

  - Arguments:
    - `backup_fn`: Function from error to fallback value.
    - `result`: Source result.
  - Returns: `Ok` value or `backup_fn error`.

  ```hc
  let timeout = result::unwrap_or_else (fn _ => 30) parsed_timeout
  ```
  *)
  let unwrap_or_else = fn backup_fn result => match result with
    | Ok value => value
    | Err error => backup_fn error

  (*>
  Checks whether a result contains a specific success value.

  - Arguments:
    - `value`: Target success value.
    - `result`: Result to inspect.
  - Returns: `true` for matching `Ok`.
  *)
  let contains = fn value result => match result with
    | Ok inner => inner == value
    | Err _ => false

  (*>
  Checks whether a result contains a specific error value.

  - Arguments:
    - `error`: Target error value.
    - `result`: Result to inspect.
  - Returns: `true` for matching `Err`.
  *)
  let contains_err = fn error result => match result with
    | Ok _ => false
    | Err inner => inner == error

  --> @HIDDEN
  let map_or = fn backup f result => match result with
    | Ok value => f value
    | Err _ => backup

  --> @HIDDEN
  let map_or_else = fn backup_fn f result => match result with
    | Ok value => f value
    | Err error => backup_fn error

  (*>
  Converts a result into an option by dropping errors.

  - Arguments:
    - `result`: Source result.
  - Returns: `Some ok` for success, otherwise `None`.
  *)
  let to_option = fn result => match result with
    | Ok value => Some value
    | Err _ => None

  (*>
  Converts a result into an option by dropping successes.

  - Arguments:
    - `result`: Source result.
  - Returns: `Some err` for failure, otherwise `None`.
  *)
  let error_option = fn result => match result with
    | Ok _ => None
    | Err error => Some error

  --> @HIDDEN
  impl bundle::Default for err ok in Result err ok where bundle::Default ok =
    let default = Ok bundle::default
  end

  --> @HIDDEN
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

  --> @HIDDEN
  impl bundle::show::Show for err ok in Result err ok where bundle::show::Show err, bundle::show::Show ok =
    let show = fn result =>
      match result with
        | Ok value => "Ok(" + (bundle::show::show value) + ")"
        | Err error => "Err(" + (bundle::show::show error) + ")"
  end

  --> @HIDDEN
  impl bundle::hkt::Applicative for err in Result err =
    let apply = fn wrapped_fn wrapped_value =>
      and_then (fn f => map f wrapped_value) wrapped_fn
  end

  --> @HIDDEN
  impl bundle::hkt::Traversable for err in Result err =
    let traverse = fn f result => match result with
      | Ok value => bundle::hkt::map (f value) Ok
      | Err error => bundle::hkt::new (Err error)
  end

  --> @HIDDEN
  impl bundle::hkt::Foldable for err in Result err =
    let fold = fn step initial result => match result with
      | Ok value => step initial value
      | Err _ => initial
  end

  --> @HIDDEN
  impl bundle::hkt::Alternative for err in Result err where bundle::Default err =
    let empty = Err bundle::default
    let or_else = fn left right => match left with
      | Ok value => Ok value
      | Err _ => right
  end

  --> @HIDDEN
  impl bundle::hkt::Functor for err in Result err =
    let fmap = fn f result => map f result
  end

  --> @HIDDEN
  impl bundle::hkt::Bifunctor Result =
    let bimap = fn left_fn right_fn result => match result with
      | Ok value => Ok (right_fn value)
      | Err error => Err (left_fn error)
  end

  --> @HIDDEN
  impl bundle::hkt::Zip for err in Result err =
    let zip_with = fn f left right =>
      and_then (fn left_value => map (fn right_value => f left_value right_value) right) left
  end

  --> @HIDDEN
  impl bundle::hkt::Filterable for err in Result err where bundle::Default err =
    let filter = fn predicate result => match result with
      | Ok value => if predicate value then Ok value else Err bundle::default
      | Err error => Err error
  end

  --> @HIDDEN
  impl bundle::hkt::Monad for err in Result err =
    let new = fn value => Ok value
    let flat_map = fn f result => match result with
      | Ok value => f value
      | Err error => Err error
  end

end
