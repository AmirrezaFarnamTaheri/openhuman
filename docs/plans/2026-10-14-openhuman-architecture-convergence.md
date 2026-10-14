
# Evidence-Driven OpenHuman Architecture Convergence Implementation Plan

> **For agentic workers:** Use the host's available task-by-task implementation workflow. Steps use checkbox (`- [ ]`) syntax for tracking; completed source edits are marked separately from unverified CI.

**Goal:** Apply the useful recommendations from the two supplied October 14 architecture reports without creating duplicate subsystems, weakening tool/credential security, or exposing UI controls whose backend has no implementation.

**Architecture:** Keep `tinyflows` / `tinyagents`, the existing flow catalog and host-owned `MemoryEngine`, inference/credential factory, security policies and Tauri IPC as the exclusive owners. Improve those layers in independently testable increments. Every remotely supplied decision is a suggestion, never an authority granting execution permissions.

**Tech Stack:** Rust/Tokio, TinyFlows, TinyAgents, TinyMemory, Tauri v2, React 19, TypeScript, xyflow, Vitest and GitHub Actions.

## Global Constraints

- The canvas originally exposed 15 editable kinds, but pinned TinyFlows commit `6447612` already implements native `scatter`, `gather`, `spawn`, `gate`, `approval`, `shell` and `void`. The canvas now exposes native `scatter` and `gather` as its 16th/17th kinds. Do not invent frontend-only `decision`, `map` or `reduce` kinds: the Rust executor, migrations, validation, permissions, resume and UI must agree.
- `crates/openhuman-core/src/flows/ops.rs` already owns durable run rows, drop-guard settlement, approval TTL, reconciliation and TinyFlows checkpointing. Do not introduce parallel `workflow_runs` and `subagent_leases` databases.
- A database transaction can atomically reserve a key and commit a local result; it cannot guarantee exactly-once external side effects without target-provided idempotency keys or an outbox/reconciliation protocol. Never use `INSERT OR REPLACE` to claim exactly-once execution.
- `crates/openhuman-core/src/memory/engine.rs` owns user/tenant/agent root scopes. Every graph fact, retrieval and deletion must inherit that root and provenance; no global unscoped memory tables or background exfiltration.
- Never send prompt contents, MCP outputs, user memories or provider credentials to a System-1 model without explicit configuration and the existing egress/approval policy. No remote classifier may auto-grant host, shell, wallet or messaging permissions.
- Scores/probabilities are *uncalibrated* until held-out reliability testing demonstrates calibration. Do not promise 50–200ms requests, 60fps streaming, zero GPU use or exact crash recovery without measurements on real targets.
- The two supplied reports contain illustrative code, not executable specifications: the proposed Rust Tarjan walk admits loop-bypass cycles; its PPR loses dangling probability mass; the Microsoft-Decision-1 JSON request is wrong; the MCP server returns fictional success results.
- Keep the PR in draft until the exact latest head passes required frontend, core Rust, Tauri, storage, lint, security and full integration gates. Never interpret a routing or boundary-check pass as the full build.

---

## Implemented, Awaiting Exact-Head Hosted Verification

### Task 1: Bounded loop authoring-safety check

**Files:**
- Create: `app/src/lib/flows/loopCycleSafety.ts`
- Test: `app/src/lib/flows/loopCycleSafety.test.ts`
- Modify: `app/src/components/flows/canvas/useFlowValidation.ts`
- Modify: `app/src/components/flows/canvas/EditableFlowCanvas.tsx`
- Modify: `app/src/components/flows/canvas/nodeConfig/loopFields.tsx`
- Test: `app/src/components/flows/canvas/nodeConfig/__tests__/loopFieldsBounds.test.tsx`

**Interfaces:** `validateLoopCycleSafety(graph: LoopSafetyGraph): LoopSafetyIssue[]`, consuming TinyFlows-compatible node IDs/kinds/config and edge endpoints/ports. Emits structured code, implicated nodes and an explanatory message; blocks unsafe cycles locally even when daemon validation is delayed.

- [x] Reproduce the SCC flaw: an inner `a → b → a` cycle still exists after removing a bounded loop controller from the same SCC.
- [x] Implement iterative SCC decomposition and a residual Kahn acyclicity check; require cycles to cross at least one loop controller **or** have a native trigger-level `recursion_limit`/`max_node_visits` bound, and permit multiple controllers as upstream does. Enforce the additional canvas restrictions for side entry, `done` re-entry, and caps outside 1–100 (native itself permits any positive iteration cap). Missing cap uses native default 25.
- [x] Integrate error visibility into canvas and form cap.
- [x] Run isolated Node+TypeScript harness: 11 loop assertions including a 14k-node chain pass.
- [ ] Run `cd app && pnpm test:unit app/src/lib/flows/loopCycleSafety.test.ts app/src/components/flows/canvas/nodeConfig/__tests__/loopFieldsBounds.test.tsx` and `pnpm format:check:prettier` on the exact head; record results.

### Task 2: Streaming voice resampling without per-quantum allocations

**Files:**
- Modify: `app/public/live-voice/pcm-capture-processor.js`
- Test: `app/src/features/human/voice/live/pcmCaptureWorklet.test.ts`

**Interfaces:** retain `openhuman-pcm-capture`, `processorOptions.targetSampleRate`, `processorOptions.frameSamples`, and transferred PCM16LE `ArrayBuffer` frames. No changes to session protocol or microphone permissions.

- [x] Remove per-quantum carry concatenation and frame buffer cloning; retain continuous fractional downsample-box / upsample interpolation behavior and binary transfer.
- [x] Run isolated worklet VM harness with nine assertions for chunk-boundary independence, rate conversion, transfer ownership and clipping.
- [ ] Run focused Vitest plus actual microphone/audio-device profiling. Treat worklet allocation reduction as an implementation property, not a measured latency claim.

### Task 3: Bounded run inspector display

**Files:**
- Modify: `app/src/components/flows/RunItemDataBrowser.tsx`
- Test: `app/src/components/flows/__tests__/RunItemDataBrowser.test.tsx`

**Interfaces:** existing `RunItemDataBrowser` props remain unchanged; text/binary/source-pairing remain accessible by stable absolute index.

- [x] Render only the first 100 table rows; incrementally reveal the next 100, preserving original row number and paired input.
- [x] Delay the expensive `formatJson` pass until JSON mode is explicitly selected; JSON mode still exposes the complete result.
- [ ] Run focused Vitest, format and inspect 10K-item run in the desktop app for layout and memory. Consider row virtualization only if incremental pagination fails the UX benchmark.

### Task 3a: Native scatter/gather workflow authoring

**Files:**
- Modify: `app/src/lib/flows/types.ts`, `nodeKindMeta.ts`, `nodeKindIcons.tsx`, `nodeSummary.ts`
- Modify: `app/src/components/flows/canvas/nodeConfig/nodeConfigForms.tsx`
- Create: `app/src/components/flows/canvas/nodeConfig/parallelFields.tsx`
- Create: `app/src/lib/i18n/parallelFlowTranslations.ts`
- Update: `app/src/lib/i18n/I18nContext.tsx`, `scripts/i18n-coverage.ts`, `scripts/i18n-find-english.ts`

**Interfaces:** native `scatter`/`gather` discriminators and the pinned TinyFlows `path`, `lanes` (ceiling 256), `from` (upstream lane-terminal node IDs), `release` (all/any/first_n/quorum/timeout_partial), `n`, `on_lane_error` and `max_polls`. No separate MapNode/ReduceNode or scheduler.

- [x] Expose native kinds in palette, icon/tile maps, card summaries and TypeScript model. New scatters default to eight lanes.
- [x] Provide typed controls and localize every control in all sixteen supported languages.
- [x] Add regression tests for source IDs, release thresholds, palette, and serialization round-trips.
- [ ] Run focused/full Vitest, exact-head TypeScript and locale audits.
- [ ] Execute a real scatter → lane task → gather graph on host, asserting input-ordered results and failed-lane policy. No claimed engine readiness based solely on UI.
- [ ] Evaluate auto-completion of gather's `from` IDs from wire edges without altering valid imported graphs.

---

## Remaining Independently Testable Deliverables

### Task 4: Native engine-level loop-soundness enforcement

**Files:**
- Modify: `crates/openhuman-core/src/flows/ops/validation.rs`
- Modify: `crates/openhuman-core/src/flows/ops.rs` authoring boundary only when needed to share the same guard
- Test (proposed): `crates/openhuman-core/src/flows/ops/validation_soundness_tests.rs`

**Interfaces:** consumes `tinyflows::model::WorkflowGraph` and emits structured `tinyflows_catalog::FlowValidationError` with the same stable codes used by the canvas. `flows_validate`, the create/update/import path, and run/resume compatibility gates must all invoke the same pure safety predicate.

- [ ] Add a Rust regression: an SCC has a loop but a subcycle bypasses every controller *and* no trigger-level bound exists. Assert native validation/save reject without side effects; also test multi-controller cycles, explicit positive trigger visit budgets, self-loops and the additional UI policy separately.
- [ ] Implement iterative SCC and residual DAG checks in the native graph policy/compatibility owner. Ensure the checked graph is the **migrated** graph, and saved descendants and resume paths use the same policy.
- [ ] Run `cargo test -p openhuman-core` with repository feature profile plus the hosted Rust/storage matrix. Keep the JS guard as fast UX feedback, not the sole security authority.
- [ ] Validate imported workflows containing legacy loop configurations, and migrate only where the current engine can faithfully execute them.

### Task 5: Durable reservations and crash recovery in existing storage

**Files:**
- Extend: `crates/openhuman-core/src/flows/tinyflows/state.rs`
- Extend: `crates/openhuman-core/src/flows/ops/run_rows.rs` and `run_management.rs` only as required
- Test (proposed): `crates/openhuman-core/src/flows/ops/durable_reservation_tests.rs`

**Interfaces:** define scope-qualified `reserve(flow_id,node_id,raw_key,run_id,lease_deadline)` returning `Acquired | Active(owner) | Committed(output)`; `settle` uses compare-and-swap ownership and stores a durable digest. Existing `DedupKv` remains supported.

- [ ] Reproduce concurrent workers claiming the same scope/key; assert exactly one reservation winner on SQLite and file drivers, and isolated roots never collide.
- [ ] Add transactional reservation and version/lease ownership in the existing storage port rather than a parallel SQLite database. Preserve committed output on duplicate, never overwrite it with `INSERT OR REPLACE`.
- [ ] Crash-inject after reservation, during dispatch, after external effect and before settlement; for effects lacking remote idempotency, mark `outcome_unknown` and require reconciliation/human action rather than blindly replay.
- [ ] Exercise cancellation, lease expiry, clock skew, restart and lost storage; recover only to an execution state supported by the TinyFlows checkpointer. No speculative side-effect replay.

### Task 6: Typed System-1 decisions, behind explicit opt-in

**Files:**
- Extend: `crates/openhuman-core/src/agent/triage` at its inference/decision boundary (not an unaudited new Tauri server)
- Keep: `app/src/chat/promptInjectionGuard.ts` synchronous rule path by default
- Tests (proposed): `crates/openhuman-core/src/agent/triage/system_one_tests.rs`

**Interfaces:** explicit provider-specific typed `questions` payloads and `answers` parsing; decision result consists of finite normalized outcome probabilities, top-two margin, normalized entropy and a reason for `abstain`. Treat unsupported response shape and model outage as abstain/escalate, never permit.

- [ ] Pin real provider API versions and their allowed endpoints, credentials and telemetry in native inference. Microsoft Foundry path is `/providers/microsoft/v1/systemone` with a deployment-name `model` and named typed `questions`; response uses `answers`. Jev has conflicting independently operated documentation/endpoints: resolve the *actual contracted provider* before sending credentials. No hardcoded latency/price claims.
- [ ] Unit-test missing or extra choice labels, NaN/inf, negatives, probabilities not summing to one, tie order, empty choices, malicious/untrusted body, binary `noul` and ordered `score`. Entropy measures distribution uncertainty; do not label it epistemic uncertainty without calibration.
- [ ] Add a bounded TTL, model/version/policy/scoped-identity cache only for explicitly non-sensitive decisions; never key only on model+state while omitting question, thresholds, tenant and allowed actions.
- [ ] Expose the decision output to triage in **advisory-only mode**. High-impact decisions always continue through existing permission/approval gates. Evaluate calibration, false-negative rates, request cost and p95 before any automatic routing.
- [ ] Only once the Rust executor supports a versioned `DecisionNode` add the React canvas handles and translated settings across all 16 locales. A frontend-only node must not be shipped.

### Task 7: Scoped graph-backed memory relevance

**Files:**
- Extend: `crates/openhuman-core/src/memory/brain.rs` and the existing TinyMemory-backed metadata/store interfaces
- Test (proposed): `crates/openhuman-core/src/memory/graph_relevance_tests.rs`

**Interfaces:** graph triples hold root-scoped source-memory provenance and valid-time intervals; retrieval is bounded by caller's resolved `MemoryEngine` namespace and opt-in lifecycle policy.

- [ ] Baseline current lexical/vector recall quality and source/forget operations. Write tests ensuring roots cannot cross-read, deleted sources disappear and expired/superseded facts never return.
- [ ] Store adjacency/relation metadata through the existing memory storage abstraction; no free-floating global `memory_entities` tables or persisted unsandboxed embedding bytes.
- [ ] Implement finite, nonnegative edge weight validation and Personalized PageRank with sink/dangling probability mass returned to the seed distribution; ensure mass conservation and convergence tolerance.
- [ ] Use reciprocal-rank fusion with deterministic ties and documented lifecycle decay from observed timestamps (never future-dated amplification); bound candidates, graph hops, runtime and injected prompt token cost.
- [ ] Run replay and adversarial multi-hop tests with actual scoped memory engines before wiring graph scores to the MemoryBrainSearch/MemoryAsk UI.

### Task 8: External MCP server only through scoped, real handlers

**Files:**
- Extend: `crates/openhuman-core/src/mcp/host.rs` and the existing security access domain
- Test (proposed): `crates/openhuman-core/src/mcp/server_scope_tests.rs`

**Interfaces:** start with opt-in **stdio** and read-only resource/tool dispatch, never a fictional `Executed` acknowledgement. The later HTTP transport must follow the deployed MCP protocol version, with per-client identity, Origin validation, loopback binding, scoped tokens, quotas and audit events.

- [ ] Add a protocol regression for initialize, tool discovery, malformed JSON, cancellation, unauthorized scopes, revoked token, resource absence and actual tool result/error mapping. No stdout logging in stdio mode.
- [ ] Route permitted memory reads through existing scoped MemoryEngine; require explicit approval and idempotency support before exposing flow triggering, notification sending, host computer use or wallets.
- [ ] Prefer current MCP Streamable HTTP over the obsolete HTTP+SSE-only sample. Consult https://modelcontextprotocol.io/specification and pin the exact version (2026-07-28 has changed transport rules).
- [ ] Fuzz request framing and confirm a malicious webpage, MCP peer or unauthenticated local process cannot trigger tools.

### Task 9: Profiling-led UI and transport isolation

**Files:**
- Inspect: `app/src/components/assistant-ui/markdown-text.tsx`, `app/src/components/flows/canvas/EditableFlowCanvas.tsx`, `app/src/hooks/useFlowRunProgress.ts` and the live-voice session adapter
- Test: existing `markdown-text.lookbehind.test.tsx`, `EditableFlowCanvas.runOverlay.test.tsx`, `pcmCaptureWorklet.test.ts`

- [ ] Profile React commit count on a 500-node workflow and 100 tokens/second chat stream before architecture change. Current markdown is already memoized and PCM is already binary.
- [ ] If graph overlay updates still relayout the entire graph, move **live-only** node progress into per-node external store subscriptions. Keep persisted graph topology/version, validation and copilot diff overlays separate.
- [ ] If Markdown parser work is justified, add adversarial tests for unfinished code fences, blockquotes, tables, links and IME; splitting on double newlines alone is not semantics-preserving.
- [ ] Keep browser worklets within CSP and supported IPC; gate any native VAD behind user configuration and microphone acceptance tests rather than unmeasured resource claims.

---

## Verification Commands and Shipping Gate

- Frontend focused tests: `cd app && pnpm test:unit src/lib/flows/loopCycleSafety.test.ts src/components/flows/__tests__/RunItemDataBrowser.test.tsx src/features/human/voice/live/pcmCaptureWorklet.test.ts src/components/flows/canvas/nodeConfig/__tests__/loopFieldsBounds.test.tsx`.
- Frontend integration: `cd app && pnpm test:unit`, `pnpm format:check:prettier`, and type checking/build commands from `app/package.json` and required CI workflows.
- Native: run the repository's cargo feature matrix including core Rust, Tauri, storage SQLite/file, coverage, lint, windows PowerShell/Pester and Linux/macOS builds where configured.
- Manual: verify large result table expansion, keyboard focus, 44.1/48k audio samples, live run status overlay, loop-validation save refusals, and accessibility at supported viewports.
- Security: run credential redirect, key-store replacement/rollback, malicious tool envelope, tenant isolation and MCP denial tests.
- Release: only after all required checks succeed on the **same exact PR head**, review findings resolve, permissions/provenance are approved, and live remote smoke tests have user-authorized disposable credentials may the PR leave draft.

## Externally Observable Product Decisions

1. **Decision models:** require explicit per-provider credential/egress consent and a review-only pilot; do not silently enable classifier calls on the user composer.
2. **Graph memory:** default off until backend provenance and deletion parity are verified; retain current memory results on rollback.
3. **Workflow replay:** unresolved side effects must be reported as ambiguous, never re-executed without remote idempotency or explicit human reconciliation.
4. **MCP exposure:** stdio read-only first; network serving and side-effect tools require separate explicit enablement and access controls.
5. **Performance:** baseline-first acceptance rather than fabricated percentages; retain backward-compatible transports and formats.

## Source Evidence

- Supplied reports: `Pasted text(20261014-200342).txt` and `Pasted text (2)(20261014-200452).txt`.
- Microsoft-Decision-1 request format: https://learn.microsoft.com/en-us/azure/foundry/foundry-models/how-to/use-foundry-models-microsoft-decision
- MCP transport versions: https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/docs/specification/2026-07-28/basic/transports/index.mdx
- Jev model contracts require exact endpoint/provider verification due materially differing independently published API limits and URL paths.

---

## Added peer sources: 9Router, Jev and Conduit (15 October 2026)

This extends the earlier two-report roadmap with **three actual peer archives**. The 16
previous proxy/API peers and two convergence-evidence archives are tracked in
the local exhaustive source-file census. Lexical symbol extraction enumerated
19 peer projects and 4,623 archived files, but is **not** a test of behavior or
proof of parity for each symbol.

### Integration 10: Native 9Router gateway and progressive account cooldown

**Observed peer contract:** `9router-master/README.md` describes an OpenAI-compatible,
locally hosted `http://localhost:20128/v1` gateway, with `/models` and
`/chat/completions` APIs. `open-sse/services/accountFallback.js` provides
progressive quota lockouts and distinguishes request errors from credential
errors. `open-sse/services/capacityAdapter.js` and `open-sse/providers/capabilities.js`
introduce modality-aware fallback candidates.

**Owned by OpenHuman:** `app/src/components/settings/panels/builtinCloudProviders.ts`,
`crates/openhuman-core/src/inference/provider/factory/key_pool.rs`, and
`credential_fleet.rs`.

- [x] Add opt-in, loopback-only `9router` gateway preset, no default credentials
  (commit `39bf60d`). Test unique slug and exact endpoint/key placeholder.
- [x] Implement bounded progressive 429 cooldown across repeated per-key quota
  errors, preserve the pre-existing first 60-second lockout, cap to five
  minutes, honor upstream Retry-After and never shorten concurrent limits
  (commit `40e5250`). Add Rust regression tests for streak, server override,
  reset, and competing responses.
- [ ] Verify exact-head Rust/Tauri + frontend CI before considering these changes released.
- [ ] Add a *provider-pinned capability decision* before native execution.
  Reject unknown or unsupported modality with an actionable error. A request
  may fall back to another provider only when user billing/content-elevation
  consent was configured. Test image/PDF/audio/video hard capabilities,
  400/403/429 error classification, streaming post-first-token no-replay,
  partial quota data, and explicit paid-route refusal.
- [ ] Correlate provider-key health with user-visible quota/usage and price
  provenance. Do not assert model entitlement from a connector's marketing
  text or open HTTP proxy without auth.

### Integration 11: Jev source search and calibrated evidence decisions

**Observed peer components:** `jev-main/internal/run/{find,ask,locate,budget,hook}.go`,
`internal/scan/{scan,index,ignore}.go`,
`internal/calibration/{calibration,compile,drift}.go`, and
`internal/evidence/{ledger,snapshot}.go`.

**Owned by OpenHuman:** `tinytools_std::filesystem` and its host `FsGate`,
`agent/tinyagents/middleware/tool_output.rs`, `memory/provenance.rs`,
`agent/triage`, and permission-gated MCP tool catalog.

- [ ] Separate *pre-read source candidate screening* from TinyJuice
  *post-read result compression*; the two optimize different costs.
  Enforce ignore files, file-size/byte-attempt ceilings, canonical workspace
  containment, no symlink escapes and no credential-store indexing.
  Add negative privacy tests and snapshot-drift invalidation before remote
  source lookups can be opted into.
- [ ] Add a finite, typed, provider-specific `choice` / `noul` / `score`
  adapter only after pinning exact Jev API wire fixtures; test normalization,
  uncertainty, stale-calibration abstention and tenant/model-version cache
  boundaries. Preserve human approval for high-risk actions.
- [ ] Audit `internal/typesafe/client.go` access-key handling and budget
  behavior before any API use. API usage is opt-in, not silently delegated.

### Integration 12: Conduit hash-anchored edits and source graph

**Observed peer modules:** `crates/change/edit/src/{lib,recovery}.rs`,
`crates/source/graph/src/{imports,queries,refresh,snapshot}.rs`,
`crates/runtime/authz/src/`, `crates/state/scope/src/`,
`crates/transport/mcp/src/`, and RTK/Zap/ripgrep subprojects.

**Owned by OpenHuman:** `tools/impl/filesystem/gate.rs` for access,
`tinytools_std` for the actual file write/edit implementation,
`memory/provenance.rs` for source/forget semantics, existing MCP server
for transport, and existing code tool output artifact pages for large results.

- [ ] Design hash-fenced tool edit contracts `preview`, `commit`,
  `pending_receipts`, `rollback` without circumventing `FsGate`.
  Validate all targets, reject symlinks/hardlinks and canonical aliases,
  preserve mode bits, journal originals before renames, and stop on
  third-party conflicts rather than guessing recovery intent. Tests must
  crash-inject at every rename boundary.
- [ ] Use one source-identity/digest graph for incremental code-search queries
  and Jev candidate evidence. Invalidating a source snapshot must invalidate
  related transient context facts and citations.
- [ ] Preserve OpenHuman native MCP permissions and TinyJuice compression.
  Do not embed a second RTK/Zap daemon or unscoped MCP server.
- [ ] Keep upstream license notices for RTK/Zap (Apache-2.0) and ripgrep
  (public domain / Unlicense) rather than marking every file MIT.

### Second-order release gates

1. **Evidence-bound developer intelligence:** source graph + semantic search +
   digest-anchored read/edit + artifact paging, with existing filesystem
   permissions enforced before index and read.
2. **Capability-aware marketplace:** quota/price/modality telemetry + account
   rotation + *explicit* cross-provider billing/content-routing policy.
3. **Unified receipts:** workflow/checkpoint, file-edit journal and memory
   evidence linked by run/source scope, without parallel database ownership.
4. **Calibrated decision layer:** typed provider adapters whose result can
   advise triage but cannot grant permissions or skip approvals.
5. **MCP interop:** local gateway and source tools exposed only through existing
   stdio/HTTP boundaries and audited scopes.

Each integration requires focused failing tests, implementation, exact-head CI
verification, and an evidence-backed rejection/deferral record for
overlapping or unsafe peer code. **Source census coverage does not mean
feature-complete convergence.**
