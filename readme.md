# Halcyon Compiler
The Halcyon language is a strongly typed compiled language for the 
[WebAssembly](https://webassembly.org/) virtual machine. Its major features
are implicit types, move semantics, and memory safety.

## Semantics
The syntax of Halcyon is designed to be minimal and readable. While Halcyon is
strongly typed, types can be assumed from context at compile time. Below is an
annotated "FizzBuzz" program
```
fizzbuzz :: (number) {
  sum := 0;
  if number == 0 {
    println("Input cannot be zero");
    break sum; // Early return
  }
  for i : 0..number {
    if (i % 3 == 0) and (i % 5 == 0) {
      println("fizzbuzz");
      sum += 1;
    }
    else if i % 3 == 0 {
      println("fizz");
    }
    else if i % 5 == 0 {
      println("buzz");
    }
  }
  sum // return the number of fizzbuzz's
}

fizzbuzz(15);
```

## In Depth
I have written a more comprehensive (but outdated) language specification
on my blog [here](https://lgatlin.dev/writings/2-language-ideas.html).
