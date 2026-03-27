module io =
  use bundle
  use bundle::ops

  (*>
  Reads one glyph from standard input.

  - Arguments: none.
  - Returns: The glyph, or `'\xFFFF'` when no data is available.
  *)
  let read = wasi::read

  (*>
  Writes a string to standard output.

  - Arguments:
    - `value`: Text to print.
  - Returns: `()`.
  *)
  let print = fn value => let _ = wasi::write_stdout value in ()

  (*>
  Writes a string followed by a newline to standard output.

  - Arguments:
    - `value`: Text to print.
  - Returns: `()`.
  *)
  let println = fn value => print (value + "\n")

  (*>
  Prompts and reads one line from standard input.

  - Arguments:
    - `s`: Prompt text.
  - Returns: Input line without the trailing newline.

  ```hc
  let name = io::readln "Please enter your name: "
  ```
  *)
  let readln = fn s => print s; wasi::readln ()

  (*>
  Writes a string to standard error.

  - Arguments:
    - `value`: Text to print.
  - Returns: `()`.
  *)
  let eprint = fn value => let _ = wasi::write_stderr value in ()

  (*>
  Writes a string followed by a newline to standard error.

  - Arguments:
    - `value`: Text to print.
  - Returns: `()`.
  *)
  let eprintln = fn value => eprint (value + "\n")
end
