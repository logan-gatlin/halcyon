module io =
  use core
  use core::ops

  let write_stdout = wasi::write_stdout
  let write_stderr = wasi::write_stderr
  let print = fn value => let _ = write_stdout value in ()
  let println = fn value => print (value + "\n")
  let eprint = fn value => let _ = write_stderr value in ()
  let eprintln = fn value => eprint (value + "\n")
end
