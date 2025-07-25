module TypeTest
  import std

  // Primitive aliases
  type a = integer
  type b = real
  type c = unit
  type d = boolean
  type e = string
  type f = glyph

  // ADT's
  type struct = {
    a : a,
    b : b,
    c : c,
    d : d,
    e : e,
    f : f,
  }
  type tuple = a * b * c * d * e * f * struct
  type variant = V_int of integer | V_real of real | V_both of integer * real

  // Struct field inference
  type specific_struct = { specific_name: integer }
  let tricky_inference = fn a => a.specific_name
  let _ = std:assert ((tricky_inference { specific_name = 1 }) + 1 == 2)
end
