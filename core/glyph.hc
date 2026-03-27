module glyph =
  use bundle
  use bundle::ops

  (*>
  Converts a glyph to a one-character string.

  - Arguments:
    - `value`: Glyph to convert.
  - Returns: A `String` containing exactly that glyph.
  *)
  let to_string : Glyph -> String = fn value => (wasm : String) => (
    get value
    struct.get $word 0
    array.new_fixed i8 1
  )

  --> @HIDDEN
  impl bundle::Default bundle::Glyph =
    let default = ' '
  end

  --> @HIDDEN
  impl bundle::show::Show bundle::Glyph =
    let show = fn value =>
      match value with
        | '\n' => "'\\n'"
        | '\r' => "'\\r'"
        | '\t' => "'\\t'"
        | '\b' => "'\\b'"
        | '\0' => "'\\0'"
        | '\\' => "'\\\\'"
        | '\'' => "'\\\''"
        | _ => "'" + (to_string value) + "'"
  end
end
