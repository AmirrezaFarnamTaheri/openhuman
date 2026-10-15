# Pass 11: Source-path provenance and journal lookup hardening

**State:** changes committed to draft PR #8, not compiled or run as Rust tests.

## Confirmed operating-system behavior

On Unix, `src/file.rs` and a filename containing a literal backslash,
`src\\file.rs`, can coexist as two different files. The earlier path formatter
applied `replace('\\', '/')` on every platform, causing the two files to
share an indistinguishable evidence path. A dependency graph, source hash,
file-location result, or copied path cursor could then be attributed to the
wrong file. A local POSIX filesystem reproduction confirmed the distinct
files and the old normalization collision.

**Correction:** source-relative paths preserve literal backslashes on Unix,
while Windows path separators are normalized to `/` only on Windows.
Pagination's `after_path` validator follows that same platform-specific
contract: a Unix backslash filename may round-trip as a cursor, but Windows
backslash separators remain forbidden in cursor input.

On Unix, filesystem names may also contain non-UTF-8 bytes that cannot
round-trip through the tool's JSON path representation. Directory walking
now marks such entries as `non_utf8_path` and `complete=false`, rather
than silently dropping them and claiming the catalog was complete. No lossy
name is treated as a valid source identifier.

## Additional behavior contracts

1. **Source locate:** long-line token evidence should display the matched
   symbol, not a misleading unrelated prefix. The existing
   `column_start` field is 1-based; the two new cases test long
   lines and Unicode scalar column accounting.
2. **Native replay:** the new read-only `run_local_checks` endpoint
   accepts only compact ASCII alphanumeric, dot, underscore and hyphen
   run IDs of at most 128 bytes. Path separators, newline/control bytes,
   whitespace and oversized values are rejected **before** workspace
   configuration and journal lookup. Existing replay APIs retain their
   previously established identifier policy.

## Tests added (not yet executed)

- Distinct `src/file.rs` / literal-backslash `src\\file.rs` hashes
  and successful hash-checked reread.
- Safe paged scanning across the backslash filename.
- Explicit partial status when an unrepresentable non-UTF-8 pathname is found.
- Locate excerpts that include a late token on long and Unicode lines,
  using the existing `column_start` JSON contract.
- Malformed / path-shaped / control-character replay IDs rejected before
  loading an event journal.

## Second-order convergence requirement

Every evidence item should be keyed by an unambiguous **workspace scope,
native path identity, content hash, and observation generation**. A prettified
display label is not an identity, and a partial catalog is not a negative
existence proof. Source evidence and verification receipts should only be
joined after a checked generation is explicitly associated with a local
process receipt and a final-commit CI result. That stronger linkage is not
implemented by these fixes.

This pass does **not** claim comprehensive AST graph coverage, full semantic
source search, trusted memory consolidation, or release readiness. The
available container lacks Rust/Cargo and outbound DNS for installing the
toolchain. GitHub Actions had not produced exact-head runs through the
connected GitHub tool at the time of review. Keep the PR draft until
focused/full Rust tests, formatting, clippy, frontend/Tauri, platform
security tests and final-head hosted checks pass.
