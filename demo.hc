array :: (T: type, size: integer) -> type {
	if size <= 0 {
		nothing
	} else {
		struct {
			value: T,
			next: array(T, size - 1)
		}
	}
}

i4 :: array(integer, 4)

() {
	print_string("hello world");
}
