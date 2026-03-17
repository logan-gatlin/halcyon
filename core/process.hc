module process =
  use core
  let yield_now = wasi::yield_now
  let arguments = wasi::arguments
  let monotonic_time_nanos = wasi::monotonic_time_nanos
  let realtime_time_nanos = wasi::realtime_time_nanos
end
