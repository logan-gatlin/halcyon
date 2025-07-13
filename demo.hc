let identity =
	fn a => (a, a, a)
in (
	identity 1;
	identity true;
	identity "asdf asdf asdf";
	identity 'a'
)
