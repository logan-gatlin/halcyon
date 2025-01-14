# Halcyon Compiler
The Halcyon language is a strongly typed compiled language for the 
[WebAssembly](https://webassembly.org/) virtual machine. Its major features
are implicit types, move semantics, and memory safety.

## Semantics
The syntax of Halcyon is designed to be minimal and readable. While Halcyon is
strongly typed, types can be assumed from context at compile time. Below is an
annotated "FizzBuzz" program
```
fizzbuzz :: (number: integer) {
  for i : 0..=number {
    if i % 3 == 0 and i % 5 == 0 {
      println("fizzbuzz");
    } else if i % 3 == 0 {
      println("fizz");
    } else {
      println("buzz");
    }
  }
}

fizzbuzz(15);
```
