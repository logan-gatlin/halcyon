# Halcyon Agent Guide

## Halcyon language notes
- File extension: `.hc`.
- Syntax is OCaml-like; see `GRAMMAR.md`.
- No `rec`, all `let`'s are recursive.
- Source files can declare a bundle root with `bundle <ident>` and then define statements directly at top level (no outer `module ... end` required).
- Bundle root files should be named `bundle.hc` by convention.
- Paths are `Path.major` (bundle name), and `Path.minor` (the declaration path inside that bundle).
- i.e. `core::foo::bar` = `Path { major: "core", minor: foo::bar }`
- Core types live in the `core` bundle.
- The `core::prelude` module contains symbols which are always in scope
- CLI bundle compilation expects exactly one bundle-root file whose first item is `bundle <ident>`; imported files are part of the same bundle and must not redeclare `bundle`.
- Prefixing a path with `bundle` makes it relative to the current bundle, i.e. `bundle::foo` resolves to `core::foo` if the current bundle is `core` - this is preferred over writing the bundle name explicitly.
- Similarly, prefixing a path with `root` makes it a global fully qualified path.
- `compile_source` uses implicit bundle name `_` when no bundle declaration is present.
- Do not special-case `core::*` symbols in the compiler. Core types/terms/traits are regular symbols inserted into the symbol table by `hc_core`, and all phases should treat them like any other module symbols.
- Do not add regression tests for syntax changes

## Naming quick reference
- Modules: `kebab-case`
- Types, traits, constructors, constructor aliases: `PascalCase`
- Trait items, let bindings, and struct fields: `snake_case`

## Type system (concise formal summary)
- Core discipline: Hindley-Milner inference with bidirectional checking and first-order unification.
- Kinds: kind annotations are not part of source syntax; constructor/trait parameter kinds are inferred and checked semantically. Partial type application (for example `m a`) is supported, and kind mismatches are rejected.
- Polymorphism: let-generalization yields rank-1 schemes; higher-rank polymorphism is predicative and annotation-only (`for a in ... [where ...]`), never inferred implicitly.
- Constraints: trait predicates (`T τ1 ... τn`) are inferred/checked, kind-checked, solved via global trait instances, then elaborated to dictionaries.
- Type structure: primitives, tuples, arrays, functions, nominal named types (`bundle::name` with parameters), explicit structural record constraints, and type-constructor application.

## User interfaces
- The target audience is programmers with only a basic understanding of FP
- Assume knowledge of primitives such as map, fold, flatmap, zip, traverse, etc
- Avoid jargon such as Monad, Kind, Skolem, polymorphism, type variable, etc
- Make error messages as long as they need to be to effectively communicate the problem - brevity is not a virtue
- NEVER make assumptions about intent, instead demonstrate contradiction with concrete examples
