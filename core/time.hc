module time =
	use bundle
	use bundle::ops

	type Duration = Integer
	type Instant = Integer

	-- Returns the current time as an `Instant`
	let current = Instant >> bundle::wasi::realtime_nanos

	-- Returns the amount of time that has passed since
	-- a given instant. If the instant is in the future,
	-- returns 0 seconds
	let time_since = fn other =>
		let Instant now_time = current () in
		let Instant other_time = other in
		if now_time > other_time then
			Duration (now_time - other_time)
		else
			Duration 0

	-- Duration -> Real
	let as_seconds = fn
		| Duration i => (bundle::real::integer_to_real i) / 1000000000.0
	-- Duration -> Integer
	let as_millis = fn
		| Duration i => i / 1000000
	-- Duration -> Integer
	let as_nanosecs = fn
		| Duration i => i

end
