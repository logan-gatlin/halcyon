module OpTest
  // Unary ops
  let _ = assert not false
  let _ = assert not not not false
  let _ = assert (-1 == 0 - 1)
  let _ = assert (-.1.0 == 0.0 -. 1.0)
  let _ = assert (-1.0 == 0.0 -. 1.0) // Special case

  // Integer arithmetic
  let _ = assert (1 + 2 == 3)
  let _ = assert (1 - 2 == -1)
  let _ = assert (1 * 2 == 2)
  let _ = assert (1 / 2 == 0)
  let _ = assert (1 % 2 == 1)

  // Real arithmetic
  let _ = assert (1.0 +. 2.0 == 3.0)
  let _ = assert (2.0 -. 1.0 == 1.0)
  let _ = assert (1.0 *. 2.0 == 2.0)
  let _ = assert (1.0 /. 2.0 == 0.5)

  // Boolean logical
  let _ = assert (true and true)
  let _ = assert (true or false)
  let _ = assert (true xor false)

  // Unit comparison
  let _ = assert (() == () == true)
  let _ = assert (() != () == false)
  let _ = assert (() <= () == true)
  let _ = assert (() >= () == true)
  let _ = assert (() < () == false)
  let _ = assert (() > () == false)

  // Boolean comparison
  let _ = assert (true == true)
  let _ = assert (true != false)
  let _ = assert (false <= true)
  let _ = assert (true >= false)
  let _ = assert (false < true)
  let _ = assert (true > false)

  // Glyph comparison
  let _ = assert ('a' == 'a')
  let _ = assert ('a' != 'b')
  let _ = assert ('a' <= 'b')
  let _ = assert ('b' >= 'a')
  let _ = assert ('a' < 'b')
  let _ = assert ('b' > 'a')

  // String comparison
  let _ = assert ("abc" == "abc")
  let _ = assert ("abc" != "def")
  let _ = assert ("abc" <= "def")
  let _ = assert ("def" >= "abc")
  let _ = assert ("abc" <= "abc")
  let _ = assert ("abc" >= "abc")
  let _ = assert ("abc" < "def")
  let _ = assert ("def" > "abc")
end
