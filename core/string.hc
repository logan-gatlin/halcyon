module string =
  use core::ops
  let empty = ""
  let concat = fn left right => left + right
  let is_empty = fn value => value == ""
  let non_empty = fn value => value != ""

  impl bundle::Default bundle::String =
    let default = ""
  end
end
