module s =
	import print : (std::integer * std::integer) -> () = sys::print_string
	let to_print = "asdf"
	let () =
		string::unsafe_memory_store to_print 0;
		print (0, string::length to_print)
end

