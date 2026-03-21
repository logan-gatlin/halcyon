# Performance Profiling

This repo now has built-in phase profiling for compiler runs.

## One-time setup

Install the CLI benchmark tool:

```bash
cargo install hyperfine
```

Optional sampling profiler:

```bash
cargo install samply
```

## Baseline timing

Build the CLI once:

```bash
cargo build -p halcyon
```

Run an end-to-end benchmark of the core-linked compile path:

```bash
hyperfine --warmup 1 --runs 5 'target/debug/halcyon build tmp_option_show.hc'
```

## Phase breakdown

Enable phase timing with `HALCYON_PROFILE=1`:

```bash
HALCYON_PROFILE=1 target/debug/halcyon build tmp_option_show.hc
```

The CLI prints a table with:

- phase name
- total time spent in the phase
- invocation count
- average and max duration
- percentage share of total sampled time

This is useful for identifying high-level bottlenecks before deeper profiling.

## Sampling profile (optional)

```bash
samply record --save-only -o target/core-profile.json -- target/debug/halcyon build tmp_option_show.hc
```

On Linux this may require `perf_event_paranoid <= 1`.

## Debug info modes

The CLI defaults to fast builds with debug metadata disabled.

Enable debug metadata when needed:

```bash
HALCYON_DEBUG_INFO=1 target/debug/halcyon build tmp_option_show.hc
```

You can also control each stream independently:

```bash
HALCYON_EMIT_SOURCE_MAP=1 target/debug/halcyon build tmp_option_show.hc
HALCYON_EMIT_DWARF=1 target/debug/halcyon build tmp_option_show.hc
```

Validation is also opt-in for fast builds:

```bash
HALCYON_VALIDATE=1 target/debug/halcyon build tmp_option_show.hc
```

## Project-local compiler cache

Compilation now uses a persistent project-local cache at:

```text
target/.halcyon-cache/
```

The cache key includes:

- compiler version
- Rust target triple
- cache unit (`core` or bundle root path)
- source dependency fingerprint (embedded core sources or transitive import graph)
- pre-compilation symbol-table fingerprint
- debug info mode (`emit_source_map`, `emit_dwarf`)

Cache lifecycle commands:

```bash
target/debug/halcyon cache warm [<bundle-root>...]
target/debug/halcyon cache warm --debug-info [<bundle-root>...]
target/debug/halcyon cache clear
```

## Current baseline

Command:

```bash
target/debug/halcyon build tmp_option_show.hc
```

Measured with `hyperfine`:

- cold cache (first run, fast mode): about `4.61s`
- warm cache (steady state, fast mode): about `0.683s`
- warm cache with debug metadata (`HALCYON_DEBUG_INFO=1`): about `5.29s`

Measured with `HALCYON_PROFILE=1`:

- warm fast path now dominated by linking/encoding and cache deserialization
- `artifact.cache.load` is ~70ms on this machine
- cold path is now dominated by core miss compile + backend encode, with much lower trait-solver cost than before

## Implemented in this pass

1. Debug metadata gating via compile options
   - `CompileOptions` now has `emit_source_map` and `emit_dwarf`
   - codegen honors these fields for both source bundles and core compilation
2. CLI fast/default profile
   - `build` / `run` now default to debug metadata disabled
   - env flags can re-enable debug metadata on demand
3. Rayon parallelism in key hotspots
    - parallel trait-impl candidate evaluation in predicate solving
    - parallel per-function source-map and DWARF row preparation
    - parallel debug metadata generation (`rayon::join`) during wasm encoding
4. Generalized persistent cache (project-local)
   - stores `{artifact + symbol table snapshot}` for core and source bundles in `target/.halcyon-cache`
   - one shared cache pipeline replaces dedicated core-only caching code
5. Predicate candidate prefiltering
   - skips impl candidates with incompatible concrete head shapes before unification-table cloning
6. Fast validation toggle
   - `HALCYON_VALIDATE=1` enables wasm validation; default fast path skips it
7. Trait solver cold-path acceleration
   - added trait-impl per-position indexing (`trait_impl_indexes`) for faster candidate narrowing
   - added ground predicate memoization in recursive predicate solving
8. Cache management command
   - `halcyon cache warm [<bundle-root>...]` and `halcyon cache clear` manage project-local cache without committed binary blobs

## Optimization roadmap

1. Eliminate expensive table cloning in hot loops
   - replace full `UnificationTable` clone-per-candidate with reversible checkpoints/rollback
   - keep canonicalized predicate forms to avoid repeated normalization work
2. Reduce cache load overhead on warm path
   - current `artifact.cache.load` (~70ms) is dominated by deserializing large symbol snapshots
   - evaluate compact wire format / split tables / mmap-friendly layout
3. Optimize debug-info-on path
   - source-map generation is the largest debug metadata cost
   - parallelize and reduce allocation in mapping encoding pipeline
4. Reduce linker decode/merge cost
   - `link.link_binaries.decode_inputs` and merge path dominate warm runtime after cache hit
   - add lightweight caching for decoded linker metadata sections
5. Add regression perf checks
   - keep `hyperfine` and `HALCYON_PROFILE` commands in CI or pre-merge scripts
   - track separate thresholds for cold cache, warm cache, and debug-info mode
