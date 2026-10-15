# Model Fleet PR consolidation — 2026-10-15

## Integration decision

PR #8 is the canonical convergence branch. PR #9 introduced exact per-model
Responses opt-in on an older base; PR #8 already contains the same Rust
provider-factory, UI/RPC, and credential-scope behavior, with an additional
explicit `chat_completions` route, localized editor content, safer key-probe
transaction order, and expanded streaming/tool authorization regressions.

The integration preserves PR #8's implementations rather than replacing them
with the older PR #9 versions. The full set of PR #9 commits remains visible
through merge ancestry as an audit trail. Changes are selectively adopted
based on actual behavior; unneeded old implementation changes are superseded.

## Follow-up correctness fixes

- Enforce the same 200 **UTF-8 byte** maximum for model IDs in TypeScript
  and Rust, including multibyte Unicode IDs (UTF-16 `string.length` was
  insufficient). Preserve the null-prototype parser result.
- Reject raw empty query/fragment delimiters on browser-entered model
  endpoint URLs. The Rust-side endpoint policy already rejects these URL
  forms; the browser had accepted `https://example/v1?` and `.../v1#`.
- Validate explicit model transports against authentication style on **both**
  native update-RPC surfaces, not just when a model is instantiated.
  `Bearer` and `None` permit explicit OpenAI protocols. Anthropic Messages
  and managed JWT retain `Auto` only. This prevents persisting a provider
  setting that can never build a compatible native model.
- Add regression fixtures for these boundaries. The older PR #9 raw-string
  test typo and untranslated labels are already corrected on PR #8.

## Verification still required

The GitHub integration supports source inspection and branch updates, but the
execution container for this pass lacks a Rust toolchain and outbound GitHub
DNS, and no latest-head CI runs were observed. The validation below is **not
claimed passed** until executed against the exact final SHA.

1. Run frontend Vitest, TypeScript, i18n parity, lint and build.
2. Run Rust fmt, clippy, core focused and full tests, including model-factory
   wire-route fixtures, config serde and both RPC boundaries.
3. Verify Tauri production build and desktop provider-editor keyboard,
   scrolling, responsive/error/loading and credential-edit states.
4. Test disposable Chat Completions/Responses gateways and a native Anthropic
   endpoint, with unauthorized, 429, timeout, cancellation and streaming cases.
5. Confirm no API key leaks across changed endpoints, redirects or stored-key
   rollback paths and no replay after streaming has produced output.
6. Require hosted CI conclusions on the **exact** integration head before
   changing draft status or merging into `main`.

The combined work deliberately does not assert broad 2API peer parity, a
fully automatic protocol capability detector, or true incremental Responses
SSE. The currently pinned Responses transport buffers a unary request.


## October 15 follow-up: editor safety and operator UX

The follow-up regression pass on canonical PR #8 closes several UI-contract
gaps without introducing a parallel inference configuration path:

- Direct keyed image-model edits use a shared UTF-8/Unicode-validated,
  null-prototype-safe updater. Its 512-entry budget, replacement behavior and
  HTTPS/loopback boundary match the existing mapping parser and native policy.
  Both textual mappings and the per-row editor are subject to the same limit.
- Preset import defaults to **add missing models only**. Refreshing metadata
  for an existing model requires a separate opt-in; the panel previews
  additions/refreshes and the fields that can be overwritten. Provider
  endpoint/auth synchronization likewise requires approval, displays its
  before/after values and refuses to re-target a stored key. Re-enter a new
  credential through the dedicated provider editor when the endpoint changes.
- Suitability results now expose the complete saved-model catalog through
  searchable, status-filtered, accessible eight-row increments. Cost, capability
  and eligibility are still metadata-only; browsing does **not** route traffic,
  validate entitlements or certify live prices.
- New suitability and consent controls go through the application's i18n
  resolver. Major controls have locale-specific translations; untranslated
  explanatory messages fall back to English rather than raw keys.
- Provider save is single-flight per editor instance. If settings persisted
  but credential storage fails, the compensation path first reads the live
  provider list and restores its original list **only if** it still matches
  the attempted write. It refuses to stomp a divergent concurrent edit and
  redacts candidate keys echoed by a failing gateway.
  
### Remaining durability and acceptance constraints

This is **not a cross-store transaction**: the configuration RPC and encrypted
credential store remain separate. The live-snapshot compensation reduces common
client races but does not provide an atomic compare-and-swap against changes
between the snapshot and the rollback RPC, nor protection against process
termination. A native transactional/revisioned persistence design is required
before advertising crash-atomic credential updates.

Regressions were added to the existing Vitest suites for image IDs, limits,
preset consent, credential re-target checks, suitability browsing/localization
and provider-save overlap/rollback. A separate isolated TypeScript/Node
image-mapping contract passed seven checks; it is **not** a substitute for
OpenHuman's Vitest, Rust, Tauri and hosted CI suites. Keep PR #8 draft until
full tests and latest-head checks have actually passed.
