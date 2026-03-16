module wasi =
  use core
  use core::ops

  wasm => (
    (import "wasi_snapshot_preview1" "fd_write"
      (func $fd_write (param i32 i32 i32 i32) (result i32)))
    (import "wasi_snapshot_preview1" "proc_exit"
      (func $proc_exit (param i32)))
    (import "wasi_snapshot_preview1" "sched_yield"
      (func $sched_yield (result i32)))
    (import "wasi_snapshot_preview1" "args_sizes_get"
      (func $args_sizes_get (param i32 i32) (result i32)))
    (import "wasi_snapshot_preview1" "clock_time_get"
      (func $clock_time_get (param i32 i64 i32) (result i32)))
  )

  let write_stdout : String -> Boolean =
    fn (value : String) => (wasm : Boolean) => (
      (local $str $string)
      (local $len i32)
      (local $index i32)

      get value
      ref.cast_array i8
      set $str

      get $str
      array.len
      set $len

      i32.const 0
      set $index

      block
      loop
        get $index
        get $len
        i32.eq
        break.if 1

        i32.const 12
        get $index
        i32.add
        get $str
        get $index
        array.get i8
        i32.store8

        get $index
        i32.const 1
        i32.add
        set $index
        break 0
      end
      end

      i32.const 0
      i32.const 12
      i32.store

      i32.const 4
      get $len
      i32.store

      i32.const 1
      i32.const 0
      i32.const 1
      i32.const 8
      call $fd_write
      i32.const 0
      i32.eq
      struct.new $word
    )

  let write_stderr : String -> Boolean =
    fn (value : String) => (wasm : Boolean) => (
      (local $str $string)
      (local $len i32)
      (local $index i32)

      get value
      ref.cast_array i8
      set $str

      get $str
      array.len
      set $len

      i32.const 0
      set $index

      block
      loop
        get $index
        get $len
        i32.eq
        break.if 1

        i32.const 12
        get $index
        i32.add
        get $str
        get $index
        array.get i8
        i32.store8

        get $index
        i32.const 1
        i32.add
        set $index
        break 0
      end
      end

      i32.const 0
      i32.const 12
      i32.store

      i32.const 4
      get $len
      i32.store

      i32.const 2
      i32.const 0
      i32.const 1
      i32.const 8
      call $fd_write
      i32.const 0
      i32.eq
      struct.new $word
    )

  let yield_now : Unit -> Boolean = fn _ => (wasm : Boolean) => (
    call $sched_yield
    i32.const 0
    i32.eq
    struct.new $word
  )

  let exit : Integer -> (for a in a) = fn i => (wasm : ()) => (
    get i
    struct.get $integer 0
    i32.wrap_i64
    call $proc_exit
    struct.new $unit
  ); bundle::intrinsics::unreachable ()

  let args_count : Unit -> Integer = fn _ =>
    let error_code = (wasm : Integer) => (
      i32.const 64
      i32.const 68
      call $args_sizes_get
      i64.extend_i32_u
      struct.new $integer
    ) in
    if ops::[==] error_code 0 then
      (wasm : Integer) => (
        i32.const 64
        i32.load
        i64.extend_i32_u
        struct.new $integer
      )
    else
      (wasm : Integer) => (
        i64.const -1
        struct.new $integer
      )

  let args_buffer_size : Unit -> Integer = fn _ =>
    let error_code = (wasm : Integer) => (
      i32.const 64
      i32.const 68
      call $args_sizes_get
      i64.extend_i32_u
      struct.new $integer
    ) in
    if ops::[==] error_code 0 then
      (wasm : Integer) => (
        i32.const 68
        i32.load
        i64.extend_i32_u
        struct.new $integer
      )
    else
      (wasm : Integer) => (
        i64.const -1
        struct.new $integer
      )

  let monotonic_time_nanos : Unit -> Integer = fn _ =>
    let error_code = (wasm : Integer) => (
      i32.const 1
      i64.const 0
      i32.const 80
      call $clock_time_get
      i64.extend_i32_u
      struct.new $integer
    ) in
    if ops::[==] error_code 0 then
      (wasm : Integer) => (
        i32.const 80
        i64.load
        struct.new $integer
      )
    else
      (wasm : Integer) => (
        i64.const -1
        struct.new $integer
      )

  let realtime_time_nanos : Unit -> Integer = fn _ =>
    let error_code = (wasm : Integer) => (
      i32.const 0
      i64.const 0
      i32.const 80
      call $clock_time_get
      i64.extend_i32_u
      struct.new $integer
    ) in
    if ops::[==] error_code 0 then
      (wasm : Integer) => (
        i32.const 80
        i64.load
        struct.new $integer
      )
    else
      (wasm : Integer) => (
        i64.const -1
        struct.new $integer
      )
end
