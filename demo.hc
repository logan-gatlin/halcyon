array :: (T: type, size: integer) -> type {
	if size <= 0 {
		nothing
	} else {
		struct {
			value: T,
			next: array(T, size - )
		}
	}
}

IntArray4 :: array(integer, 4);

