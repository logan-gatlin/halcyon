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

/*
IntArray4 :: {
	loop i: 0 {
		print_integer(i);
		i + 1
	}
};
*/
IntArray2 :: array(integer, 2);

main :: () {
	IntArray2.{
		value: 0,
		next: .{
			value: 1,
			next: (),
		}
	};
}

