# Pass 8: Evidence, provider presets, and verified completion

## Scope and proof boundary

The 20 additional peer ZIP archives yielded 25,264 central-directory entries, 10,750 eligible source candidates, and 82,739 lexical declaration occurrences. A second strict fingerprint analysis indexed 9,768 source files and 1,045,322 normalized 12-line windows. Only the Mem0/Ruflo alert-dialog scaffolding yielded 108 exact cross-peer windows. Neither method-name overlap nor absence of exact code windows establishes semantic equivalence or difference.

## Structural parser deep pass

An additional offline syntax-tree audit read **9,724** eligible JS/TS and Python
peer source files without executing archive code. TypeScript 5.8.3 parsed
**6,759** JS/TS files and indexed 133,330 declarations, 29,226
imports, and 562,846 call occurrences. Python 3.13 accepted 2,937 of 2,965
files (28 recorded parse failures), indexing 40,239 declarations,
20,344 imports and 225,170 call occurrences. The combined AST inventories
contain **173,569 declarations, 49,570 import references and 788,016 calls**.

The more selective normalized method-body comparison, after additionally
excluding tiny Python method bodies, found **three** cross-peer pairs with
qualifying exact matches: Mem0/Ruflo (16), CLI-Anything/Mem0 (2), and
Jev-Auto-Router/Jev-Codex Router (1). A former SWE-agent/Mem0 overlap was
an undersized stub and was deliberately excluded.
Many are UI/test scaffolds; one shared Mem0/Ruflo cosine-similarity helper
lacks explicit zero-norm and dimensional-mismatch guards. It must not
supersede OpenHuman's vector implementation without numerical and
runtime parity tests. ASTs reveal structure, not semantic superiority.

The separate audit bundle includes per-file source hashes, AST method ranges,
body fingerprints, import references, parser-error records and scripts.
Rust, C#, Go and other peer languages remain in the lexical catalog but
were not processed by these Python/TypeScript AST parsers.

## Actual code committed on this PR

1. Source Evidence: opt-in SHA-256 evidence, per-file host authorization, bounded scans, pagination, hash-checked line retrieval, explicit partial results and JSON-safe output.
2. Derived Source Graph: Rust module syntax, JS/TS relative imports/requires and dynamic literal imports, plus Python relative from-imports. Edges are syntactic observations, not full parser/AST or runtime dataflow. Ambiguous, unresolved and partial catalogs never justify definitive dependency impact.
3. Root TinyAgents final-verification middleware: short consequential edits, git operations and native delegation/handoff tools now trigger a requirements check even below the previous five-round threshold. The check requires tests after the last write, latest exact commit checks and observable exit status.
4. Four new key-only, OpenAI-compatible provider presets on both Rust and frontend catalogs: Cohere (https://api.cohere.ai/compatibility/v1), TokenRouter (https://api.tokenrouter.io/v1), BytePlus ModelArk Asia Pacific (https://ark.ap-southeast.bytepluses.com/api/v3) and Europe (https://ark.eu-west.bytepluses.com/api/v3). API keys and endpoints are region-scoped for BytePlus. No model-level quota, entitlement or pricing is implied.

### System-One context convergence

The existing Jev evaluator already sent current intent, recent user turns and
shortlisted tool choices (plus a yes/no needs-tool question). The new bounded
adapter retains at most six recent user turns, limits each to 500 Unicode scalar
values, and preserves the initial and latest halves of exceptionally long
intents. It emits explicit truncation metadata and refuses duplicate/oversized
Choice options before remote egress. It does not silently add system prompts,
agent/tool transcripts, file contents or memory to the external route.

The next higher-order step is an **opt-in, provenance-aware task-state summary**
that includes only user-authorized minimal evidence necessary for selection.
Do not implement hidden raw transcript forwarding to a decision endpoint.

## Executed checks

| Check | Evidence | What it does not verify |
| --- | --- | --- |
| Jev model-router peer fixtures | 60 passed, 1 skipped | OpenHuman inference, Rust |
| Canny command classifier | 7 offline exit-status fixtures passed | Entire Vitest suite or Rust integration |
| Python import AST versus regex | 2,936 Python peer files parsed, 481 relative-from import matches | Runtime module resolution or Rust code |
| Cross-language presets | 4 labels/endpoints/auth styles matched in the fetched Rust and UI sources | Authentication and provider model availability |
| Archive provenance | All 20 ZIPs SHA-256 hashed | Licenses, manual function-by-function semantic review |

## Second-order architecture

- Evidence and source graph must be derived within OpenHuman SecurityPolicy/workspace boundaries, not a second authoritative file database.
- Hard model/capability/billing constraints must be checked before probabilistic Jev/System-One ranking.
- Real verified software completion requires persisted post-edit test, commit, CI and execution receipts from the existing TinyAgents ledger. Model prompts alone do NOT constitute such receipts.
- Memory capture and deletion must remain scoped, provenance-aware and consented via the existing memory owner.
- Per-application automation and artifacts must reuse approved typed tools and native document/rendering engines; no unmanaged parallel agent runners.
- Operator UX must distinguish unobserved, incomplete, stale, failed and verified states.

## Critical unfinished release gates

No Rust compiler/cargo existed in the execution container. The changed Rust modules have NOT been cargo-checked, fmt-checked, clippy-checked, test-run or measured for coverage. No workflows appeared for the latest exact connector-pushed PR head at the point of inspection. An older successful run is not proof of this PR head. Frontend/Tauri full builds, live provider smoke tests, source graph parser benchmarks and adversarial cross-platform path tests remain unverified. Draft PR #8 must NOT be merged.

A ZIP license is determined by its actual source terms, not a direction to assume all projects are MIT; preserve provenance and resolve notices before distributing copied code.

More details: SOURCE_EVIDENCE.md, NEW_PEERS_PASS7.md, SECOND_ORDER_JEV_9ROUTER_CONDUIT.md. Full archive-method and source-hash audit evidence is in the separately delivered pass-8 bundle.
