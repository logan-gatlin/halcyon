module demo =
	let f = fn a b => a
	let _ = f 1 2
end

module demo2 =
	let g = demo::f
end
