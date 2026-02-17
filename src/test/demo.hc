module demo =
	type Result : a b = | Ok a | Err b

	let map = fn r f => match r with
		| Result.Ok a => Result.Ok (f a)
		| Result.Err e => Result.Err e
end
