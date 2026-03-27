module show =
  use bundle

  (*>
  Converts values into user-facing text.

  Implement `Show` for a type when you want stable, readable diagnostics,
  logs, and REPL output.

  ```hc
  let rendered = show 42
  ```
  *)
  trait Show: self =
    let show : self -> String
  end
end
