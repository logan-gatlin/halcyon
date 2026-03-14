module bool =
  use core::opt

  let select = fn condition value =>
    if condition then Some value else None

  let select_else = fn condition when_true when_false =>
    if condition then when_true () else when_false ()

  let guard = fn condition value => select condition value

  impl bundle::Default bundle::Boolean =
    let default = false
  end
end
