# Convergence pass 12 — evidence previews and workflow validation freshness

## Ownership and implementation

OpenHuman already owns flow graph serialization, TinyFlows validation, and
source-tool security. This pass corrects two cross-plane evidence/freshness
failures **inside those owners**, rather than adding a new workflow validator,
code-index database, or model-facing cache.

### Workflow editor: revision-keyed validation

`useFlowValidation` used a monotonic request ID to prevent older RPCs from
overwriting newer RPCs, but the ID advanced only when a new validation *began*.
After editing a draft, the next debounced validation waits 500 ms. The previous
draft's pending RPC could complete in that interval and publish its errors or
success to the new draft. That could temporarily re-block or unblock Save.

The hook now stores `{ key, result }` rather than an unscoped validation, and
exposes that result only if the exact current serialized graph key matches.
Pending state is similarly keyed. Each RPC captures its checked key before
starting; a request from an older graph or superseded manual validation returns
`null` instead of an obsolete result. Server-side flow update/persistence
validation remains authoritative.

Regression tests are committed in
`app/src/components/flows/canvas/__tests__/useFlowValidation.test.tsx`:
- Old RPC finishing during a new draft's debounce cannot populate the new result.
- An already displayed error disappears synchronously on graph revision change.
- Concurrent validations finishing in reverse order cannot overwrite the newer result.

### Source evidence: excerpts must display the actual hit

The existing `source_evidence.locate` implementation already centered long-line
excerpts on matched lexical tokens and supplied `column_start`. This pass
adds an explicit `excerpt_truncated` marker, with tests for long/multibyte
lines and unchanged short-line excerpts.

Literal `source_evidence.scan` previously returned the *beginning* of a matched
line, hiding hits far to the right. It now displays a bounded, Unicode-safe
160-scalar excerpt around the exact case-sensitive substring match, preserving
the original `line`, `column_start`, and `excerpt_truncated`. Search results
remain hash-anchored, non-atomic observations, not AST proof.

### Second-order insight

A common invariant across the source intelligence, workflow editor, agent
verification, model routing, and memory retrieval planes is that **evidence
belongs to the exact state it describes**. Source hashes, graph draft keys,
route/credential generations, and post-edit check offsets must be attached to
the observation, not inferred from whichever state happens to be current when
an asynchronous response finishes.

## Verification and remaining scope

Code and regression tests are committed to draft PR #8. **Not yet validated by
execution**: the container has no Rust toolchain or checked-out OpenHuman
node_modules, and GitHub Actions has reported no runs for connector-created
latest-head commits. No claims of Rust compilation, Vitest passing, Tauri/E2E
success, complete peer-method equivalence, or release readiness are made.

The larger convergence program remains open: full AST-derived incremental
repository graph, machine-verifiable final-commit CI receipts, typed model
capability/consent/feasibility routing, unified memory provenance/lifecycle,
and typed artifact/application integrations need independent acceptance gates.

Preserve the actual license and third-party notice obligations for any source
copied from peers. Treating source material as available for comparison does
not change a repository's published license.
