# Halcyon Agent Guide

## Halcyon language notes
- File extension: `.hc`.
- Syntax is OCaml-like; see `GRAMMAR.md`.
- Paths are always `module::name` (two parts).
- Core types live in the `core` module.
- Nothing is in scope by default; resolve through core or module scope.
