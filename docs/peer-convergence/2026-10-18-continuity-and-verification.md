# Peer audit: agent memory, trajectories, and verification

Inspected on 2026-10-18 through the GitHub connector, using the exact default-branch commits below. This is a source audit, not a claim that the peer projects' tests or performance claims were reproduced. The structural-code and generic execution-observation owner contributions are published and CI-validated; the OpenHuman host assembly has separate pending gates. No peer code has been installed or ported.

## Recommendation

The strongest next contribution is a bounded transcript importer informed by **trajectory**, owned by `tinymemory-integrations`. Next, extract the idea of a typed, budgeted evidence packet from **ContextLattice** into existing memory/context interfaces. Selected **scientific-agents** profiles can supply optional content and inspire native task criteria, artifact requirements and domain-specific review flows. **pstack** contributes useful verification records and queue/test patterns, but its merge-readiness policy cannot serve as OpenHuman's approval gate unchanged.

The current contracts already cover important foundations. TinyMemory owns `MemoryEngine`, validated recall/fetch/store requests, namespace reach, provenance markers and replay-safe storage. TinyAgents owns admission, tool execution, middleware, context management and durable orchestration. OpenHuman owns composition, credentials, workspace policy and the user experience. Contributions should strengthen these owners rather than introduce a second storage or execution authority.

| Repository                                                           | Inspected main commit                                                                                                                         | Material reviewed                                                                              | Initial disposition                                                                  |
| -------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| [ContextLattice](https://github.com/sheawinkler/ContextLattice)      | [`e0daa914fc3f3a9a21363470242ea3fc0bd88964`](https://github.com/sheawinkler/ContextLattice/commit/e0daa914fc3f3a9a21363470242ea3fc0bd88964)   | Rust retrieval, Go context compiler/agent packet, cognition contracts, tests and license       | Extract narrow primitives/contracts; whole runtime is a separate integration project |
| [trajectory](https://github.com/letta-ai/trajectory)                 | [`b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773`](https://github.com/letta-ai/trajectory/commit/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773)          | Normalization types, canonical identity, Codex adapter, bounds, canonical tests and package CI | Highest-value module candidate for authorized transcript import                      |
| [pstack](https://github.com/backnotprop/pstack)                      | [`6f131f137de64fdaf928278c0fd9ecec520ccbee`](https://github.com/backnotprop/pstack/commit/6f131f137de64fdaf928278c0fd9ecec520ccbee)           | PR watcher policy/tests, plan checker, skill dependencies and license                          | Adopt evidence/queue patterns selectively; rewrite readiness semantics               |
| [scientific-agents](https://github.com/K-Dense-AI/scientific-agents) | [`98c7fae46648724e1a43a1f2b207b0c82fcc462e`](https://github.com/K-Dense-AI/scientific-agents/commit/98c7fae46648724e1a43a1f2b207b0c82fcc462e) | Catalog/build validation, CI, research software profile and license                            | Optional domain content plus catalog validation patterns                             |

Effort below means an engineering estimate for one contributor familiar with these repositories, including focused tests and documentation. It excludes waiting for owner review and deployment. Estimates are not measured delivery commitments.

## Inspiration trajectories

A useful result of this audit may be a new native behavior rather than imported code. The following trajectories separate observed source mechanisms or guidance from designs that OpenHuman and its owners would specify and validate independently. A Markdown research profile can inspire a typed requirement as readily as a parser can inspire a codec. Source/text reuse still needs the applicable license; independent design remains eligible even when direct copying is unavailable.

| Peer              | Reusable implementation question                                    | Independent native feature                                                                                    | Richer composition                                                                                                      |
| ----------------- | ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| trajectory        | Which codecs/types are worth adapting with Apache notices?          | A loss-aware import contract with stable source identity and explicit unknown outcomes                        | Imported history, TinyMemory recall and coding handoffs preserve continuity without granting old instructions authority |
| ContextLattice    | Which fusion or packet mechanics fit current memory APIs?           | An evidence packet explains selection, omission, freshness and budget                                         | Recall, context preparation and reviewer exports can share the same selected-source receipt                             |
| pstack            | Which pure watcher/test utilities fit the connector contract?       | A durable repository-state subscription explains why verification is pending, failed or unknown               | A coding workflow resumes from exact revision/check evidence and proposes the next authorized action                    |
| scientific-agents | Which selected content and catalog checks are useful to distribute? | A task profile becomes native criteria, prerequisites and required evidence, with a bounded prompt supplement | Optional research workflows combine planning, admitted tools, artifacts and generation-scoped review                    |

For example, “verify reproducibility” in a profile can inspire a native requirement that identifies a source revision, environment record, command result and retained output. That requirement is stronger and more inspectable than repeating the sentence in a prompt. The profile does not thereby gain permission to run a command, and satisfying the mechanical requirement does not certify the scientific conclusion. The candidate sections below define the smaller mechanisms and failure cases needed for such a composition.

## Owner baseline for the deeper comparison

This expansion re-read the selected peer implementations at the same immutable
commits in the snapshot table. It also checked the active owner contracts:

| Existing owner surface                                                                                                                                                                                | What it already guarantees                                                                                                      | Remaining gap relevant to this audit                                                                                 |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| [TinyMemory conversation reader](https://github.com/tinyhumansai/tinymemory/blob/41c34f36d912ffecac24b229cb657f331c7fc607/crates/tinymemory-integrations/src/sources/readers/conversation/mod.rs)     | Selected thread files, containment checks, known-role turns and source metadata                                                 | External format codecs, complete tool/result relationships, explicit loss accounting and incremental import identity |
| [TinyMemory item/meta contracts](https://github.com/tinyhumansai/tinymemory/blob/41c34f36d912ffecac24b229cb657f331c7fc607/crates/tinymemory-api/src/meta/mod.rs)                                      | Namespace, source, thread/turn range, observed time/actor, tool-call references and optional `derive: false`                    | `ToolCallRef` contains name and optional ID, not a lossless transcript event or result-outcome schema                |
| [TinyAgents finish activity](https://github.com/tinyhumansai/tinyagents/blob/e4021670f652ba0ec59c244aad4f660f216c455b/crates/tinyagents-harness/src/middleware/library/verify_before_finish/types.rs) | Separate requested tools, concrete started executors and argument-aware potential side effects; legacy deserialization defaults | Per-attempt results, durable artifact lineage and proof of successful verification are separate records              |
| [TinyTools code adapter](https://github.com/tinyhumansai/tinytools/blob/daec6c7a563336dd83d2e65ae38b2b169179aada/crates/tinytools-std/src/code/README.md)                                             | Bounded reads, content revisions, explicit custom manifests, preview/write classification and atomic publication receipts       | Multi-file imports, content selection policy and long-lived evidence storage belong elsewhere                        |
| OpenHuman [coding handoff](../../crates/openhuman-core/src/agent/orchestration/coding_handoff/)                                                                                                       | Bounded caller-supplied context carried as untrusted reports                                                                    | Importing a report must not convert it into execution evidence or increase the receiver's permissions                |

TinyMemory is the local memory/retrieval owner. TinySearch is web search; a local
transcript index should not be added to TinySearch merely because both expose
search-like operations. The tables below propose new contracts; their suggested
field names are design sketches, not claims that those APIs already exist.

## 1. trajectory: preserve the meaning of imported sessions

### The concrete gap

The current TinyMemory [`ConversationReader`](https://github.com/tinyhumansai/tinymemory/blob/41c34f36d912ffecac24b229cb657f331c7fc607/crates/tinymemory-integrations/src/sources/readers/conversation/mod.rs) reads OpenHuman-shaped JSON thread files. Its `thread_turns` accepts nonempty string content, skips unknown roles, and initializes `tool_calls` as empty. The existing [`Turn` / `StoreItem::Conversation`](https://github.com/tinyhumansai/tinymemory/blob/41c34f36d912ffecac24b229cb657f331c7fc607/crates/tinymemory-api/src/item/mod.rs) contract can retain timestamps and tool-call references, so format-aware import can improve fidelity without replacing the memory engine.

trajectory supplies a useful reference contract: separate message, reasoning, observation, tool-call and tool-result records; diagnostics for repairs; explicit bounds; source context for chunked input; and an optional outcome flag that is absent when the source does not provide an authoritative result. Its [`types.ts`](https://github.com/letta-ai/trajectory/blob/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773/src/types.ts) does not infer success from result prose.

### Absorption options

| Scale                       | Candidate and owner                                                                                                  | Why it fits                                                                                        | Estimated effort                         |
| --------------------------- | -------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- | ---------------------------------------- |
| Primitive                   | Source identity kind, diagnostics, explicit unknown outcome, Unicode-safe output bounds in `tinymemory-integrations` | Prevents silent loss and ambiguous import receipts                                                 | 2–4 days                                 |
| Focused module              | Codex/Claude transcript codec producing typed source records, then `StoreItem::Conversation`                         | Adds new import formats while keeping namespace and storage policy in TinyMemory                   | 1–2 weeks for two formats                |
| Whole normalization package | An explicitly invoked, pinned TypeScript normalizer used by a host import tool                                       | Reuses the adapter suite, but requires a supported Node runtime and a bridge to Rust storage types | 2–4 weeks including packaging and parity |

The [`canonical.ts`](https://github.com/letta-ai/trajectory/blob/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773/src/canonical.ts) distinction between native, location-based and content-based identity is particularly valuable. Content fallback supports duplicate detection, but cannot establish that changed content is a new version of the same native record. Absolute byte offsets make Codex record identity stable across append-only chunk boundaries. Preserve that distinction as import provenance; do not replace `StoreItem::fingerprint` or engine-assigned identities with untrusted transcript IDs.

The [`Codex adapter`](https://github.com/letta-ai/trajectory/blob/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773/src/adapters/codex/index.ts) recognizes session metadata, ordinary and custom tool calls, and injected environment/instruction prefixes. The default [`bounds`](https://github.com/letta-ai/trajectory/blob/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773/src/bounds.ts) are character limits, not tokenizer guarantees. Integrating them into model context still requires TinyAgents' own budget accounting.

### Contract boundaries and acceptance evidence

- Source access remains explicit and workspace-scoped. Import should receive approved bytes or a selected source; it should not silently discover other agents' private session directories.
- Historical transcript statements remain imported evidence. They must never call the execution observer or populate `FinishActivity.tools_executed` as if the current harness ran them.
- The four-role TinyMemory turn vocabulary does not directly represent every trajectory record. A conversion policy must explicitly handle observations/reasoning and retain diagnostics for omissions. System/instruction content must not become active application policy.
- Preserve actual tool-call IDs and source-supplied outcomes; never invent success from text. Apply namespace/provenance policy before storage.
- Test full versus chunked imports, reordered call/results, malformed lines, conflicting session IDs, retries, duplicate imports, omitted fields and multibyte truncation. The peer's [`canonical tests`](https://github.com/letta-ai/trajectory/blob/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773/test/canonical.test.ts) already provide a substantive inventory of these cases.

### A usable import module, from bytes to retained meaning

The smallest useful feature has three separable layers: a pure decoder of
explicit bytes, a loss-aware projection into existing TinyMemory items, and a
source driver that persists progress under the host-selected namespace. Only
the source driver reads a selected file or stores results. A normalizer should
not probe private session directories, mint a new live TinyAgents session, or
derive tenancy from a transcript's `cwd`, user name or session ID.

The peer separates `record_id`, `record_hash` and `content_hash`. Its canonical
record ID binds the source group, stable record identity and semantic component
key. A native tool-call ID identifies a tool component; message/reasoning
components use type-local ordinals so adding a second component does not shift
the first one's identity. The record hash covers serialized normalized output,
while the content hash excludes transport metadata. This distinction supports
a concrete import policy: changing synthesized timestamps need not masquerade
as new semantic content, while changing tool arguments must remain visible.
See [canonical projection and component keys](https://github.com/letta-ai/trajectory/blob/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773/src/canonical.ts).

An owner design should carry an explicit source generation, codec version and
effective bounds/filter digest with each import receipt. Append-only byte
anchors are stable within a generation; rewriting or rotating a file must start
a new generation. Absolute source timestamps are descriptive data, not resume
cursors. The peer deliberately avoids using its synthesized record timestamp
as the canonical order key. A connector's arrival order is equally unsuitable
as durable transcript order.

| Normalized input                  | Proposed TinyMemory projection                                      | Information that must remain explicit                                                                                                        |
| --------------------------------- | ------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| User/assistant message            | Known-role `Turn` with text and source-supplied time                | Original event identity and whether time was synthesized                                                                                     |
| Assistant tool call               | `Turn.tool_calls` for name/ID, with linked normalized source record | Full arguments exceed the existing `ToolCallRef` vocabulary; preserve them in a separately scoped source record or extend the owner contract |
| Tool result                       | Tool turn plus source linkage                                       | Parent call ID, absent/false/true source-native outcome and truncation; do not guess outcome from prose                                      |
| Environment observation           | Typed source record or explicitly labeled quoted evidence           | It is not necessarily a tool result and must not be linked to a made-up call                                                                 |
| Reasoning record                  | Default omission or separately authorized retained source material  | Diagnostic loss count and source location; it is not user testimony or new system policy                                                     |
| System/developer/injected context | Inert source material or explicit filter omission                   | Never install it as the resumed session's live system prefix                                                                                 |
| Metadata/session header           | Source descriptor                                                   | A claimed path/session ID cannot choose namespace, credential or live agent identity                                                         |
| Unknown/malformed event           | Quarantine/diagnostic with bounded source reference                 | Retained versus omitted bytes, parse location and whether the import is partial                                                              |

The current conversation reader drops unknown roles and accepts string content;
the current metadata can mark machine-produced source material with
`derive: false`. Those are concrete reuse points, not sufficient substitutes
for a lossless event store. The first owner PR should choose whether normalized
events are retained as a source document plus a conversation projection or as
an explicit new typed item. Hiding an arbitrary event object inside tags would
make querying and migration ambiguous. The decision affects fingerprinting,
erase/export behavior and replay compatibility and belongs in TinyMemory's
contract review.

The native path can keep the first two codecs in Rust to avoid shipping another
runtime. A pinned TypeScript normalizer is an alternative when coverage of its
existing adapter suite is worth the Node bridge: run it only through the host's
approved execution surface, pass bounded bytes, forbid discovery side effects,
and compare normalized fixtures with the Rust projection. Node 20+ and any
checkpoint/Python support are separate deployment requirements. A Python-only
checkpoint reader should not silently become a dependency of ordinary JSONL
imports.

### Proposed acceptance and failure cases

These cases are **required future validation**, not tests executed in this
audit. They make the module estimate reviewable.

| Case                | Fixture and expected result                                                                                                                                                      |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Chunk identity      | Normalize the same append-only UTF-8/CRLF transcript whole and at every complete-line boundary; group, record IDs, component keys and logical order agree after combining chunks |
| Partial result      | A result whose call is in a previous chunk remains retained in partial mode; an unresolved result in a declared complete transcript receives the documented diagnostic           |
| Conflicting group   | Caller source context and detected session metadata disagree; refuse/quarantine the chunk before any item or resume cursor is committed                                          |
| Rewritten source    | Same path but changed earlier bytes; do not continue at the old byte offset or misclassify the new generation as an append                                                       |
| Replay/crash        | Kill between item persistence and progress update; replay stores no duplicate logical event and does not skip the first uncommitted event                                        |
| Changed version     | Same native identity with changed content records a conflict/version according to policy; content-fallback identity only supports exact duplicate detection                      |
| Truthful outcome    | Source-native `false` stays false, absent stays unknown, and a string saying “passed” never becomes a successful process receipt                                                 |
| Loss and bounds     | Oversized arguments/results, malformed JSON and filtered roles produce bounded diagnostics and omitted counts; final serialized output obeys its byte limit                      |
| Scope and authority | A transcript names another workspace, an absolute credential path or a new system prompt; the import remains in the selected namespace with inert historical content             |
| Link fidelity       | Interleaved calls, retries and duplicate tool IDs cannot attach the wrong result to a call merely because it is the nearest preceding event                                      |

Implementation should proceed with decoder fixtures and dry-run import receipts
first, then idempotent persistence, then resumable source-driver behavior. An
export/erase round trip and migration test are the completion boundary for the
module. Adding another source format requires its own malformed/partial and
identity fixtures, not only one happy-path transcript.

### Maturity and license

The inspected [`package.json`](https://github.com/letta-ai/trajectory/blob/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773/package.json) is version `0.4.3`, requires Node 20+, and declares Bun/TypeScript development tooling. Its [`CI`](https://github.com/letta-ai/trajectory/blob/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773/.github/workflows/ci.yml) includes tests, type checking, Python parity, real Deep Agents checkpoint compatibility and installation as a package consumer. This is a useful young implementation with meaningful compatibility tests; the source formats and pre-1.0 API can still change.

Source license: [Apache-2.0](https://github.com/letta-ai/trajectory/blob/b8c508fd1e05d1eeb1e2c7e13f41b8c2f3f09773/LICENSE). Preserve its license, applicable notices and change notices in copied or adapted files.

## 2. ContextLattice: bounded evidence with inspectable omissions

### What is already concrete

The repository provides both guidance and implementation; these are different kinds of evidence, not a ranking of value. Its Rust [`context_retrieval`](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/crates/context_retrieval/src/lib.rs) implements source-aware candidate fusion and lexical/vector retrieval. Its Go [`memory_context_pack.go`](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/services/gateway-go/memory_context_pack.go) defines budget inputs, evidence identity/provenance, selected and omitted items, token metadata and allocation records. [`agent_packet.go`](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/services/gateway-go/agent_packet.go) projects that information into a bounded transport surface.

TinyMemory already has [`RecallMode::Evidence`, citations and `FetchMode::Hybrid`](https://github.com/tinyhumansai/tinymemory/blob/41c34f36d912ffecac24b229cb657f331c7fc607/crates/tinymemory-api/src/query/mod.rs). Its [`provenance contract`](https://github.com/tinyhumansai/tinymemory/blob/41c34f36d912ffecac24b229cb657f331c7fc607/crates/tinymemory-api/src/provenance/README.md) treats unknown/external context conservatively. The best fit is an evidence selection/reporting layer over these interfaces, with final model-context accounting retained in TinyAgents.

### Absorption options

| Scale           | Candidate and owner                                                                                  | Required adaptation                                                                                                                                         | Estimated effort |
| --------------- | ---------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| Primitive       | Deterministic source-aware fusion in an owner retrieval implementation                               | Finite-score validation, explicit source identity and a calibrated boost; avoid extra bitmap/SIMD dependencies for tiny candidate lists without a benchmark | 2–4 days         |
| Contract/module | Typed evidence packet and omission receipt in TinyMemory, consumed by TinyAgents context preparation | Citation IDs, namespace reach, provenance, available token budget, truncation/degradation status                                                            | 1–2 weeks        |
| Separate module | Temporal claims with support/opposition, validity interval and supersession                          | Extend existing learning/consolidation contracts; preserve history and require explicit verification evidence                                               | 2–4 weeks        |
| Whole service   | Optional ContextLattice engine/sidecar adapter                                                       | Full `MemoryEngine` conformance, health, access scope, export/erase behavior, lifecycle, credentials and deployment ownership                               | 4–8+ weeks       |

### Specific limitations to carry into design

The fusion function uses each item's best source score and adds a boost per distinct source. That is an implementable heuristic, not proof that sources are independent. The shown comparison path does not reject non-finite floats; a Rust adaptation should validate them and make tie behavior explicit. A benchmark must establish whether the additional [`FST/bitmap/SIMD dependencies`](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/crates/context_retrieval/Cargo.toml) improve the intended OpenHuman workload.

The Go budget path falls back to a configured default when its derived target is nonpositive, and packet shrinking uses bounded passes. Those are reasons to specify stronger local invariants before copying: an explicitly exhausted model budget must remain exhausted, and the final serialized packet must either fit or report a refusal/degradation result. An allocation estimate must not be labeled an exact tokenizer count unless that exact tokenizer ran. These are code-derived review findings, not reproduced runtime failures.

The [`cognition proof contract`](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/docs/cognition-proof-core.md) separates observations, verification, contradiction and advisory retrieval planning. That is a useful design direction. However, the active TinyAgents observer records executor entry, including unsuccessful attempts. It cannot supply a successful outcome or independently verified claim by itself. Any future skill promotion needs separate, attributable verification and held-out evaluation.

### Proposed evidence-packet contract and allocation invariants

The most valuable transfer is an inspectable selection decision, not another
memory database. Start from existing `RecallMode::Evidence` results and retain
their source citations. A pure selector can receive a caller-owned residual
budget, validated candidate scores, source identity and a deterministic tie
rule. TinyAgents then serializes the actual model-visible form and validates
the final budget. The retrieval implementation should not invent a new model
window or silently restore a default after the caller has exhausted it.

The peer's request parser accepts only positive integer hints; a zero value is
therefore indistinguishable from an omitted hint at that point. Its target
calculation falls back to a default if the resulting available amount is
nonpositive. Its packet transport repeatedly shrinks and reattaches accounting
metadata in bounded passes. These are directly observed source behaviors in
[budget parsing](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/services/gateway-go/memory_context_pack.go)
and [transport finalization](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/services/gateway-go/agent_packet.go);
they motivate stronger local guarantees rather than an unsupported assertion
that every peer response is oversized.

A proposed packet needs four independently meaningful parts:

| Part       | Required meaning                                                                                                               | Avoided ambiguity                                                                  |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------- |
| Selection  | Stable candidate/citation ID, source revision, selected span and ranking reason                                                | A high score does not mean the claim was verified                                  |
| Omission   | Counts and bounded references classified by budget, duplicate, stale revision, scope, unsupported content or retrieval failure | “No evidence” differs from “evidence could not be included”                        |
| Accounting | Requested residual budget, tokenizer/model identity, exact/estimated flag and final model-visible count                        | Byte or character counts cannot be labeled exact tokens                            |
| Provenance | Retrieval namespace, source identity, observed/inferred status and freshness                                                   | Multiple mirrors of one upstream statement do not become independent corroboration |

Do not expose forbidden candidate titles or IDs while explaining omissions:
scope filtering must happen before public omission projection. A private
allocator can count rejected candidates without leaking their identities.
Similarly, a citation to a source omitted from the final packet must either
remain resolvable through an authorized fetch handle or be removed from the
answer's evidence set. A packet that lists a citation but cannot recover its
supporting span is not a complete evidence transport.

The Rust fusion rule uses the best candidate score plus a nonnegative boost for
each additional distinct normalized source. It sorts equal scored fused results
by ID, but the selected summary for equal scores can still depend on input
order when both summaries are nonempty. Its `partial_cmp` fallback also allows
non-finite score comparisons to collapse to equality. An independent owner
implementation should reject non-finite input, specify equal-score summary
selection, and distinguish source identity from evidence independence.
For small candidate lists, ordinary bounded maps may be cheaper to maintain
than importing the peer's bitmap/SIMD stack.

### Acceptance gates for a packet module

| Case                  | Required behavior                                                                                                                                                         |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Exhausted context     | An explicit zero residual budget produces an empty/refused packet; configured defaults do not create new capacity                                                         |
| Final serialization   | Add long citation IDs, Unicode text and accounting fields; the final serialized model-visible packet fits or returns a bounded refusal/degradation result                 |
| Determinism           | Shuffle equal-score candidates and duplicate source aliases; selected items, summary choice, order and omission reasons remain stable                                     |
| Invalid scores        | NaN, infinity, out-of-range weights and malformed source identity fail validation before ranking                                                                          |
| False corroboration   | Three mirrors of the same source remain one provenance family for any corroboration display; retrieval diversity and evidential independence are separate fields          |
| Missing evidence      | Retrieval timeout, namespace refusal and size omission produce different statuses and do not become “no matching facts”                                                   |
| Stale citations       | The retained source revision differs when fetched; mark stale/refetch, and do not silently cite current bytes as the selected historical version                          |
| Cross-tenant omission | An inaccessible candidate never leaks its path, title, memory ID or text in public diagnostics                                                                            |
| Promotion boundary    | A started tool attempt, a model-written positive verdict and a passed external test remain distinct evidence kinds; none silently upgrades a memory into a verified skill |

A complete optional ContextLattice engine adapter would also need the
TinyMemory conformance suite, explicit unsupported operations, bounded health
checks, credentials, tenant isolation, source export and deletion semantics.
The gateway's public/private surfaces and mixed-language deployment make that
a substantially larger project than the evidence packet. The packet is useful
even if that service is never adopted.

### Maturity and license

There are focused tests for packet transport, default budgets, learned activation, numeric bounds, outcome bindings and persistence. The codebase also spans Go services, Rust crates, a dashboard, installers and older archived paths. Its [`engine API`](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/docs/engine-api.md) describes several versioned public and private surfaces. That breadth increases integration cost; a repository-wide import would duplicate substantial existing owners. No independent performance or end-to-end reliability result is claimed here.

Public source license: [Apache-2.0](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/LICENSE). The [legal summary](https://github.com/sheawinkler/ContextLattice/blob/e0daa914fc3f3a9a21363470242ea3fc0bd88964/docs/legal/README.md) distinguishes public source from separately licensed hosted/private offerings. Do not assume a described private capability is present or licensed for source absorption.

## 3. pstack: evidence and queue primitives, with a different gate policy

The inspected [README](https://github.com/backnotprop/pstack/blob/6f131f137de64fdaf928278c0fd9ecec520ccbee/README.md) identifies this repository as a standalone mirror of Cursor's pstack and documents skill-to-skill dependencies. Some skills work alone; the full orchestration mode brings many additional playbooks and model preferences. It should not silently replace OpenHuman's agent definitions or repository instructions.

The most reusable implementation is the separation of a typed PR snapshot, pure classification, queue scheduling and a fake clock in [`watch-pr/policy.ts`](https://github.com/backnotprop/pstack/blob/6f131f137de64fdaf928278c0fd9ecec520ccbee/skills/poteto-mode/scripts/watch-pr/policy.ts), with [`policy tests`](https://github.com/backnotprop/pstack/blob/6f131f137de64fdaf928278c0fd9ecec520ccbee/skills/poteto-mode/scripts/watch-pr/policy.test.ts) for readiness and queue cadence. This could inform a host-owned, read-only PR status tool or a durable task worker, without putting GitHub-specific policy into the generic TinyAgents execution core.

### The important incompatibility

`assessGitHubMerge` returns an allowed assessment for `BLOCKED` unless the head rollup is explicitly `ERROR` or `FAILURE`. `readSnapshot` can then classify a state with no failed or pending checks as `ci-clean`. The earlier OpenHuman PR #14 head with no exact-head check runs is an observed example of a state that must remain **unknown/unverified**, not become a passed gate. Historical passing commits and a review-bot success are insufficient. Copying this policy unchanged would contradict the current publication requirements.

| Scale             | Useful absorption                                                                   | Required boundary                                                              | Estimated effort                        |
| ----------------- | ----------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ | --------------------------------------- |
| Primitive         | Typed evidence records, per-head check identity, fake clock and queue/backoff tests | Required checks must be explicit; missing/stale results remain unknown         | 1–3 days                                |
| Focused module    | Read-only PR watcher with connector-backed snapshots and reviewable status output   | No implicit merge, enablement or messaging; host owns allowed actions          | 1–2 weeks                               |
| Content           | Small verification and handoff templates                                            | Fit the repository's actual gates and user's requested rigor                   | 1–2 days                                |
| Full orchestrator | pstack's multi-skill mode and autonomous PR program                                 | Would duplicate existing tasks/orchestration, assumptions and permission flows | Not recommended as an absorption target |

The [`check-plan.mjs`](https://github.com/backnotprop/pstack/blob/6f131f137de64fdaf928278c0fd9ecec520ccbee/skills/poteto-mode/scripts/check-plan.mjs) parser correctly distinguishes fenced code when checking plan structure. Its required ten live lanes, fixed headings and typography rules are specific workflow policy, not a general measure of completion. Reuse the idea of evidence attached to each completed item, not those requirements wholesale.

### A host-owned watcher with auditable state transitions

A portable contribution should retain the separation of the peer's reader,
pure policy and scheduling clock, while using the existing GitHub connector
for authorized reads. Snapshot collection is not itself permission to merge,
enable Actions, comment on a PR or rerun a workflow. Those operations belong to
an explicit host action policy. A read-only watcher can still provide a useful
complete feature: a durable subscription to a repository/PR/head and an evidence
receipt that says exactly what is pending, failed, unavailable or accepted.

The minimally useful snapshot must bind repository identity, PR number, head
SHA, base/ref policy, workflow run/attempt, individual check IDs and the time of
observation. Check names alone are insufficient: two apps can publish the same
name, and a rerun can replace an earlier attempt. Required-check configuration
may itself be inaccessible through an installed connector. That should remain
`requirements_unknown` unless the repository supplies an explicit policy; a
zero-length API response is not a policy saying no checks are required.

| State transition         | Required evidence or refusal                                                                                   |
| ------------------------ | -------------------------------------------------------------------------------------------------------------- |
| Unobserved → pending     | The current head has required work queued/running, or a required result is not yet available                   |
| Pending → passed         | Every required result for the accepted head/attempt satisfies the repository's actual completion policy        |
| Any → unknown            | Required policy, check pages, status API or authorization information is incomplete                            |
| Any → failed             | A decisive current-head failure/cancellation/timeout is final under the repository policy                      |
| Passed → stale           | The head, base policy or dependency/source selection changes after the receipt                                 |
| Passed → action eligible | Separate review/mergeability/authorization requirements also pass; passing CI alone does not authorize a write |

Use API pagination or a declared truncation result before claiming the snapshot
is complete. In the peer, `pendingHistory` changes what evidence is collected
for queue processing. A port should ensure this optimization cannot suppress a
required current-head pending job. Store a monotonically advancing observation
sequence locally and use a monotonic clock for backoff; wall-clock timestamps
remain useful display metadata but should not cause an elapsed deadline to
move backward.

Proposed acceptance cases:

1. No checks, one review-bot success, or only historical-head success remains
   unknown/pending according to explicit requirements.
2. Fetch PR head A, then checks for head B after a push; invalidate the mixed
   snapshot and obtain a consistent head rather than issuing a passed receipt.
3. A required job is skipped, neutral, cancelled or approval-blocked; classify
   it according to repository policy and expose the specific reason.
4. A same-name untrusted app status cannot satisfy a required GitHub Actions
   check; run attempts and check identity remain part of the proof.
5. Rate limiting and transient read failures preserve the last receipt as stale,
   carry resumable queue position and never publish an optimistic replacement.
6. Fake-clock tests exercise deadline equality, long pauses, cancellation,
   backoff caps and fair progress across independent PRs.
7. A passing watcher receipt cannot merge, send a message or mutate a workflow
   without the host's separate authorized action.

The whole pstack playbook includes workflow-specific headings, lane counts and
model preferences. Those may be appropriate in its originating environment.
A useful OpenHuman module would import only the proven snapshot/scheduling
ideas and define repository-specific requirements as data. It should not
impose those playbook conventions on every project.

Maturity evidence consists of executable watcher tests and a substantial, actively maintained skill/playbook set. The mirror relationship adds an upstream synchronization concern. License: [MIT, copyright 2026 Lauren Tan](https://github.com/backnotprop/pstack/blob/6f131f137de64fdaf928278c0fd9ecec520ccbee/LICENSE). Preserve that attribution and permission notice in copied/substantially adapted content, as well as the recorded upstream path and commit.

## 4. scientific-agents: optional expertise content with generation checks

This is principally a catalog of profession-specific instructions, not an alternative runtime. The [inspected README](https://github.com/K-Dense-AI/scientific-agents/blob/98c7fae46648724e1a43a1f2b207b0c82fcc462e/README.md) advertises 503 profiles and multiple generated agent/plugin formats. The [`research-software-engineer` profile](https://github.com/K-Dense-AI/scientific-agents/blob/98c7fae46648724e1a43a1f2b207b0c82fcc462e/scientific-agents/research-software-engineer/AGENTS.md) gives concrete numerical-validation, environment-reproduction and artifact-reporting practices. Such content can improve a selected specialist's task framing, but does not install the scientific tools it names or validate scientific conclusions.

### Absorption options

- **Small content contribution, 1–2 days:** curated research-software, statistics or information-retrieval guidance as an optional skill reference, selected for the user's task. Give it normal skill/content priority; do not overwrite a project's existing `AGENTS.md` or promote imported text to trusted system policy.
- **Catalog module, 3–5 days:** a pinned index with profession, work mode, version and summary, rendered through existing discovery/skill loading. Load a chosen profile on demand rather than inserting the full catalog into every prompt.
- **Validation primitive, 1–3 days:** generated-file parity, catalog/schema checks, content-size bounds and duplicate-profile detection adapted to the owner skill pipeline.
- **Whole catalog, 1–2 weeks plus continuing review:** technically feasible as optional content, but needs source updates, profile selection, attribution, token budgets and domain-specific evaluation. It is not a replacement for TinyAgents orchestration or tool permissions.

The [`validator`](https://github.com/K-Dense-AI/scientific-agents/blob/98c7fae46648724e1a43a1f2b207b0c82fcc462e/scripts/validate.py) checks profile headings, a 32 KiB file cap, metadata, generated-file freshness and excessive shared long lines. Its [`CI`](https://github.com/K-Dense-AI/scientific-agents/blob/98c7fae46648724e1a43a1f2b207b0c82fcc462e/.github/workflows/validate.yml) adds plugin/skill schema conformance. These are useful structural quality controls. They do not establish that all named domain standards are current or that every profile improves task outcomes. The linked research paper was not evaluated in this audit.

### A curated profile pack with a measurable contract

The useful unit can be smaller than a persona: a numerical-validation checklist,
a dependency/environment reporting paragraph, or a reproducibility rubric
attached to a relevant task. At the next scale, a selected research-software
skill can declare the kinds of evidence it asks the agent to collect. A full
catalog importer should wait until these smaller units demonstrate improved
task outcomes under a fixed prompt and tool budget.

A proposed profile manifest should bind upstream path/commit, content digest,
license notice, display summary, domain, supported work modes and declared
tool prerequisites. Prerequisites are descriptive availability checks, not
permission grants or installation instructions. Resolve them against actual
installed tools and show unsupported capabilities explicitly. A profile that
mentions a compiler, literature service or simulation package does not make
that capability available in the host.

The validator's size cap and duplicate-line checks are useful hygiene. They
cannot detect subtly incorrect scientific advice, contradictory profiles or
tool instructions that are inapplicable to the user's project. A maintained
pack therefore needs both structural checks and a small behavioral corpus:
for example, an incomplete numerical result, a missing environment lockfile,
an unavailable referenced tool, and a task with an intentionally irrelevant
specialty. Evaluate whether the profile improves uncertainty reporting and
evidence collection without increasing unnecessary tool use.

| Acceptance/failure case       | Required outcome                                                                                                                                                   |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Project instructions conflict | Imported profile remains ordinary optional skill content; existing project and user authority is preserved                                                         |
| Tool prerequisite missing     | The agent reports the missing capability and uses an authorized available alternative; it does not invent a result or install software automatically               |
| Generated catalogs stale      | Rebuilding profile indexes is deterministic and the parity check identifies the changed generated file                                                             |
| Oversized/duplicate profile   | Bounded validation refuses or flags it before prompt assembly; selection stays within the allocated context budget                                                 |
| Unsupported conclusion        | A plausible domain paragraph without data/results does not satisfy the task's verification criterion                                                               |
| Removal/update                | Removing a selected pack removes it from future discovery; an existing frozen session records its earlier content/version according to TinyAgents' resume contract |
| License provenance            | Every redistributed profile or substantial excerpt retains the corresponding source and MIT notice; external assets and cited papers receive separate review       |

The absence of a reproduced peer evaluation is the main uncertainty. The audit
establishes inspectable content and packaging mechanics, not a quantified gain
from 503 profiles. Start with a small comparison against the current coding
assistant on predeclared tasks, score unsupported claims and useful evidence
separately, and expand only if the narrower pack helps.

License: [MIT, copyright 2026 K-Dense, Inc.](https://github.com/K-Dense-AI/scientific-agents/blob/98c7fae46648724e1a43a1f2b207b0c82fcc462e/LICENSE.md). Retain the copyright and permission notice when distributing profiles or substantial derivatives. Keep the optional requested research citation distinct from the legal license notice.

### A Markdown profile can become a native reproducibility view

The selected research-software-engineer skill distinguishes rerunning the same code/data/environment from testing a conclusion on new data. It asks for pinned environments, numerical tolerances, citation metadata and evidence tied to the release being discussed. It also contains environment-specific advice about container and HPC workflows. These are instructions and method suggestions, not implemented probes or verified standards compliance. [Complete selected skill](https://github.com/K-Dense-AI/scientific-agents/blob/98c7fae46648724e1a43a1f2b207b0c82fcc462e/scientific-agents/research-software-engineer/skills/research-software-engineer/SKILL.md)

An independent native design can turn that guidance into three inspectable records: the intended reproducibility target, the relevant environment/input/code identities, and the observed checks with declared numerical tolerances. Optional citation-file inspection can report syntactic completeness without inventing authorship, a DOI or a publication. The product should select requirements for the actual artifact and execution surface; a local exploratory script need not acquire an HPC deployment or a public release merely because the profile mentions them.

The smallest primitive is a typed requirement whose satisfaction points to a current artifact or observed check. The feature is a reproducibility view that explains missing pins, unsupported platforms and unverified outputs. The larger composition joins that view to resource planning, retained tool results and a reviewer export. This is meaningful native functionality inspired by Markdown even if no profile text is distributed.

**Acceptance cases:** the task targets only a laptop but the profile mentions a cluster; numerical outputs differ within a declared tolerance; the environment digest changes after validation; citation metadata parses but names an unverified release; and the only available evidence is a model-written assertion. Show the applicable requirements and evidence limits. Do not claim independent replication, scientific validity or released-artifact provenance merely from satisfying the mechanical fields.

## License compatibility and contribution rules

The inspected [OpenHuman manifest](https://github.com/AmirrezaFarnamTaheri/openhuman/blob/0a0ac335a3ed7210797eeaa2691d6f843ee018a3/Cargo.toml) and [TinyMemory manifest](https://github.com/tinyhumansai/tinymemory/blob/41c34f36d912ffecac24b229cb657f331c7fc607/Cargo.toml) declare `GPL-3.0-only`; TinyAgents also carries GPLv3. MIT and Apache-2.0 source can be incorporated into GPLv3 distributions while retaining the applicable original notices and satisfying the combined distribution's obligations. The [FSF's license list](https://www.gnu.org/licenses/license-list.en.html#apache2) explicitly identifies Apache-2.0 as GPLv3-compatible; its MIT/Expat entry covers the standard permissive form used by the two content repositories.

For an accepted contribution, record the source repository, exact commit, copied/adapted paths, original license/copyright, and modifications. Preserve any applicable NOTICE material and Apache change notices. Check the transitive dependency and asset licenses for a whole-module adoption; the repository's top-level license alone is not an audit of every dependency or private service.

## Recommended order and proof before expansion

1. Complete the OpenHuman host gates against the published packed-permission, actual-execution and structural-code owner revisions. Their owner CI passes; application validation remains distinct.
2. Prototype two transcript formats against TinyMemory's existing source/import seam. Demand loss accounting, stable replay, correct tool linkage, scope enforcement and unknown-outcome preservation.
3. Add a small evidence-packet contract and selected/omitted receipt around existing recall evidence. Verify token exhaustion, final serialization size, deterministic selection, citations and untrusted provenance before adding learned ranking.
4. Offer a few selected scientific profiles through normal skill discovery and measure task-specific utility. Port only the relevant catalog checks.
5. Consider a PR watcher only if the product needs it; establish an explicit unknown state and exact-head required-check policy before reusing pstack's scheduling code.

No whole-runtime absorption is necessary to deliver these gains. Each step has a focused owner, a reviewable contract, and independent acceptance evidence.
