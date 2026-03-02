# Halcyon Agent Guide

## Halcyon language notes
- File extension: `.hc`.
- Syntax is OCaml-like; see `GRAMMAR.md`.
- Paths are always `module::name` (two parts).
- Core types live in the `core` module.
- Nothing is in scope by default; resolve through core or module scope.
- Do not special-case `core::*` symbols in the compiler. Core types/terms/traits are regular symbols inserted into the symbol table by `hc_core`, and all phases should treat them like any other module symbols.
