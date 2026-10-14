
# Evidence-Driven OpenHuman Architecture Convergence Implementation Plan

> **For agentic workers:** Use the host's available task-by-task implementation workflow. Steps use checkbox (`- [ ]`) syntax for tracking; completed source edits are marked separately from unverified CI.

**Goal:** Apply the useful recommendations from the two supplied October 14 architecture reports without creating duplicate subsystems, weakening tool/credential security, or exposing UI controls whose backend has no implementation.

**Architecture:** Keep `tinyflows` / `tinyagents`, the existing flow catalog and host-owned `MemoryEngine`, inference/credential factory, security policies and Tauri IPC as the exclusive owners. Improve those layers in independently testable increments. Every remotely supplied decision is a suggestion, never an authority granting execution permissions.

**Tech Stack:** Rust/Tokio, TinyFlows, TinyAgents, TinyMemory, Tauri v2, React 19, TypeScript, xyflow, Vitest and GitHub Actions.

## Global Constraints

- The 15 `NodeKind` variants in `app/src/lib/flows/types.ts` mirror native TinyFlows. Do not add `decision`, `map` or `reduce` in only the React palette: the Rust engine, migrations, serializer, executor, validation, handles and resume tests must all recognize them first.
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
- [x] Implement iterative SCC decomposition and a residual Kahn acyclicity check; reject controllers missing or multiple per cyclic SCC, external side entry, `done` returning to its SCC and caps outside 1–100; missing cap uses native default 25.
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

---

## Remaining Independently Testable Deliverables

### Task 4: Native engine-level loop-soundness enforcement

**Files:**
- Modify: `crates/openhuman-core/src/flows/ops/validation.rs`
- Modify: `crates/openhuman-core/src/flows/ops.rs` authoring boundary only when needed to share the same guard
- Test (proposed): `crates/openhuman-core/src/flows/ops/validation_soundness_tests.rs`

**Interfaces:** consumes `tinyflows::model::WorkflowGraph` and emits structured `tinyflows_catalog::FlowValidationError` with the same stable codes used by the canvas. `flows_validate`, the create/update/import path, and run/resume compatibility gates must all invoke the same pure safety predicate.

- [ ] Add a Rust regression: one SCC has exactly one loop, but a subcycle does not pass through it. Assert native validation and save both reject without side effects; similarly test self-loop, side entry, `done` back-edge and cap bounds.
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
