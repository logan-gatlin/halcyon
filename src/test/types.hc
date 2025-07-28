module TypeTest
  import std

  // Primitive aliases
  type integer = std:integer
  type real = std:real
  type unit = std:unit
  type boolean = std:boolean
  type string = std:string
  type glyph = std:glyph

  // ADT's
  type struct = {
    a : integer,
    b : real,
    c : unit,
    d : boolean,
    e : string,
    f : glyph,
  }
  type tuple = integer * boolean * real * unit * string * glyph * struct
  type variant = V_int of integer | V_real of real | V_both of integer * real

  // Struct field inference
  type specific_struct = { specific_name: integer }
  let tricky_inference = fn a => a.specific_name
  let a = std:assert ((tricky_inference { specific_name = 1 }) + 1 == 2)
end
