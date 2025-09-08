module test =
	(* wasm module *)
	let () = 69
		|> wasm::store_i32 0
	let () = wasm::load_i32_sx 0
		|> format::integer
		|> std::println

	(* integer module *)
	let () = 0.1
		|> integer::from_real
		|> std::assert_eq 0
	let () = 'a'
		|> integer::from_glyph
		|> std::assert_eq 97
	let () = integer::pow 4 2
		|> std::assert_eq 16
	let () = integer::pow 1 0
		|> std::assert_eq 1
	let () = integer::pow 0 0
		|> std::assert_eq 1
	let () = integer::pow (-4) 2
		|> std::assert_eq 16

	(* real module *)
	let () = 0
		|> real::from_integer
		|> std::assert_eq 0.0
	let () = 16.0
		|> real::sqrt
		|> std::assert_eq 4.0
	let () = 2.5
		|> real::round
		|> std::assert_eq 2.0
	
	(* format module *)
	let () = 12345
		|> format::integer 
		|> std::assert_eq "12345"
	let () = -54321
		|> format::integer
		|> std::assert_eq "-54321"

	(* string module *)
	let () = "12345678"
		|> string::length
		|> std::assert_eq 8
	let () = ""
		|> string::length
		|> std::assert_eq 0
end
