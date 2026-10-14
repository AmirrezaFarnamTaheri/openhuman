# OpenHuman multi-peer integration: source-grounded audit

## Scope and limits

Five local ZIP files were scanned without running untrusted installers. The four external peers contribute 940 regular files (695 source/config files) and the prior Prodex hardening bundle contributes 11 regular files (9 source/config files): **951 files / 704 source-config files**. `PEER_SOURCE_CENSUS.json` identifies each file by archive, relative path, byte size, SHA-256, source symbols when discoverable, and folder/path-derived domain tags. This is a **complete file census**, not proof that every method has been manually evaluated or that the merged application compiles.

The four-peer contracts were initially prepared offline; subsequent repository connector access was restored. This file identifies source-derived candidates, not a claim that the host is fully wired.

## Source provenance and licensing

| Peer | Actual license in supplied ZIP | Architecture | Main valuable scope |
|---|---|---|---|
| `antigravity-panel-main` | Apache-2.0 | VS Code extension; TypeScript MVVM and Lit webview | Read-only quota pools, consumption/history, context gauge, local service diagnostics, cache review, localization, status bar |
| `SwitchAI-Account-Manager-main` | MIT | Tauri Rust + React | Native OS secret-vault interfaces, atomic backup/recovery, quota freshness, stale-revision rejection, refresh coalescing, account sorting and switching |
| `Antigravity-Shield-main` | CC BY-NC-SA 4.0 | Tauri Rust + React, Axum gateway | Image/media model aliases and variants, multi-format protocol mapping, circuit breakers, 403/429 classification, account quarantine, proxy monitoring, MCP/IDE integrations, UI widgets |
| `zero-gravity-main` | MIT | Rust Axum, Language Server and MITM transport | OpenAI/Gemini surface contracts, SSE session semantics, measured token usage, session and quota health, event lifecycle |
| Prior `openhuman_prodx_port_hardened_files.zip` | local patch bundle | Node.js CI/adapters + Python Git audit | Prodex catalog boundary, inference-only smoke, provenance, 30 offline contract tests |

The instruction to "assume everything MIT" does not change the license files. No external source is relicensed by this package; the new library implements concepts independently and retains provenance. Before importing any third-party code verbatim, inspect the corresponding license, notices and usage restrictions.

## File-and-folder architectural assessment

### 1. Antigravity Panel (176 files)

- `src/model/services/quota.service.ts`: live quota API parsing, weekly limits and recoverable errors; adapt a read-only host adapter and keep its CSRF/IDE communication local. The authoritative service must own quota collection, not the webview.
- `src/model/services/context.service.ts`: context-window estimate, actual sent tokens and truncation checkpoint; use the view-model invariant `max(estimated, observed input)` and separate compressed state.
- `src/model/services/storage.service.ts`: bounded quota time series with reset markers; use provider/account/model/window-keyed persistent measurements, not a second incompatible history store.
- `src/model/services/cache.service.ts`: verified path roots, UUID candidates and cleanup plans; defer destructive operations until a host-native dry-run, user confirmation, safe realpath checks and recovery support exist.
- `src/model/services/connection.service.ts`, `src/shared/platform/*`: ambient discovery of local language servers and WSL/multi-platform processes; only if the host opts into IDE diagnostics. No unattended process discovery to infer credentials.
- `src/model/services/automation.service.ts`: auto-accept through IDE/CDP. Not part of core model routing; disabled by default and not copied into the agent harness.
- `src/view-model/app.vm.ts`, `src/view/status-bar.ts`, `src/view/webview/components/*`: dashboard/view-model source, including gauge/usage/credits/context and tooltip patterns. Reuse normalized widget input contracts rather than Lit components directly in React.
- `src/test/suite/*`, `l10n/*`, `scripts/check_l10n.js`: unit and locale checks; localization consistency passed locally.

### 2. SwitchAI Account Manager (162 files)

- `src-tauri/src/providers/{chatgpt,gemini}/*`: provider-specific OAuth, quota and sync adapters. These are sensitive host operations. Use existing OpenHuman credential ownership and OS keyring, not an imported independent token store.
- `src-tauri/src/{secret_store,atomic_file,storage,persisted}.rs`: AES-GCM/keyring vault, private-file writes, temporary fsync/rename, backup and recovery. Native integration needs migration tests, strict file permissions and platform key management before reuse.
- `src-tauri/src/{auto_refresh,refresh_service,app_state}.rs`: bounded concurrency, lock guards, per-account refresh identity and scheduler policies. Ported core ideas: coalescing refreshes and backoff with account metadata instead of token-bearing frontend state.
- `src-tauri/src/{process,shell,portable_updater,subscription,geo}.rs`: process and app-surface switching, subscription metadata, platform updates and reachability. These require explicit host capabilities and permission review; do not start processes from a provider widget.
- `ui/src/utils/{stateMerge,quotaWindows,accountInsights,refreshQueue,subscriptionFilters,accountReorder,privacy,focusSync}.ts`: important small primitives (stale-response arbitration, heterogeneous quota columns, privacy, focus-safe updates). Source-generated read contracts implemented in `packages/peer-fleet/` and guarded with regression tests.
- `ui/src/components/accounts/*`, `ui/src/components/layout/*`, `ui/src/components/modals/*`: reusable *interaction patterns* for quota cells, recommendation, switch-target selection, modal focus and startup recovery. Direct React component transfer requires evaluating OpenHuman's actual frontend tree.
- `ui/src/utils/*.test.mjs`: 46 unit tests passed locally under Node 22 type stripping.

### 3. Antigravity Shield (549 files)

- `src-tauri/src/proxy/{server,config,token_manager,rate_limit,proxy_pool,session_manager,sticky_config}.rs`: broad independent proxy/control plane. Do **not** embed this alongside OpenHuman's existing provider fleet or optional Prodex gateway: that would create competing route, credentials, retry and quota authorities. Instead preserve a clear delegated-gateway boundary.
- `src-tauri/src/proxy/common/{model_mapping,variant_mapping,rate_limiter,session,tool_adapter,json_schema}.rs`: model alias and reasoning tiers, request shape and tool schema normalization. Their ideas are worth adapting into provider-specific capability/format adapters after a source compatibility review. Static model IDs alone are not proof of entitlement or live availability.
- `src-tauri/src/proxy/mappers/{openai,gemini,claude}/*`, `src-tauri/src/proxy/handlers/*`, `src-tauri/src/proxy/upstream/*`: OpenAI, Claude and Gemini message/streaming transforms, multimodal, retries and upstream error surfaces. Review them *against* OpenHuman's existing transports rather than replacing stable implementations wholesale.
- `src-tauri/src/proxy/{image_pool,audio,video,model_specs}.rs` and `src-tauri/resources/model_specs.json`: independent media scheduling and model defaults. Offline model names are imported only as **unverified declarations**; image/audio/video flags remain unknown until a live capability endpoint establishes them.
- `src-tauri/src/proxy/middleware/{auth,ip_filter,cors,logging,monitor}.rs`: host-only access control and monitoring. Retain network binding restrictions, redact secrets and avoid logging full prompts by default.
- `src-tauri/src/modules/{account,oauth,quota,brain_scanner,cache,remote_mobile,device,token_stats,security_db,network_pulse,scheduler}.rs`: support and observability domains. Separate read-only metrics from potentially invasive native state edits and account identity/fingerprint operations.
- `src/components/{accounts,dashboard,proxy,settings,security,stats,ecosystem,common}/*`: quota grids, countdown, heatmap, context/health charts, router and sync widgets, MCP/skills hub, error boundaries and modals. Reconcile design primitives and interaction contracts before porting UI pieces.
- `src/config/modelConfig.ts` and `src-tauri/resources/model_specs.json`: static declarations used in pinned `peer-declared-models.json`, with provenance and unverified state. Aliases targeting different provider families are not auto-enabled.
- `tests/{e2e,skills,adversarial}/*`, `tests/test_code_structure.py`, `tests/test_token_bucket_simulation.py`: protocol/reliability test patterns. Both standalone Python script suites passed; full Rust/JS E2E could not run without their required toolchain/dependencies.

### 4. ZeroGravity (53 files)

- `src/api/{completions,responses,gemini,models,types,polling,search}.rs`: API compatibility surfaces and stream semantics to compare to existing OpenHuman API adapters. Some endpoints are described as WIP by the peer itself.
- `src/{session,quota,trace,warmup,backend}.rs`: request/session state, quota probes, trace observability and backend transport. Session affinity, idempotent resource release and telemetry are reusable concepts.
- `src/proto/*`, `src/standalone/*`: binary-specific protobuf and Language Server process handling. Do not tie the general-purpose agent harness to an undocumented private binary protocol.
- `src/mitm/*` and `scripts/setup-*.{sh,ps1}`: certificate interception, traffic modification, firewall routing and LS impersonation. Not enabled or imported: a separate privileged security/compatibility review would be required.
- `docs/{architecture,mitm,traces,extension-server-analysis,ls-binary-analysis}.md`: architecture/source hypotheses. Treat claimed internals and provider behavior as source assertions, not verified facts about present upstream APIs.
- `src/api/models.rs`: static six-entry alias source imported only as source declarations.

### 5. Previous Prodex patch (11 files)

- `.github/workflows/prodex-boundary-tests.yml` provides offline contract checks.
- `scripts/ci/prodex-catalog-overlay.mjs` retains live-only capabilities and pricing unknown unless verified.
- `scripts/ci/prodex-gateway-smoke.mjs` is an opt-in, inference-scoped gateway diagnostic and does not own credential rotation.
- `scripts/ci/audit-peer-commits.py` examines exhaustive Git history when a full clone exists; it must reject shallow repositories.
- `docs/prodex/SECOND_ORDER_CONVERGENCE.md` resolves duplicate-body findings and the single-owner trust boundary.
- `scripts/ci/tests/*` retains the previous Node and Python regression cases.

## Converged features implemented in this bundle

| Capability | Implementation | Proof |
|---|---|---|
| Provider-scoped model catalog | `packages/peer-fleet/catalog.mjs` | Declarations do not become live entitlements, prices or context limits; same ID across providers is distinct; alias cycles rejected |
| Unverified peer model presets | `packages/peer-fleet/peer-declared-models.json` | Extracted across Panel, Shield, ZeroGravity with concrete path provenance; no alias activation or fictional pricing |
| Heterogeneous quota windows | `packages/peer-fleet/quota.mjs` | Five-hour/weekly/monthly/fallback windows, stale/reset/future/conflicted handling, tightest available quota, history runway |
| Fail-closed direct-key routing | `packages/peer-fleet/routing.mjs` | Health/quota freshness, cooldown, session affinity, capacity and idempotent leases; 429,401,403 classified conservatively |
| Delegated gateway separation | `packages/peer-fleet/routing.mjs`, `convergence.mjs` | Prodex route returns delegated state; no second internal account selection or hidden token rotation |
| Safe multiple model/endpoint presets | `packages/peer-fleet/presets.mjs` | HTTPS and loopback-only HTTP, no embedded URL credentials, revisioned bulk edit, conflict-aware sync, tombstones, model groups |
| Single-flight refresh | `packages/peer-fleet/refresh.mjs` | Burst coalescing, one replay, failures recover, bounded backoff scheduling |
| UI-ready quota/context/history view models | `packages/peer-fleet/widgets.mjs` | Distinguish unknown from exhausted, context compression, consumption/reset boundaries, no inferred across-day split |
| Higher-order fleet composition | `packages/peer-fleet/convergence.mjs` | Catalog+quota+groups+advisory routing with credential-free render shape and scoped account observation merging |
| Source provenance/review | `scripts/ci/audit-peer-archives.py` | ZIP traversal guards, size limits, per-file hashes, symbol and domain indexing; exact cross-archive hash comparison |
| CI | `.github/workflows/peer-fleet-contracts.yml` | Node 22 and Python standard-library offline contract checks, read-only checkout, no secrets |

## Not implemented and not yet verifiable

- No GitHub push, PR #6 update, review-thread resolution, full OpenHuman build, CodeRabbit response or remote CI status verification. The connector and outbound git were unavailable.
- No production OpenHuman Rust dependency additions or Tauri React widget attachment, since the actual branch structure and existing comparable features could not be inspected.
- No working provider OAuth/keychain migration, account switching of external apps, native browser/CDP bridge, privileged MITM, LS binary impersonation, native backup/restore or destructive cache cleaning.
- No real Anthropic/OpenAI/Gemini multimodal gateway, remote media generation, tool-call compatibility, pricing verification or entitlement confirmation. Only the independent contract primitives above are implemented.
- No full source-level semantic equivalence proof for every one of the peer files. The file census is exhaustive; runtime review and behavioral comparisons remain bounded by missing build dependencies and target source.

## Local validations executed

- `node --test packages/peer-fleet/test/*.test.mjs` passes (64 tests at this audit revision).
- `python3 -m unittest discover -s scripts/ci/tests -p 'test_*.py' -v` passes with the earlier Prodex Git audit and new archive tests.
- SwitchAI `ui/src/utils/*.test.mjs` passes (46 source tests under Node 22 experimental TypeScript stripping).
- Panel `node scripts/check_l10n.js` passes (14 additional locales plus English default).
- Shield `python3 tests/test_code_structure.py` and `python3 tests/test_token_bucket_simulation.py` pass.
- Rust compilation and peer end-to-end tests were unavailable (`cargo` absent; npm dependencies not installed and network DNS disabled).

## Extra lexical convergence index

`PEER_CROSS_SOURCE_SYMBOLS.json` enumerates **372 symbol names** and **34 file basenames** occurring in at least two external peer archives. The source list for every candidate is retained. This is a useful exhaustive lexical cross-reference for small-method comparisons, **not** a claim of matching behavior, supersession, or source-level integration. Integration decisions must compare invariants and ownership, not just symbol names.

`PEER_FILE_DECISIONS.json` records a preliminary decision for **every one of the 951 files**; e.g. 245 source files are deliberately left for target-aware semantic comparison, while protocol, UI, telemetry, credential and security modules have distinct gates. Classifications are path-based discovery aids and cannot replace a code review.
