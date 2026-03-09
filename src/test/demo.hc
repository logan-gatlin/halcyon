module test =
	let x = 1
end

module demo =
	let x = 2
	use test
	let y = x
end
