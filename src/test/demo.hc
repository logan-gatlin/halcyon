module demo =
  type opt =  Left | Right
  let _ = match Left with
  | Left => true
  | Right => false
end
