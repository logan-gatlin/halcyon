# Halcyon fuzzing

This directory contains local `cargo-fuzz` targets for full-stack compiler fuzzing.

## Setup

```bash
cargo install cargo-fuzz
```

## Priority order

1. `type_unify_ops`
2. `type_scheme_ops`
3. `type_resolve_only`
4. `lexer_only`
5. `parser_roundtrip`
6. `full_source_pipeline`
7. `full_source_with_imports`
8. `ir_pipeline`
9. `linker_inputs`
10. `custom_section_decoders`
11. `tooling_positions`

## Quick runs

```bash
cargo fuzz run type_unify_ops fuzz/corpus/type_unify_ops
cargo fuzz run lexer_only fuzz/corpus/lexer_only -- -dict=fuzz/dictionaries/halcyon.dict
cargo fuzz run parser_roundtrip fuzz/corpus/parser_roundtrip -- -dict=fuzz/dictionaries/halcyon.dict
cargo fuzz run full_source_pipeline fuzz/corpus/full_source_pipeline -- -dict=fuzz/dictionaries/halcyon.dict
```

## Profile runner

```bash
./fuzz/run-profile.sh quick
./fuzz/run-profile.sh smoke
./fuzz/run-profile.sh long
```

## Notes

- Corpora live in `fuzz/corpus/<target>`.
- Shared token dictionary is `fuzz/dictionaries/halcyon.dict`.
- CI wiring is intentionally not included yet.
