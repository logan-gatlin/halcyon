module time =
  use bundle
  use bundle::ops

  (*>
  Duration in nanoseconds.

  Use `Duration` to represent elapsed time.
  *)
  type Duration = Integer

  (*>
  Instant in nanoseconds from the runtime clock.

  Use `Instant` for timestamp snapshots.
  *)
  type Instant = Integer

  (*>
  Returns the current realtime clock value.

  - Arguments: none.
  - Returns: Current `Instant`.
  *)
  let current = Instant >> bundle::wasi::realtime_nanos

  (*>
  Computes elapsed time since a prior instant.

  - Arguments:
    - `other`: Earlier (or future) instant.
  - Returns: Non-negative `Duration`; returns `0` when `other` is in the future.

  ```hc
  let start = time::current () in
  let _ = do_work () in
  let elapsed = time::time_since start
  ```
  *)
  let time_since = fn other =>
    let Instant now_time = current () in
    let Instant other_time = other in
    if now_time > other_time then
      Duration (now_time - other_time)
    else
      Duration 0

  (*>
  Converts a duration to fractional seconds.

  - Arguments:
    - `duration`: Duration to convert.
  - Returns: Duration in seconds as `Real`.
  *)
  let as_seconds = fn
    | Duration i => (bundle::real::integer_to_real i) / 1000000000.0

  (*>
  Converts a duration to milliseconds.

  - Arguments:
    - `duration`: Duration to convert.
  - Returns: Whole milliseconds as `Integer`.
  *)
  let as_millis = fn
    | Duration i => i / 1000000

  (*>
  Returns raw nanoseconds from a duration.

  - Arguments:
    - `duration`: Duration to convert.
  - Returns: Whole nanoseconds as `Integer`.
  *)
  let as_nanosecs = fn
    | Duration i => i

end
