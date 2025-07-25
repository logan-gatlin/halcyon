module LiteralTest
  import std
  let _ = ()
  let _ = 0.1
  let _ = std:assert (1e9 == 1000000000.0)
  let _ = 999
  // Hex
  let _ = std:assert (0xFF == 255)
  let _ = std:assert (0xff == 255)
  let _ = std:assert (0XFF == 255)
  let _ = std:assert (0Xff == 255)
  // Octal
  let _ = std:assert (0o777 == 511)
  let _ = std:assert (0O777 == 511)
  // Binary
  let _ = std:assert (0b111 == 7)
  let _ = std:assert (0B111 == 7)
  // Strings, characters, escapes
  let _ = 'a'
  let _ = '\t'
  let _ = "a"
  let _ = "\r \n \t \b \\ \0  \' \x00 \w0000"
end
