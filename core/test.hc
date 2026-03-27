module test =
  use bundle

  (*>
  Aborts execution with a panic message.

  - Arguments:
    - `message`: Human-readable failure reason.
  - Returns: Never returns; execution stops.

  ```hc
  let value =
    if is_valid then
      computed
    else
      panic "unexpected state"
  ```
  *)
  let panic : for a in String -> a = fn message =>
    let _ = wasi::write_stderr (bundle::ops::[+] (bundle::ops::[+] "panic: " message) "\n") in
      intrinsics::unreachable ()

  (*>
  Fails fast when a condition is false.

  - Arguments:
    - `condition`: Predicate that must hold.
    - `message`: Failure message used when the assertion fails.
  - Returns: `()` when `condition` is true; otherwise panics.

  ```hc
  let _ = assert (result == expected) "result should match expected value" in
  ()
  ```
  *)
  let assert : Boolean -> String -> Unit = fn condition message =>
    if condition then
      ()
    else
      panic message
end
