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
  let _ = assert (core::opt::is_some (Some 1)) "option is_some" in
  let _ = assert (core::opt::contains 1 (Some 1)) "option contains" in
  ()

let test_result_and_default = fn _ =>
  let _ = assert (core::result::is_ok (Ok 1)) "result is_ok" in
  let _ = assert (core::opt::is_some (core::result::ok (Ok 1))) "result ok" in
  let _ = assert (core::result::contains 1 (Ok 1)) "result contains" in
  ()

let run = fn _ =>
  let _ = test_integer_ops () in
  let _ = test_real_ops () in
  let _ = test_string_and_bool () in
  let _ = test_function_helpers () in
  let _ = test_array_and_option () in
  let _ = test_result_and_default () in
  ()

let () = run ()
