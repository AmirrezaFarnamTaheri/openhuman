# Pass 11: Verified boundaries, source semantics and Model Fleet consistency

**Status:** selective native fixes on draft PR #8; the overall 37-peer convergence is not finished. The code below is committed, but Rust/desktop/hosted CI have not passed on this head.

## Code-level corrections in this pass

| Module | Evidence-backed correction | Limits |
| --- | --- | --- |
| `tinyhumans/jev/evaluator_tests.rs` | Existing exact-instruction assertion was inconsistent with the mandatory appended untrusted-data guard. Retains original wording and asserts the guard. | Guard is advisory to the decision model; sandbox policy is independent |
| `filesystem/source_graph.rs` | Lexical state suppresses dependency edges found in quoted JS require/import examples, multiline JS and Rust comments, and Python triple-quoted docstrings. Custom Rust `#[path]` module attributes survive intervening comments and stop guessed module targets; inline nested Rust scopes are not falsely resolved at the file root. | Not full AST resolution or runtime dependency proof |
| `filesystem/source_locate.rs` | Returned snippet now centers around a whole token match rather than blindly returning a line prefix; carries one-based Unicode-scalar `column_start` and `line_char_count`. | Candidate scoring is local lexical heuristic, not source-of-truth verification |
| `filesystem/source_evidence.rs` | `verify` accepts bounded `start_column`, applies it to SHA-checked line excerpts and returns matching cursor metadata. | Opt-in source snippets may reach the active main model |
| `modelFleetOps.ts` | Provider bulk image mutations refuse remote HTTP destinations via the strict image transport predicate; HTTPS and loopback HTTP are allowed, empty/null clearing preserved. | Generic text-provider endpoint rules remain separate |
| `ModelFleetPanel.tsx` | Both group and per-model image form submissions use the strict image validator for every provider, not just Prodex. | Rust's `cloud_slug::image_destinations_are_safe` was already an execution-time safeguard |

All changed behavior has focused regression cases in `source_graph_tests.rs`, `source_locate_tests.rs`, `source_evidence_tests.rs`, `modelFleetOps.test.ts` and `evaluator_tests.rs`. Those newly authored tests are **not marked passing** without execution on the latest head.

## Second-order convergence decisions

- **Evidence fabric:** Keep one SecurityPolicy-controlled, SHA-anchored source-read owner. Peer graph parsers and code-search services can provide adapters or retrieval ideas, but they must not produce parallel authoritative repositories or invent certainty from incomplete reads.
- **Decision/model fleet:** Keep deterministic billing, credential, permission, transport, context and budget feasibility outside Jev probabilistic ranking. Native roles and provider keys remain independently configured.
- **Verified execution:** Use TinyAgents durable events and host-only observed check outcomes as raw evidence. A passing local command does not establish complete requirements or final-commit CI.
- **UI/API convergence:** Validate sensitive endpoint transport at form, bulk helper and Rust execution boundaries. Avoid changing existing managed/non-credentialed generic endpoint behavior unnecessarily.
- **Memory/skills/artifacts:** These larger planes remain incomplete; import only capabilities with explicit workspace scope, consent, provenance, retention/deletion and native render/verification contracts.

## Concurrent settings persistence safety

The Model Fleet panel previously checked only the React `busy` state in its
mutation handler. Two clicks in the same render frame can both observe
`busy=false` and issue overlapping writes from stale prop snapshots. A small
pure `createSingleFlightRunner` now acquires an imperative lock before an
asynchronous mutation and releases it in `finally`, including after a failed
save. Model Fleet's shared `run` handler holds one stable runner in a React
ref, suppressing a second write until the first settles. Existing visual
disabled controls remain in place as user feedback.

Focused Vitest regressions specify duplicate suppression and post-error
recovery. Three isolated Node 22 smoke cases covering equivalent single-flight
logic and the image URL predicate executed successfully; this is **not** an
OpenHuman module import, UI mount, or substitute for its Vitest/E2E runs.

## Reference-suite findings and limitations

The earlier 17-peer archive audit (1,372 members; 980 indexed files; 8,404 declaration records) remains available in the conversation evidence bundle. This pass additionally attempted offline peer tests: Blink's three Node invocations failed on Bun-only imports, Hono's on missing dev dependency, Stanley reported 22 passing/14 failing with missing dependencies among failures, Pi discovered zero tests, and Switchboard could not finish its bounded suite because of missing dependencies and incompatible TypeScript child execution. These do not prove functional peer failures.

## Required production gates

1. In a proper Rust checkout, run focused `source_graph`, `source_evidence`, `source_locate` and `jev` regression tests; then `cargo fmt --check`, workspace `cargo check/test/clippy`.
2. Run frontend TypeScript/Vitest tests including Model Fleet bulk edits, and actual browser/Tauri UI flows that attempt remote HTTP on a non-Prodex image route.
3. Recheck GitHub Actions jobs against **the final exact PR head SHA**; old runs and reference-suite outcomes cannot validate it.
4. Run adversarial symlink, Unicode, workspace/privacy, graph pagination and real provider credential tests in disposable environments.
5. Keep PR in draft until these gates pass. No blanket peer license assumption changes original license obligations when copying verbatim code.

**Audit artifacts:** downloadable PASS11_ENGINEERING_AUDIT.md and OPENHUMAN_PASS11_EVIDENCE.zip include test logs and source inventories.
