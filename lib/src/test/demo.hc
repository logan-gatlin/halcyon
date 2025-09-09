module test = 
	let foo = array::length [1, 2]
		|> std::assert_eq 2
end
