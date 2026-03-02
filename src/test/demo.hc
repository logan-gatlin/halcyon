module demo =
	trait Id : self =
		let id : self -> self
	end

	impl Id : core::integer =
		let id = fn a => a
	end

	impl Id : core::boolean =
		let id = fn a => a
	end

	let a = id "a"
end
