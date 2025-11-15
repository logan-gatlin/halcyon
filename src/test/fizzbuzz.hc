module FizzBuzz = 
  let fizzbuzz = fn from to =>
    let to_str = fn num =>
      match (num % 3, num % 5) with
      | (0, 0) => "FizzBuzz"
      | (0, _) => "Fizz"
      | (_, 0) => "Buzz"
      | (_, _) => "---"
    in
      if from <= to then
        std::println (to_str from);
        fizzbuzz (from + 1) to
      else ()
  do fizzbuzz 1 30
end
