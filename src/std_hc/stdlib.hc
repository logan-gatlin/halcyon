module std =
  import builtin

  type integer = builtin:integer
  type real = builtin:real
  type string = builtin:string
  type unit = builtin:unit
  type glyph = builtin:glyph
  type boolean = builtin:boolean
  
  let panic = fn => builtin:panic ()
  let assert = fn condition => if condition then () else panic ()
  let string_length = fn s => builtin:string_length s
  let print_string = fn s => builtin:print_string s
  let string_concatenate = fn s1 s2 => builtin:string_concatenate s1 s2
end
