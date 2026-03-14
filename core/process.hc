module process =
  use core
  let yield_now = wasi::yield_now
  let exit_success = wasi::exit_success
  let exit_failure = wasi::exit_failure
  let args_count = wasi::args_count
  let args_buffer_size = wasi::args_buffer_size
  let monotonic_time_nanos = wasi::monotonic_time_nanos
  let realtime_time_nanos = wasi::realtime_time_nanos
end
