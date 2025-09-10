module test = 
	let a = array::map (fn a =>
		std::println "Hello" ;
	()) [1, 2, 3]
	let b = array::get 2 a
end
