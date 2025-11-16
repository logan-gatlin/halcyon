# Halcyon Compiler
The Halcyon language is a strongly typed functional programming language for the 
[WebAssembly](https://webassembly.org/) virtual machine.

```
module Demo = 
  import std
  let fizzbuzz = fn from to =>
    let to_str = fn num =>
      match (num % 3, num % 5) with
      | (0, 0) => "FizzBuzz"
      | (0, _) => "Fizz"
      | (_, 0) => "Buzz"
      | (_, _) => "---"
    in
      if from <= to then
        std:print_string (to_str from);
        fizzbuzz (from + 1) to
  let _ = fizzbuzz 1 30
end
```
