array :: (T: type, size: integer)->type {
	if size <= 0 {
		nothing
	} else {
		struct {
			value: T,
			next: array(T, size - 1)
		}
	}
}

() {
	print_string("Hello world")
}
