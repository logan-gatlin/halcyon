module bool =
  use bundle::opt

  (*>
  Wraps `value` in `Some` when `condition` is true.

  - Arguments:
    - `condition`: Gate that decides whether to keep the value.
    - `value`: Value to keep when `condition` is true.
  - Returns: `Some value` or `None`.
  *)
  let select = fn condition value =>
    if condition then Some value else None

  (*>
  Chooses one of two thunks based on a boolean.

  - Arguments:
    - `condition`: Branch selector.
    - `when_true`: Thunk executed when `condition` is true.
    - `when_false`: Thunk executed when `condition` is false.
  - Returns: The selected thunk result.

  ```hc
  let outcome = bool::select_else is_ready (fn _ => "go") (fn _ => "wait")
  ```
  *)
  let select_else = fn condition when_true when_false =>
    if condition then when_true () else when_false ()

  (*>
  Alias of `select` for guard-style pipelines.

  - Arguments:
    - `condition`: Predicate.
    - `value`: Value to keep.
  - Returns: `Some value` when predicate holds, otherwise `None`.
  *)
  let guard = fn condition value => select condition value

  --> @HIDDEN
  impl bundle::Default bundle::Boolean =
    let default = false
  end

  --> @HIDDEN
  impl bundle::show::Show bundle::Boolean =
    let show = fn value => if value then "true" else "false"
  end
end
