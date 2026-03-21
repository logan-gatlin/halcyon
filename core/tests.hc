bundle core-tests

let panic_integer : Unit -> Integer = fn _ => panic "panic integer"
let panic_boolean : Unit -> Boolean = fn _ => panic "panic boolean"

let test_integer_ops = fn _ =>
  let _ = assert (1 + 2 == 3) "integer addition" in
  let _ = assert (7 - 5 == 2) "integer subtraction" in
  let _ = assert (4 * 3 == 12) "integer multiplication" in
  let _ = assert (9 / 3 == 3) "integer division" in
  let _ = assert (7 % 4 == 3) "integer remainder" in
  ()

let test_real_ops = fn _ =>
  let _ = assert (1.5 + 2.5 == 4.0) "real addition" in
  let _ = assert (5.5 - 2.5 == 3.0) "real subtraction" in
  let _ = assert (1.5 * 2.0 == 3.0) "real multiplication" in
  let _ = assert (7.5 / 2.5 == 3.0) "real division" in
  ()

let test_string_and_bool = fn _ =>
  let _ = assert (core::opt::is_some (core::bool::select true 1)) "bool select some" in
  let _ = assert (core::bool::select_else true (fn _ => 1) (fn _ => 2) == 1) "bool select_else true" in
  let _ = assert (core::opt::contains 3 (core::bool::guard true 3)) "bool guard true" in
  let _ = assert (core::string::replace "banana" "na" "xo" == "baxoxo") "string replace repeated substring" in
  let _ = assert (core::string::replace "aaaa" "aa" "b" == "bb") "string replace non-overlapping" in
  let _ = assert (core::string::replace "hello" "" "x" == "hello") "string replace empty needle no-op" in
  let split_with_substring = core::string::split "a--b--c" "--" in
  let _ = assert
    (match split_with_substring with
      | ["a", "b", "c"] => true
      | _ => false)
    "string split substring delimiter"
  in
  let split_without_delimiter = core::string::split "hello" "-" in
  let _ = assert
    (match split_without_delimiter with
      | ["hello"] => true
      | _ => false)
    "string split delimiter missing"
  in
  let split_with_empty_delimiter = core::string::split "hello" "" in
  let _ = assert
    (match split_with_empty_delimiter with
      | ["hello"] => true
      | _ => false)
    "string split empty delimiter returns original"
  in
  let split_with_empty_segments = core::string::split "a,,b," "," in
  let _ = assert
    (match split_with_empty_segments with
      | ["a", "", "b", ""] => true
      | _ => false)
    "string split preserves empty segments"
  in
  ()

let test_function_helpers = fn _ =>
  let subtract = fn left right => left - right in
  let _ = assert (core::function::identity 9 == 9) "function identity" in
  let _ = assert (core::function::constant 9 false == 9) "function constant" in
  let _ = assert (core::function::flip subtract 1 4 == 3) "function flip" in
  let _ = assert (core::function::compose (fn n => n + 1) (fn n => n * 2) 3 == 8) "function compose" in
  let _ = assert (core::function::pipe 3 (fn n => n + 2) == 5) "function pipe" in
  ()

let test_array_and_option = fn _ =>
  let _ = assert ([1, 2, 3] == [1, 2, 3]) "array integer equality" in
  let _ = assert (["a", "b"] == ["a", "b"]) "array string equality" in
  let _ = assert (not ([1, 2] == [1, 2, 3])) "array equality length mismatch" in
  let _ = assert (core::opt::is_some (Some 1)) "option is_some" in
  let _ = assert (core::opt::contains 1 (Some 1)) "option contains" in
  ()

let test_result_and_default = fn _ =>
  let _ = assert (core::result::is_ok (Ok 1)) "result is_ok" in
  let _ = assert (core::opt::is_some (core::result::ok (Ok 1))) "result ok" in
  let _ = assert (core::result::contains 1 (Ok 1)) "result contains" in
  ()

let test_big_num_ops = fn _ =>
  let natural = core::big-num::natural_from_integer in
  let integer = core::big-num::integer_from_integer in
  let n0 = natural 0 in
  let n1 = natural 1 in
  let n3 = natural 3 in
  let n5 = natural 5 in
  let n8 = natural 8 in
  let _ = assert (n5 + n8 == natural 13) "big-num natural addition" in
  let _ = assert (n8 - n3 == natural 5) "big-num natural subtraction" in
  let _ = assert (n3 - n8 == n0) "big-num natural subtraction saturates" in
  let _ = assert (core::ops::[~] n8 == n0) "big-num natural unary subtraction saturates" in
  let _ = assert (n8 * n3 == natural 24) "big-num natural multiplication" in
  let _ = assert (n8 / n3 == natural 2) "big-num natural division" in
  let _ = assert (n8 % n3 == natural 2) "big-num natural remainder" in
  let _ = assert (n3 < n8) "big-num natural compare" in
  let _ = assert (n8 > n3) "big-num natural compare reverse" in
  let _ = assert ((n5 and n3) == n1) "big-num natural bitwise and" in
  let _ = assert ((n5 or n3) == natural 7) "big-num natural bitwise or" in
  let _ = assert ((n5 xor n3) == natural 6) "big-num natural bitwise xor" in
  let _ = assert ((not n5) == natural 2) "big-num natural bitwise not" in
  let huge = core::big-num::natural_pow10 30 in
  let _ = assert (show huge == "1000000000000000000000000000000") "big-num natural large show" in
  let _ = assert (show (huge + huge) == "2000000000000000000000000000000") "big-num natural large arithmetic" in
  let _ = assert (show (core::big-num::integer_from_natural huge) == "1000000000000000000000000000000") "big-num integer from natural" in
  let i1 = integer 1 in
  let i3 = integer 3 in
  let i5 = integer 5 in
  let i7 = integer 7 in
  let in2 = integer (-2) in
  let in7 = integer (-7) in
  let _ = assert (i5 + in7 == integer (-2)) "big-num integer addition" in
  let _ = assert (i5 - in7 == integer 12) "big-num integer subtraction" in
  let _ = assert (in2 * i7 == integer (-14)) "big-num integer multiplication" in
  let _ = assert (in7 / i3 == integer (-2)) "big-num integer division" in
  let _ = assert (in7 % i3 == integer (-1)) "big-num integer remainder" in
  let _ = assert (in7 < in2) "big-num integer compare" in
  let _ = assert (i5 > in2) "big-num integer compare reverse" in
  let _ = assert ((in2 and i7) == integer 6) "big-num integer bitwise and" in
  let _ = assert ((in2 or i1) == integer (-1)) "big-num integer bitwise or" in
  let _ = assert ((in2 xor i1) == integer (-1)) "big-num integer bitwise xor" in
  let _ = assert ((not i1) == integer (-2)) "big-num integer bitwise not" in
  ()

let test_show_trait = fn _ =>
  let min_integer = (0 - 9223372036854775807) - 1 in
  let _ = assert (show () == "()") "show unit" in
  let _ = assert (show true == "true") "show boolean true" in
  let _ = assert (show false == "false") "show boolean false" in
  let _ = assert (show 42 == "42") "show integer" in
  let _ = assert (show (-42) == "-42") "show integer negative" in
  let _ = assert (show min_integer == "-9223372036854775808") "show integer min" in
  let _ = assert (show 1.5 == "1.5") "show real positive" in
  let _ = assert (show (-0.25) == "-0.25") "show real negative" in
  let _ = assert (show 2.0 == "2") "show real integer value" in
  let _ = assert (show (0.0 / 0.0) == "nan") "show real nan" in
  let _ = assert (show (1.0 / 0.0) == "inf") "show real positive infinity" in
  let _ = assert (show ((0.0 - 1.0) / 0.0) == "-inf") "show real negative infinity" in
  let _ = assert (show "hello" == "\"hello\"") "show string quotes" in
  let _ = assert (show "a\nb" == "\"a\\nb\"") "show string escapes" in
  let _ = assert (show 'a' == "'a'") "show glyph" in
  let _ = assert (show '\n' == "'\\n'") "show glyph escape" in
  let none_integer : Option Integer = None in
  let _ = assert (show (Some 1) == "Some(1)") "show option some" in
  let _ = assert (show none_integer == "None") "show option none" in
  let ok_value : Result String Integer = Ok 7 in
  let err_value : Result String Integer = Err "bad" in
  let _ = assert (show ok_value == "Ok(7)") "show result ok" in
  let _ = assert (show err_value == "Err(\"bad\")") "show result err" in
  let _ = assert (show [1, 2, 3] == "[1, 2, 3]") "show array" in
  let nested : Array (Option Integer) = [Some 1, None] in
  let _ = assert (show nested == "[Some(1), None]") "show nested array" in
  ()

let run = fn _ =>
  let _ = test_integer_ops () in
  let _ = test_real_ops () in
  let _ = test_string_and_bool () in
  let _ = test_function_helpers () in
  let _ = test_array_and_option () in
  let _ = test_result_and_default () in
  let _ = test_big_num_ops () in
  let _ = test_show_trait () in
  ()

let () = run ()
