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
