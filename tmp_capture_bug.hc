bundle demo

trait Show : a =
  let show : a -> core::String
end

let pass = fn value =>
  let rendered = show value in
  rendered
