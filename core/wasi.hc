module wasi =
  use core
  use bundle::ops

  wasm => (
    (import "wasi_snapshot_preview1" "fd_write"
      (func $fd_write (param i32 i32 i32 i32) (result i32)))
    (import "wasi_snapshot_preview1" "fd_read"
      (func $fd_read (param i32 i32 i32 i32) (result i32)))
    (import "wasi_snapshot_preview1" "proc_exit"
      (func $proc_exit (param i32)))
    (import "wasi_snapshot_preview1" "sched_yield"
      (func $sched_yield (result i32)))
    (import "wasi_snapshot_preview1" "args_sizes_get"
      (func $args_sizes_get (param i32 i32) (result i32)))
    (import "wasi_snapshot_preview1" "args_get"
      (func $args_get (param i32 i32) (result i32)))
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

  let read : Unit -> Integer = fn _ => (wasm : Integer) => (
    (local $error_code i32)
    (local $bytes_read i32)
    (local $result i64)

    i64.const -1
    set $result

    i32.const 0
    i32.const 16
    i32.store

    i32.const 4
    i32.const 1
    i32.store

    i32.const 0
    i32.const 0
    i32.const 1
    i32.const 8
    call $fd_read
    set $error_code

    get $error_code
    i32.const 0
    i32.eq
    if
      i32.const 8
      i32.load
      set $bytes_read

      get $bytes_read
      i32.const 0
      i32.ne
      if
        i32.const 16
        i32.load
        i32.const 255
        i32.and
        i64.extend_i32_u
        set $result
      end
    end

    get $result
    struct.new $integer
  )

  let readln : Unit -> String = fn _ => (wasm : String) => (
    (local $result $string)
    (local $next $string)
    (local $len i32)
    (local $byte i32)
    (local $error_code i32)
    (local $bytes_read i32)

    i32.const 0
    array.new_default i8
    set $result

    block
    loop
      i32.const 0
      i32.const 16
      i32.store

      i32.const 4
      i32.const 1
      i32.store

      i32.const 0
      i32.const 0
      i32.const 1
      i32.const 8
      call $fd_read
      set $error_code

      get $error_code
      i32.const 0
      i32.ne
      break.if 1

      i32.const 8
      i32.load
      set $bytes_read

      get $bytes_read
      i32.const 0
      i32.eq
      break.if 1

      i32.const 16
      i32.load
      i32.const 255
      i32.and
      set $byte

      get $byte
      i32.const 10
      i32.eq
      break.if 1

      get $result
      array.len
      set $len

      get $len
      i32.const 1
      i32.add
      array.new_default i8
      set $next

      get $next
      i32.const 0
      get $result
      i32.const 0
      get $len
      array.copy i8 i8

      get $next
      get $len
      get $byte
      array.new_fixed i8 1
      i32.const 0
      i32.const 1
      array.copy i8 i8

      get $next
      set $result
      break 0
    end
    end

    get $result
  )

  let yield_now : Unit -> Boolean = fn _ => (wasm : Boolean) => (
    call $sched_yield
    i32.const 0
    i32.eq
    struct.new $word
  )

  let arguments : Unit -> Array String = fn _ => (wasm : Array String) => (
    (local $count i32)
    (local $argv_ptr i32)
    (local $error_code i32)
    (local $index i32)
    (local $argument_ptr i32)
    (local $argument_length i32)
    (local $byte_index i32)
    (local $argument $string)
    (local $result (array any))

    i32.const 0
    array.new_default any
    set $result

    i32.const 64
    i32.const 68
    call $args_sizes_get
    set $error_code

    get $error_code
    i32.const 0
    i32.eq
    if
      i32.const 64
      i32.load
      set $count

      get $count
      array.new_default any
      set $result

      i32.const 96
      set $argv_ptr

      get $argv_ptr
      get $argv_ptr
      get $count
      i32.const 4
      i32.mul
      i32.add
      call $args_get
      set $error_code

      get $error_code
      i32.const 0
      i32.eq
      if
        i32.const 0
        set $index

        block
        loop
          get $index
          get $count
          i32.eq
          break.if 1

          get $argv_ptr
          get $index
          i32.const 4
          i32.mul
          i32.add
          i32.load
          set $argument_ptr

          i32.const 0
          set $argument_length

          block
          loop
            get $argument_ptr
            get $argument_length
            i32.add
            i32.load
            i32.const 255
            i32.and
            i32.const 0
            i32.eq
            break.if 1

            get $argument_length
            i32.const 1
            i32.add
            set $argument_length
            break 0
          end
          end

          get $argument_length
          array.new_default i8
          set $argument

          i32.const 0
          set $byte_index

          block
          loop
            get $byte_index
            get $argument_length
            i32.eq
            break.if 1

            get $argument
            get $byte_index
            get $argument_ptr
            get $byte_index
            i32.add
            i32.load
            i32.const 255
            i32.and
            array.new_fixed i8 1
            i32.const 0
            i32.const 1
            array.copy i8 i8

            get $byte_index
            i32.const 1
            i32.add
            set $byte_index
            break 0
          end
          end

          get $result
          get $index
          get $argument
          array.new_fixed any 1
          i32.const 0
          i32.const 1
          array.copy any any

          get $index
          i32.const 1
          i32.add
          set $index
          break 0
        end
        end
      else
        i32.const 0
        array.new_default any
        set $result
      end
    end

    get $result
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
