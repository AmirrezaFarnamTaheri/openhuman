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
