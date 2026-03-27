bundle core

-- Core documentation guidelines:
-- * Use doc comments (`-->` or `(*> ... *)`) immediately above public declarations.
-- * Keep doc comments consecutive; only the trailing consecutive doc block is attached.
-- * For term docs, include `- Arguments:` and `- Returns:` sections.
-- * Add fenced `hc` examples for non-trivial behavior.
-- * Use (but don't over-use) markdown formatting for emphasis
-- * Mark internal declarations with `--> @HIDDEN` to omit them from generated docs.
-- * Use regular comments (`--` or `(* ... *)`) for maintainer-only notes.

wasm => (
  (type $integer (struct i64))
  (type $natural (struct i64))
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
  "integer.hc",
  "natural.hc",
  "big-num.hc",
  "real.hc",
  "function.hc",
  "time.hc",
  "prelude.hc"
