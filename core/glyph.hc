module glyph =
  use bundle
  use bundle::ops

  let to_string : Glyph -> String = fn value => (wasm : String) => (
    get value
    struct.get $word 0
    array.new_fixed i8 1
  )

  impl bundle::Default bundle::Glyph =
    let default = ' '
  end

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
