# Prodex -> OpenHuman source and second-order convergence audit

Source: user-provided `prodex-main.zip`, reported version `0.437.10`. Original license: `Apache-2.0` (do not relabel MIT). Target: this branch, stacked on `sync/upstream-2026-10-14`; see draft PR #6.

## Coverage / evidence

The local file-level audit enumerated **all 2,544 files** in **3,052 archive entries**, including **40 Rust crate directories** plus the root package, with **758,625 text lines** and **19,601 heuristic Rust source symbol declarations**. Each file has a SHA256 checksum, tags, category and source-path classification; nested folders were indexed, and cross-crate exact-file/hash as well as name-overlap candidates were screened. File coverage is **not** the same as a semantic line-by-line review or an equivalence proof of all 19,601 symbols.

The offline source checks passed the crate boundaries (41 packages, 131 edges), provider catalog (82 models, 7 provider families), 23 JS CI-runner tests, application/domain/provider-SPI/production/runtime-hotpath guards. Several other checks failed because they reference absent source. In particular, dedicated `src/bin/prodex-gateway.rs`, `src/bin/prodex-control-plane.rs`, and `src/enterprise_serve.rs` are absent; `scripts/ci/enterprise-binaries-guard.mjs` fails, as do auth, control-plane and gateway core/HTTP guards looking for non-existent crates. The normal CLI gateway implementation **does exist** under `crates/prodex-app/src/app_commands/gateway.rs`; do not conflate the two. There are no broken explicit Rust `#[path]` references in this snapshot.

## Crate-group decisions

| Layer / Prodex source | Relationship to OpenHuman | Decision |
|---|---|---|
| Provider interface and adapters: `prodex-provider-core`, `prodex-provider-spi`, `prodex-runtime-proxy` | Native `tinyinference` and host-owned routes already exist | **Integrated bridge** via `prodex` builtin provider, not a second in-process adapter stack |
| Gateway/quota/health: `prodex-app`, `prodex-quota`, `prodex-runtime-quota`, `prodex-runtime-policy` | Quota and admission need one authoritative downstream owner | External service; potential *read-only* scoped status and metrics UI later |
| Secret/profile management: `prodex-secret-store`, `prodex-profile-identity`, `prodex-runtime-cookies`, `prodex-profile-export` | Host credential storage and Prodex provider OAuth are separate trust roots | **No raw token import**, no browser cookie scraping; only a least-privilege inference virtual key in OpenHuman |
| Conversation affinity and retry: `prodex-runtime-proxy/src/error_policy`, `health`, `lineage` | Stream commitment and deferred-response identity are critical | Prodex handles upstream identity; never double rotate the gateway key or replay emitted content |
| Context compaction: `prodex-context`, `prodex-mojo-core` | OpenHuman already owns transcript and compaction | Explore critical-signal-preservation tests; do not graft a second Mojo ABI/runtime |
| MCP/tools: `prodex-mcp-stdio`, `prodex-optional-tools`, `prodex-runtime-launch` | OpenHuman permission and MCP lifecycle already exist | Exclude duplicate permission authority; port only validated tool-diagnostics ideas |
| Governance/enterprise: `prodex-domain`, `prodex-presidio`, enterprise docs | Distinct Prodex admin RBAC and policy versions | External admin service, not automatically implemented by documented but missing binaries |
| Operational safety: `prodex-redaction`, `prodex-audit-log`, `prodex-observability`, `prodex-runtime-log` | Host telemetry/redaction already exist | Candidate targeted redaction and bounded-label contracts with offline negative tests |
| Diagnostics: `prodex-runtime-doctor`, `prodex-runtime-tuning` | Native model-connection test UI already exists | Candidate read-only gateway diagnostics; no privileged host writes |
| State and migrations: `prodex-session-store`, `prodex-runtime-store`, `prodex-state` | Two transaction histories and schemas | **Never** share-write stores; import-only migration with rollback tests |
| Deployment/CLI: `prodex-cli`, `prodex-terminal-ui`, `prodex-runtime-broker`, `prodex-update-notice` | Prodex remains installable on its own | Do not introduce a second CLI, update daemon or release pipeline into OpenHuman |

## Second-order architecture invariants

1. **User/agent policy** is owned by OpenHuman; Prodex may deny a model request but cannot broaden tool/filesystem access.
2. **Upstream provider selection and account quotas** for a `prodex:` route are owned by Prodex. OpenHuman's multi-token key fleet remains in use for other direct BYOK routes; do not nest two rotations for Prodex.
3. **Session transcript and memory** are owned by OpenHuman, while Prodex can own only the provider continuation handles necessary for its transport.
4. **Gateway inference auth** uses a separate `Authorization: Bearer` **inference virtual key**. Never store Prodex admin tokens or Codex OAuth credentials as the OpenHuman provider key.
5. **Admin policy/ledger/quota** stays inside Prodex. Inference virtual keys must not gain admin access; an eventual status UI needs separate read-only scoped authorization.
6. **Streaming** permits credential rotation/retry only before content is exposed; never replay accepted tool responses after an interrupted turn.
7. **Capability signaling** must be model-specific: the gateway exposes chat, Responses, embedding, speech, images and other routes only where the selected upstream supports each.
8. **Privacy** requires explicit disclosure: loopback gateway traffic may still egress to remote model providers. OpenHuman Privacy mode must remain authoritative.

## Changes currently implemented in this PR

- Native Prodex gateway preset in Rust and frontend catalogs, default loopback URL and model picker.
- Early route validation blocks remote plaintext HTTP and URL-embedded secrets for the `prodex` slug (includes localhost and IPv4/IPv6 loopback regression cases).
- Responses API capability gate (Prodex implements `/v1/responses`).
- Offline tests for preset uniqueness, canonical endpoint, Bearer auth and scoped discovery.
- Documentation with explicit `policy.toml` virtual-key example, usage isolation and admin-token warning.
- Dedicated frontend + core GitHub Actions workflow.

## Remaining verification and integration scope

- Tests: run final Rust, frontend, CI lane, lint and Windows checks. Record the precise failures; no silent skips.
- Runtime: with an actual Prodex process and a disposable scoped key, test model listing, chat, Responses, stream cancellation, token usage and unsupported media.
- More modules: time-box each candidate as a separate TDD-backed PR with pre-existing behavior fixture, negative auth tests, performance benchmark and migration/rollback plan.
- Source license: Prodex Apache-2.0, OpenHuman GPL-3.0-only. Preserve applicable notices if original Prodex code is ever directly copied.
- The local full source manifest and detailed second-order checklist are downloadable in the audit bundle. This repository-level synopsis does not falsely claim every distinct feature was implemented or manually validated.

**Acceptance state:** draft; the file census is comprehensive as an *inventory*, but full semantic review, every-feature integration and live gateway E2E are not complete.