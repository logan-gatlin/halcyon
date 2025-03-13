main :: () {
	a = loop i: 0 {
		if true {
			break .{a: 1, b: 2};
		} else {
			break .{a: 2, b: 3};
		}
		"asdf"
	};
}

