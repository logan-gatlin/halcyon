bundle core

wasm => (
  (type $integer (struct i64))
  (type $real (struct f64))
  (type $word (struct i32))
  (type $string (array i8))
  (type $unit (struct))
  (memory $mem 1)
)

import 
  "intrinsics.hc",
  "ops.hc",
  "wasi.hc",
  "default.hc",
  "show.hc",
  "test.hc",
  "hkt.hc",
  "io.hc",
  "process.hc",
  "unit.hc",
  "glyph.hc",
  "opt.hc",
  "result.hc",
  "array.hc",
  "bool.hc",
  "string.hc",
  "big-num.hc",
  "integer.hc",
  "real.hc",
  "function.hc",
  "prelude.hc"
