module fs =
  use bundle
  use bundle::ops
  use bundle::opt

  wasm => (
    (import "wasi_snapshot_preview1" "path_open"
      (func $path_open (param i32 i32 i32 i32 i32 i64 i64 i32 i32) (result i32)))
    (import "wasi_snapshot_preview1" "fd_read"
      (func $fd_read (param i32 i32 i32 i32) (result i32)))
    (import "wasi_snapshot_preview1" "fd_write"
      (func $fd_write (param i32 i32 i32 i32) (result i32)))
    (import "wasi_snapshot_preview1" "fd_close"
      (func $fd_close (param i32) (result i32)))
  )

  --> @HIDDEN
  let rights_fd_read = 2
  --> @HIDDEN
  let rights_fd_write = 64
  --> @HIDDEN
  let oflags_create = 1
  --> @HIDDEN
  let oflags_truncate = 8
  --> @HIDDEN
  let fdflags_append = 1

  --> @HIDDEN
  let open_path : String -> Integer -> Integer -> Integer -> Integer =
    fn path oflags rights fdflags => (wasm : Integer) => (
      (local $path_value $string)
      (local $path_len i32)
      (local $index i32)
      (local $error_code i32)
      (local $result i64)

      i64.const -1
      set $result

      get path
      ref.cast_array i8
      set $path_value

      get $path_value
      array.len
      set $path_len

      i32.const 0
      set $index

      block
      loop
        get $index
        get $path_len
        i32.eq
        break.if 1

        i32.const 256
        get $index
        i32.add
        get $path_value
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

      i32.const 3
      i32.const 0
      i32.const 256
      get $path_len
      get oflags
      struct.get $integer 0
      i32.wrap_i64
      get rights
      struct.get $integer 0
      i64.const 0
      get fdflags
      struct.get $integer 0
      i32.wrap_i64
      i32.const 128
      call $path_open
      set $error_code

      get $error_code
      i32.const 0
      i32.eq
      if
        i32.const 128
        i32.load
        i64.extend_i32_u
        set $result
      end

      get $result
      struct.new $integer
    )

  --> @HIDDEN
  let open_read = fn path => open_path path 0 rights_fd_read 0

  --> @HIDDEN
  let open_write = fn path => open_path path (oflags_create + oflags_truncate) rights_fd_write 0

  --> @HIDDEN
  let open_append = fn path => open_path path oflags_create rights_fd_write fdflags_append

  --> @HIDDEN
  let close_fd : Integer -> Unit = fn fd => (wasm : Unit) => (
    (local $ignored i32)

    get fd
    struct.get $integer 0
    i32.wrap_i64
    call $fd_close
    set $ignored

    struct.new $unit
  )

  --> @HIDDEN
  let read_fd_to_string : Integer -> String = fn fd => (wasm : String) => (
    (local $fd_i32 i32)
    (local $result $string)
    (local $next $string)
    (local $result_len i32)
    (local $bytes_read i32)
    (local $index i32)
    (local $byte i32)
    (local $error_code i32)

    get fd
    struct.get $integer 0
    i32.wrap_i64
    set $fd_i32

    i32.const 0
    array.new_default i8
    set $result

    block
    loop
      i32.const 0
      i32.const 1024
      i32.store

      i32.const 4
      i32.const 1024
      i32.store

      get $fd_i32
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

      get $result
      array.len
      set $result_len

      get $result_len
      get $bytes_read
      i32.add
      array.new_default i8
      set $next

      get $next
      i32.const 0
      get $result
      i32.const 0
      get $result_len
      array.copy i8 i8

      i32.const 0
      set $index

      block
      loop
        get $index
        get $bytes_read
        i32.eq
        break.if 1

        i32.const 1024
        get $index
        i32.add
        i32.load
        i32.const 255
        i32.and
        set $byte

        get $next
        get $result_len
        get $index
        i32.add
        get $byte
        array.new_fixed i8 1
        i32.const 0
        i32.const 1
        array.copy i8 i8

        get $index
        i32.const 1
        i32.add
        set $index
        break 0
      end
      end

      get $next
      set $result
      break 0
    end
    end

    get $result
  )

  --> @HIDDEN
  let write_fd_string : Integer -> String -> Boolean = fn fd value => (wasm : Boolean) => (
    (local $fd_i32 i32)
    (local $value_data $string)
    (local $value_len i32)
    (local $offset i32)
    (local $chunk_len i32)
    (local $index i32)
    (local $error_code i32)
    (local $bytes_written i32)
    (local $success i32)

    get fd
    struct.get $integer 0
    i32.wrap_i64
    set $fd_i32

    get value
    ref.cast_array i8
    set $value_data

    get $value_data
    array.len
    set $value_len

    i32.const 0
    set $offset

    i32.const 1
    set $success

    block
    loop
      get $offset
      get $value_len
      i32.eq
      break.if 1

      get $value_len
      get $offset
      i32.sub
      set $chunk_len

      get $chunk_len
      i32.const 1024
      i32.gt
      if
        i32.const 1024
        set $chunk_len
      end

      i32.const 0
      set $index

      block
      loop
        get $index
        get $chunk_len
        i32.eq
        break.if 1

        i32.const 1024
        get $index
        i32.add
        get $value_data
        get $offset
        get $index
        i32.add
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
      i32.const 1024
      i32.store

      i32.const 4
      get $chunk_len
      i32.store

      get $fd_i32
      i32.const 0
      i32.const 1
      i32.const 8
      call $fd_write
      set $error_code

      get $error_code
      i32.const 0
      i32.ne
      if
        i32.const 0
        set $success
        break 1
      end

      i32.const 8
      i32.load
      set $bytes_written

      get $bytes_written
      i32.const 0
      i32.eq
      if
        i32.const 0
        set $success
        break 1
      end

      get $offset
      get $bytes_written
      i32.add
      set $offset
      break 0
    end
    end

    get $success
    i32.const 1
    i32.eq
    struct.new $word
  )

  (*>
  Reads an entire file into a string.

  - Arguments:
    - `path`: Path to the file.
  - Returns: `Some text` on success, otherwise `None`.

  ```hc
  let contents = fs::read_to_string "./data.txt"
  ```
  *)
  let read_to_string : String -> opt::Option String = fn path =>
    let fd = open_read path in
    if fd < 0 then
      None
    else
      let contents = read_fd_to_string fd in
      let _ = close_fd fd in
      Some contents

  (*>
  Writes text to a file, replacing existing contents.

  - Arguments:
    - `path`: Path to the destination file.
    - `value`: Text content to write.
  - Returns: `true` on success, otherwise `false`.

  ```hc
  let ok = fs::write_string "./data.txt" "hello"
  ```
  *)
  let write_string : String -> String -> Boolean = fn path value =>
    let fd = open_write path in
    if fd < 0 then
      false
    else
      let ok = write_fd_string fd value in
      let _ = close_fd fd in
      ok

  (*>
  Appends text to the end of a file.

  Creates the file when it does not exist.

  - Arguments:
    - `path`: Path to the destination file.
    - `value`: Text content to append.
  - Returns: `true` on success, otherwise `false`.

  ```hc
  let ok = fs::append_string "./data.txt" "\nworld"
  ```
  *)
  let append_string : String -> String -> Boolean = fn path value =>
    let fd = open_append path in
    if fd < 0 then
      false
    else
      let ok = write_fd_string fd value in
      let _ = close_fd fd in
      ok
end
