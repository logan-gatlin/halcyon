module OpTest
  import std
  // Unary ops
  let _ = std:assert not false
  let _ = std:assert not not not false
  let _ = std:assert (-1 == 0 - 1)
  let _ = std:assert (-.1.0 == 0.0 -. 1.0)
  let _ = std:assert (-1.0 == 0.0 -. 1.0) // Special case

  // Integer arithmetic
  let _ = std:assert (1 + 2 == 3)
  let _ = std:assert (1 - 2 == -1)
  let _ = std:assert (1 * 2 == 2)
  let _ = std:assert (1 / 2 == 0)
  let _ = std:assert (1 % 2 == 1)

  // Real arithmetic
  let _ = std:assert (1.0 +. 2.0 == 3.0)
  let _ = std:assert (2.0 -. 1.0 == 1.0)
  let _ = std:assert (1.0 *. 2.0 == 2.0)
  let _ = std:assert (1.0 /. 2.0 == 0.5)

  // Boolean logical
  let _ = std:assert (true and true)
  let _ = std:assert (true or false)
  let _ = std:assert (true xor false)

  // Unit comparison
  let _ = std:assert (() == () == true)
  let _ = std:assert (() != () == false)
  let _ = std:assert (() <= () == true)
  let _ = std:assert (() >= () == true)
  let _ = std:assert (() < () == false)
  let _ = std:assert (() > () == false)

  // Boolean comparison
  let _ = std:assert (true == true)
  let _ = std:assert (true != false)
  let _ = std:assert (false <= true)
  let _ = std:assert (true >= false)
  let _ = std:assert (false < true)
  let _ = std:assert (true > false)

  // Glyph comparison
  let _ = std:assert ('a' == 'a')
  let _ = std:assert ('a' != 'b')
  let _ = std:assert ('a' <= 'b')
  let _ = std:assert ('b' >= 'a')
  let _ = std:assert ('a' < 'b')
  let _ = std:assert ('b' > 'a')

  // String comparison
  let _ = std:assert ("abc" == "abc")
  let _ = std:assert ("abc" != "def")
  let _ = std:assert ("abc" <= "def")
  let _ = std:assert ("def" >= "abc")
  let _ = std:assert ("abc" <= "abc")
  let _ = std:assert ("abc" >= "abc")
  let _ = std:assert ("abc" < "def")
  let _ = std:assert ("def" > "abc")
end
