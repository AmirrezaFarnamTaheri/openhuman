# Prodex → OpenHuman: second-order convergence, exact duplicates, and trust boundaries

This is a code-grounded follow-up to the 2,544-file source audit in `prodex_openhuman_convergence_audit.zip`.
The source parser inspected 1,870 Rust-bearing files and 12,958 function bodies, finding 74 duplicate-body groups, of which 18 cross crate scopes. Duplicate *bodies* are a screening signal—not proof that two functions have the same semantic owner. This report adjudicates all 18 cross-crate cases.

## Exact duplicate-body review (all cross-crate groups)

| Fingerprint | Pair / scope | Recommendation | Proof obligation |
|---|---|---|---|
| `9cf46aa1` | `prodex-app` vs root auto-rotate test `Drop` | **Keep** isolated test fixture Drop guards; independent process cleanup is intentional. | Both suites still dispose threads/ports independently. |
| `024a9c89` | `prodex-app` and `prodex-runtime-state` state mutation tests | **Keep** cross-layer invariant checks; centralize only fixture data after import. | Both source and application state save paths honor typed mutation scope. |
| `bc682832` | App vs proxy transient 5xx precommit retry tests | **Keep** two-layer policy assertions; source-derived transport contract is valuable. | Test both precommit 5xx retry and postcommit fail-closed; no duplicate billing. |
| `e982cdb8` | `prodex-runtime-proxy` and `prodex-runtime-store` backoff sort key | **Converge later** to one shared host adapter around Mojo `profile_backoff_sort_key`, not duplicate policy. | Differential property tests over retry/transport/circuit tie cases and clock edges. |
| `fdfd8fea` | App and runtime-proxy startup circuit softening | **Converge** app wrapper to proxy-authoritative helper if dependency graph allows; do not extend app-level policy. | Cold restart with persisted probe timestamps, half-open transitions. |
| `d2d0bb02` | App and provider-core Gemini unavailable-model tests | **Keep** source contract + end-to-end adapter tests independently; avoid copying vendor auth. | Gemini alias suppression equal through catalog and runtime launch. |
| `442be7cb` | App and quota display scaling tests | **Keep** display contract tests at both consumers; implementation belongs in quota formatting owner. | Integer-rounding, localization, % clamp boundaries. |
| `530f072c` | Proxy vs store persisted-backoff softening | **Converge** common low-level Mojo adapter or single runtime-state API, if it avoids circular deps. | Idempotent truncation, stale entries deletion, overflow/time regressions. |
| `77328af9` | App info JSON field renderer vs runtime-doctor JSON renderer | **Candidate** for shared bounded display helper; avoid adding new domain crate for two simple wrappers. | Invalid/null/escaped fields, log-safety; no secret serialization. |
| `2d9abc8e` | App vs root usage-server test header builder | **Keep** local fixture builders: unrelated test process boundaries. | Header serialization and required auth remain explicit in each fixture. |
| `b78ee934` | Runtime-doctor vs proxy log-field parser | **Candidate** for lower `prodex-runtime-log` typed parser, not two private parsers. | Escapes, invalid UTF-8, huge/untrusted lines, redaction. |
| `60d8bbf9` | App support and root support `wait_for_poll` | **Candidate** single generic test helper; safe only if the two test binaries can share test-support dependency without feature fanout. | Timeouts under monotonic clocks and no leaked async tasks. |
| `2d6df225` | Proxy vs store transport-backoff maximum | **Converge** with route-key/parsing owner; preserve stable map ordering. | Expired values, simultaneous endpoints, malformed keys. |
| `a2c31a73` | App doctor test vs Mojo doctor rendering test | **Keep** cross-layer parity tests; do not collapse UI/system-boundary checks into Mojo-only tests. | Bad config renders a safe actionable message in actual CLI. |
| `46edb69c` | App and proxy selection-profile test builders | **Keep** per-crate fixture shape; share a type only if the public API contract requires it. | Profiles with OAuth vs virtual keys, conflict semantics. |
| `f3bf7223` | Session-store vs shared-codex-fs compressed session filename checks | **Candidate** single read-only Codex filesystem predicate in `prodex-shared-codex-fs`, after verifying allowed dependency direction. | `.jsonl.gz`, mixed case, path traversal, symlink and archive false positives. |
| `229286c6` | App and proxy test label helpers | **Keep** local test fixtures; unification would be cosmetic. | Labels used in both selection suites still match. |
| `2ddbe7a6` | App Gemini project normalization vs runtime-state lineage component normalization | **Converge conditionally** on `runtime-state` trimming helper only if project IDs and lineage identities have the *same documented* constraints. | Unicode whitespace, empty value, stale credentials and canonicalization invariants. |

## Two control-plane invariants (higher-order pass)

**One provider-routine owner.** OpenHuman owns user-visible model choices and its existing credential-fleet selection for direct BYOK. For the optional `prodex` route, the Prodex gateway owns retries, credential rotation, provider fallback, continuation affinity, quota, and auth. The gateway diagnostic adapter must never construct or mutate a second provider key pool. Native `/v1/responses` and `/v1/chat/completions` are explicitly selected only for opt-in smoke tests.

**One fact owner for capabilities and pricing.** The Prodex ZIP contains 82 model catalog rows across seven provider families, but all 82 have `pricing_known=false`. Several model IDs occur in more than one provider. Their labels and feature flags cannot be presented as *live availability*, *verified supported modalities*, or *free prices*. Only live `/v1/models` can establish availability. The `prodex-catalog-overlay.mjs` implementation preserves this boundary by reporting unknown price/capability states unless the live gateway supplies explicit facts and by refusing to guess among ambiguous provider matches.

## Trust and runtime gates

- No privileged `/v1/prodex/gateway/*` operator endpoint is called by the diagnostic scripts. `PRODEX_GATEWAY_CLIENT_KEY` must be one inference-scoped virtual key; operator/admin token scope cannot be proven by client-side string checks.
- Remote HTTP URLs are refused; HTTPS is required off loopback. Redirects and embedded credentials are rejected; model listing is the default read-only operation.
- The direct inference probe requires `--inference --model MODEL`. It never silently changes endpoints or retries a streamed/committed request.
- Model IDs with Unicode control/format characters are rejected to prevent terminal bidi and log spoofing.
- Reader/transport errors are sanitized so an exception carrying `Authorization` cannot appear in CLI output.
- Deferred Responses results are not reported as a successful completed call. A real gateway with credentials is needed to verify streaming, deferred, long-running and media behavior.
- Source-supplied corporate/admin binaries and Enterprise/SCIM policy are **not** merged into OpenHuman. This ZIP omits standalone `prodex-gateway`, control-plane and `enterprise_serve.rs` source files referenced by parts of its docs and guards.

## CI and boundary-regression verification

The dedicated `prodex-boundary-tests.yml` workflow runs the offline Node.js and Python contract suites on pull requests and relevant main-branch changes. It uses read-only checkout permissions and does not require gateway tokens or privileged admin APIs. The tests cover terminal/control-character injection, credential-safe error handling, hard timeouts, nonempty assistant outputs, offline-vs-live provenance, and shallow-Git clone detection.

## Integration acceptance

**Implemented here:** inference-scoped gateway smoke (`--catalog`, opt-in `responses`/`chat-completions`), live-only/ambiguous-safe catalog overlay, 82-item pinned offline data snapshot, branch-complete Git patch-ID audit tool, and 30 local tests (26 Node and four Python).

**Not demonstrated here:** reading the latest CodeRabbit threads for OpenHuman PRs #5/#6, applying any changes to those remote branches, a full OpenHuman Rust/TypeScript build, a working live Prodex gateway, or complete manual review of all 12,958 Rust bodies. Do not mark a review thread resolved or claim an upstream merge on the strength of this local bundle.
