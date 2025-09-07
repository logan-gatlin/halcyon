module test =
	(*
	let () = 4
		|> integer::pow 2
		|> format::integer
		|> std::println
	*)

	(* integer module *)
	let () = 0.1
		|> integer::from_real
		|> std::assert_eq 0

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
	
	(* strings module *)
	let () = 12345
		|> format::integer 
		|> std::assert_eq "12345"
	let () = -54321
		|> format::integer
		|> std::assert_eq "-54321"
end
