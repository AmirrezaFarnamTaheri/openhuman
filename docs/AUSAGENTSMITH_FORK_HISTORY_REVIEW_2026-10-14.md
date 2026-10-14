# AusAgentSmith/openhuman — 2,342-commit historical audit

Date: 2026-10-14. Source: https://github.com/AusAgentSmith/openhuman.
Target: https://github.com/AmirrezaFarnamTaheri/openhuman/pull/5, draft branch `sync/upstream-2026-10-14`.

## Scope and evidence

- GitHub repository metadata identifies a public fork of `tinyhumansai/openhuman`, created 2026-05-14, last pushed 2026-05-26.
- **2,342 commits on its default `main`**: enumerated all pages of GitHub REST `/commits?sha=main&per_page=100&page=1..24` (23 full pages and 42 commits on page 24). The earliest committer timestamp is 2026-01-26, the final is 2026-05-26.
- Approximately **554 first-parent-history entries have author dates on or after 2026-05-14**. Author-date order and commit history may not map exactly to when a change landed in this fork.
- `GET /commits?sha=main&author=AusAgentSmith` returned no authored commits. Do not infer authorship solely from the hosting account: commits from others and integrations still require inspection.
- All 2,342 commit messages and source/author metadata were screened. **This is not a statement that 2,342 individual diffs were inspected line by line.** Focused changed-file and content review was carried out for the highest-signal candidates identified below.
- `GET /repos/tinyhumansai/openhuman/commits/{sha}` resolves historical commit SHAs sampled across the fork's complete age range, including its current tip `ec350c15dd4e5fc36bc35cf0f941c94d5009004f`. The current upstream mainline is rewritten/divergent enough that `compare/main...AusAgentSmith:main` returns `404 No common ancestor`. GitHub forks also share Git object storage; a SHA being retrievable under the upstream API alone does **not** prove that it was merged into upstream's current branch.
- This is therefore predominantly a **historical-implementation audit**, not an `ahead_by: 2342` finding. Attempting a 2,342-commit cherry-pick would be wrong.

## Current architectural difference

The historical fork retains `src/openhuman/...` as the Rust module root; the target today is organized into `crates/openhuman-core/...`, external TinyAgents/TinyMemory modules, native provider registries and separate React feature modules. Changes must be validated against the modern contracts, not copied across paths blindly.

## Findings and decisions

| Historical commit | Finding | Evidence in modern integration | Decision |
|---|---|---|---|
| [`42618b520d`](https://github.com/AusAgentSmith/openhuman/commit/42618b520d371a379a12a8058feb0848d57617e2) | 401/invalid Composio key caused an indefinite 5-second connection polling loop | The modern `app/src/lib/composio/hooks.ts` still called `listConnections` on every interval after a 401, while supporting `composio:config-changed` resets | **Integrated** a guarded auth-failure latch and tests; resets on key/mode changes and rejects stale-generation errors |
| [`82b7303bf6`](https://github.com/AusAgentSmith/openhuman/commit/82b7303bf66fa14d0596100679d795852c301dd8) | User-configurable Kokoro / OpenAI-compatible local TTS (10 files; HTTP/WAV, defaults, UI and RPC) | Modern `crates/openhuman-core/src/voice/factory/entry.rs` explicitly says Kokoro is not part of this cut | **Valuable, not integrated**: requires a native speech adapter, voice schemas, settings UI, network restrictions and tests; do not copy old factory |
| [`38a944d5ab`](https://github.com/AusAgentSmith/openhuman/commit/38a944d5ab847cb2e61fc44ec3a912aa715b9aa3) | LLM namespace-graph entity extraction in the legacy UnifiedMemory ingestion pipeline | Current memory delegates engines, extraction, and lifecycle to vendored TinyMemory | **Research candidate only**: evaluate as an engine capability or plugin, not a host-side legacy port |
| [`62cd6b5154`](https://github.com/AusAgentSmith/openhuman/commit/62cd6b5154e4cc4af7a635c0681fe7439b13f9f8) | Recover some Responses-shaped bodies from compatible chat endpoints | New inference routing is owned by the native registry / TinyInference provider adapter | **Compatibility test candidate**: prove a live/mock wire-shape failure before porting |
| [`1974ed2bc4`](https://github.com/AusAgentSmith/openhuman/commit/1974ed2bc465cb2603284b1804d52675f63663b2) | Local multilingual PII redaction at memory ingestion | Modern `security::scrub` uses `tinyredact` under `host_policy`, and `memory::guard::ScrubbingEngine` scrubs every write | **Superseded**; copying old regex rules would duplicate policy and risk divergence |
| [`0ef21203d4`](https://github.com/AusAgentSmith/openhuman/commit/0ef21203d417195b305538e03870e266485295e7) | Legacy memory chunk splitting, DNS guard, thread-local regex cache | Chunker and URL guards now live behind new module boundaries; the proposed old regex cache is unbounded | **Do not cherry-pick**; separately test boundedness and DNS/SSRF invariants in modern modules |
| [`cf640c7c66`](https://github.com/AusAgentSmith/openhuman/commit/cf640c7c66) / [`8a69877a4d`](https://github.com/AusAgentSmith/openhuman/commit/8a69877a4d) | Provider editor live model selection, testing and advanced endpoint management | Modern `CloudProviderEditor.tsx` + `useCloudProviderEditorSubmit.ts` already perform `/models` probe and persist settings | **Already represented** in newer architecture; no direct port |
| [`fdc469e879`](https://github.com/AusAgentSmith/openhuman/commit/fdc469e879) | Credential-file lock recovery on Windows | `security/credentials/profiles/lock.rs` provides multi-process coordination and self-healing | **Superseded** by newer lock code |
| [`ec350c15dd`](https://github.com/AusAgentSmith/openhuman/commit/ec350c15dd4e5fc36bc35cf0f941c94d5009004f) | After saving a Composio key, launch immediate sync rather than wait for a timer | Modern `ComposioConfigChangedSubscriber` invalidates and eagerly re-warms integrations and publishes `ComposioIntegrationsChanged` | **Largely replaced** by an eager event path; verify background task synchronization separately |
| [`19d8ff9dfd`](https://github.com/AusAgentSmith/openhuman/commit/19d8ff9dfd147f73d5a66019c9c9b68a6ffb7966) | Legacy `hint:agentic` / `hint:coding` routing regression | Today's session/role routing contracts moved across agent and provider factory modules | **Historical regression guard concept**; old code paths not portable |

## Other branches

GitHub reports seven branches (including `main`): `feat/jakedismo-fork-integration`, `fix/handle-dont-show-again-preference-8051465356393478955`, `improve-rewards-api-tests-7711497934610124399`, `jules-7537295491042602194-472f8276`, `senamakel-patch-1`, `test-decrypt-handoff-blob-14953444737016356940`.
Their heads date to May 2026 and include historical UI experiments, tests, a README-only patch, and a large formatting change. GitHub also returns `No common ancestor` for their comparisons to this fork's May 26 main tip; treat them as separate historical snapshots, not inferred ahead counts. No open PRs were returned for this fork.

## Validation and security boundaries

- Only the small 401 polling fix has been transplanted into PR #5. Its tests assert pause-after-401, recovery after credential change, and continued retries for non-auth 503 failures.
- The 401 latch checks the current client-generation id before suppressing polling; an old tenant's rejected call cannot disable a new tenant's poll.
- Tests were committed before the implementation. **GitHub CI remains the authority on passing status**; a queued or cancelled workflow is not a test pass.
- Historical Kokoro HTTP endpoints need explicit user opt-in and SSRF / local-loopback safety review before adding them to a modern app. Local inference may legitimately run on trusted LAN hosts, so choose a clear permission model rather than either silently opening arbitrary network targets or hardcoding only localhost.
- No merges, releases, or main-branch changes are implied by this audit.
