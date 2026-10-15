# Pass 10 — Seventeen additional peer archives and second-order convergence

## Scope, provenance and limits

An additional 17 user-supplied archives (1,372 archive members) were inventoried including nested folders and all file extensions. Their eligible source candidates include 851 files. The expanded source/config/documentation audit emitted **980 per-file SHA-256 records** and **8,404 AST/lexical declaration records**. TypeScript 5.8 and Python 3.13 parsed 393 JS/TS and 163 Python files respectively with no syntax diagnostics in that selected set; Rust symbols were indexed lexically and need compiler confirmation. There were zero cross-peer identical **entire source files** with at least 128 bytes, which does not exclude copied methods or semantically equivalent implementations. The full inventory and method index are available in the conversation's delivered evidence files.

| Source peer | Candidate capability | Native OpenHuman disposition |
| --- | --- | --- |
| Stanley | Deterministic coding workflows, trust promotion and bounded evidence | Adapt verification lifecycle into existing TinyAgents journal/workflow owner; not a second Pi runtime |
| Jev-tree | Recursive partitioned choice above the 255-option limit | Extend existing Jev/ToolRanker only with explicit bounded per-level choices and abstention, not a parallel router |
| TypeSafe Jev Workflow | Typed intent routing through branches | Existing workflows suffice; handlers in the peer are demos only |
| Skillbox | Versioned skill library, scoped clients, MCP recommendations | Existing skills catalog/registry, preflight and approval gate own this |
| JevRouter | Policy/risk/permission/availability before probabilistic decisions | Use native authorization and capability gates before any Jev score |
| Hono Jev Router | Meaning-based HTTP dispatch | Do not put non-deterministic semantic routing in the authenticated API critical path without hard protocol checks |
| Todo-Jev | Skill match includes both positive and exclusion conditions | Extend existing skill discovery/preflight without making skills grant permissions |
| TypeSafe Skill Router | Optional single-skill suggestion with failure isolation | Reuse existing opt-in skill/tool discovery, no unsolicited prompt injection |
| Jev Auto Router | Per-call model effort, fallback, outcome/cost receipts | Keep provider factory/key pool, account scope and explicit BYOK consent authoritative |
| Switchboard | Capability catalog and model effort/usage dashboard | Reuse Model Fleet and provider routing; no hidden traffic interception proxy |
| Pi Jev Router | Session-pinned model and effort | Existing correlation/run routing should remain stable unless user explicitly changes role |
| Jevbus | Typed event dispatch, breakers, delivery ledger | Existing orchestration/event controls own delivery and replay |
| JevLoop | Deterministic decision documents, hard gates and replay | Adopt fail-closed policy and threshold provenance; do not replace the agent harness |
| Jev Model Router | Live capability catalog, deterministic cost feasibility | Integrate curated capability snapshots only after authorization/cost/transport verification |
| Winx | Durable PTY/MCP, multi-principal guard, remote agent lifecycle | Existing TinyAgents workspace/shell sandbox and MCP auth remain authority; remote PTY requires isolation review |
| Jevex | Source-of-truth location packets with uncertainty and line provenance | Native opt-in Source Evidence locate + SHA verification |
| Blink | Codebase file search via concurrent walkers | Native bounded lexical candidate navigation; do not copy unbounded parallel walkers or send source to Jev by default |

## New implementation on this PR

- **Jev external decision hardening:** bounded model and instruction inputs, explicit untrusted-data guard, rejection of NaN/infinite/out-of-range response probabilities, unknown option identifiers, and invalid needs-tool probability. The failed earlier patch was completed; regression tests are committed.
- **Source Evidence locate:** local, deterministic path/token and line scoring over individually authorized observed UTF-8 files; up to five candidates with three line-numbered evidence excerpts per file and SHA-256 provenance. Candidate status is explicitly not source-of-truth; no remote request or new index. It reuses the same opt-in tool and sensitive-path exclusions.
- **Source scanner correctness:** sort the bounded full eligible catalog before slicing pages, so directory-prefix paths cannot be skipped across cursors. If traversal itself is partial, do not issue an unsafe continuation. Bound the **final** graph JSON *after* adding partial-catalog observed candidates and response metadata.

## Verified peer reference baselines and limitations

Reference suites reported JevLoop 311/311 passing, Jev Model Router 60 passed/1 skipped, and TypeSafe Skill Router 65 passing in offline environments. Stanley, Todo-Jev and the TypeSafe workflow did not pass their limited local harnesses due to test-runner/dependency or fixture gaps; these are **not** OpenHuman CI results.

All new Rust production code/tests require `cargo test`, `cargo check`, `cargo fmt` and `cargo clippy`, followed by frontend/Tauri/E2E checks against the latest exact commit. A syntactic AST index or successful peer test cannot replace that evidence. No new GitHub Actions runs have been observed for connector-origin branch heads to date, and PR #8 stays draft.

## Second-order planes not fully implemented

1. Hierarchical typed skill/tool/model routing with deterministic permission, resource, context and cost feasibility.
2. Scoped, incremental symbol/call graph with AST parser provenance and conservative cross-language edges.
3. Machine-verifiable completion receipts tied to source generation, observed local checks and final commit CI.
4. Unified consent/provenance/deletion of memory/context from external capture sources.
5. Approved application adapters and artifact storyboard/rendering reuse.
6. Operator cockpit for routing, permission, cost, source freshness and verification telemetry.

Adapt conceptually from peer code; the instruction to assume peer projects are MIT does not change their real licenses or third-party notices. In particular, review Apache/dual-licensed or unlicensed dependencies before distributing adapted code.
