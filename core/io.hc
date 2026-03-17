module io =
  use core
  use core::ops

  let write_stdout = wasi::write_stdout
  let write_stderr = wasi::write_stderr
  let read = wasi::read
  let print = fn value => let _ = write_stdout value in ()
  let println = fn value => print (value + "\n")
  let readln = fn s => print s; wasi::readln ()
  let eprint = fn value => let _ = write_stderr value in ()
  let eprintln = fn value => eprint (value + "\n")
end
