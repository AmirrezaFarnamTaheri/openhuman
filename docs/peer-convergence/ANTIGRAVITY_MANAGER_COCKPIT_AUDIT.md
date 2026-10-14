# Second expansion: Antigravity Manager and Cockpit Tools source-to-host convergence

## Scope and evidence

This review covers **every ZIP member path** in the two supplied archives, totaling **3,974 files** (Antigravity Manager 486, Cockpit Tools 3,488). The accompanying locally generated `NEW_PEER_SOURCE_CENSUS.json` enumerates per-file size, SHA-256, path-derived tags and source symbols; `NEW_PEER_FILE_DECISIONS.json` triages each entry into an owner/action category. Such machine-assisted enumeration **does not mean every method has been manually proven equivalent**. Three byte-identical cross-archive source matches were found. The audit scripts at `scripts/ci/audit-peer-archives.py` and `classify-peer-census.py` can regenerate the reports from the original archives without executing untrusted source.

| Area | Manager | Cockpit | OpenHuman convergence |
|---|---|---|---|
| General architecture | Rust/Tauri+React, Axum protocol proxy | Rust/Tauri+React, Cockpit Core, Go CLIProxy sidecar | Existing OpenHuman app/core/Tauri remains host; Prodex is delegated gateway |
| Accounts and secrets | OAuth, token pool, rotation, 403 quarantine | 16+ IDE account families, OAuth, injection/instance management | Only existing secure credentials and approved native host manage tokens; UI receives opaque read-only state |
| Quota | per-model percentage, weekly/5h grouped buckets, observed timestamp | per-model percentage, cache with timestamp and tier hints | `observations.mjs` preserves distinct windows, rejects invalid percentages and stale overwrites |
| Routing | model mappings, account tier preference, protocol bridge | Go model routing, cooldown selector, affinity/replay | Reuse OpenHuman's direct fleet or separately deployed Prodex; never run two rotation owners |
| Diagnostics | request logs, proxy monitoring | per-attempt output speed, quota cooldown, streaming lifecycle | `telemetry.mjs` counts unique completed upstream attempts, redacts raw data |
| UI | quota dashboard, health bars, model selectors, settings | overview, per-provider quota/reset widgets, status card layouts | Adapt these concepts to the existing `ModelFleetPanel`; no second navigation shell |
| Process/IDE | account switching, process and local state inspection | multi-instance IDE discovery, IDE-state injection and switching | Native capability only, explicit consent, backups and permission boundaries |
| Transports | OpenAI/Claude/Gemini transcoding, huge streaming pipeline | Go CLIProxyAPI vendored transport and extensions | Differential protocol/parity tests required; no wholesale import of competing relay servers |

## Concrete source-backed findings

1. `Antigravity-Manager-main/src-tauri/src/models/quota.rs` defines `QuotaBucket` with bucket/window, fractional remainder, RFC3339 reset and optional observation timestamp, while `QuotaData` separates per-model quota from grouped windows. The new adapter preserves **both** views instead of collapsing a family to a single percentage.
2. The manager's `resolve_subscription_tier` uses a fallback of `FREE` on unknown tier strings and documents why model-catalog inference is invalid. For OpenHuman, unknown remains **unknown**, because the fallback must not misrepresent account entitlements.
3. `cockpit-tools-main/crates/cockpit-core/src/models/quota.rs` shares the per-model percentage/last-updated vocabulary. Its `modules/quota_cache.rs` maps missing fractions to a numeric zero during cache recovery, potentially reporting exhausted quota. OpenHuman's normalizer rejects missing/invalid fractions rather than inventing exhaustion.
4. `cockpit-tools-main/sidecars/cockpit-cliproxy/request_output_speed.go` ties output speed to the **exact upstream attempt** and requires at least 100 output tokens and 100 ms after its first fragment; it explicitly labels the metric approximate. The new `telemetry.mjs` maintains that narrow semantics without collecting prompts, URLs, cookies or raw tokens.
5. `cockpit-tools-main/sidecars/cockpit-cliproxy/quota_cooldown.go` filters exhausted accounts before sticky selection and prevents recursive auto-recovery. For OpenHuman this must live behind the existing direct-credential owner, not inside a second UI key selector; the optional Prodex gateway owns its own routing.
6. `Antigravity-Manager-main/src-tauri/src/proxy/token_manager.rs`, `thinking_store.rs` and `handlers/openai.rs` are large independent transport/account/session subsystems. They are not drop-in replacements for OpenHuman's factory or Prodex's gateway; tool ordering, streaming, continuation and error parity tests would be required.
7. `cockpit-tools-main/crates/cockpit-core/src/modules/atomic_write.rs` implements temp rename, backup and automatic JSON restore. It does not demonstrate cross-platform fsync/durability parity with OpenHuman's native persistence and must not replace its key vault or config migration unreviewed.
8. The Cockpit archive contains a large vendored `sidecars/cockpit-cliproxy/third_party/CLIProxyAPI` tree. Vendor/package files and duplicated Go adapters belong to dependency provenance and differential tests, not the OpenHuman Rust runtime by default.

## Implemented on this expansion

- `packages/peer-fleet/observations.mjs` and tests: strict scoped identifiers, source provenance, freshness, forbidden accounts, explicit tier provenance, independent model and grouped quota windows, reset normalization, conflict-safe updates and secret-free projections.
- `packages/peer-fleet/telemetry.mjs` and tests: true attempt identity, conservative output throughput, no metrics for incomplete streams, monotonic timing checks, deduplicated aggregates, provider/model isolation and p50/p90 latency.
- `app/src/components/settings/panels/ai/fleetHealthInsights.ts`: adapts the existing **live** OpenHuman model-key status RPC into an aggregate read model; healthy/cooling/evicted and active leases are never described as provider account quota.
- `ModelFleetPanel.tsx`: renders the real fleet aggregate with accessible status messaging. This is read-only; no new secrets, routes or task side effects.

## Second-order architectural convergence

`native credential owner → authenticated provider observation → normalized, revisioned quota windows → safe fleet UI`; separately, `selected host/gateway route → upstream attempt identity → measured completion → redacted aggregate diagnostics`. Account ID is scoped to provider; quota observations and model catalog assertions retain their source evidence. Unverified peer model names must not activate providers or claim capability. A dashboard may advise a route, but cannot steal ownership of its selector or retry/charge logic.

### What is explicitly *not* complete

Porting IDE account switching, protocol servers, Go sidecar clients, media generation, OAuth, credential migration, or privileged process hooks would require a separate product/security design, native owner migration, conformance fixtures, licensing review, and deployment verification. This review does not imply those features were silently incorporated. **Both uploaded projects state CC BY-NC-SA 4.0** in their supplied license material; no content is assumed MIT or relicensed in this work.
