module test =
	(* strings module *)
	let () = 12345 |> string::from_integer |> std::println
	let () = 12345
		|> string::from_integer
		|> (==) "12345"
		|> std::assert


end
