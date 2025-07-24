module TypeTest
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
  type nested_struct = {
    a : {
      a : {
        a : {
          a : {
            a : unit
          }
        }
      }
    }
  }
  type tuple = a * b * c * d * e * f * struct

  // Recursion
  type direct_recursion = direct_recursion
  type recursive_struct = {
    cycle: recursive_struct,
  }
  type recursive_tuple = integer * recursive_tuple

  // Struct field inference
  type specific_struct = { specific_name: integer }
  let tricky_inference = fn a => a.specific_name
  let _ = assert ((tricky_inference { specific_name = 1 }) + 1 == 2)
end
