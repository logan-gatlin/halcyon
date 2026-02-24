# Halcyon Agent Guide

## Project setup
- Rust 2024 workspace; build/test with latest stable `cargo`.
- Main crate: `halcyon-lib` (library + `src/main.rs` demo binary).
- Compiler for the Halcyon language (`.hc`), OCaml-like syntax.
- Strict scoping: nothing in scope by default, even primitives.
- Paths are exactly `module::identifier` (two components, no nesting yet).
- Hindley-Milner type inference across modules.

## Compilation pipeline
- `src/parse/`: tokenization + lossless CST + typed AST wrapper.
- `src/ir/`: lowers AST into IR and resolves identifiers into `Path`.
- `src/types/`: unification + inference + traits; produces typed IR.
- `src/asm/`: codegen (WIP/commented out).
- `src/hc_core/`: builtin core module (types/traits/terms).

## Commands
### Build
- `cargo build`
- `cargo build --release`
- `cargo build -p format`
- `cargo check`

### Run demo
- `cargo run` (uses `src/test/demo.hc`)

### Format and lint
- `cargo fmt --all`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Test
- `cargo test --workspace`
- `cargo test -p halcyon-lib`
- `cargo test -p format`
- Single test by name/pattern: `cargo test <pattern>`
- Single test (module-qualified): `cargo test parse::test::round_trip_empty`
- Single test (nested module): `cargo test types::infer::tests::infer_polymorphic_let`
- Prefer `cargo test --lib <pattern>` when targeting library tests only.
- Add `-- --nocapture` only when you need test output.

## Test layout
- Unit tests live next to code or in `tests.rs` (e.g. `src/types/tests.rs`).
- Parser tests are in `src/parse/test.rs`.
- Type inference tests also appear inside module files (`#[cfg(test)]`).
- End-to-end fixtures live under `src/test/`.
- `src/test/demo.hc` is a scratch file; overwriting is OK.
- `src/test/inference.hc` exercises type inference across constructs.
- Add new `.hc` fixtures under `src/test/` for language features.
- `src/test/mod.rs` contains a commented-out E2E harness.

## Code style
### General
- Prefer functional style and immutable data; keep scopes small.
- Prefer iterators to explicit loops unless mutation/side effects are needed.
- Use the standard library before adding dependencies.
- Make invalid states unrepresentable; use enums/newtypes/type aliases.
- Use builder patterns for complex structs when appropriate.
- Avoid unnecessary side effects in core compiler logic.

### Imports
- Group imports by origin: std, external crates, then crate/super.
- Separate groups with a blank line.
- Prefer explicit import lists over globbing.

### Formatting
- Use rustfmt defaults (4-space indent, trailing commas, multiline params).
- Keep function signatures wrapped as rustfmt prefers.
- Avoid manual alignment; let rustfmt handle it.

### Naming
- Types/traits/enums: `UpperCamelCase`.
- Functions/variables/modules/files: `snake_case`.
- Constants: `SCREAMING_SNAKE_CASE`.
- Use `type_`, `match_`, etc. for keyword collisions.
- Use descriptive names for globals; locals may be shorter when scoped.

### Types and collections
- Use `IndexMap` when deterministic ordering matters (fields/traits/IR).
- Prefer `HashMap` only when ordering is irrelevant.
- Prefer `Box<[T]>` over `Vec<T>` for collections that do not grow.
- For function parameters, prefer `impl IntoIter<T>` or `&[T]` over `Vec<T>`.
- Use `Vec::with_capacity` when sizes are known.
- Use `Path::new(major, minor)` or `Path::core(minor)` for names.

### Error handling and logging
- Clippy lints are strict; `panic!` and `unwrap` are disallowed in non-test code.
- If something is statically known to always be `Ok` or `Some`, use `unwrap_or_else(|| unreachable!())`.
- All user facing diagnostics are routed through `FileLogger` + `WithContext`.
- For cases that should never happen, prefer emitting a lint with the level `Bug` (implying a compiler bug) rather than using `unreachable!()`.
- Attach `Span` to diagnostics; use `Span::Generated` for synthetic nodes.
- Compilation should continue even when an error is encountered; avoid repeating errors that would have been emitted in a previous phase already.

### Logging patterns
- `Logger` owns files and diagnostics; `FileLogger` is file-scoped.
- Create logs with `Logger::new_file(...)` and merge via `consume_file`.
- `MockLogger` is handy in tests for a single file.
- `WithContext` returns a `LogBuilder`; call `.done()` to submit.

### Lint expectations
- `src/lib.rs` denies `clippy::all`, `clippy::exit`, and other style lints.
- `clippy::panic` is denied outside tests (`#[cfg(test)]` relaxes it).
- `clippy::print_stdout`/`print_stderr` are warnings; keep logging in `Logger`.
- `clippy::large_enum_variant` and `clippy::result_large_err` are allowed.

### AST/IR conventions
- Keep `Span` and `Spanned<T>` consistent when transforming nodes.
- Preserve comments where applicable (`comments: String` fields).
- When lowering AST, resolve names through `ModuleScope` and `Scope`.
- Use `NameSpace` to distinguish term/type/constructor bindings.
- Use `TypeExprKind::alias` for non-parameterized type instantiations.
- Normalize types through the unification table when comparing/reporting.
- Keep core types and operators in `hc_core`.

### Type system notes
- `TypeScheme` represents generalized types; use it in environments.
- Use `InferenceContext` to instantiate/generalize; normalize before comparing.
- When adding a new `Type` variant, update equality and pretty printer.
- `Type` contains a number of helper methods for constructing large types, USE THESE ALWAYS.

### Parsing conventions
- Use `Parser::start_node`/`finish_node` pairs; use markers to enforce closure.
- For trivia-sensitive nodes, use `start_node_with_leading_comments`.
- Use `error_recover` to skip to recovery tokens and keep CST lossless.
- Always attach spans to diagnostics via `FileLogger`.

### Parser/lexer updates
- Token changes require updates in `src/token.rs` and `src/parse/mod.rs`.
- Add/adjust grammar in `src/parse/grammar/`.
- Maintain round-trip tests when changing syntax.

### Testing guidance
- Use `Logger`/`FileLogger` in tests; avoid stdout assertions.

## Halcyon language notes
- File extension: `.hc`.
- Syntax is OCaml-like; see `GRAMMAR.md`.
- Paths are always `module::name` (two parts).
- Core types live in the `core` module.
- Nothing is in scope by default; resolve through core or module scope.

## Repo-specific cautions
- The `asm` module is incomplete; avoid extending it unless requested.
