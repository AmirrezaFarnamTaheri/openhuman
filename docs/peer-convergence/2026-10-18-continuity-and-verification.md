# Peer audit: agent memory, trajectories, and verification

Inspected on 2026-10-18 through the GitHub connector, using the exact default-branch commits below. This is a source audit, not a claim that the peer projects' tests or performance claims were reproduced. Implementation and publication of the active TinyAgents contribution remain the first priority; no peer code has been installed or ported.

## Recommendation

The strongest next contribution is a bounded transcript importer informed by **trajectory**, owned by `tinymemory-integrations`. Next, extract the idea of a typed, budgeted evidence packet from **ContextLattice** into existing memory/context interfaces. Selected **scientific-agents** profiles can be optional skill content. **pstack** contributes useful verification records and queue/test patterns, but its merge-readiness policy cannot serve as OpenHuman's approval gate unchanged.

The current contracts already cover important foundations. TinyMemory owns `MemoryEngine`, validated recall/fetch/store requests, namespace reach, provenance markers and replay-safe storage. TinyAgents owns admission, tool execution, middleware, context management and durable orchestration. OpenHuman owns composition, credentials, workspace policy and the user experience. Contributions should strengthen these owners rather than introduce a second storage or execution authority.

| Repository | Inspected main commit | Material reviewed | Initial disposition |
| --- | --- | --- | --- |
| [ContextLattice](https://github.com/sheawinkler/ContextLattice) | [`e0daa914fc3f3a9a21363470242ea3fc0bd88964`](https://github.com/sheawinkler/ContextLattice/commit/e0daa914fc3f3a9a21363470242ea3fc0bd88964) | Rust retrieval, Go context compiler/agent packet, cognition contracts, tests and license | Extract narrow primitives/contracts; whole runtime is a separate integration project |
| [trajectory](https://github.com/letta-ai/trajectory) | [`b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773`](https://github.com/letta-ai/trajectory/commit/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773) | Normalization types, canonical identity, Codex adapter, bounds, canonical tests and package CI | Highest-value module candidate for authorized transcript import |
| [pstack](https://github.com/backnotprop/pstack) | [`6f131f137de64fdaf928278c0fd9ecec520ccbee`](https://github.com/backnotprop/pstack/commit/6f131f137de64fdaf928278c0fd9ecec520ccbee) | PR watcher policy/tests, plan checker, skill dependencies and license | Adopt evidence/queue patterns selectively; rewrite readiness semantics |
| [scientific-agents](https://github.com/K-Dense-AI/scientific-agents) | [`98c7fae46648724e1a43a1f2b207b0c82fcc462e`](https://github.com/K-Dense-AI/scientific-agents/commit/98c7fae46648724e1a43a1f2b207b0c82fcc462e) | Catalog/build validation, CI, research software profile and license | Optional domain content plus catalog validation patterns |

Effort below means an engineering estimate for one contributor familiar with these repositories, including focused tests and documentation. It excludes waiting for owner review and deployment. Estimates are not measured delivery commitments.

## 1. trajectory: preserve the meaning of imported sessions

### The concrete gap

The current TinyMemory [`ConversationReader`](https://github.com/tinyhumansai/tinymemory/blob/41c34f36d912ffecac24b229cb657f331c7fc607/crates/tinymemory-integrations/src/sources/readers/conversation/mod.rs) reads OpenHuman-shaped JSON thread files. Its `thread_turns` accepts nonempty string content, skips unknown roles, and initializes `tool_calls` as empty. The existing [`Turn` / `StoreItem::Conversation`](https://github.com/tinyhumansai/tinymemory/blob/41c34f36d912ffecac24b229cb657f331c7fc607/crates/tinymemory-api/src/item/mod.rs) contract can retain timestamps and tool-call references, so format-aware import can improve fidelity without replacing the memory engine.

trajectory supplies a useful reference contract: separate message, reasoning, observation, tool-call and tool-result records; diagnostics for repairs; explicit bounds; source context for chunked input; and an optional outcome flag that is absent when the source does not provide an authoritative result. Its [`types.ts`](https://github.com/letta-ai/trajectory/blob/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773/src/types.ts) does not infer success from result prose.

### Absorption options

| Scale | Candidate and owner | Why it fits | Estimated effort |
| --- | --- | --- | --- |
| Primitive | Source identity kind, diagnostics, explicit unknown outcome, Unicode-safe output bounds in `tinymemory-integrations` | Prevents silent loss and ambiguous import receipts | 2–4 days |
| Focused module | Codex/Claude transcript codec producing typed source records, then `StoreItem::Conversation` | Adds new import formats while keeping namespace and storage policy in TinyMemory | 1–2 weeks for two formats |
| Whole normalization package | An explicitly invoked, pinned TypeScript normalizer used by a host import tool | Reuses the adapter suite, but requires a supported Node runtime and a bridge to Rust storage types | 2–4 weeks including packaging and parity |

The [`canonical.ts`](https://github.com/letta-ai/trajectory/blob/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773/src/canonical.ts) distinction between native, location-based and content-based identity is particularly valuable. Content fallback supports duplicate detection, but cannot establish that changed content is a new version of the same native record. Absolute byte offsets make Codex record identity stable across append-only chunk boundaries. Preserve that distinction as import provenance; do not replace `StoreItem::fingerprint` or engine-assigned identities with untrusted transcript IDs.

The [`Codex adapter`](https://github.com/letta-ai/trajectory/blob/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773/src/adapters/codex/index.ts) recognizes session metadata, ordinary and custom tool calls, and injected environment/instruction prefixes. The default [`bounds`](https://github.com/letta-ai/trajectory/blob/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773/src/bounds.ts) are character limits, not tokenizer guarantees. Integrating them into model context still requires TinyAgents' own budget accounting.

### Contract boundaries and acceptance evidence

- Source access remains explicit and workspace-scoped. Import should receive approved bytes or a selected source; it should not silently discover other agents' private session directories.
- Historical transcript statements remain imported evidence. They must never call the execution observer or populate `FinishActivity.tools_executed` as if the current harness ran them.
- The four-role TinyMemory turn vocabulary does not directly represent every trajectory record. A conversion policy must explicitly handle observations/reasoning and retain diagnostics for omissions. System/instruction content must not become active application policy.
- Preserve actual tool-call IDs and source-supplied outcomes; never invent success from text. Apply namespace/provenance policy before storage.
- Test full versus chunked imports, reordered call/results, malformed lines, conflicting session IDs, retries, duplicate imports, omitted fields and multibyte truncation. The peer's [`canonical tests`](https://github.com/letta-ai/trajectory/blob/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773/test/canonical.test.ts) already provide a substantive inventory of these cases.

### Maturity and license

The inspected [`package.json`](https://github.com/letta-ai/trajectory/blob/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773/package.json) is version `0.4.3`, requires Node 20+, and declares Bun/TypeScript development tooling. Its [`CI`](https://github.com/letta-ai/trajectory/blob/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773/.github/workflows/ci.yml) includes tests, type checking, Python parity, real Deep Agents checkpoint compatibility and installation as a package consumer. This is a useful young implementation with meaningful compatibility tests; the source formats and pre-1.0 API can still change.

Source license: [Apache-2.0](https://github.com/letta-ai/trajectory/blob/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773/LICENSE). Preserve its license, applicable notices and change notices in copied or adapted files.

## 2. ContextLattice: bounded evidence with inspectable omissions

### What is already concrete

This repository is more than prompting guidance. Its Rust [`context_retrieval`](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/crates/context_retrieval/src/lib.rs) implements source-aware candidate fusion and lexical/vector retrieval. Its Go [`memory_context_pack.go`](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/services/gateway-go/memory_context_pack.go) defines budget inputs, evidence identity/provenance, selected and omitted items, token metadata and allocation records. [`agent_packet.go`](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/services/gateway-go/agent_packet.go) projects that information into a bounded transport surface.

TinyMemory already has [`RecallMode::Evidence`, citations and `FetchMode::Hybrid`](https://github.com/tinyhumansai/tinymemory/blob/41c34f36d912ffecac24b229cb657f331c7fc607/crates/tinymemory-api/src/query/mod.rs). Its [`provenance contract`](https://github.com/tinyhumansai/tinymemory/blob/41c34f36d912ffecac24b229cb657f331c7fc607/crates/tinymemory-api/src/provenance/README.md) treats unknown/external context conservatively. The best fit is an evidence selection/reporting layer over these interfaces, with final model-context accounting retained in TinyAgents.

### Absorption options

| Scale | Candidate and owner | Required adaptation | Estimated effort |
| --- | --- | --- | --- |
| Primitive | Deterministic source-aware fusion in an owner retrieval implementation | Finite-score validation, explicit source identity and a calibrated boost; avoid extra bitmap/SIMD dependencies for tiny candidate lists without a benchmark | 2–4 days |
| Contract/module | Typed evidence packet and omission receipt in TinyMemory, consumed by TinyAgents context preparation | Citation IDs, namespace reach, provenance, available token budget, truncation/degradation status | 1–2 weeks |
| Separate module | Temporal claims with support/opposition, validity interval and supersession | Extend existing learning/consolidation contracts; preserve history and require explicit verification evidence | 2–4 weeks |
| Whole service | Optional ContextLattice engine/sidecar adapter | Full `MemoryEngine` conformance, health, access scope, export/erase behavior, lifecycle, credentials and deployment ownership | 4–8+ weeks |

### Specific limitations to carry into design

The fusion function uses each item's best source score and adds a boost per distinct source. That is an implementable heuristic, not proof that sources are independent. The shown comparison path does not reject non-finite floats; a Rust adaptation should validate them and make tie behavior explicit. A benchmark must establish whether the additional [`FST/bitmap/SIMD dependencies`](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/crates/context_retrieval/Cargo.toml) improve the intended OpenHuman workload.

The Go budget path falls back to a configured default when its derived target is nonpositive, and packet shrinking uses bounded passes. Those are reasons to specify stronger local invariants before copying: an explicitly exhausted model budget must remain exhausted, and the final serialized packet must either fit or report a refusal/degradation result. An allocation estimate must not be labeled an exact tokenizer count unless that exact tokenizer ran. These are code-derived review findings, not reproduced runtime failures.

The [`cognition proof contract`](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/docs/cognition-proof-core.md) separates observations, verification, contradiction and advisory retrieval planning. That is a useful design direction. However, the active TinyAgents observer records executor entry, including unsuccessful attempts. It cannot supply a successful outcome or independently verified claim by itself. Any future skill promotion needs separate, attributable verification and held-out evaluation.

### Maturity and license

There are focused tests for packet transport, default budgets, learned activation, numeric bounds, outcome bindings and persistence. The codebase also spans Go services, Rust crates, a dashboard, installers and older archived paths. Its [`engine API`](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/docs/engine-api.md) describes several versioned public and private surfaces. That breadth increases integration cost; a repository-wide import would duplicate substantial existing owners. No independent performance or end-to-end reliability result is claimed here.

Public source license: [Apache-2.0](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/LICENSE). The [legal summary](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/docs/legal/README.md) distinguishes public source from separately licensed hosted/private offerings. Do not assume a described private capability is present or licensed for source absorption.

## 3. pstack: evidence and queue primitives, with a different gate policy

The inspected [README](https://github.com/backnotprop/pstack/blob/6f131f137de64fdaf928278c0fd9ecec520ccbee/README.md) identifies this repository as a standalone mirror of Cursor's pstack and documents skill-to-skill dependencies. Some skills work alone; the full orchestration mode brings many additional playbooks and model preferences. It should not silently replace OpenHuman's agent definitions or repository instructions.

The most reusable implementation is the separation of a typed PR snapshot, pure classification, queue scheduling and a fake clock in [`watch-pr/policy.ts`](https://github.com/backnotprop/pstack/blob/6f131f137de64fdaf928278c0fd9ecec520ccbee/skills/poteto-mode/scripts/watch-pr/policy.ts), with [`policy tests`](https://github.com/backnotprop/pstack/blob/6f131f137de64fdaf928278c0fd9ecec520ccbee/skills/poteto-mode/scripts/watch-pr/policy.test.ts) for readiness and queue cadence. This could inform a host-owned, read-only PR status tool or a durable task worker, without putting GitHub-specific policy into the generic TinyAgents execution core.

### The important incompatibility

`assessGitHubMerge` returns an allowed assessment for `BLOCKED` unless the head rollup is explicitly `ERROR` or `FAILURE`. `readSnapshot` can then classify a state with no failed or pending checks as `ci-clean`. OpenHuman PR #14's observed absence of exact-head check runs is precisely a state that must remain **unknown/unverified**, not become a passed gate. Historical passing commits and a review-bot success are insufficient. Copying this policy unchanged would contradict the current publication requirements.

| Scale | Useful absorption | Required boundary | Estimated effort |
| --- | --- | --- | --- |
| Primitive | Typed evidence records, per-head check identity, fake clock and queue/backoff tests | Required checks must be explicit; missing/stale results remain unknown | 1–3 days |
| Focused module | Read-only PR watcher with connector-backed snapshots and reviewable status output | No implicit merge, enablement or messaging; host owns allowed actions | 1–2 weeks |
| Content | Small verification and handoff templates | Fit the repository's actual gates and user's requested rigor | 1–2 days |
| Full orchestrator | pstack's multi-skill mode and autonomous PR program | Would duplicate existing tasks/orchestration, assumptions and permission flows | Not recommended as an absorption target |

The [`check-plan.mjs`](https://github.com/backnotprop/pstack/blob/6f131f137de64fdaf928278c0fd9ecec520ccbee/skills/poteto-mode/scripts/check-plan.mjs) parser correctly distinguishes fenced code when checking plan structure. Its required ten live lanes, fixed headings and typography rules are specific workflow policy, not a general measure of completion. Reuse the idea of evidence attached to each completed item, not those requirements wholesale.

Maturity evidence consists of executable watcher tests and a substantial, actively maintained skill/playbook set. The mirror relationship adds an upstream synchronization concern. License: [MIT, copyright 2026 Lauren Tan](https://github.com/backnotprop/pstack/blob/6f131f137de64fdaf928278c0fd9ecec520ccbee/LICENSE). Preserve that attribution and permission notice in copied/substantially adapted content, as well as the recorded upstream path and commit.

## 4. scientific-agents: optional expertise content with generation checks

This is principally a catalog of profession-specific instructions, not an alternative runtime. The [inspected README](https://github.com/K-Dense-AI/scientific-agents/blob/98c7fae46648724e1a43a1f2b207b0c82fcc462e/README.md) advertises 503 profiles and multiple generated agent/plugin formats. The [`research-software-engineer` profile](https://github.com/K-Dense-AI/scientific-agents/blob/98c7fae46648724e1a43a1f2b207b0c82fcc462e/scientific-agents/research-software-engineer/AGENTS.md) gives concrete numerical-validation, environment-reproduction and artifact-reporting practices. Such content can improve a selected specialist's task framing, but does not install the scientific tools it names or validate scientific conclusions.

### Absorption options

- **Small content contribution, 1–2 days:** curated research-software, statistics or information-retrieval guidance as an optional skill reference, selected for the user's task. Give it normal skill/content priority; do not overwrite a project's existing `AGENTS.md` or promote imported text to trusted system policy.
- **Catalog module, 3–5 days:** a pinned index with profession, work mode, version and summary, rendered through existing discovery/skill loading. Load a chosen profile on demand rather than inserting the full catalog into every prompt.
- **Validation primitive, 1–3 days:** generated-file parity, catalog/schema checks, content-size bounds and duplicate-profile detection adapted to the owner skill pipeline.
- **Whole catalog, 1–2 weeks plus continuing review:** technically feasible as optional content, but needs source updates, profile selection, attribution, token budgets and domain-specific evaluation. It is not a replacement for TinyAgents orchestration or tool permissions.

The [`validator`](https://github.com/K-Dense-AI/scientific-agents/blob/98c7fae46648724e1a43a1f2b207b0c82fcc462e/scripts/validate.py) checks profile headings, a 32 KiB file cap, metadata, generated-file freshness and excessive shared long lines. Its [`CI`](https://github.com/K-Dense-AI/scientific-agents/blob/98c7fae46648724e1a43a1f2b207b0c82fcc462e/.github/workflows/validate.yml) adds plugin/skill schema conformance. These are useful structural quality controls. They do not establish that all named domain standards are current or that every profile improves task outcomes. The linked research paper was not evaluated in this audit.

License: [MIT, copyright 2026 K-Dense, Inc.](https://github.com/K-Dense-AI/scientific-agents/blob/98c7fae46648724e1a43a1f2b207b0c82fcc462e/LICENSE.md). Retain the copyright and permission notice when distributing profiles or substantial derivatives. Keep the optional requested research citation distinct from the legal license notice.

## License compatibility and contribution rules

The inspected [OpenHuman manifest](https://github.com/AmirrezaFarnamTaheri/openhuman/blob/0a0ac335a3ed7210797eeaa2691d6f843ee018a3/Cargo.toml) and [TinyMemory manifest](https://github.com/tinyhumansai/tinymemory/blob/41c34f36d912ffecac24b229cb657f331c7fc607/Cargo.toml) declare `GPL-3.0-only`; TinyAgents also carries GPLv3. MIT and Apache-2.0 source can be incorporated into GPLv3 distributions while retaining the applicable original notices and satisfying the combined distribution's obligations. The [FSF's license list](https://www.gnu.org/licenses/license-list.en.html#apache2) explicitly identifies Apache-2.0 as GPLv3-compatible; its MIT/Expat entry covers the standard permissive form used by the two content repositories.

For an accepted contribution, record the source repository, exact commit, copied/adapted paths, original license/copyright, and modifications. Preserve any applicable NOTICE material and Apache change notices. Check the transitive dependency and asset licenses for a whole-module adoption; the repository's top-level license alone is not an audit of every dependency or private service.

## Recommended order and proof before expansion

1. Finish and publish the active owner changes: packed minimum permissions, actual execution observation and structural code tools. They establish the gate/evidence boundary used by later work.
2. Prototype two transcript formats against TinyMemory's existing source/import seam. Demand loss accounting, stable replay, correct tool linkage, scope enforcement and unknown-outcome preservation.
3. Add a small evidence-packet contract and selected/omitted receipt around existing recall evidence. Verify token exhaustion, final serialization size, deterministic selection, citations and untrusted provenance before adding learned ranking.
4. Offer a few selected scientific profiles through normal skill discovery and measure task-specific utility. Port only the relevant catalog checks.
5. Consider a PR watcher only if the product needs it; establish an explicit unknown state and exact-head required-check policy before reusing pstack's scheduling code.

No whole-runtime absorption is necessary to deliver these gains. Each step has a focused owner, a reviewable contract, and independent acceptance evidence.
