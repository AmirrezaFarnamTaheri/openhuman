# Source evidence: Jev + Conduit native convergence (experimental)

OpenHuman PR #8 includes a read-only `source_evidence` tool, behind a default-disabled **Source Evidence** tool toggle. It uses the existing OpenHuman `SecurityPolicy` for each file and honors per-call TinyAgents workspace descriptors; there is no additional credential store, remote request, graph database, or mutation path.

## Contract

- `{"operation":"scan","path":".","query":"fn example","max_files":32}` enumerates up to 32 eligible files in deterministic path order. The optional query is a literal, case-sensitive substring; each matching line is numbered and excerpted. Scans return a SHA-256 `source_hash` per inspected UTF-8 file, a deterministic generation digest over paths/hashes, the observed byte total, an explicit `complete` flag, and `truncated_reason`.
- `{"operation":"verify","path":"src/main.rs","expected_sha256":"<64 lowercase hex chars>","start_line":1,"end_line":40}` rereads and hashes a file before returning numbered excerpts; if content changed, it refuses the stale evidence. The line window is limited to 80 lines.
- Each file is revalidated against the existing workspace, trusted-root and sensitive-path checks. Symbolic links, ignored directories, common credential filenames, large/binary/non-UTF-8 files are excluded. Individual files are capped at 256 KiB and a scan at 2 MiB; traversal depth, directory fan-out, query length, matches and output lengths are limited.
- Results always include `snapshot_atomic: false`: this is **not** an atomic filesystem snapshot and does not promise cross-file consistency under concurrent edits. Call verify before relying on any retrieved excerpt.
- `complete: false` means the scan hit a declared limit or skipped oversized material, not that search found no other matches.

## Security and limits

This is not an authorization mechanism and provides no write access. Sensitive-content classification is not exhaustive: users should explicitly opt in only for repositories whose contents may be surfaced in agent context. Path authorization is delegated to OpenHuman's existing policy. For strictly sensitive source work, use local-only inference and existing privacy settings.

The feature is a **foundation** for Jev/Conduit convergence, not a completed semantic search engine, AST/call graph, replay ledger, or hash-anchored edit system. Those remain independent future acceptance gates. Merely naming a source file here is not evidence of implementing all upstream features.

## Tests and release gates

Regression cases cover deterministic source generation, numbered matches, stale hash refusal, traversal/out-of-root denial, symlink and sensitive-file exclusion, bounded scans, workspace-descriptor isolation, and invalid digest/line ranges. Run `cargo test -p openhuman source_evidence` and the full core/frontend/Tauri gates on the exact PR head. **Do not mark this implementation verified until the hosted Rust checks pass.**
