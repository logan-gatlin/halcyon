bundle demo

trait Show : a =
  let show : a -> core::String
end

let outer = fn value =>
  let render = fn x => show value in
  render
