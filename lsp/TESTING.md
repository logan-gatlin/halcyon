# halcyon-lsp testing strategy

The LSP test suite is organized as a reliability pyramid, with crash prevention
as the primary goal.

## 1) Protocol boundary tests

Location: `lsp/src/protocol.rs`, `lsp/src/server.rs`

These tests verify:

- invalid request params return JSON-RPC `InvalidParams` errors
- malformed notifications are ignored instead of crashing the server
- unknown methods return `MethodNotFound`
- response serialization and error shaping stay stable

## 2) In-memory session tests

Location: `lsp/src/server.rs`

These tests use `Connection::memory()` and drive `handle_request` /
`handle_notification` directly to validate real request-response behavior without
spawning an external process.

Covered flows include:

- completion requests returning structured completion lists
- hover requests returning symbol markdown payloads
- rename requests returning workspace edits
- notification + request sequencing in one session
- hover/rename namespace matrix coverage for `module`, `type`, `constructor`,
  `trait`, `term`, and `wasm`
- per-namespace coverage validates both in-file spans and cross-file spans
- hover doc-comment coverage validates declaration docs, cross-file docs, and
  hidden-doc suppression (`@HIDDEN`)

## 3) State and concurrency tests

Location: `lsp/src/server.rs`

These tests validate asynchronous typecheck behavior, especially generation
handling and stale-result dropping.

## 4) Diagnostics lifecycle tests

Location: `lsp/src/diagnostics.rs`

These tests validate:

- stale URI cleanup (`publishDiagnostics` with empty diagnostics)
- conversion fallback when primary labels have no available source
- graceful skipping when diagnostics cannot be anchored

## 5) UTF-16 / Unicode safety tests

Location: `lsp/src/completion.rs`, `lsp/src/keyword_hover.rs`

These tests verify that cursor handling rejects mid-surrogate positions and that
keyword/completion behavior remains correct with non-ASCII characters before the
cursor.

## Running tests

```bash
cargo test -p halcyon-lsp
```

## Rules for future tests

- prefer in-memory session coverage for user-facing behavior
- add regression tests for every crash or panic report
- include malformed payload tests for any new LSP method
- include generation/staleness assertions for async state updates
