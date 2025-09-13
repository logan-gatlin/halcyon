module test = 
	let a = ["a", "b"]
	do match [] with
		| [] => std::println "Empty"
		| [a] => std::println "one"
		| [a, b] => std::println "two"
		| [a, b, c] => std::println "three"
end
