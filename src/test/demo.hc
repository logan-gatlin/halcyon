module demo =
	let i = 1
	wasm => (
		(type $integer (struct i64))
		(global $asdf $integer)
		(func $foo
			get i
			get $asdf
			struct.get $integer 0
		)
	)
end
