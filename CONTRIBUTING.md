# Project setup
## Building
```sh
cargo build
```
## Compile a file
The `main.rs` file compiles a Halcyon source file to WebAssembly.
It can be used in two ways.
When just the input path is specified, the WebAssembly text format (wat) is printed to stdout.
Note that error messages may be printed to stderr, so filter the output if needed.
```sh
cargo run -- <input-path>
```
The second way is to provide an output binary path.
The compiled binary (wasm) file will be written to this location
```sh
cargo run -- <input-path> <output-path>
```

# Language design
Halcyon is an immutable functional programming language based on ML.
It's type system uses type variables to achieve parametric polymorphism.
The target platform for Halcyon is WebAssembly.

# Implementation
The compiler runs in phases:
* [Tokenization](src/token.rs)
* [Parsing](src/parse/)
* [IR generation](src/ir/)
* [Type checking](src/semantic/)
* [Code generation](src/asm/)

## IR generation
The syntax tree produced by the parser is reduced into a kind of lambda calculus which is the IR.
Several simplifications are made:
* Binary and unary operators become function calls
* Array construction is reduced to a sequence of append function calls
* `let`, `if`, and `match` constructs are unified into `let <pattern> = <predicate> else <fallback>`

## Code generation
Internally, we rely on the GC proposal for handling data.
Our type system does not translate 1:1 with WebAssembly's type system, so concessions have to be made.

Closures are implemented as boxed structs that look like:
```
{
    captured_args: [any],
    function: any -> any,
}
```
This is because WASM does not support type variables, so types like 'a -> 'a are unrepresentable.
After a closure is called, it is necessary to cast the result to its appropriate type.
This should never fail.

It is not always possible or practical to construct a type all at once.
Therefore, generated code uses nullable and mutable references.
In practice, a reference should never be null or mutated after construction.
