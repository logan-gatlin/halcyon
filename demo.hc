fizzbuzz :: limit => loop i = 0
	if i > limit then break else
	(match (i % 5, i % 3) with
	| 0, 0 then "FizzBuzz"
	| 0, _ then "Fizz"
	| _, 0 then "Buzz"
	| _, _ then to_string(i))
	|> print ;
	i + 1
