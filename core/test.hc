module test =
  use core

  let panic : for a in String -> a = fn message =>
    let _ = wasi::write_stderr (bundle::ops::[+] (bundle::ops::[+] "panic: " message) "\n") in
      intrinsics::unreachable ()

  let assert : Boolean -> String -> Unit = fn condition message =>
    if condition then
      ()
    else
      panic message
end
