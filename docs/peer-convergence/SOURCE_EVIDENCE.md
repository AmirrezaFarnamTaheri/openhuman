# Source evidence: Jev + Conduit native convergence (experimental)

OpenHuman PR #8 includes a read-only `source_evidence` tool, behind a default-disabled **Source Evidence** tool toggle. It uses the existing OpenHuman `SecurityPolicy` for each file and honors per-call TinyAgents workspace descriptors; there is no additional credential store, remote request, graph database, or mutation path.

## Contract

- `{"operation":"scan","path":".","query":"fn example","max_files":32}` enumerates up to 32 eligible files in deterministic path order, with an optional after_path cursor for successive pages. Cursor pages are not a single atomic snapshot. The optional query is a literal, case-sensitive substring; each matching line is numbered and excerpted. Scans return a SHA-256 `source_hash` per inspected UTF-8 file, a deterministic generation digest over paths/hashes, the observed byte total, an explicit `complete` flag, and `truncated_reason`.
- `{"operation":"verify","path":"src/main.rs","expected_sha256":"<64 lowercase hex chars>","start_line":1,"end_line":40}` rereads and hashes a file before returning numbered excerpts; if content changed, it refuses the stale evidence. The line window is limited to 80 lines and returned JSON is bounded to 26 KiB; when verbose escaping would exceed the limit, `lines_complete=false` and `next_line` identifies the first omitted line for a follow-up verify call with the same content hash.
- `{"operation":"graph","path":"src","max_edges":64,"focus_path":"src/lib.rs"}` derives a read-only local import/module graph from the same bounded, individually authorized source reads. Edges are `resolved`, `ambiguous`, or `unresolved`. A resolved edge only means that one observed indexed file matches the supported syntax; it is **not proof of a runtime dependency**. Impact results traverse resolved inbound edges and are never claimed to be exhaustive. The current syntactic-only extractor supports Rust `mod name;`, JS/TS relative import/export, CommonJS relative require, literal dynamic `import('./path')`, and Python relative `from .module import name` or `from . import child`. It does not parse full ASTs, name binding, Python absolute imports, dynamic specifiers, or cross-language dataflow.
- Each file is revalidated against the existing workspace, trusted-root and sensitive-path checks. Symbolic links, ignored directories, common credential filenames, large/binary/non-UTF-8 files are excluded. Individual files are capped at 256 KiB and a scan at 2 MiB; traversal depth, directory fan-out, query length, matches and output lengths are limited. Read handles enforce the file-size ceiling (not just a post-read truncation); Unix source reads reject final-component symlink substitution.
- Results always include `snapshot_atomic: false`: this is **not** an atomic filesystem snapshot and does not promise cross-file consistency under concurrent edits. Call verify before relying on any retrieved excerpt.
- `complete: false` means the scan hit a declared limit or skipped oversized material, not that search found no other matches.

## Security and limits

This is not an authorization mechanism and provides no write access. Sensitive-content classification is not exhaustive: users should explicitly opt in only for repositories whose contents may be surfaced in agent context. Path authorization is delegated to OpenHuman's existing policy. For strictly sensitive source work, use local-only inference and existing privacy settings.

The feature is a **foundation** for Jev/Conduit convergence, not a completed semantic search engine, AST/call graph, replay ledger, or hash-anchored edit system. Those remain independent future acceptance gates. Merely naming a source file here is not evidence of implementing all upstream features.

## Tests and release gates

Regression cases cover deterministic source generation, numbered matches, stale hash refusal, traversal/out-of-root denial, symlink and sensitive-file exclusion, bounded scans, workspace-descriptor isolation, and invalid digest/line ranges. Run `cargo test -p openhuman source_evidence` and the full core/frontend/Tauri gates on the exact PR head. **Do not mark this implementation verified until the hosted Rust checks pass.**

## Large repositories and integration boundaries

A scan can page using the returned `next_after_path` as the next request's `after_path`, but the catalog is intentionally not cached or persisted and **there is no cross-page atomic generation**. Files inserted, deleted, or changed during pagination require a new scan for consistent evidence. Incomplete scans and graph edge budgets surface as explicit partial flags. A graph from one page must not claim to represent the full repository.

The tool call performs no independent outbound network request. However **tool outputs may be sent to the active language model as part of the agent conversation**; enabling source evidence is not a guarantee that source metadata/snippets remain local unless the active model and privacy settings ensure that. This is why the feature is opt-in by default.

Derived graphs and evidence are different from Canny verification receipts, Open SWE/SWE-agent execution history, Jev decision model scores, or the memory engine. Combining them requires explicit provenance and lifecycle contracts; this change does not establish parity with those larger subsystems.

## Verified-output and partial-catalog safety

A page's graph must not claim unique resolution from an incomplete catalog: after directory/file/byte/output budget truncation, edges are marked `catalog_incomplete`, confirmed `to` targets are omitted, and reverse impact is withheld with `impact_omitted_reason=incomplete_catalog`. When a target was actually observed within the partial catalog, it is exposed only as `observed_candidate` for follow-up verification, never as a confirmed dependency. A syntactically matched import can still be a false dependency; all graph results are explicitly `syntactic_only`.

Scan results trim matched excerpts before exceeding the tool-output limit, preserving valid JSON and marking `complete=false` with `truncated_reason=max_response_bytes`. Hashes and result generation describe the observed bytes, **not** an atomic snapshot or completeness of excluded files. Explicitly addressed symbolic links are refused just as linked files discovered while walking are skipped.

The proof strength is restricted to focused regression cases until Rust unit tests and platform integration tests execute on the latest exact PR head. Do not represent a passing peer-suite baseline or successful TypeScript check as proof that this Rust feature compiles.

Graph results also include `scope_path` and `scope_is_workspace`. Even a scan that finishes its specified subdirectory is not evidence that imports outside that directory were absent from the whole repository. Graph impact remains explicitly non-exhaustive.

## Pass 9 direct-path, pagination and resource-resolution corrections

- Explicitly addressed files are now checked for excluded ancestors, not only their basename. The exclusion is case-insensitive and includes Git/mercurial metadata, vendor/build/cache folders, and common hidden SSH/cloud/Kubernetes/PKI credential stores. A selected file under an excluded directory cannot bypass the walker's normal deny list.
- For scans stopped by the aggregate byte limit, `next_after_path` points to the last examined/accepted file **before** the byte-limited file. A subsequent request resumes without silently skipping that unprocessed candidate.
- A nonempty `after_path` is a partial file catalog, even when the last page has no further items. `catalog_complete=false` and `scan_complete=false` are returned for derived graph results; targets and impact remain provisional. Regular scans retain `complete` as page-exhaustion status and additionally expose `catalog_complete` to avoid claiming full-repository coverage.
- Literal JS/TS import specifiers naming observed JSON, CSS or other text resources can resolve to that exact file. This remains syntactic, read-only and non-exhaustive evidence.

The Source Evidence and Local Check APIs are separate read-only projections. A source digest is not proof that a test ran on that digest. Combining them as an authoritative post-edit verification receipt requires persisted linkage from a specific checked source generation to a successful command and final CI commit.

## Pass 10: local locate and correct global lexical paging

The same opt-in tool now accepts `{"operation":"locate","query":"where is the session checked","path":".","max_results":5}`. It derives a locally ranked candidate shortlist from the individually authorized observed files, using bounded path tokens and line-token evidence. Up to five candidates carry a source SHA-256, a heuristic integer score, matched terms and up to three numbered line excerpts (160 Unicode scalars each). No remote Jev or general LLM call is made by this operation; **tool outputs may still be sent to the configured main model**. Output declares `ranking_kind=local_lexical_heuristic`, `source_of_truth_verified=false`, and `no_match_is_exhaustive=false`. Always `verify` a claimed excerpt's hash before relying on it.

A previous cursor implementation wrongly stopped after the first `max_files + 1` entries during depth-first traversal, then sorted only that prefix. Directory `a/child.rs` could be visited before lexically preceding `a.rs`, causing the cursor to skip `a.rs`. The implementation now enumerates the independently bounded traversal first, sorts all observed eligible files by normalized relative path, and only then selects a page. If the traversal budget itself is exceeded, the result is partial **without issuing a cursor that could skip unobserved files**. The final enriched graph JSON is also independently bounded after partial-catalog metadata has been attached; omitted edges invalidate graph-derived impact.

This is candidate discovery and conservative syntactic dependency evidence, not a full repository-scale semantic code navigator, full AST/call graph, or atomic snapshot. The source graph still requires real Rust/compiler and multi-platform regression verification before shipping.
