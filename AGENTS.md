# Halcyon Agent Guide

## Halcyon language notes
- File extension: `.hc`.
- Syntax is OCaml-like; see `GRAMMAR.md`.
- Paths are always `module::name` (two parts).
- Core types live in the `core` module.
- The `core::prelude` module contains symbols which are always in scope
- Do not special-case `core::*` symbols in the compiler. Core types/terms/traits are regular symbols inserted into the symbol table by `hc_core`, and all phases should treat them like any other module symbols.
- Do not add regression tests for syntax changes

## Type system (concise formal summary)
- Core discipline: Hindley-Milner inference with bidirectional checking and first-order unification.
- Polymorphism: let-generalization yields rank-1 schemes; higher-rank polymorphism is predicative and annotation-only (`for a in ... [where ...]`), never inferred implicitly.
- Constraints: trait predicates (`T τ1 ... τn`) are inferred/checked and solved via global trait instances, then elaborated to dictionaries.
- Type structure: primitives, tuples, arrays, functions, nominal named types (`module::name` with parameters), and explicit structural record constraints.
