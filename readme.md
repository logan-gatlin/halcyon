# Halcyon Compiler
The Halcyon language is a strongly typed functional programming language for the 
[WebAssembly](https://webassembly.org/) virtual machine.

```
module Demo
  let fizzbuzz = fn from to => 
    if from < to then
      (print_string
        match (num % 3, num % 5) with
        | (0, 0) = "FizzBuzz"
        | (0, _) = "Fizz"
        | (_, 0) = "Buzz"
        | (_, _) = integer_to_string num);
      fizzbuzz (from + 1) to
    else ()
  
  let () = fizzbuzz 0 30
end
```
