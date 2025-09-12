module test = 
	let a = [1, 2, 3]
	let b = [a.., a..]
	do array::length b
		|> format::integer
		|> std::println
end
