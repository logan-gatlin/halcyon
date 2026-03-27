module process =
  use bundle

  (*>
  Returns process command-line arguments.

  - Arguments: none.
  - Returns: `Array String` containing the program arguments in order.

  ```hc
  let args = process::arguments ()
  ```
  *)
  let arguments = wasi::arguments
end
